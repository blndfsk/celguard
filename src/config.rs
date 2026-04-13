use std::{
    collections::HashMap,
    fmt::Debug,
    fs::File,
    io::{self, BufReader, Read},
};

use anyhow::{Error, Result};
use cel::Program;
use http_wasm_guest::host;
use log::LevelFilter;

use serde::{Deserialize, de::Deserializer};

#[derive(Deserialize, Debug, Default)]
#[serde(tag = "type", rename_all = "lowercase")]
pub struct Action {
    pub response: Option<r#Response>,
    #[serde(default)]
    pub r#continue: bool,
}

#[derive(Deserialize, Debug)]
pub struct r#Response {
    #[serde(default = "default_status")]
    pub status: i32,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub header: HashMap<String, String>,
}

fn default_status() -> i32 {
    403
}

impl Action {
    pub fn execute(&self, response: &host::Response) -> bool {
        if let Some(resp) = &self.response {
            for (key, value) in &resp.header {
                response.header().set(key.as_bytes(), value.as_bytes());
            }
            if let Some(body) = &resp.body {
                response.body().write(body.as_bytes());
            }
            response.set_status(resp.status);
        };
        self.r#continue
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub actions: HashMap<String, Action>,
    pub rules: Vec<Rule>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub name: String,
    #[serde(default)]
    pub disabled: bool,
    #[serde(deserialize_with = "deserialize_level", default = "default_level")]
    pub log: LevelFilter,
    #[serde(deserialize_with = "deserialize_filters")]
    pub tests: Vec<Program>,
    pub action: Option<String>,
}

fn deserialize_filters<'de, D>(d: D) -> Result<Vec<Program>, D::Error>
where
    D: Deserializer<'de>,
{
    let filters: Vec<String> = Vec::deserialize(d)?;
    filters
        .iter()
        .map(|s| Program::compile(s).map_err(serde::de::Error::custom))
        .collect()
}

fn deserialize_level<'de, D>(d: D) -> Result<LevelFilter, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    s.parse().map_err(serde::de::Error::custom)
}

fn default_level() -> LevelFilter {
    LevelFilter::Off
}

#[derive(Debug, Deserialize)]
struct HostConfig {
    #[serde(default)]
    paths: Vec<String>,
    config: Option<Config>,
}

pub(crate) fn read() -> Result<Config> {
    let hc: HostConfig = serde_saphyr::from_slice(&host::admin::config()).map_err(Error::from)?;
    hc.config.map_or_else(|| read_from(&hc.paths), Ok)
}

pub fn read_from(paths: &[String]) -> Result<Config> {
    if paths.is_empty() {
        return Err(Error::msg("no config paths provided"));
    }
    let readers: Result<Vec<_>, io::Error> = paths
        .iter()
        .map(|f| File::open(f).map(BufReader::new))
        .collect();

    let reader = chain_readers(readers?);

    let config: Config = serde_saphyr::from_reader(reader)?;
    Ok(config)
}

fn chain_readers(mut readers: Vec<BufReader<File>>) -> Box<dyn Read> {
    // Start by popping the last reader
    let mut combined: Box<dyn Read> = match readers.pop() {
        Some(reader) => Box::new(reader),
        None => Box::new(io::empty()), // No readers, empty reader
    };

    // Chain remaining readers one by one
    while let Some(reader) = readers.pop() {
        combined = Box::new(reader.chain(combined));
    }

    combined
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
}
