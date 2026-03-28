use super::EnvOr;
use anyhow::Result;
use axum::http::HeaderValue;
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawServerConfig {
    address: EnvOr<SocketAddr>,
    frontend_dir: EnvOr<PathBuf>,
    logs_dir: EnvOr<PathBuf>,
    max_upload_size: EnvOr<usize>,
    allowed_origins: Vec<EnvOr<String>>,
}

impl RawServerConfig {
    pub(super) fn into_config(self) -> Result<ServerConfig> {
        let allowed_origins = self
            .allowed_origins
            .into_iter()
            .map(|o| {
                o.resolve()
                    .and_then(|s| s.parse::<HeaderValue>().map_err(Into::into))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(ServerConfig {
            address: self.address.resolve()?,
            frontend_dir: self.frontend_dir.resolve()?,
            logs_dir: self.logs_dir.resolve()?,
            max_upload_size: self.max_upload_size.resolve()?,
            allowed_origins,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub address: SocketAddr,
    pub frontend_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub max_upload_size: usize,
    pub allowed_origins: Vec<HeaderValue>,
}
