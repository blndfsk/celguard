use cel::{ExecutionError, ResolveResult, Value, extractors::This, objects::KeyRef};

/// Returns the first element of a list, or `null` if the list is empty.
///
/// Registered as the CEL function `get_first` so rules can select a single value,
/// e.g. `request.header['x-real-ip'].get_first()` returns the first value of the
/// `x-real-ip` header, or `null` if it is not present.
///
/// Fails with `NoSuchOverload` if the receiver is not a list.
pub fn get_first(This(this): This<Value>) -> ResolveResult {
    Ok(match this {
        Value::List(v) => Value::from(v.first().unwrap_or_else(|| &Value::Null)),
        _ => return Err(cel::ExecutionError::NoSuchOverload),
    })
}

#[allow(dead_code)]
pub fn get(This(this): This<Value>, arg: Value) -> ResolveResult {
    Ok(match this {
        Value::Map(v) => Value::from(
            v.get(&KeyRef::try_from(&arg).map_err(ExecutionError::UnsupportedKeyType)?)
                .unwrap_or_else(|| &Value::Null),
        ),
        _ => return Err(cel::ExecutionError::NoSuchOverload),
    })
}
