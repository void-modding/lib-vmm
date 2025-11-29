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
    /// Creates a CapabilityBuilder from a strong `Arc` by downgrading it to a `Weak` reference.
    ///
    /// The returned builder holds a non-owning `Weak` reference to the provided `Arc` and an empty
    /// capability list ready for fluent construction.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use crate::capabilities::builder::CapabilityBuilder;
    ///
    /// let svc = Arc::new(());
    /// let builder = CapabilityBuilder::new_from_arc(&svc);
    /// let caps = builder.finish();
    /// assert!(caps.is_empty());
    /// ```
    pub fn new_from_arc(arc: &Arc<T>) -> Self {
        Self { weak: Arc::downgrade(arc), caps: Vec::new() }
    }

    /// Creates a CapabilityBuilder from an existing weak reference.
    ///
    /// The builder is initialised with the provided `Weak<T>` and an empty capability list.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use std::sync::Weak;
    /// // assume CapabilityBuilder is in scope
    /// let arc = Arc::new(());
    /// let weak: Weak<()> = Arc::downgrade(&arc);
    /// let builder = CapabilityBuilder::new_from_weak(weak);
    /// assert_eq!(builder.finish().len(), 0);
    /// ```
    pub fn new_from_weak(weak: Weak<T>) -> Self {
        Self {weak, caps: Vec::new()}
    }

    /// Consume the builder and return the accumulated capability references.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Weak;
    /// use crate::capabilities::builder::CapabilityBuilder;
    ///
    /// let builder = CapabilityBuilder::<()>::new_from_weak(Weak::new());
    /// let caps = builder.finish();
    /// assert!(caps.is_empty());
    /// ```
    ///
    /// # Returns
    ///
    /// A `Vec<CapabilityRef>` containing the capability references collected by the builder.
    pub fn finish(self) -> Vec<CapabilityRef> {
        self.caps
    }
}

impl<T: RequiresApiKey + Send + Sync + 'static> CapabilityBuilder<T> {
    /// Appends an `ApiKeyCapability` to the builder and returns the builder for chaining.
    ///
    /// The new capability is created from the builder's weak reference and added to the accumulated
    /// capability list.
    ///
    /// # Returns
    ///
    /// The same `CapabilityBuilder` with the `ApiKeyCapability` appended.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use std::sync::Weak;
    /// use crate::capabilities::builder::CapabilityBuilder;
    ///
    /// // use a unit value as the provider type for the example
    /// let weak: Weak<()> = Arc::new(()).downgrade();
    /// let builder = CapabilityBuilder::new_from_weak(weak).api_key();
    /// let caps = builder.finish();
    /// assert_eq!(caps.len(), 1);
    /// ```
    pub fn api_key(mut self) -> Self {
        self.caps.push(Arc::new(ApiKeyCapability::new(self.weak.clone())) as CapabilityRef);
        self
    }
}