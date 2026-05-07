fn has(This(this): This<Value>, key: Arc<String>) -> ResolveResult {
    match this {
        Value::Map(v) => Ok(Value::Bool(v.get(&Key::from(key)).is_some())),
        _ => Err(cel::ExecutionError::NoSuchOverload),
    }
}
