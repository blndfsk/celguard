use crate::config::deserialize;
use cel::Program;
use log::LevelFilter;
use serde::Deserialize;
use serde_saphyr::RcAnchor;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rule {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) disabled: bool,
    #[serde(default = "default_level", deserialize_with = "deserialize::deserialize_level")]
    pub(crate) log: LevelFilter,
    #[serde(default, deserialize_with = "deserialize::deserialize_vec_program")]
    pub(crate) tests: Vec<Program>,
    pub(crate) action: Option<RcAnchor<Action>>,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            name: Default::default(),
            disabled: Default::default(),
            log: LevelFilter::Off,
            tests: Default::default(),
            action: Default::default(),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Action {
    #[serde(default)]
    pub(crate) response: Option<Response>,
    #[serde(default)]
    pub(crate) r#continue: bool,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Response {
    pub(crate) status: i32,
    pub(crate) body: Option<String>,
    pub(crate) header: Option<HashMap<String, String>>,
}

fn default_level() -> LevelFilter {
    LevelFilter::Off
}
