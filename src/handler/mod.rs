use std::collections::HashMap;

use http_wasm_guest::host::Response;

use crate::config::Action;

pub(crate) struct Handler {
    actions: HashMap<String, Action>,
}

impl Handler {
    pub(crate) fn new(actions: HashMap<String, Action>) -> Self {
        Handler { actions }
    }
    pub(crate) fn execute(&self, name: &str, response: &Response) -> bool {
        if let Some(act) = self.actions.get(name) {
            execute(act, response)
        } else {
            execute(&Action::default(), response)
        }
    }
}

fn execute(action: &Action, response: &Response) -> bool {
    if let Some(resp) = &action.response {
        for (key, value) in &resp.header {
            response.header.set(key.as_bytes(), value.as_bytes());
        }
        response.set_status(resp.status);
        if let Some(body) = &resp.body {
            response.body.write(body.as_bytes());
        }
    }
    action.r#continue
}
