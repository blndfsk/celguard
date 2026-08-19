use std::sync::Arc;

use cel::{ResolveResult, Value, extractors::This, objects::Key};

/// Checks whether a given element exists within a map or list.
///
/// # Overloads
///
/// - **Map**: `has(map, key)` - Returns `true` if the map contains the specified key.
/// - **List**: `has(list, element)` - Returns `true` if the list contains the specified element.
///
/// # Examples
///
/// ```
/// // Map check
/// has({a: 1, b: 2}, "b")  // => true
/// has({a: 1, b: 2}, "c")  // => false
///
/// // List check
/// has([1, 2, 3], 2)       // => true
/// has([1, 2, 3], 4)       // => false
/// ```
pub(crate) fn has(This(this): This<Value>, elem: Arc<String>) -> ResolveResult {
    match this {
        Value::Map(map) => Ok(Value::Bool(map.get(&Key::from(elem)).is_some())),
        Value::List(list) => Ok(Value::Bool(list.contains(&Value::from(elem)))),
        _ => Err(cel::ExecutionError::NoSuchOverload),
    }
}
