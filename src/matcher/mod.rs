use crate::{
    matcher::request::Request,
    model::{Action, Rule},
};
use anyhow::Result;
use cel::{Context, Program, Value, extractors::This};
use http_wasm_guest::host;
use log::log;
use std::sync::Arc;

mod function;
mod request;

pub(crate) struct Matcher<'a> {
    context: Context<'a>,
    rules: Vec<Rule>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Outcome<'a> {
    Match(&'a Action),
    NoMatch,
}

impl<'a> Matcher<'a> {
    pub(crate) fn new(rules: Vec<Rule>) -> Self {
        let mut context = cel::Context::default();
        context.add_function("to_lower", |This(s): This<Arc<String>>| s.to_lowercase());
        context.add_function("equals", |This(s): This<Value>, o: Value| s.eq(&o));
        context.add_function("has", function::has);

        Matcher { context, rules }
    }

    pub(crate) fn evaluate(&self, request: &host::Request) -> Result<Outcome<'_>> {
        let request = Request::try_from_host(request)?;
        self.eval(&request)
    }

    fn eval(&self, request: &Request) -> Result<Outcome<'_>> {
        let mut context = self.context.new_inner_scope();
        context.add_variable("request", &request)?;

        for rule in &self.rules {
            if !rule.disabled
                && (rule.tests.is_empty() || rule.tests.iter().any(|p| is_match(p, &context)))
            {
                if let Some(level) = rule.log.to_level() {
                    log!(level, "{} => {}", rule.name, request);
                }
                return Ok(Outcome::Match({
                    match &rule.action {
                        Some(anchor) => &anchor.0,
                        None => Action::default_action(),
                    }
                }));
            }
        }
        Ok(Outcome::NoMatch)
    }
}

fn is_match(program: &Program, context: &Context) -> bool {
    match program.execute(context) {
        Ok(Value::Bool(b)) => b,
        Ok(val) => {
            log::warn!("program must return bool: {:?}", val);
            false
        } //wrong type
        Err(e) => {
            log::error!("{}", e);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_saphyr::RcAnchor;
    use std::{collections::HashMap, ptr, rc::Rc};
    use testresult::TestResult;

    #[test]
    fn test_to_lower() {
        let matcher = Matcher::new(Vec::new());
        let program = Program::compile("to_lower('HeLLo') == 'hello'").unwrap();
        assert!(is_match(&program, &matcher.context));
    }

    #[test]
    fn test_request_header_match() -> TestResult {
        let req = Request::from_parts(
            "/foo/bar",
            "GET",
            "HTTP/1.1",
            HashMap::from([("user-agent".to_string(), vec!["Curl/123".to_string()])]),
        );
        let m = Matcher::new(vec![Rule::from_parts(
            "test",
            false,
            log::LevelFilter::Off,
            vec!["request.header['user-agent'].contains((?i)'curl/123')"],
            None,
        )]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::Match(Action::default_action()), out);
        Ok(())
    }

    #[test]
    fn test_request() -> TestResult {
        let req = Request::from_parts(
            "/foo/bar",
            "GET",
            "HTTP/1.1",
            HashMap::from([("user-agent".to_string(), vec!["curl/123".to_string()])]),
        );
        let m = Matcher::new(vec![Rule::from_parts(
            "test",
            false,
            log::LevelFilter::Off,
            vec!["request.method == 'GET'"],
            None,
        )]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::Match(Action::default_action()), out);
        Ok(())
    }

    #[test]
    fn test_disabled_rule_is_skipped() -> TestResult {
        let req = Request::from_parts("/foo", "GET", "HTTP/1.1", HashMap::new());
        let m = Matcher::new(vec![Rule::from_parts(
            "test_disabled",
            true,
            log::LevelFilter::Off,
            vec!["request.method == 'GET'"],
            None,
        )]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::NoMatch, out);
        Ok(())
    }

    #[test]
    fn test_first_matching_rule_wins() -> TestResult {
        let req = Request::from_parts("/foo", "GET", "HTTP/1.1", HashMap::new());
        let action1 = RcAnchor::from(Rc::from(Action { response: None, r#continue: true }));
        let action2 = RcAnchor::from(Rc::from(Action { response: None, r#continue: false }));
        let m = Matcher::new(vec![
            Rule::from_parts(
                "first",
                false,
                log::LevelFilter::Off,
                vec!["request.method == 'GET'"],
                Some(RcAnchor::from(action1.clone())),
            ),
            Rule::from_parts(
                "second",
                false,
                log::LevelFilter::Off,
                vec!["request.method == 'GET'"],
                Some(RcAnchor::from(action2.clone())),
            ),
        ]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::Match(&action1), out);
        Ok(())
    }

    #[test]
    fn test_no_rules_returns_no_match() -> TestResult {
        let req = Request::from_parts("/foo", "GET", "HTTP/1.1", HashMap::new());
        let m = Matcher::new(vec![]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::NoMatch, out);
        Ok(())
    }

    #[test]
    fn test_match_returns_action() -> TestResult {
        let req = Request::from_parts("/foo", "GET", "HTTP/1.1", HashMap::new());
        let action = RcAnchor::from(Rc::from(Action::default()));
        let m = Matcher::new(vec![Rule::from_parts(
            "test",
            false,
            log::LevelFilter::Off,
            vec!["request.method == 'GET'"],
            Some(RcAnchor::from(action.clone())),
        )]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::Match(&action), out);
        Ok(())
    }

    #[test]
    fn test_non_matching_rule() -> TestResult {
        let req = Request::from_parts("/foo", "POST", "HTTP/1.1", HashMap::new());
        let m = Matcher::new(vec![Rule::from_parts(
            "get_only",
            false,
            log::LevelFilter::Off,
            vec!["request.method == 'GET'"],
            None,
        )]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::NoMatch, out);
        Ok(())
    }

    #[test]
    fn test_matching_rule_without_action() -> TestResult {
        let req = Request::from_parts("/foo", "GET", "HTTP/1.1", HashMap::new());
        let m = Matcher::new(vec![Rule::from_parts(
            "get_only",
            false,
            log::LevelFilter::Off,
            vec!["request.method == 'GET'"],
            None,
        )]);
        let out = m.eval(&req)?;
        let Outcome::Match(action) = out else { panic!() };
        assert_eq!(Action::default_action(), action);
        assert!(ptr::addr_eq(Action::default_action(), action));
        assert!(action.response.is_none());
        Ok(())
    }

    #[test]
    fn test_non_bool_expression_is_no_match() {
        let matcher = Matcher::new(Vec::new());
        let program = Program::compile("'hello'").unwrap();
        assert!(!is_match(&program, &matcher.context));
    }

    #[test]
    fn test_multiple_tests_any_matches() -> TestResult {
        let req = Request::from_parts("/foo", "POST", "HTTP/1.1", HashMap::new());
        let m = Matcher::new(vec![Rule::from_parts(
            "get_only",
            false,
            log::LevelFilter::Off,
            vec!["request.method == 'GET'", "request.method == 'POST'"],
            None,
        )]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::Match(Action::default_action()), out);
        Ok(())
    }

    #[test]
    fn test_path_matching() -> TestResult {
        let req = Request::from_parts("/api/v1/users", "GET", "HTTP/1.1", HashMap::new());
        let m = Matcher::new(vec![Rule::from_parts(
            "get_only",
            false,
            log::LevelFilter::Off,
            vec!["request.path.matches('^/api')"],
            None,
        )]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::Match(Action::default_action()), out);
        Ok(())
    }
    #[test]
    fn test_invalid_program() -> TestResult {
        let req = Request::from_parts("/api/v1/users", "GET", "HTTP/1.1", HashMap::new());
        let m = Matcher::new(vec![Rule::from_parts(
            "returns_wrong_type",
            false,
            log::LevelFilter::Off,
            vec!["request.path"],
            None,
        )]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::NoMatch, out);
        Ok(())
    }
}
