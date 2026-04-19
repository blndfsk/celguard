mod deserialize;
mod model;

pub(crate) use model::{Action, Config, Rule};

use std::{
    io::{self, BufReader, Read},
    path::PathBuf,
};

use anyhow::{Error, Result};
use http_wasm_guest::host;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct HostConfig {
    #[serde(default)]
    paths: Vec<PathBuf>,
    config: Option<Config>,
}

pub(crate) fn read() -> Result<Config> {
    let hc: HostConfig = serde_saphyr::from_slice(&host::admin::config()).map_err(Error::from)?;
    hc.config.map_or_else(|| read_from(&hc.paths), Ok)
}

fn read_from(paths: &[PathBuf]) -> Result<Config> {
    if paths.is_empty() {
        return Err(Error::msg("no config paths provided"));
    }

    let config: Config = serde_saphyr::from_reader(combine(paths))?;
    Ok(config)
}

fn combine(paths: &[PathBuf]) -> Box<dyn Read> {
    let mut readers = Vec::with_capacity(paths.len());

    for path in paths {
        match std::fs::File::open(path) {
            Ok(file) => readers.push(BufReader::new(file)),
            Err(err) => log::warn!("unable to open file: {}, error: {}", path.display(), err),
        }
    }

    match readers.len() {
        0 => Box::new(io::empty()),
        1 => Box::new(readers.into_iter().next().unwrap()),
        _ => {
            let mut combined: Box<dyn Read> = Box::new(readers.pop().unwrap());
            while let Some(reader) = readers.pop() {
                combined = Box::new(reader.chain(combined));
            }
            combined
        }
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use super::*;

    #[test_log::test]
    fn test_read() -> TestResult {
        let cfg = r#"
            actions:
              myjail:
                response: { status: 403, body: forbidden, header: {allow: 'GET'} }
                continue: false
              response_without_body:
                response: { status: 400 }
            rules:
              - name: get_foobar
                disabled: false
                log: off
                tests:
                  - request.method == "GET" && request.path.matches('^/api')
                action: myjail"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        assert!(config.actions.contains_key("myjail"));
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules.first().unwrap().name, "get_foobar");
        assert!(
            config
                .actions
                .get("response_without_body")
                .and_then(|a| a.response.as_ref())
                .map(|r| &r.body)
                .is_some()
        );
        assert!(
            config
                .actions
                .get("myjail")
                .and_then(|a| a.response.as_ref())
                .map(|r| &r.header)
                .is_some()
        );

        Ok(())
    }

    #[test_log::test]
    fn test_defaults() -> TestResult {
        let cfg = r#"
            rules:
              - name: get_foobar
                tests:
                  - request.method == "GET" && request.path.matches('^/api')"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules.first().unwrap().name, "get_foobar");
        assert!(!config.rules[0].disabled);
        Ok(())
    }

    #[test_log::test]
    fn test_invalid_cel_expression() {
        let cfg = r#"
            rules:
              - name: bad_rule
                tests:
                  - "this is not valid CEL @@!""#;
        let result: Result<Config, _> = serde_saphyr::from_str(cfg);
        assert!(result.is_err());
    }

    #[test_log::test]
    fn test_unknown_field_rejected() {
        let cfg = r#"
            rules:
              - name: bad_rule
                tests:
                  - request.method == 'GET'
                unknown_field: oops"#;
        let result: Result<Config, _> = serde_saphyr::from_str(cfg);
        assert!(result.is_err());
    }

    #[test_log::test]
    fn test_response_default_status() -> TestResult {
        let cfg = r#"
            actions:
              block:
                response: {}
            rules:
              - name: test
                tests:
                  - request.method == 'GET'"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        let action = config.actions.get("block").unwrap();
        let resp = action.response.as_ref().unwrap();
        assert_eq!(resp.status, 403);
        Ok(())
    }

    #[test_log::test]
    fn test_action_default_continue_is_false() -> TestResult {
        let cfg = r#"
            actions:
              block:
                response: { status: 403 }
            rules:
              - name: test
                tests:
                  - request.method == 'GET'"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        let action = config.actions.get("block").unwrap();
        assert!(!action.r#continue);
        Ok(())
    }

    #[test_log::test]
    fn test_multiple_rules() -> TestResult {
        let cfg = r#"
            rules:
              - name: rule_one
                tests:
                  - request.method == 'GET'
              - name: rule_two
                tests:
                  - request.method == 'POST'"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.rules[0].name, "rule_one");
        assert_eq!(config.rules[1].name, "rule_two");
        Ok(())
    }

    #[test_log::test]
    fn test_empty_actions_map() -> TestResult {
        let cfg = r#"
            rules:
              - name: test
                tests:
                  - request.method == 'GET'"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        assert!(config.actions.is_empty());
        Ok(())
    }

    #[test_log::test]
    fn test_read_from_empty_paths() {
        let result = read_from(&[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "no config paths provided");
    }

    #[test_log::test]
    fn test_read_from_nonexistent_file() {
        let p = PathBuf::from("nonexistant.yml");
        let result = read_from(&[p]);
        assert!(result.is_err());
    }

    #[test_log::test]
    fn test_disabled_rule_parsing() -> TestResult {
        let cfg = r#"
            rules:
              - name: disabled_rule
                disabled: true
                tests:
                  - request.method == 'GET'"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        assert!(config.rules[0].disabled);
        Ok(())
    }

    #[test_log::test]
    fn test_rule_with_log_level() -> TestResult {
        let cfg = r#"
            rules:
              - name: logged_rule
                log: info
                tests:
                  - request.method == 'GET'"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        assert_eq!(config.rules[0].log, log::LevelFilter::Info);
        Ok(())
    }

    #[test_log::test]
    fn test_default_log_level_is_off() -> TestResult {
        let cfg = r#"
            rules:
              - name: test
                tests:
                  - request.method == 'GET'"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        assert_eq!(config.rules[0].log, log::LevelFilter::Off);
        Ok(())
    }
}
