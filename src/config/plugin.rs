use crate::config::rule::{Action, Response};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Config {
    pub(crate) error_action: Action,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            error_action: Action {
                response: Some(Response { status: 500, body: None, header: None }),
                r#continue: false,
            },
        }
    }
}
