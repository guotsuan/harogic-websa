//! Capability reporting for external providers whose process adapters require Linux.
#[path = "event.rs"]
mod event;
#[path = "onair_modes.rs"]
mod onair_modes;
pub use event::ExternalDecodeEvent;

use serde_json::Value;

const UNAVAILABLE: &str = "External decoder providers are unavailable on Windows";

pub enum ExternalDecoder {}

impl ExternalDecoder {
    pub fn new(_mode: &str, _rate: u32, _center_hz: u64) -> Result<Self, String> {
        Err(UNAVAILABLE.into())
    }
    pub fn buffer_status(&self) -> Value {
        match *self {}
    }
    pub fn push_iq(&mut self, _iq: &[f32]) -> Result<Vec<ExternalDecodeEvent>, String> {
        match *self {}
    }
    pub fn retire(&mut self) -> Result<Vec<ExternalDecodeEvent>, String> {
        match *self {}
    }
    pub fn finish(&mut self) -> Result<Vec<ExternalDecodeEvent>, String> {
        match *self {}
    }
}

pub(crate) struct Dependency {
    pub modes: Vec<String>,
}
impl Dependency {
    pub(crate) fn unavailable_reason(&self) -> Option<String> {
        Some(UNAVAILABLE.into())
    }
}
pub(crate) fn multimon_dependency() -> &'static Dependency {
    static UNAVAILABLE: Dependency = Dependency { modes: Vec::new() };
    &UNAVAILABLE
}
pub(crate) use multimon_dependency as rtl433_dependency;

pub mod onair {
    pub use super::onair_modes::{
        band_limited, default_frequency, frequency_supported, validate_frequency, OnAirCapability,
    };

    pub fn capabilities() -> Vec<OnAirCapability> {
        super::onair_modes::MODES
            .iter()
            .map(|&(mode, rate, minimum, bandwidth)| OnAirCapability {
                mode: mode.into(),
                available: false,
                unavailable_reason: Some(super::UNAVAILABLE.into()),
                input_rate_hz: rate,
                minimum_rate_hz: minimum,
                bandwidth_hz: bandwidth,
            })
            .collect()
    }
}

pub(crate) fn unavailable_reason() -> Option<String> {
    Some(UNAVAILABLE.into())
}

pub mod rtty {
    pub(crate) use super::unavailable_reason;
}

pub mod gnss {
    pub(crate) use super::unavailable_reason;
}

pub mod dtmb {
    pub(crate) use super::unavailable_reason;

    pub async fn play(_id: String, _program: u16) -> axum::response::Response {
        use axum::{http::StatusCode, response::IntoResponse};
        (StatusCode::SERVICE_UNAVAILABLE, super::UNAVAILABLE).into_response()
    }
}
