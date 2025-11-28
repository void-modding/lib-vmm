use std::sync::{Arc, Weak};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{capabilities::{base::Capability, builder::CapabilityError, form::FormSchema, ids}, capability};

/// What the runtime should do with a successfully provided key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum KeyAction {
    /// The runtime will store the key for the future.
    Store,
    /// The runtime will NOT store the key
    DontStore,
}

#[derive(Error, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum ApiKeyValidationError {
    #[error("API key cannot be empty")]
    Empty,
    #[error("API key too short (min {min_len})")]
    TooShort { min_len: usize },
    #[error("API key is invalid")]
    Invalid,
    #[error("An error occured while working with the provider.")]
    ProviderError,
    #[error("{0}")]
    Other(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ApiSubmitResponse {
    pub id: String,
    pub value: String
}

/// Behavior-only trait (no Capability)
pub trait RequiresApiKey: Send + Sync {
    /// Called when the user submits a key.
    /// Return Err(message) to indicate validation failure.
    fn on_provided(&self, values: &Vec<ApiSubmitResponse>) -> Result<KeyAction, ApiKeyValidationError>;

    /// Called when the user explicitly rejects entering a key (e.g. cancels).
    fn on_rejected(&self) {}

    /// Whether the UI should prompt for a key (e.g. missing or invalid).
    fn needs_prompt(&self, existing_key: Option<&str>) -> bool;

    /// Returns the form schema used to render the API key collection UI.
    fn render(&self) -> FormSchema;
}

/// Wrapper giving this behavior a concrete Capability
pub struct ApiKeyCapability<T: RequiresApiKey + Send + Sync + 'static>(Weak<T>);

impl <T: RequiresApiKey + Send + Sync + 'static> ApiKeyCapability<T> {
    pub fn new(inner: Weak<T>) -> Self { Self(inner) }
    pub fn inner(&self) -> Result<Arc<T>, CapabilityError> {
        self.upgrade().ok_or(CapabilityError::ProviderDropped)
    }
    fn upgrade(&self) -> Option<Arc<T>> {
        self.0.upgrade()
    }
}

impl <T:RequiresApiKey + Send + Sync + 'static> Capability  for ApiKeyCapability<T> {
    fn id(&self) -> &'static str { ids::REQUIRES_API_KEY }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_requires_api_key(&self) -> Option<&dyn RequiresApiKey> { Some(self) }
}

/// Delegate back to underlying behvaior for ergonomics
impl <T: RequiresApiKey + Send + Sync + 'static> RequiresApiKey for ApiKeyCapability<T> {
    fn on_provided(&self, values: &Vec<ApiSubmitResponse>) -> Result<KeyAction, ApiKeyValidationError> {
        match self.inner() {
            Ok(p) => p.on_provided(values),
            Err(_) => Err(ApiKeyValidationError::ProviderError),
        }
    }
    fn on_rejected(&self) {
        if let Ok(p) = self.inner() {
            p.on_rejected();
        }
    }
    fn needs_prompt(&self, existing_key: Option<&str>) -> bool {
        match self.inner() {
            Ok(p) => p.needs_prompt(existing_key),
            Err(_) => false,
        }
    }
    fn render(&self) -> FormSchema {
        match self.inner() {
            Ok(p) => p.render(),
            Err(_) => panic!("An error occurred while working with the provider."),
        }
    }
}
