use std::sync::Arc;

use anyhow::Result;
use cel::{Context, Program, Value, extractors::This};
use http_wasm_guest::host;
use log::log;

use crate::{config::Rule, matcher::request::Request};
mod request;

pub(crate) struct Matcher<'a> {
    context: Context<'a>,
    rules: Vec<Rule>,
}
#[derive(Debug, PartialEq)]
pub(crate) enum Outcome<'a> {
    Match(Option<&'a str>),
    NoMatch,
}

impl<'a> Matcher<'a> {
    pub(crate) fn new(rules: Vec<Rule>) -> Self {
        let mut context = cel::Context::default();
        context.add_function("to_lower", |This(s): This<Arc<String>>| s.to_lowercase());
        context.add_function("equals", |This(s): This<Arc<String>>, o: Arc<String>| {
            s.eq(&o)
        });

        Matcher { context, rules }
    }

    pub(crate) fn evaluate<'b>(&'a self, request: &'b host::Request) -> Result<Outcome<'a>> {
        let request = Request::try_from_host(request)?;
        self.eval(&request)
    }

    fn eval<'b>(&'a self, request: &'b Request) -> Result<Outcome<'a>> {
        let mut context = self.context.new_inner_scope();
        context.add_variable("request", &request)?;

        for rule in &self.rules {
            if !rule.disabled && rule.tests.iter().any(|program| is_match(program, &context)) {
                if let Some(level) = rule.log.to_level() {
                    log!(level, "{} => {}", rule.name, request);
                }
                return Ok(Outcome::Match(rule.action.as_deref()));
            }
        }
        Ok(Outcome::NoMatch)
    }
}

fn is_match(program: &Program, context: &Context) -> bool {
    match program.execute(context) {
        Ok(Value::Bool(b)) => b,
        Ok(_) => false, //wrong type
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

    use crate::config::Rule;

    use super::*;

    #[test]
    fn test_to_lower() {
        let matcher = Matcher::new(Vec::new());
        let program = Program::compile("to_lower('HeLLo') == 'hello'").unwrap();
        assert!(is_match(&program, &matcher.context));
    }

    #[test]
    fn test_request_header_wrong_use() {
        let req = Request::from_parts(
            "/foo/bar",
            "GET",
            "HTTP/1.1",
            HashMap::from([("user-agent".to_string(), "curl/123".to_string())]),
        );
        let m = Matcher::new(vec![Rule {
            name: "test".to_string(),
            disabled: false,
            log: log::LevelFilter::Off,
            tests: vec![
                Program::compile("request.header['user-agent'].all(h, h.matches('(?i)curl'))")
                    .unwrap(),
            ],
            action: None,
        }]);
        let out = m.eval(&req).unwrap();
        assert_eq!(Outcome::NoMatch, out);
    }

    #[test]
    fn test_request() -> TestResult {
        let req = Request::from_parts(
            "/foo/bar",
            "GET",
            "HTTP/1.1",
            HashMap::from([("user-agent".to_string(), "curl/123".to_string())]),
        );
        let m = Matcher::new(vec![Rule {
            name: "test".to_string(),
            disabled: false,
            log: log::LevelFilter::Off,
            tests: vec![Program::compile("request.method == 'GET'")?],
            action: None,
        }]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::Match(None), out);
        Ok(())
    }
}
