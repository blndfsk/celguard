use anyhow::{Context, Error, Result};
use http_wasm_guest::host;
use serde::Deserialize;
use std::{
    io::{self, BufReader, Read},
    path::PathBuf,
};

mod deserialize;
pub(crate) mod matcher;
pub(crate) mod plugin;
pub(crate) mod rule;

#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) plugin: plugin::Config,
    pub(crate) matcher: matcher::Config,
}

#[derive(Debug, Deserialize)]
struct HostConfig {
    #[serde(default)]
    paths: Vec<PathBuf>,
    config: Option<Config>,
}

pub(crate) fn read() -> Result<Config> {
    let hc: HostConfig =
        serde_saphyr::from_slice(&host::admin::config()).context("failed to parse host config")?;
    hc.config.map_or_else(|| read_from(&hc.paths), Ok)
}

fn read_from(paths: &[PathBuf]) -> Result<Config> {
    if paths.is_empty() {
        return Err(Error::msg("no config paths provided"));
    }

    let where_from = paths.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>().join(", ");
    let options = serde_saphyr::options! { with_snippet: false };
    let config: Config = serde_saphyr::from_reader_with_options(combine(paths), options)
        .with_context(|| format!("failed to read config from {where_from}"))?;
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

    use crate::config::rule::Response;

    use super::*;

    #[test]
    fn test_read_full() -> TestResult {
        let cfg = r#"
            plugin:
              default_action:
                response: { status: 401 }
            actions:
              - &myjail
                response: { status: 403, body: forbidden, header: {allow: 'GET'} }
              - &response_without_body
                response: { status: 400 }
            matcher:
              source_ip: request.source_ip
              rules:
                - name: get_foobar
                  disabled: false
                  tests:
                    - request.method == "GET" && request.path.matches('^/api')
                  action: *myjail"#;
        let r = BufReader::new(cfg.as_bytes());
        let config: Config = serde_saphyr::from_reader(r)?;

        let pc = config.plugin;
        assert_eq!(
            pc.default_action.response,
            Some(Response { status: Some(401), body: None, header: None })
        );

        let mc = config.matcher;

        assert!(mc.source_ip.is_some());
        assert_eq!(mc.rules.len(), 1);

        let rule = mc.rules.first().unwrap();
        assert_eq!(rule.name, "get_foobar");
        assert!(rule.action.is_some());

        let action = rule.action.as_ref().unwrap();
        assert!(!action.r#continue);
        assert!(action.response.is_some());
        Ok(())
    }

    #[test]
    fn test_read_minimal() -> TestResult {
        let cfg = r#"
            matcher:
              rules:
                - name: get_foobar
                  tests: []"#;
        let r = BufReader::new(cfg.as_bytes());
        let config: Config = serde_saphyr::from_reader(r)?;

        let pc = config.plugin;
        assert!(pc.default_action.response.is_some());

        let mc = config.matcher;

        assert!(mc.source_ip.is_none());
        assert_eq!(mc.rules.len(), 1);

        let rule = mc.rules.first().unwrap();
        assert_eq!(rule.name, "get_foobar");
        assert!(rule.action.is_none());
        Ok(())
    }

    #[test]
    fn test_rule_without_action() -> TestResult {
        let cfg = r#"
            matcher:
              rules:
                - name: get_foobar
                  tests:
                    - request.method == "GET" && request.path.matches('^/api')"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        let mc = config.matcher;
        assert_eq!(mc.rules.len(), 1);
        assert_eq!(mc.rules.first().unwrap().name, "get_foobar");
        assert!(!mc.rules[0].disabled);
        assert!(mc.rules[0].action.is_none());
        Ok(())
    }

    #[test]
    fn test_invalid_cel_expression() {
        let cfg = r#"
            matcher:
              rules:
                - name: bad_rule
                  tests:
                    - "this is not valid CEL @@!""#;
        let result: Result<Config, _> = serde_saphyr::from_str(cfg);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_field_rejected() {
        let cfg = r#"
            matcher:
              rules:
                - name: bad_rule
                  tests:
                    - request.method == 'GET'
                  unknown_field: oops"#;
        let result: Result<Config, _> = serde_saphyr::from_str(cfg);
        assert!(result.is_err());
    }

    #[test]
    fn test_action_default_continue_is_false() -> TestResult {
        let cfg = r#"
            actions:
              - &block
                response: { status: 403 }
            matcher:
              rules:
                - name: test
                  action: *block"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        let mc = config.matcher;
        let action = mc.rules.first().unwrap().action.as_ref();
        assert!(!action.unwrap().r#continue);
        Ok(())
    }

    #[test]
    fn test_multiple_rules() -> TestResult {
        let cfg = r#"
            matcher:
              rules:
                - name: rule_one
                  tests:
                    - request.method == 'GET'
                - name: rule_two
                  tests:
                    - request.method == 'POST'"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        let mc = config.matcher;
        assert_eq!(mc.rules.len(), 2);
        assert_eq!(mc.rules[0].name, "rule_one");
        assert_eq!(mc.rules[1].name, "rule_two");
        Ok(())
    }

    #[test]
    fn test_multiple_rules_same_action() -> TestResult {
        let cfg = r#"
            actions:
              - &action1
                response: { status: 403 }
            matcher:
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
        let mc = config.matcher;
        let a = mc.rules[0].action.as_ref().unwrap().0.as_ref();
        let b = mc.rules[1].action.as_ref().unwrap().0.as_ref();
        assert!(ptr::addr_eq(a, b));
        Ok(())
    }

    #[test]
    fn test_read_from_empty_paths() {
        let result = read_from(&[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "no config paths provided");
    }

    #[test]
    fn test_read_from_nonexistent_file() {
        let p = PathBuf::from("nonexistent.yml");
        let result = read_from(&[p]);
        assert!(result.is_err());
    }

    #[test]
    fn test_disabled_rule_parsing() -> TestResult {
        let cfg = r#"
            matcher:
              rules:
                - name: disabled_rule
                  disabled: true
                  tests:
                  - request.method == 'GET'"#;
        let config: Config = serde_saphyr::from_str(cfg)?;
        let mc = config.matcher;
        assert!(mc.rules[0].disabled);
        Ok(())
    }
}
