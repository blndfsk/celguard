use anyhow::Error;
use http_wasm_guest::{Guest, HostLogger, host, register};
use log::Level;
use std::panic;

use crate::{
    agent::Agent,
    matcher::{Matcher, Outcome},
};

mod agent;
mod config;
mod matcher;

struct CelGuard<'a> {
    matcher: Matcher<'a>,
    agent: Agent,
}

impl<'a> Guest for CelGuard<'a> {
    fn handle_request(&self, request: &host::Request, response: &host::Response) -> (bool, i32) {
        match self.matcher.evaluate(request).unwrap_or_else(handle_err) {
            Outcome::Match(Some(action)) => (self.agent.perform(action, response), 0),
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
    panic::set_hook(Box::new(|info| {
        log::error!(target: "panic", "{}", info);
    }));

    let _ = HostLogger::init_with_level(Level::Debug);

    match config::read() {
        Ok(config) => {
            let celguard = CelGuard {
                matcher: Matcher::new(config.rules),
                agent: Agent::new(config.actions),
            };
            register(celguard);
        }
        Err(err) => log::error!(target: "celguard", "Config: {}", err),
    }
}
