use wasm_bindgen::prelude::*;

use larptrix_protocol::ServerMessage;

use crate::ClientCore;

#[wasm_bindgen]
pub struct LarptrixClientCore {
    inner: ClientCore,
}

#[wasm_bindgen]
impl LarptrixClientCore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: ClientCore::new(),
        }
    }

    pub fn apply_server_message_json(&mut self, raw: &str) -> Result<(), JsValue> {
        let event: ServerMessage = serde_json::from_str(raw)
            .map_err(|error| JsValue::from_str(&format!("invalid server event: {error}")))?;
        self.inner.apply_server_message(event);
        Ok(())
    }

    pub fn snapshot_json(&self) -> Result<String, JsValue> {
        self.inner
            .snapshot_json()
            .map_err(|error| JsValue::from_str(&format!("could not serialize client state: {error}")))
    }

    pub fn take_platform_events_json(&mut self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner.take_platform_events())
            .map_err(|error| JsValue::from_str(&format!("could not serialize platform events: {error}")))
    }
}
