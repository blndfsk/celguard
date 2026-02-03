use cel::{
    objects::{Key, Opaque},
    Context, Value,
};

use std::{
    collections::HashMap,
    fmt::{Debug, Formatter, Result as FmtResult},
    sync::Arc,
};

use crate::config::Config;

pub(crate) fn evaluate(
    config: &Config,
    request: &dyn http_wasm_guest::api::Request,
) -> (bool, i32) {
    let mut context = Context::default();
    context.add_variable_from_value("request", map_request(request));

    for rule in &config.rules {
        match rule.trigger.execute(&context) {
            Ok(val) => match val {
                Value::Bool(b) => {
                    if b {
                        log::warn!(
                            "{:?} {} for {:?}",
                            rule.action,
                            request.source_addr(),
                            rule.duration
                        );
                        return (true, 0);
                    }
                }
                _ => log::error!("wrong return type"),
            },
            Err(e) => log::error!("{}", e),
        }
    }
    (true, 0)
}

#[derive(Eq, PartialEq)]
struct Request {
    source_ip: String,
    header: HashMap<Key, Value>,
}
impl Opaque for Request {
    fn runtime_type_name(&self) -> &str {
        "request"
    }
}
impl Debug for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Request({}, {:?})", self.source_ip, self.header)
    }
}

fn map_request(request: &dyn http_wasm_guest::api::Request) -> Value {
    let header_map: HashMap<Key, Value> = request
        .header()
        .get()
        .iter()
        .map(|(key, value)| {
            (
                Key::String(Arc::new(key.to_string())),
                Value::List(Arc::new(
                    value
                        .iter()
                        .map(|i| Value::String(Arc::new(i.to_string())))
                        .collect(),
                )),
            )
        })
        .collect();
    let req = Request {
        source_ip: request.source_addr().to_string(),
        header: header_map,
    };

    Value::Opaque(Arc::new(req))
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use cel::{objects::Key, Value};
    use http_wasm_guest::api::{Body, Bytes, Header};
    use mockall::mock;

    use crate::engine::{map_request, Request};

    mock! {
        Header{}
            impl http_wasm_guest::api::Header for Header {
            fn names(&self) -> Vec<Bytes>;
            fn values(&self, name: &[u8]) -> Vec<Bytes>;
            fn set(&self, name: &[u8], value: &[u8]);
            fn add(&self, name: &[u8], value: &[u8]);
            fn remove(&self, name: &[u8]);
            fn get(&self) -> HashMap<Bytes, Vec<Bytes>>;
        }
    }
    mock! {
        Request {}
        impl http_wasm_guest::api::Request for Request {
            fn source_addr(&self) -> Bytes;
            fn version(&self) -> Bytes;
            fn method(&self) -> Bytes;
            fn set_method(&self, method: &[u8]);
            fn uri(&self) -> Bytes;
            fn set_uri(&self, uri: &[u8]);
            fn header(&self) -> &dyn Header;
            fn body(&self) -> &dyn Body;
        }
    }

    #[test]
    fn test() {
        let header_map = HashMap::from([(Bytes::from("FOO"), vec![Bytes::from("bar")])]);
        let mut header = MockHeader::new();
        header.expect_get().return_const(header_map);
        let mut request = MockRequest::new();
        request.expect_header().return_const(Box::new(header));
        request
            .expect_source_addr()
            .return_const(Bytes::from("test"));

        let res = map_request(&request);
        assert_eq!(
            Value::Opaque(Arc::new(Request {
                source_ip: "test".to_string(),
                header: HashMap::from([(
                    Key::String(Arc::new("FOO".to_string())),
                    Value::List(Arc::new(vec!(Value::String(Arc::new("bar".to_string())))))
                )])
            })),
            res
        )
    }
}
