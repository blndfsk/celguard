use std::{error::Error, time::Duration};

use cel::Program;
use duration_str::deserialize_duration;

use http_wasm_guest::{api, host};
use serde::{
    de::{self, Deserializer},
    Deserialize,
};

#[derive(Deserialize, Debug, PartialEq)]
pub enum Action {
    Jail,
    Log,
}

#[derive(Deserialize, Debug)]
pub struct Rule {
    pub action: Action,
    #[serde(default, deserialize_with = "deserialize_duration")]
    pub duration: Duration,
    #[serde(deserialize_with = "program")]
    pub trigger: Program,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub rules: Vec<Rule>,
}

pub(crate) fn read() -> Result<Config, Box<dyn Error>> {
    let bytes = host::config();
    parse_rules(&bytes)
}

fn parse_rules(bytes: &api::Bytes) -> Result<Config, Box<dyn Error>> {
    let config: Config = serde_json::from_slice(bytes)?;
    Ok(config)
}

fn program<'de, D>(d: D) -> Result<Program, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    Program::compile(&s).map_err(de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let bytes =
            api::Bytes::from(r#"{"rules":[{"action":"Jail","duration":"5h","trigger":"1 == 1"}]}"#);
        let c = parse_rules(&bytes);
        assert!(c.is_ok());
        let config = c.unwrap();
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules.get(0).unwrap().action, Action::Jail);
        assert_eq!(
            config.rules.get(0).unwrap().duration,
            Duration::from_hours(5)
        );
    }
}
