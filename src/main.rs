use crate::{
    config::{Config, plugin, rule::Action},
    matcher::{Matcher, Outcome},
};
use http_wasm_guest::{Guest, HostLogger, HostLoggerConfig, host, register};
use log::{error, warn};

mod config;
mod matcher;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Plugin<'a> {
    config: plugin::Config,
    matcher: Matcher<'a>,
}

impl<'a> Plugin<'a> {
    fn new(cfg: Config) -> Self {
        Self { config: cfg.plugin, matcher: Matcher::new(cfg.matcher) }
    }
}

impl<'a> Guest for Plugin<'a> {
    fn handle_request(&self, request: &host::Request, response: &host::Response) -> (bool, i32) {
        match self.matcher.evaluate(request) {
            Ok(Outcome::Match(action)) => execute(action, response), //rule match
            Ok(Outcome::NoMatch) => (true, 0),                       //no match - continue
            Err(err) => {
                warn!("Matcher: {}", err);
                execute(&self.config.error_action, response)
            }
        }
    }
}

fn execute(action: &Action, response: &host::Response) -> (bool, i32) {
    if let Some(resp) = action.response.as_ref() {
        if let Some(map) = resp.header.as_ref() {
            for (key, value) in map {
                response.header.set(key.as_bytes(), value.as_bytes());
            }
        }
        response.set_status(resp.status);
        if let Some(str) = resp.body.as_ref() {
            response.body.write(str.as_bytes());
        }
    }
    (action.r#continue, 0)
}

fn main() {
    let _ =
        HostLogger::init_with_config(HostLoggerConfig { with_target: true, ..Default::default() });

    match config::read() {
        Ok(config) => {
            register(Plugin::new(config));
        }
        Err(err) => {
            register(Plugin::new(Config::default()));
            log::error!("{}, {}", VERSION, err);
            if err.source().is_some() {
                for line in err.root_cause().to_string().split("\\n") {
                    error!("{}", line);
                }
            }
        }
    }
}
