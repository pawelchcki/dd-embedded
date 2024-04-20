mod payloads;

use heapless::String;
pub use payloads::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ApiVersion {
    #[serde(rename = "v1")]
    V1,
    #[serde(rename = "v2")]
    V2,
}

impl ApiVersion {
    pub fn to_str(&self) -> &'static str {
        match self {
            ApiVersion::V1 => "v1",
            ApiVersion::V2 => "v2",
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Telemetry<'a> {
    pub api_version: ApiVersion,
    pub tracer_time: u64,
    pub runtime_id: &'a str,
    pub seq_id: u64,
    pub application: &'a Application,
    pub host: &'a Host,
    #[serde(flatten)]
    pub payload: &'a Payload,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Application {
    pub service_name: String<20>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_version: Option<String<20>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String<30>>,
    pub language_name: String<30>,
    pub language_version: String<30>,
    pub tracer_version: String<30>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_name: Option<String<30>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_version: Option<String<30>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_patches: Option<String<30>>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Host {
    pub hostname: String<100>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String<100>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String<100>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String<100>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel_name: Option<String<100>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel_release: Option<String<30>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel_version: Option<String<30>>,
}
