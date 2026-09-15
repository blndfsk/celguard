use serde::Deserialize;

use crate::config::rule::{Action, Response};

#[derive(Deserialize, Debug)]
#[serde(default)]
pub(crate) struct Config {
    pub(crate) default_action: Action,
}

impl Default for Config {
    fn default() -> Self {
        Self { default_action: DEFAULT_ACTION }
    }
}
/// Default action used when a rule matches without an explicit action.
const DEFAULT_ACTION: Action = Action {
    response: Some(Response { status: 400, body: None, header: None }),
    log: log::LevelFilter::Off,
    r#continue: false,
};
