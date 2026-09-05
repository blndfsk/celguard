use std::collections::HashMap;

use crate::config::deserialize;
use cel::Program;
use log::LevelFilter;
use serde::Deserialize;
use serde_saphyr::RcAnchor;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rule {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) disabled: bool,
    #[serde(
        deserialize_with = "deserialize::deserialize_level",
        default = "deserialize::default_level"
    )]
    pub(crate) log: LevelFilter,
    #[serde(default, deserialize_with = "deserialize::deserialize_vec_program")]
    pub(crate) tests: Vec<Program>,
    pub(crate) action: Option<RcAnchor<Action>>,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            name: Default::default(),
            disabled: false,
            log: LevelFilter::Off,
            tests: Default::default(),
            action: Default::default(),
        }
    }
}

#[derive(Deserialize, Default, Debug, PartialEq)]
pub(crate) struct Action {
    pub(crate) response: Option<Response>,
    #[serde(default)]
    pub(crate) r#continue: bool,
}

#[derive(Deserialize, Debug, PartialEq)]
pub(crate) struct Response {
    pub(crate) status: Option<i32>,
    #[serde(default)]
    pub(crate) body: Option<String>,
    #[serde(default)]
    pub(crate) header: Option<HashMap<String, String>>,
}
