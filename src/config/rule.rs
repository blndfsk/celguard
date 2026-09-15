use crate::config::deserialize;
use cel::Program;
use log::LevelFilter;
use serde::Deserialize;
use serde_saphyr::RcAnchor;
use std::collections::HashMap;

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rule {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) disabled: bool,
    #[serde(default, deserialize_with = "deserialize::deserialize_vec_program")]
    pub(crate) tests: Vec<Program>,
    pub(crate) action: Option<RcAnchor<Action>>,
}

impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Action {
    pub(crate) response: Option<Response>,
    #[serde(default = "default_level", deserialize_with = "deserialize::deserialize_level")]
    pub(crate) log: LevelFilter,
    #[serde(default)]
    pub(crate) r#continue: bool,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Response {
    pub(crate) status: Option<i32>,
    #[serde(default)]
    pub(crate) body: Option<String>,
    #[serde(default)]
    pub(crate) header: Option<HashMap<String, String>>,
}

fn default_level() -> LevelFilter {
    LevelFilter::Off
}
