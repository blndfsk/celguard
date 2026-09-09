use crate::{
    config::{plugin, rule::Action},
    matcher::{Matcher, Outcome},
};
use http_wasm_guest::{
    Guest, HostLogger, HostLoggerConfig,
    host::{Request, Response},
    register,
};
use std::collections::HashMap;

mod config;
mod matcher;

struct Plugin<'a> {
    config: plugin::Config,
    matcher: Matcher<'a>,
}

impl<'a> Guest for Plugin<'a> {
    fn handle_request(&self, request: &Request, response: &Response) -> (bool, i32) {
        match self.matcher.evaluate(request) {
            Ok(Outcome::Match(action)) => self.execute(action, response), //rule match with action
            Ok(Outcome::NoMatch) => (true, 0),                            //no match - continue
            Err(err) => {
                log::error!("Matcher: {}", err);
                (true, 0)
            }
        }
    }
}

impl<'a> Plugin<'a> {
    fn execute(&self, action: &Action, response: &Response) -> (bool, i32) {
        if let Some(resp) = action.response.as_ref() {
            self.write_header(response, &resp.header);
            self.write_status(response, &resp.status);
            self.write_body(response, &resp.body);
        }
        (action.r#continue, 0)
    }

    fn write_header(&self, response: &Response, header: &Option<HashMap<String, String>>) {
        if let Some(map) = header {
            for (key, value) in map {
                response.header.set(key.as_bytes(), value.as_bytes());
            }
        }
    }
    fn write_status(&self, response: &Response, status: &Option<i32>) {
        response.set_status(status.unwrap_or(self.config.default_status));
    }
    fn write_body(&self, response: &Response, body: &Option<String>) {
        if let Some(str) = body {
            response.body.write(str.as_bytes());
        }
    }
}

// order matters

fn main() {
    let _ =
        HostLogger::init_with_config(HostLoggerConfig { with_target: true, ..Default::default() });

    match config::read() {
        Ok(config) => {
            let plugin = Plugin { config: config.plugin, matcher: Matcher::new(config.matcher) };
            register(plugin);
        }
        Err(err) => log::error!(target: "celguard", "Config {}", err),
    }
}
