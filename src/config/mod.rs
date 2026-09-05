use std::{
    io::{self, BufReader, Read},
    path::PathBuf,
};

use anyhow::{Error, Result};
use cel::Program;
use http_wasm_guest::host;
use serde::Deserialize;

use crate::config::model::Response;

mod deserialize;
mod model;

pub(crate) use model::{Action, Rule};

#[derive(Deserialize, Debug, Default)]
pub(crate) struct Config {
    #[serde(default, deserialize_with = "deserialize::deserialize_opt_program")]
    pub(crate) source_ip: Option<Program>,
    pub(crate) rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
struct HostConfig {
    #[serde(default)]
    paths: Vec<PathBuf>,
    config: Option<Config>,
}

/// Default action used when a rule matches without an explicit action.
const DEFAULT_ACTION: Action = Action {
    response: Some(Response { status: None, body: None, header: None }),
    r#continue: false,
};

impl Config {
    pub(crate) fn default_action() -> &'static Action {
        &DEFAULT_ACTION
    }
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
    let mut iter = readers.into_iter();
    let Some(first) = iter.next() else {
        return Box::new(io::empty());
    };
    let mut combined: Box<dyn Read> = Box::new(first);
    for reader in iter {
        combined = Box::new(reader.chain(combined));
    }
    combined
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use testresult::TestResult;

    use super::*;

    #[test_log::test]
    fn test_read() -> TestResult {
        let cfg = r#"
            source_ip: request.source_ip
            actions:
              - &myjail
                response: { status: 403, body: forbidden, header: {allow: 'GET'} }
                continue: false
              - &response_without_body
                response: { status: 400 }
            rules:
              - name: get_foobar
                disabled: false
                log: off
                tests:
                  - request.method == "GET" && request.path.matches('^/api')
                action: *myjail"#;
        let r = BufReader::new(cfg.as_bytes());
        let config: Config = serde_saphyr::from_reader(r)?;

        assert!(config.source_ip.is_some());
        assert_eq!(config.rules.len(), 1);

        let rule = config.rules.first().unwrap();
        assert_eq!(rule.name, "get_foobar");
        assert!(rule.action.is_some());

        let action = rule.action.as_ref().unwrap();
        assert!(!action.r#continue);
        assert!(action.response.is_some());
        Ok(())
    }

    #[test_log::test]
    fn test_rule_without_action() -> TestResult {
        let cfg = r#"
            rules:
              - name: get_foobar
                tests:
                  - request.method == "GET" && request.path.matches('^/api')"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules.first().unwrap().name, "get_foobar");
        assert!(!config.rules[0].disabled);
        assert!(config.rules[0].action.is_none());
        assert_eq!(config.rules[0].log, log::LevelFilter::Off);
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
    fn test_action_default_continue_is_false() -> TestResult {
        let cfg = r#"
            actions:
              - &block
                response: { status: 403 }
            rules:
              - name: test
                action: *block"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        let action = config.rules.first().unwrap().action.as_ref();
        assert!(!action.unwrap().r#continue);
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
    fn test_multiple_rules_same_action() -> TestResult {
        let cfg = r#"
            actions:
              - &action1
                response: { status: 403 }
            rules:
              - name: rule_one
                tests:
                  - request.method == 'GET'
                action: *action1
              - name: rule_two
                tests:
                  - request.method == 'POST'
                action: *action1"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        let a = config.rules[0].action.as_ref().unwrap().0.as_ref();
        let b = config.rules[1].action.as_ref().unwrap().0.as_ref();
        assert!(ptr::addr_eq(a, b));
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
        let p = PathBuf::from("nonexistent.yml");
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
}
