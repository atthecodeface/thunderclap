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
        serde_json::to_value(s) // from_str(s)
    }
    fn value_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    fn is_array(&self) -> bool {
        self.is_array()
    }
    fn is_map(&self) -> bool {
        self.is_map()
    }
    fn len(&self) -> Option<usize> {
        if let Some(array) = self.as_array() {
            Some(array.len())
        } else if let Some(obj) = self.as_object() {
            Some(obj.len())
        } else {
            None
        }
    }
    fn index(&self, n: usize) -> Option<Self> {
        if !self.is_array() {
            None
        } else {
            self.get(n).cloned()
        }
    }
    fn key(&self, n: usize) -> Option<&str> {
        if let Some(obj) = self.as_object() {
            obj.keys().skip(n).next().map(|x| x.as_str())
        } else {
            None
        }
    }
    fn get(&self, _s: &str) -> Option<Self> {
        None
    }
}
