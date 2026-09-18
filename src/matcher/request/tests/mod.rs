pub mod mocks;
use super::tests::mocks::*;
use super::*;
use testresult::TestResult;

#[test]
fn test_request() {
    let req = Request {
        path: "/foo/bar".to_string().into(),
        method: "GET".to_string().into(),
        version: "HTTP/1.1".to_string().into(),
        headers: test_headers(&[("user-agent", &["curl/8.0"])]),
        source_ip: "127.0.0.1".to_string().into(),
    };
    assert_eq!(format!("{}", req), "127.0.0.1 \"GET /foo/bar HTTP/1.1\" \"curl/8.0\"");
}

#[test]
fn test_display_without_user_agent() {
    let req = Request {
        path: "/foo/bar".to_string().into(),
        method: "POST".to_string().into(),
        version: "HTTP/2.0".to_string().into(),
        headers: test_headers(&[]),
        source_ip: "127.0.0.1".to_string().into(),
    };
    assert_eq!(format!("{}", req), "127.0.0.1 \"POST /foo/bar HTTP/2.0\" -");
}

#[test]
fn test_display_with_empty_user_agent() {
    let req = Request {
        path: "/".to_string().into(),
        method: "GET".to_string().into(),
        version: "HTTP/1.0".to_string().into(),
        headers: test_headers(&[("user-agent", &[])]),
        source_ip: "127.0.0.1:123".to_string().into(),
    };
    assert_eq!(format!("{}", req), "127.0.0.1:123 \"GET / HTTP/1.0\" -");
}

#[test]
fn test_parse_socket_addr() -> TestResult {
    assert!(parse_socket_addr(b"127.0.0.1:80")? == "127.0.0.1");
    assert!(parse_socket_addr(b"203.0.113.7:443")? == "203.0.113.7");
    assert!(parse_socket_addr(b"[::1]:443")? == "::1");
    assert!(parse_socket_addr(b"[2001:db8::1]:8080")? == "2001:db8::1");
    assert!(parse_socket_addr(b"[fe80::1%eth0]:8080")? == "fe80::1");
    Ok(())
}

#[test]
fn test_parse_socket_addr_invalid() {
    assert!(parse_socket_addr(b"").is_err());
    assert!(parse_socket_addr(b"1.2.3:80").is_err());
    assert!(parse_socket_addr(b"1.2.3.4:").is_err());
    assert!(parse_socket_addr(b"1.2.3.4:1:2").is_err());
    assert!(parse_socket_addr(b"::1:80").is_err()); // unbracketed ipv6
    assert!(parse_socket_addr(b"[::1").is_err()); // missing ']'
    assert!(parse_socket_addr(b"[]:80").is_err());
    assert!(parse_socket_addr(b"[not-ipv6]:80").is_err());
    assert!(parse_socket_addr(b"foo%bar").is_err()); // '%' without ']'
    assert!(parse_socket_addr(b"]:80%zone").is_err()); // ']' before '%'
}
