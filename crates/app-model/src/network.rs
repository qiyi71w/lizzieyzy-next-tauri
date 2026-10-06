use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkModeDto {
    #[default]
    Direct,
    System,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NetworkSettingsDto {
    pub mode: NetworkModeDto,
    pub manual_host: String,
    pub manual_port: u16,
}

impl Default for NetworkSettingsDto {
    fn default() -> Self {
        Self {
            mode: NetworkModeDto::Direct,
            manual_host: "127.0.0.1".into(),
            manual_port: 7897,
        }
    }
}

impl NetworkSettingsDto {
    pub fn validate(&self) -> Result<(), String> {
        let host = &self.manual_host;
        // A host, never a URL or credentials. Values in inactive modes also cannot store secrets.
        if host.contains(['@', '/', '\\', '?', '#']) || host.chars().any(char::is_whitespace) {
            return Err(
                "Proxy host must contain only a hostname or IP address, without credentials or URL syntax."
                    .into(),
            );
        }
        if host.contains(':')
            && host
                .trim_matches(['[', ']'])
                .parse::<std::net::Ipv6Addr>()
                .is_err()
        {
            return Err("Enter the proxy port separately from its host.".into());
        }
        if self.mode == NetworkModeDto::Manual && (host.is_empty() || self.manual_port == 0) {
            return Err("Manual proxy requires a nonempty host and port 1–65535.".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkSnapshotDto {
    pub settings: NetworkSettingsDto,
    pub policy_revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRequestIdentityDto {
    pub request_id: u64,
    pub policy_revision: u64,
    pub document_identity: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkRouteDto {
    pub mode: NetworkModeDto,
    pub source: String,
    pub target: String,
    pub proxy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderNetworkResultDto {
    pub identity: ProviderRequestIdentityDto,
    pub result: crate::ProviderFetchResult,
    pub routes: Vec<NetworkRouteDto>,
}
