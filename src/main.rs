use http_wasm_guest::{host, register, Guest, Request, Response};
use log::Level;

mod config;
mod engine;

use config::Config;

struct Plugin {
    config: Config,
}

impl Guest for Plugin {
    fn handle_request(&self, request: Request, _response: Response) -> (bool, i32) {
        // test source_ip against the stored ips in jail
        // response.set_status(403);
        // return (false, 0);

        engine::evaluate(&self.config, request.as_ref())
    }
    fn handle_response(&self, _request: Request, _response: Response) {
        // create context with request and response variables
    }
}

fn main() {
    host::log::init_with_level(Level::Debug).expect("no logging");
    let config = config::read().expect("no valid config");
    let plugin = Plugin { config };
    register(plugin);
}
