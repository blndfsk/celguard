use std::sync::Arc;

use anyhow::Result;
use cel::{Context, Program, Value, extractors::This};
use http_wasm_guest::host;
use log::log;

use crate::{config::Rule, matcher::request::Request};
mod request;

pub struct Matcher<'a> {
    context: Context<'a>,
    pub rules: Vec<Rule>,
}
#[derive(Debug, PartialEq)]
pub enum Outcome<'a> {
    Match(Option<&'a String>),
    NoMatch,
}

impl<'a> Default for Matcher<'a> {
    fn default() -> Self {
        let mut default = cel::Context::default();
        default.add_function("to_lower", to_lower);
        Self {
            context: default,
            rules: Vec::default(),
        }
    }
}

impl<'a> Matcher<'a> {
    pub fn new(rules: Vec<Rule>) -> Self {
        Matcher {
            rules,
            ..Default::default()
        }
    }

    pub fn evaluate<'b>(&'a self, request: &'b host::Request) -> Result<Outcome<'a>> {
        let request = Request::try_from_host(request)?;
        self.eval(&request)
    }

    fn eval<'b>(&'a self, request: &'b Request) -> Result<Outcome<'a>> {
        let mut context = self.context.new_inner_scope();
        context.add_variable("request", &request)?;

        for rule in &self.rules {
            if rule.tests.iter().any(|program| execute(&context, program)) {
                if let Some(level) = rule.log.to_level() {
                    log!(level, "{} => {}", rule.name, request);
                }
                return Ok(Outcome::Match(rule.action.as_ref()));
            }
        }
        Ok(Outcome::NoMatch)
    }
}

fn to_lower(This(s): This<Arc<String>>) -> String {
    s.to_lowercase()
}

fn execute(context: &Context, program: &Program) -> bool {
    match program.execute(context) {
        Ok(val) => match val {
            Value::Bool(b) => b,
            _ => false, //wrong type
        },
        Err(e) => {
            log::error!("{}", e);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use testresult::TestResult;

    use super::*;

    #[test]
    fn test_to_lower() {
        let matcher = Matcher::new(Vec::default());
        let program = Program::compile("to_lower('HeLLo') == 'hello'").unwrap();
        assert!(execute(&matcher.context, &program));
    }
    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_request_header_panics() {
        // request.header['user-agent'] returns a String. The cel crate panics when
        // .all() is called on a String, because it tries to iterate over its characters
        // and hits an unimplemented code path converting them to cel objects.
        let req = Request::from_parts(
            "192.168.1.1",
            "/foo/bar",
            "GET",
            "HTTP/1.1",
            HashMap::from([("user-agent".to_string(), "curl/123".to_string())]),
        );
        let m = Matcher::new(vec![Rule {
            name: "test".to_string(),
            log: log::LevelFilter::Off,
            tests: vec![
                Program::compile("request.header['user-agent'].all(h, h.matches('(?i)curl'))")
                    .unwrap(),
            ],
            action: None,
        }]);
        let out = m.eval(&req).unwrap();
        assert_eq!(Outcome::Match(None), out);
    }

    #[test]
    fn test_request() -> TestResult {
        let req = Request::from_parts(
            "192.168.1.1",
            "/foo/bar",
            "GET",
            "HTTP/1.1",
            HashMap::from([("user-agent".to_string(), "curl/123".to_string())]),
        );
        let m = Matcher::new(vec![Rule {
            name: "test".to_string(),
            log: log::LevelFilter::Off,
            tests: vec![Program::compile("request.method == 'GET'")?],
            action: None,
        }]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::Match(None), out);
        Ok(())
    }
}
