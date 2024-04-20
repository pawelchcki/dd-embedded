use heapless::String;
use heapless::Vec;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(tag = "request_type", content = "payload")]
#[serde(rename_all = "kebab-case")]
pub enum Payload {
    AppStarted(AppStarted),
    AppHeartbeat(#[serde(skip_serializing)] ()),
}

#[derive(Serialize, Debug)]
pub struct AppStarted {
    pub configuration: Vec<Configuration, 20>,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
pub struct Configuration {
    pub name: String<30>,
    pub value: String<30>,
    pub origin: ConfigurationOrigin,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
#[repr(C)]
pub enum ConfigurationOrigin {
    EnvVar,
    Code,
    DdConfig,
    RemoteConfig,
    Default,
}
