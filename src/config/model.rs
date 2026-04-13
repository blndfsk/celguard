use std::collections::HashMap;

use cel::Program;
use log::LevelFilter;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(tag = "type", rename_all = "lowercase")]
pub(crate) struct Action {
    pub(crate) response: Option<Response>,
    #[serde(default)]
    pub(crate) r#continue: bool,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Response {
    #[serde(default = "super::deserialize::default_status")]
    pub(crate) status: i32,
    #[serde(default)]
    pub(crate) body: Option<String>,
    #[serde(default)]
    pub(crate) header: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Default)]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) actions: HashMap<String, Action>,
    pub(crate) rules: Vec<Rule>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rule {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) disabled: bool,
    #[serde(
        deserialize_with = "super::deserialize::deserialize_level",
        default = "super::deserialize::default_level"
    )]
    pub(crate) log: LevelFilter,
    #[serde(deserialize_with = "super::deserialize::deserialize_filters")]
    pub(crate) tests: Vec<Program>,
    pub(crate) action: Option<String>,
}
