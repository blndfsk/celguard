use std::{collections::HashMap, fmt::Display};

use anyhow::{Error, Result};
use cel::objects::Opaque;
use http_wasm_guest::host;
use serde::Serialize;

#[derive(Eq, PartialEq, Serialize, Debug)]
pub(super) struct Request {
    path: String,
    method: String,
    version: String,
    header: HashMap<String, Vec<String>>,
}
impl Opaque for Request {
    fn runtime_type_name(&self) -> &str {
        "request"
    }
}

///"GET /apache_pb.gif HTTP/1.0" curl/
impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let user_agent = self.header.get("user-agent").map_or_else(
            || "-".to_string(),
            |v| {
                if v.is_empty() { "-".to_string() } else { v.join(", ") }
            },
        );
        write!(f, "\"{} {} {}\" {}", self.method, self.path, self.version, user_agent)
    }
}

impl Request {
    pub(super) fn try_from_host(request: &host::Request) -> Result<Self> {
        Self::try_from(request)
    }
    #[cfg(test)]
    pub(super) fn from_parts(
        path: &str,
        method: &str,
        version: &str,
        header: HashMap<String, Vec<String>>,
    ) -> Self {
        Request { path: path.into(), method: method.into(), version: version.into(), header }
    }
}

impl TryFrom<&host::Request> for Request {
    type Error = Error;

    fn try_from(request: &host::Request) -> std::result::Result<Self, Self::Error> {
        Ok(Request {
            path: request.uri().into(),
            method: request.method().into(),
            version: request.version().into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_with_user_agent() {
        let req = Request::from_parts(
            "/foo/bar",
            "GET",
            "HTTP/1.1",
            HashMap::from([("user-agent".to_string(), vec!["curl/8.0".to_string()])]),
        );
        assert_eq!(format!("{}", req), "\"GET /foo/bar HTTP/1.1\" curl/8.0");
    }

    #[test]
    fn test_display_without_user_agent() {
        let req = Request::from_parts("/foo/bar", "POST", "HTTP/2.0", HashMap::new());
        assert_eq!(format!("{}", req), "\"POST /foo/bar HTTP/2.0\" -");
    }

    #[test]
    fn test_display_with_empty_user_agent() {
        let req = Request::from_parts(
            "/",
            "GET",
            "HTTP/1.0",
            HashMap::from([("user-agent".to_string(), vec![])]),
        );
        assert_eq!(format!("{}", req), "\"GET / HTTP/1.0\" -");
    }

    #[test]
    fn test_runtime_type_name() {
        let req = Request::from_parts("/", "GET", "HTTP/1.1", HashMap::new());
        assert_eq!(req.runtime_type_name(), "request");
    }
}
