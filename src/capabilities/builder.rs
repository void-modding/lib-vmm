use std::sync::{Arc, Weak};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::capabilities::{api_key_capability::{ApiKeyCapability, RequiresApiKey}, base::CapabilityRef};

#[derive(Error, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityError {
    #[error("The provider was dropped before the refrence could be upgraded.")]
    ProviderDropped
}

/// Fluent builder use by providers to handle constructors
pub struct CapabilityBuilder<T> {
    weak: Weak<T>,
    caps: Vec<CapabilityRef>,
}

impl <T> CapabilityBuilder<T> {
    pub fn new_from_arc(arc: &Arc<T>) -> Self {
        Self { weak: Arc::downgrade(arc), caps: Vec::new() }
    }

    pub fn new_from_weak(weak: Weak<T>) -> Self {
        Self {weak, caps: Vec::new()}
    }

    pub fn finish(self) -> Vec<CapabilityRef> {
        self.caps
    }
}

impl<T: RequiresApiKey + Send + Sync + 'static> CapabilityBuilder<T> {
    pub fn api_key(mut self) -> Self {
        self.caps.push(Arc::new(ApiKeyCapability::new(self.weak.clone())) as CapabilityRef);
        self
    }
}
