//! Preserve the server-mode API while reporting the Linux Soapy adapter as unavailable.
pub(super) use super::server_bind::parse_bind;
use serde_json::{json, Value};
use std::net::{IpAddr, Ipv4Addr};

const UNAVAILABLE: &str = "SoapySDRServer handoff is unavailable on Windows";

pub(super) struct SoapyServer {
    bind: (IpAddr, u16),
    graph_mode: String,
}

impl Default for SoapyServer {
    fn default() -> Self {
        Self {
            bind: (Ipv4Addr::LOCALHOST.into(), 55132),
            graph_mode: "std".into(),
        }
    }
}

impl SoapyServer {
    pub(super) fn snapshot(&mut self) -> Value {
        json!({"state":"stopped", "bind_address":self.bind.0.to_string(), "port":self.bind.1,
            "pid":null, "uptime_seconds":0, "exit_code":null, "error":null, "logs":[],
            "log_capacity":200, "driver_available":false, "dependency_error":UNAVAILABLE,
            "graph_mode":self.graph_mode})
    }

    pub(super) fn remember_graph(&mut self, mode: &str) {
        if matches!(mode, "std" | "rta" | "sdr") {
            self.graph_mode = mode.into();
        }
    }

    pub(super) fn refresh_dependencies(&mut self) {}
    pub(super) fn poll(&mut self) {}
    pub(super) fn stop(&mut self) -> Result<bool, String> {
        Ok(false)
    }

    pub(super) fn command(&mut self, data: &Value) -> Result<bool, (&'static str, String)> {
        let bind = parse_bind(data, self.bind.0, self.bind.1).map_err(|e| ("bad_parameter", e))?;
        match data["action"].as_str() {
            Some("configure") => {
                let changed = self.bind != bind;
                self.bind = bind;
                Ok(changed)
            }
            Some("stop") => Ok(false),
            Some("start" | "restart") => Err(("server_error", UNAVAILABLE.into())),
            _ => Err((
                "bad_parameter",
                "action must be start, stop, restart, or configure".into(),
            )),
        }
    }
}
