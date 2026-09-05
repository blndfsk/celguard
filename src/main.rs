use std::collections::HashMap;

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
}

impl<'a> Guest for Plugin<'a> {
    fn handle_request(&self, request: &Request, response: &Response) -> (bool, i32) {
        match self.matcher.evaluate(request) {
            Ok(Outcome::Match(action)) => execute(action, response), //rule match with action
            Ok(Outcome::NoMatch) => (true, 0),                       //no match - continue
            Err(err) => {
                log::error!("Matcher: {}", err);
                (true, 0)
            }
        }
    }
}

// order matters
fn execute(action: &Action, response: &Response) -> (bool, i32) {
    if let Some(resp) = action.response.as_ref() {
        write_header(response, resp.header.as_ref());
        write_status(response, resp.status.as_ref());
        write_body(response, resp.body.as_ref());
    }
    (action.r#continue, 0)
}

fn write_header(response: &Response, header: Option<&HashMap<String, String>>) {
    if let Some(map) = header {
        for (key, value) in map {
            response.header.set(key.as_bytes(), value.as_bytes());
        }
    }
}
fn write_status(response: &Response, status: Option<&i32>) {
    match status {
        Some(value) => response.set_status(*value),
        None => response.set_status(403), //TODO read from config
    };
}
fn write_body(response: &Response, body: Option<&String>) {
    if let Some(str) = body {
        response.body.write(str.as_bytes());
    }
}

fn main() {
    let _ = HostLogger::init();

    match config::read() {
        Ok(config) => {
            let plugin = Plugin { matcher: Matcher::new(config) };
            register(plugin);
        }
        Err(err) => log::error!(target: "celguard", "Config: {}", err),
    }
}
