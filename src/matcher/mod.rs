use crate::{
    config::{matcher::Config, rule::Action},
    matcher::request::Request,
};
use anyhow::{Error, Result};
use cel::{Context, Program, Value, extractors::This};
use http_wasm_guest::host;
use log::log;
use std::sync::Arc;

mod functions;
pub mod request;

pub(crate) struct Matcher<'a> {
    context: Context<'a>,
    config: Config,
}

#[derive(Debug)]
pub(crate) enum Outcome<'a> {
    Match(&'a Action),
    NoMatch,
}

impl<'a> Matcher<'a> {
    pub(crate) fn new(config: Config) -> Self {
        let mut context = cel::Context::default();
        context.add_function("contains", cel::functions::contains);
        context.add_function("lower", |This(s): This<Arc<String>>| s.to_lowercase());
        context.add_function("trim", |This(s): This<Arc<String>>| s.trim().to_string());
        Matcher { context, config }
    }

    pub(crate) fn evaluate(&self, host_request: &host::Request) -> Result<Outcome<'_>> {
        let mut request = Request::try_from(host_request)?;
        if let Some(ip) = self.real_ip(&request)? {
            request.source_ip = ip;
        }
        self.eval(&request)
    }

    fn eval(&self, request: &Request) -> Result<Outcome<'_>> {
        let mut context = self.context.new_inner_scope();
        context.add_variable_from_value("request", request.value());

        for rule in &self.config.rules {
            if !rule.disabled
                && (rule.tests.is_empty() || rule.tests.iter().any(|p| is_match(p, &context)))
            {
                let action =
                    rule.action.as_ref().map_or(&self.config.default_action, |a| a.0.as_ref());

                if let Some(level) = rule.log.to_level() {
                    log!(level, "{} => {}", rule.name, request);
                }

                return Ok(Outcome::Match(action));
            }
        }
        Ok(Outcome::NoMatch)
    }
    fn real_ip(&self, request: &Request) -> Result<Option<Arc<String>>> {
        match &self.config.source_ip {
            Some(source_ip) => {
                let mut context = self.context.new_inner_scope();
                context.add_variable_from_value("request", request.value());
                match source_ip.execute(&context) {
                    Ok(Value::String(s)) => Ok(Some(s)),
                    Ok(val) => anyhow::bail!("source_ip must return string: {:?}", val),
                    Err(e) => Err(Error::from(e)),
                }
            }
            _ => Ok(None),
        }
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
    use crate::config::rule::Rule;
    use std::assert_matches;
    use testresult::TestResult;

    #[test]
    fn test_function_lower() {
        let matcher = Matcher::new(Config::default());
        let program = Program::compile("lower('HeLLo') == 'hello'").unwrap();
        assert!(is_match(&program, &matcher.context));
    }

    #[test]
    fn test_disabled_rule_is_skipped() -> TestResult {
        let req = Request::get_request();
        let m = Matcher::new(Config {
            rules: vec![Rule {
                name: "rule1".to_string(),
                tests: vec![],
                action: None,
                disabled: true,
                log: log::LevelFilter::Off,
            }],
            ..Config::default()
        });
        let out = m.eval(&req)?;
        assert_matches!(out, Outcome::NoMatch);
        Ok(())
    }

    #[test]
    fn test_first_matching_rule_wins() -> TestResult {
        let req = Request::get_request();
        let m = Matcher::new(Config {
            rules: vec![
                Rule {
                    name: "rule1".to_string(),
                    tests: vec![Program::compile("request.method == 'GET'")?],
                    ..Default::default()
                },
                Rule {
                    name: "rule2".to_string(),
                    tests: vec![Program::compile("request.method == 'GET'")?],
                    ..Default::default()
                },
            ],
            ..Config::default()
        });
        let out = m.eval(&req)?;
        assert_matches!(out, Outcome::Match(a) if a == &m.config.default_action);
        Ok(())
    }

    #[test]
    fn test_no_rules_returns_no_match() -> TestResult {
        let req = Request::get_request();
        let m = Matcher::new(Config::default());
        let out = m.eval(&req)?;
        assert_matches!(out, Outcome::NoMatch);
        Ok(())
    }

    #[test]
    fn test_non_matching_rule() -> TestResult {
        let req = Request::post_request();
        let m = Matcher::new(Config {
            rules: vec![Rule {
                name: "rule1".to_string(),
                tests: vec![Program::compile("request.method == 'GET'")?],
                ..Default::default()
            }],
            ..Config::default()
        });
        let out = m.eval(&req)?;
        assert_matches!(out, Outcome::NoMatch);
        Ok(())
    }

    #[test]
    fn test_header_rule_matches() -> TestResult {
        let req = Request::get_request();
        let m = Matcher::new(Config {
            rules: vec![Rule {
                name: "rule1".to_string(),
                tests: vec![Program::compile("request.headers.contains('user-agent')")?],
                ..Default::default()
            }],
            ..Config::default()
        });
        let out = m.eval(&req)?;

        assert_matches!(out, Outcome::Match(a) if a == &m.config.default_action);
        Ok(())
    }
    #[test]
    fn test_source_ip() -> TestResult {
        let req = Request::get_request();

        let m = Matcher::new(Config {
            source_ip: Some(Program::compile("request.headers['x-real-ip']")?),
            ..Default::default()
        });
        let ip = m.real_ip(&req)?;
        assert_eq!(ip.unwrap(), "1.1.1.1".to_string().into());
        Ok(())
    }
    #[test]
    fn test_non_bool_expression_is_no_match() {
        let matcher = Matcher::new(Config::default());
        let program = Program::compile("'hello'").unwrap();
        assert!(!is_match(&program, &matcher.context));
    }
}
