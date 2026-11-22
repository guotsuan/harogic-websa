use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct ExternalDecodeEvent {
    pub provider: String,
    pub mode: String,
    pub kind: String,
    pub data: Value,
}
impl ExternalDecodeEvent {
    pub fn into_value(self) -> Value {
        json!({"type":"decoder-result","provider":self.provider,"mode":self.mode,
            "kind":self.kind,"data":self.data})
    }
}
