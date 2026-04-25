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
        match self.matcher.evaluate(request) {
            Ok(Outcome::Match(Some(action))) => (self.handler.execute(action, response), 0),
            Ok(Outcome::Match(None)) => (false, 0),
            Ok(Outcome::NoMatch) => (true, 0),
            Err(err) => {
                log::error!("Matcher: {}", err);
                (true, 0)
            }
        }
    }
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
