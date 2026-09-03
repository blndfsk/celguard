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
        context.add_function("trim", |This(s): This<Arc<String>>| s.trim().to_string());
        context.add_function("has", function::has);
        Matcher { context, rules }
    }

    pub(crate) fn evaluate(&self, request: &host::Request) -> Result<Outcome<'_>> {
        let request = Request::from(request);
        self.eval(&request)
    }

    fn eval(&self, request: &Request) -> Result<Outcome<'_>> {
        let mut context = self.context.new_inner_scope();
        context.add_variable_from_value("request", request.value());

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
    use std::{ptr, rc::Rc};
    use testresult::TestResult;

    #[test]
    fn test_to_lower() {
        let matcher = Matcher::new(Vec::new());
        let program = Program::compile("to_lower('HeLLo') == 'hello'").unwrap();
        assert!(is_match(&program, &matcher.context));
    }

    #[test]
    fn test_disabled_rule_is_skipped() -> TestResult {
        let req = Request::get_request();
        let m = Matcher::new(vec![Rule { disabled: true, ..Rule::default() }]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::NoMatch, out);
        Ok(())
    }

    #[test]
    fn test_first_matching_rule_wins() -> TestResult {
        let req = Request::get_request();
        let action1 = RcAnchor::from(Rc::from(Action { response: None, r#continue: false }));
        let action2 = RcAnchor::from(Rc::from(Action { response: None, r#continue: true }));
        let m = Matcher::new(vec![
            Rule {
                tests: vec![Program::compile("request.method == 'GET'")?],
                action: Some(RcAnchor::from(action1.clone())),
                ..Rule::default()
            },
            Rule {
                tests: vec![Program::compile("request.method == 'GET'")?],
                action: Some(RcAnchor::from(action2.clone())),
                ..Rule::default()
            },
        ]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::Match(&action1), out);
        Ok(())
    }

    #[test]
    fn test_no_rules_returns_no_match() -> TestResult {
        let req = Request::get_request();
        let m = Matcher::new(vec![]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::NoMatch, out);
        Ok(())
    }

    #[test]
    fn test_match_returns_action() -> TestResult {
        let req = Request::get_request();
        let action = RcAnchor::from(Rc::from(Action::default()));
        let m = Matcher::new(vec![Rule {
            tests: vec![Program::compile("request.method == 'GET'")?],
            action: Some(RcAnchor::from(action.clone())),
            ..Rule::default()
        }]);

        let out = m.eval(&req)?;
        assert_eq!(Outcome::Match(&action), out);
        Ok(())
    }

    #[test]
    fn test_non_matching_rule() -> TestResult {
        let req = Request::post_request();
        let m = Matcher::new(vec![Rule {
            tests: vec![Program::compile("request.method == 'GET'")?],
            ..Rule::default()
        }]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::NoMatch, out);
        Ok(())
    }

    #[test]
    fn test_matching_rule_without_action() -> TestResult {
        let req = Request::get_request();
        let m = Matcher::new(vec![Rule {
            tests: vec![Program::compile("request.method == 'GET'")?],
            ..Rule::default()
        }]);
        let out = m.eval(&req)?;
        let Outcome::Match(action) = out else { panic!() };
        assert_eq!(Action::default_action(), action);
        assert!(ptr::addr_eq(Action::default_action(), action));
        assert!(action.response.is_some());
        Ok(())
    }

    #[test]
    fn test_rule_without_tests_and_action() -> TestResult {
        let req = Request::get_request();
        let m = Matcher::new(vec![Rule { ..Default::default() }]);
        let out = m.eval(&req)?;
        let Outcome::Match(action) = out else { panic!() };
        assert_eq!(Action::default_action(), action);
        assert!(ptr::addr_eq(Action::default_action(), action));
        assert!(action.response.is_some());
        Ok(())
    }

    #[test]
    fn test_header_rule_matches() -> TestResult {
        let req = Request::user_agent_request();
        let action = RcAnchor::from(Rc::from(Action::default()));
        let m = Matcher::new(vec![Rule {
            tests: vec![Program::compile("request.header.has('user-agent')")?],
            action: Some(RcAnchor::from(action.clone())),
            ..Rule::default()
        }]);
        let out = m.eval(&req)?;
        assert_eq!(Outcome::Match(&action), out);
        Ok(())
    }

    #[test]
    fn test_non_bool_expression_is_no_match() {
        let matcher = Matcher::new(Vec::new());
        let program = Program::compile("'hello'").unwrap();
        assert!(!is_match(&program, &matcher.context));
    }
}
