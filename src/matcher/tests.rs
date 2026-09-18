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
    let req = request::tests::mocks::get_request();
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
    let req = request::tests::mocks::get_request();
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
fn test_no_rules_returns_err() -> TestResult {
    let req = request::tests::mocks::get_request();
    let m = Matcher::new(Config::default());
    let out = m.eval(&req);
    assert!(out.is_err());
    Ok(())
}

#[test]
fn test_non_matching_rule() -> TestResult {
    let req = request::tests::mocks::post_request();
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
    let req = request::tests::mocks::get_request();
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
    let req = request::tests::mocks::get_request();

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
