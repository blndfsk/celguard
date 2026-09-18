use super::*;

pub fn get_request() -> Request {
    Request {
        path: "/".to_string().into(),
        method: "GET".to_string().into(),
        version: "HTTP/1.1".to_string().into(),
        headers: test_headers(&[("user-agent", &["curl/8.0"]), ("x-real-ip", &["1.1.1.1"])]),
        source_ip: "127.0.0.1".to_string().into(),
    }
}

pub fn post_request() -> Request {
    Request {
        path: "/".to_string().into(),
        method: "POST".to_string().into(),
        version: "HTTP/1.1".to_string().into(),
        headers: test_headers(&[("user-agent", &["curl/8.0"]), ("x-real-ip", &["1.1.1.1"])]),
        source_ip: "127.0.0.1".to_string().into(),
    }
}

/// Builds a CEL header value from (name, values) test pairs.
pub fn test_headers(pairs: &[(&str, &[&str])]) -> Map {
    let map = pairs
        .iter()
        .map(|(name, values)| {
            (
                Key::String(name.to_string().into()),
                match values.len() {
                    0 => Value::Null,
                    1 => Value::String(values[0].to_string().into()),
                    _ => Value::List(Arc::new(
                        values.iter().map(|v| Value::String(v.to_string().into())).collect(),
                    )),
                },
            )
        })
        .collect();
    Map { map: Arc::new(map) }
}
