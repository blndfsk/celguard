use crate::{
    config::{plugin, rule::Rule},
    matcher::{Matcher, Outcome},
};
use http_wasm_guest::{Guest, HostLogger, HostLoggerConfig, host, register};

mod config;
mod matcher;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Plugin<'a> {
    config: plugin::Config,
    matcher: Matcher<'a>,
}

impl<'a> Guest for Plugin<'a> {
    fn handle_request(&self, request: &host::Request, response: &host::Response) -> (bool, i32) {
        match self.matcher.evaluate(request) {
            Ok(Outcome::Match(rule)) => self.execute(rule, response), //rule match
            Ok(Outcome::NoMatch) => (true, 0),                        //no match - continue
            Err(err) => {
                log::error!("Matcher: {}", err);
                (true, 0)
            }
        }
    }
}

impl<'a> Plugin<'a> {
    fn execute(&self, rule: &Rule, response: &host::Response) -> (bool, i32) {
        let action = match &rule.action {
            Some(anchor) => &anchor.0,
            None => &self.config.default_action,
        };

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
}

fn main() {
    let _ =
        HostLogger::init_with_config(HostLoggerConfig { with_target: true, ..Default::default() });

    match config::read() {
        Ok(config) => {
            let plugin = Plugin { config: config.plugin, matcher: Matcher::new(config.matcher) };
            register(plugin);
            log::info!("Started Version {}", VERSION);
        }
        Err(err) => {
            log::error!("Config {}", err);
            if err.source().is_some() {
                for line in err.root_cause().to_string().split("\\n") {
                    log::error!("{}", line);
                }
            }
        }
    }
}
