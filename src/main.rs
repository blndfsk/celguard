use std::collections::HashMap;

use anyhow::Error;
use http_wasm_guest::{
    Guest, HostLogger,
    host::{Request, Response},
    register,
};

use crate::{
    config::Action,
    matcher::{Matcher, Outcome},
};

mod config;
mod matcher;

struct Plugin<'a> {
    matcher: Matcher<'a>,
    actions: HashMap<String, Action>,
}

impl<'a> Guest for Plugin<'a> {
    fn handle_request(&self, request: &Request, response: &Response) -> (bool, i32) {
        match self.matcher.evaluate(request).unwrap_or_else(handle_err) {
            Outcome::Match(Some(action_name)) => (self.execute(action_name, response), 0),
            Outcome::Match(None) => (false, 0),
            Outcome::NoMatch => (true, 0),
        }
    }
}
impl<'a> Plugin<'a> {
    fn execute(&self, name: &str, response: &Response) -> bool {
        if let Some(action) = self.actions.get(name) {
            action.execute(response)
        } else {
            Action::default().execute(response)
        }
    }
}

fn handle_err<'a>(err: Error) -> Outcome<'a> {
    log::error!("Matcher: {}", err);
    Outcome::NoMatch
}

fn main() {
    let _ = HostLogger::init();

    match config::read() {
        Ok(config) => {
            let plugin = Plugin {
                matcher: Matcher::new(config.rules),
                actions: config.actions,
            };
            register(plugin);
        }
        Err(err) => log::error!(target: "celguard", "Config: {}", err),
    }
}
