use std::path::PathBuf;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::registry::model::ProviderSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameIcon {
    Path(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMetadata {
    pub id: String,
    pub display_name: String,
    pub short_name: String,
    pub icon: GameIcon,
    pub provider_source: ProviderSource
}

#[async_trait]
pub trait GameProvider: Send + Sync {
    fn game_id(&self) -> &str;
    fn mod_provider_id(&self) -> &str;
    fn metadata(&self) -> GameMetadata;
    fn get_external_id(&self) -> &str;
    fn install_mod(&self, path: &PathBuf) -> Result<(), ()>;
}
