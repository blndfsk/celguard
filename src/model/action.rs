use std::{collections::HashMap, ptr};

use http_wasm_guest::host;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub(crate) struct Action {
    pub(crate) response: Option<Response>,
    #[serde(default)]
    pub(crate) r#continue: bool,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Response {
    pub(crate) status: Option<i32>,
    #[serde(default)]
    pub(crate) body: Option<String>,
    #[serde(default)]
    pub(crate) header: Option<HashMap<String, String>>,
}

/// Default action used when a rule matches without an explicit action.
const DEFAULT_ACTION: Action = Action { response: None, r#continue: false };

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        ptr::addr_eq(self, other)
    }
}

impl Action {
    pub(crate) fn default_action() -> &'static Action {
        &DEFAULT_ACTION
    }

    // order matters
    pub(crate) fn execute(&self, response: &host::Response) -> (bool, i32) {
        let resp = self.response.as_ref();
        write_header(response, resp.and_then(|r| r.header.as_ref()));
        write_status(response, resp.and_then(|r| r.status.as_ref()));
        write_body(response, resp.and_then(|r| r.body.as_ref()));
        (self.r#continue, 0)
    }
}
fn write_header(response: &host::Response, header: Option<&HashMap<String, String>>) {
    if let Some(map) = header {
        for (key, value) in map {
            response.header.set(key.as_bytes(), value.as_bytes());
        }
    }
}
fn write_status(response: &host::Response, status: Option<&i32>) {
    match status {
        Some(value) => response.set_status(*value),
        None => response.set_status(403), //TODO read from config
    };
}
fn write_body(response: &host::Response, body: Option<&String>) {
    if let Some(str) = body {
        response.body.write(str.as_bytes());
    }
}
