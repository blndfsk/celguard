#[cfg(test)]
use std::sync::OnceLock;
use std::{collections::HashMap, fmt::Display, net::IpAddr, str::FromStr};

use anyhow::{Error, Result};
use cel::objects::Opaque;
use http_wasm_guest::host;
use serde::Serialize;

#[derive(Eq, Default, PartialEq, Serialize, Debug)]
pub(super) struct Request {
    path: String,
    method: String,
    version: String,
    header: HashMap<String, Vec<String>>,
    source_addr: String,
}
impl Opaque for Request {
    fn runtime_type_name(&self) -> &str {
        "request"
    }
}

///"GET /apache_pb.gif HTTP/1.0" curl/
impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{} {} {}\" ", self.method, self.path, self.version)?;
        match self.header.get("user-agent") {
            Some(ua) if !ua.is_empty() => {
                let mut sep = std::iter::once("");
                ua.iter().for_each(|elem| {
                    write!(f, "{}{}", sep.next().unwrap_or(", "), elem).unwrap_or_default();
                });
            }
            _ => write!(f, "-")?,
        }
        write!(f, " {}", self.source_addr)?;
        Ok(())
    }
}

impl TryFrom<&host::Request> for Request {
    type Error = Error;

    fn try_from(request: &host::Request) -> std::result::Result<Self, Self::Error> {
        Ok(Request {
            path: request.uri().into(),
            method: request.method().into(),
            version: request.version().into(),
            source_addr: parse_socket_addr(&request.source_addr())?.to_string(),
            header: map_header(&request.header),
        })
    }
}

fn map_header(header: &host::Header) -> HashMap<String, Vec<String>> {
    header
        .names_iter()
        .map(|name| {
            let val = header.values_iter(&name).map(|i| i.into()).collect::<Vec<_>>();
            let mut key: String = name.into();
            key.make_ascii_lowercase();
            (key, val)
        })
        .collect()
}
/// Parses a socket address from the request source address.
/// valid formats: `ipv4:port`, `[ipv6]:port`, `[ipv6%zone]:port`, `[ipv6]`
/// returns the addr-part as a string
fn parse_socket_addr(input: &[u8]) -> Result<IpAddr> {
    // bracketed form: `[ipv6]:port`, `[ipv6%zone]:port`, or `[ipv6]`
    let addr = if input.first() == Some(&b'[') {
        let (inner, _) =
            byte_split(&input[1..], b']').ok_or_else(|| Error::msg("right bracket missing"))?;
        match byte_split(inner, b'%') {
            Some((ip, zone)) if !zone.is_empty() => Ok(ip),
            Some((_, _)) => Err(Error::msg("zone missing")),
            None => Ok(inner),
        }
    } else {
        match byte_split(input, b':') {
            Some((ip, port)) if !port.is_empty() && port.iter().all(u8::is_ascii_digit) => Ok(ip),
            Some((_, _)) => Err(Error::msg("port missing")),
            None => Ok(input),
        }
    };
    IpAddr::from_str(str::from_utf8(addr?)?)
        .map_err(|e| Error::msg(format!("invalid ip address: {}", e)))
}

fn byte_split(slice: &[u8], delim: u8) -> Option<(&[u8], &[u8])> {
    slice.iter().position(|&b| b == delim).map(|i| (&slice[..i], &slice[i + 1..]))
}

#[cfg(test)]
impl Request {
    pub(super) fn get_request() -> &'static Request {
        static GET_REQUEST: OnceLock<Request> = OnceLock::new();
        GET_REQUEST.get_or_init(|| Request {
            path: "/".to_string(),
            method: "GET".to_string(),
            version: "HTTP/1.1".to_string(),
            header: HashMap::new(),
            source_addr: String::new(),
        })
    }

    pub(super) fn post_request() -> &'static Request {
        static POST_REQUEST: OnceLock<Request> = OnceLock::new();
        POST_REQUEST.get_or_init(|| Request {
            path: "/".to_string(),
            method: "POST".to_string(),
            version: "HTTP/1.1".to_string(),
            header: HashMap::new(),
            source_addr: String::new(),
        })
    }
}

#[cfg(test)]
mod tests {

    use testresult::TestResult;

    use super::*;

    #[test]
    fn test_display_with_user_agent() {
        let req = Request {
            path: "/foo/bar".to_string(),
            method: "GET".to_string(),
            version: "HTTP/1.1".to_string(),
            header: HashMap::from([("user-agent".to_string(), vec!["curl/8.0".to_string()])]),
            source_addr: "127.0.0.1".to_string(),
        };
        assert_eq!(format!("{}", req), "\"GET /foo/bar HTTP/1.1\" curl/8.0 127.0.0.1");
    }

    #[test]
    fn test_display_without_user_agent() {
        let req = Request {
            path: "/foo/bar".to_string(),
            method: "POST".to_string(),
            version: "HTTP/2.0".to_string(),
            ..Request::default()
        };
        assert_eq!(format!("{}", req), "\"POST /foo/bar HTTP/2.0\" - ");
    }

    #[test]
    fn test_display_with_empty_user_agent() {
        let req = Request {
            path: "/".to_string(),
            method: "GET".to_string(),
            version: "HTTP/1.0".to_string(),
            header: HashMap::from([("user-agent".to_string(), vec![])]),
            source_addr: "127.0.0.1:123".to_string(),
        };
        assert_eq!(format!("{}", req), "\"GET / HTTP/1.0\" - 127.0.0.1:123");
    }

    #[test]
    fn test_runtime_type_name() {
        let req = Request { ..Default::default() };
        assert_eq!(req.runtime_type_name(), "request");
    }

    #[test]
    fn test_parse_socket_addr() -> TestResult {
        assert!(parse_socket_addr(b"127.0.0.1:80").map(|a| a.to_string() == "127.0.0.1")?);
        assert!(parse_socket_addr(b"203.0.113.7:443").map(|a| a.to_string() == "203.0.113.7")?);
        assert!(parse_socket_addr(b"[::1]:443").map(|a| a.to_string() == "::1")?);
        assert!(parse_socket_addr(b"[2001:db8::1]:8080").map(|a| a.to_string() == "2001:db8::1")?);
        assert!(parse_socket_addr(b"[fe80::1%eth0]:8080").map(|a| a.to_string() == "fe80::1")?);
        assert!(parse_socket_addr(b"[::1]").map(|a| a.to_string() == "::1")?);
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
        assert!(parse_socket_addr(b"[::1%]:80").is_err()); // empty zone
    }
}
