use http_wasm_guest::{
    Guest, HostLogger,
    host::{Request, Response},
    register,
};

use crate::matcher::{Matcher, Outcome};

mod config;
mod matcher;
mod model;

struct Plugin<'a> {
    matcher: Matcher<'a>,
}

impl<'a> Guest for Plugin<'a> {
    fn handle_request(&self, request: &Request, response: &Response) -> (bool, i32) {
        match self.matcher.evaluate(request) {
            Ok(Outcome::Match(action)) => action.execute(response), //rule match with action
            Ok(Outcome::NoMatch) => (true, 0),                      //no match - continue
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
            let plugin = Plugin { matcher: Matcher::new(config.rules) };
            register(plugin);
        }
        Err(err) => log::error!(target: "celguard", "Config: {}", err),
    }
}
