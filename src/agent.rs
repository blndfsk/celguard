use std::collections::HashMap;

use http_wasm_guest::host::Response;

use crate::config::Action;

pub struct Agent {
    default: Action,
    actions: HashMap<String, Action>,
}
impl Agent {
    pub fn new(actions: HashMap<String, Action>) -> Self {
        Self {
            actions,
            default: Action::default(),
        }
    }
    pub fn perform(&self, action_name: &str, response: &Response) -> bool {
        let action = self.actions.get(action_name).unwrap_or(&self.default);
        if let Some(jail) = &action.jail {
            log::info!("{:?}", jail);
        };
        if let Some(resp) = &action.response {
            response.set_status(resp.status);
            if let Some(body) = &resp.body {
                response.body().write(body.as_bytes());
            }
        };
        action.r#continue
    }
}
