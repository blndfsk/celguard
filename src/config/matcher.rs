use crate::config::{deserialize, rule::Rule};
use cel::Program;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    #[serde(default, deserialize_with = "deserialize::deserialize_opt_program")]
    pub(crate) source_ip: Option<Program>,
    pub(crate) rules: Vec<Rule>,
}
