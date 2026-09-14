use serde::Deserialize;

use crate::config::rule::{Action, Response};

#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    pub(crate) default_status: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self { default_status: 400 }
    }
}
/// Default action used when a rule matches without an explicit action.
const DEFAULT_ACTION: Action = Action {
    response: Some(Response { status: None, body: None, header: None }),
    r#continue: false,
};

impl Config {
    pub(crate) fn default_action(&self) -> &'static Action {
        &DEFAULT_ACTION
    }
}
