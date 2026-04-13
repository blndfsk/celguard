use anyhow::Error;
use http_wasm_guest::{
    Guest, HostLogger,
    host::{Request, Response},
    register,
};

use crate::{
    handler::Handler,
    matcher::{Matcher, Outcome},
};

mod config;
mod handler;
mod matcher;

struct Plugin<'a> {
    matcher: Matcher<'a>,
    handler: Handler,
}

impl<'a> Guest for Plugin<'a> {
    fn handle_request(&self, request: &Request, response: &Response) -> (bool, i32) {
        match self.matcher.evaluate(request).unwrap_or_else(handle_err) {
            Outcome::Match(Some(action)) => (self.handler.execute(action, response), 0),
            Outcome::Match(None) => (false, 0),
            Outcome::NoMatch => (true, 0),
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
                handler: Handler::new(config.actions),
            };
            register(plugin);
        }
        Err(err) => log::error!(target: "celguard", "Config: {}", err),
    }
}
