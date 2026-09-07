use crate::config::{
    deserialize,
    rule::{Action, Response, Rule},
};
use cel::Program;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub(crate) struct Config {
    #[serde(default, deserialize_with = "deserialize::deserialize_opt_program")]
    pub(crate) source_ip: Option<Program>,
    pub(crate) rules: Vec<Rule>,
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
