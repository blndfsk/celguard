use crate::config::rule::{Action, Response};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Config {
    pub(crate) error_action: Action,
}

impl Default for Config {
    fn default() -> Self {
        Self { error_action: DEFAULT_ACTION }
    }
}
/// Default action used when a rule matches without an explicit action.
const DEFAULT_ACTION: Action = Action {
    response: Some(Response { status: 500, body: None, header: None }),
    r#continue: false,
};
