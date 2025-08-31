use crate::CommandArgsValue;

pub use serde_json::from_value;
pub use serde_json::to_value;
pub use serde_json::Value;
///
/// JsonValue implements
///
/// JsonValue::to_string()
///
/// pub fn from_value<T>(value: Value) -> Result<T, Error>
///    where T: DeserializeOwned,
///
/// pub fn to_value<T>(value: T) -> Result<Value, Error>
/// where T: Serialize,

//ip CommandArgsValue for Value
impl CommandArgsValue for Value {
    const CAN_INDEX: bool = true;
    const CAN_GET: bool = true;
    type FromStrError = serde_json::Error;
    fn is_none(&self) -> bool {
        self == &serde_json::Value::Null
    }
    fn from_str(s: &str) -> Result<Self, Self::FromStrError> {
        dbg!(s);
        serde_json::to_value(s) // from_str(s)
    }
    fn value_string(&self) -> String {
        dbg!(self);
        serde_json::to_string(self).unwrap()
    }
    fn index(&self, n: usize) -> Option<Self> {
        if !self.is_array() {
            None
        } else {
            self.get(n).cloned()
        }
    }
    fn get(&self, _s: &str) -> Option<Self> {
        None
    }
}
