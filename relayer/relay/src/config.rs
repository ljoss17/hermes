//! Read the relayer configuration into the Config struct, in examples for now
//! to support ADR validation..should move to relayer/src soon

use std::path::Path;
use std::time::Duration;

use serde_derive::{Deserialize, Serialize};
use tendermint::chain::Id as ChainId;
use tendermint::net;

use crate::error;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub global: GlobalConfig,
    pub chains: Vec<ChainConfig>,
    pub connections: Option<Vec<Connection>>, // use all for default
}

fn default_timeout() -> Duration {
    Duration::from_secs(10)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Strategy {
    #[serde(rename = "naive")]
    Naive,
}

impl Default for Strategy {
    fn default() -> Self {
        Self::Naive
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlobalConfig {
    #[serde(default = "default_timeout", with = "humantime_serde")]
    pub timeout: Duration,
    #[serde(default)]
    pub strategy: Strategy,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            strategy: Strategy::default(),
        }
    }
}

fn default_gas() -> u64 {
    200_000
}

fn default_rpc_addr() -> net::Address {
    "localhost:26657".parse().unwrap()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChainConfig {
    pub id: ChainId,
    #[serde(default = "default_rpc_addr")]
    pub rpc_addr: net::Address,
    pub account_prefix: String,
    pub key_name: String,
    pub client_ids: Vec<String>,
    #[serde(default = "default_gas")]
    pub gas: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Connection {
    pub src: Option<ConnectionEnd>,    // use any source
    pub dest: Option<ConnectionEnd>,   // use any destination
    pub paths: Option<Vec<RelayPath>>, // use any port, direction bidirectional
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionEnd {
    pub client_id: String,
    pub connection_id: Option<String>, // use all connections to this client
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Direction {
    #[serde(rename = "unidirectional")]
    Unidirectional,
    #[serde(rename = "bidirectional")]
    Bidirectional,
}

impl Default for Direction {
    fn default() -> Self {
        Self::Bidirectional
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RelayPath {
    pub src_port: Option<String>,  // default from any source port
    pub dest_port: Option<String>, // default from any dest port
    #[serde(default)]
    pub direction: Direction, // default bidirectional
}

/// Attempt to load and parse the config file into the Config struct.
///
/// TODO: If a file cannot be found, return a default Config allowing relayer
///       to relay everything from any to any chain(?)
pub fn parse(path: impl AsRef<Path>) -> Result<Config, error::Error> {
    let config_toml =
        std::fs::read_to_string(&path).map_err(|e| error::Kind::ConfigIo.context(e))?;

    let config =
        toml::from_str::<Config>(&config_toml[..]).map_err(|e| error::Kind::Config.context(e))?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parse_valid_config() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/config/fixtures/relayer_conf_example.toml"
        );

        let config = parse(path);
        assert!(config.is_ok());
    }
}
