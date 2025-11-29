use std::sync::{Arc, Weak};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{capabilities::{base::Capability, builder::CapabilityError, form::FormSchema, ids}};

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
    fn render(&self) -> Result<FormSchema, CapabilityError>;
}

/// Wrapper giving this behavior a concrete Capability
pub struct ApiKeyCapability<T: RequiresApiKey + Send + Sync + 'static>(Weak<T>);

impl <T: RequiresApiKey + Send + Sync + 'static> ApiKeyCapability<T> {
    /// Creates a new `ApiKeyCapability` that wraps the given weak reference.

///

/// # Parameters

///

/// - `inner`: a `Weak<T>` pointing to the underlying provider implementing `RequiresApiKey`.

///

/// # Returns

///

/// A new `ApiKeyCapability<T>` that delegates to the provided weak reference.

///

/// # Examples

///

/// ```no_run

/// use std::sync::{Arc, Weak};

/// // Assume `MyProvider` implements `RequiresApiKey`.

/// // let provider: Arc<MyProvider> = Arc::new(MyProvider::new());

/// // let weak = Arc::downgrade(&provider);

/// // let cap = ApiKeyCapability::new(weak);

/// ```
pub fn new(inner: Weak<T>) -> Self { Self(inner) }
    /// Obtain a strong `Arc` reference to the underlying provider if it still exists.
    ///
    /// Returns `Ok(Arc<T>)` with the upgraded strong reference, or `Err(CapabilityError::ProviderDropped)` if the underlying provider has been dropped.
    pub fn inner(&self) -> Result<Arc<T>, CapabilityError> {
        self.upgrade().ok_or(CapabilityError::ProviderDropped)
    }
    /// Attempts to upgrade the stored `Weak<T>` to a strong `Arc<T>`.
    
    ///
    
    /// Returns `Some(Arc<T>)` if the underlying value is still alive, `None` if it has been dropped.
    
    ///
    
    /// # Examples
    
    ///
    
    /// ```
    
    /// use std::sync::{Arc, Weak};
    
    ///
    
    /// let strong = Arc::new(42);
    
    /// let weak: Weak<i32> = Arc::downgrade(&strong);
    
    /// assert!(weak.upgrade().is_some());
    
    ///
    
    /// drop(strong);
    
    /// assert!(weak.upgrade().is_none());
    
    /// ```
    fn upgrade(&self) -> Option<Arc<T>> {
        self.0.upgrade()
    }
}

impl <T:RequiresApiKey + Send + Sync + 'static> Capability  for ApiKeyCapability<T> {
    /// Returns the static capability identifier for the "requires API key" capability.
///
/// # Returns
///
/// The `ids::REQUIRES_API_KEY` static string.
fn id(&self) -> &'static str { ids::REQUIRES_API_KEY }
    /// Expose the capability as `Any` to enable runtime downcasting.
///
/// This method returns a reference to the capability as a `dyn Any`, allowing callers
/// to attempt downcasts (for example with `downcast_ref`) when they need concrete access.
///
/// # Examples
///
/// ```
/// // Given a value `cap` that implements the capability:
/// let any_ref = cap.as_any();
/// // Attempt to downcast to a concrete type:
/// // let concrete = any_ref.downcast_ref::<ApiKeyCapability<MyType>>();
/// ```
fn as_any(&self) -> &dyn std::any::Any { self }
    /// Expose this capability as a `RequiresApiKey` trait object.
///
/// # Examples
///
/// ```
/// // Given an `Arc` to a type implementing `RequiresApiKey`:
/// let arc = std::sync::Arc::new(MyApiKeyProvider::new());
/// let cap = ApiKeyCapability::new(std::sync::Arc::downgrade(&arc));
/// assert!(cap.as_requires_api_key().is_some());
/// ```
///
/// # Returns
///
/// `Some(&dyn RequiresApiKey)` containing a trait object referencing this capability.
fn as_requires_api_key(&self) -> Option<&dyn RequiresApiKey> { Some(self) }
}

/// Delegate back to underlying behvaior for ergonomics
impl <T: RequiresApiKey + Send + Sync + 'static> RequiresApiKey for ApiKeyCapability<T> {
    /// Forwards submitted API key responses to the underlying provider and returns its validation result.
    ///
    /// If the wrapped provider has been dropped, returns `ApiKeyValidationError::ProviderError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Weak};
    /// // Minimal stubs to demonstrate usage; real implementations live in the same module.
    /// struct DummyProvider;
    /// impl RequiresApiKey for DummyProvider {
    ///     fn on_provided(&self, _values: &Vec<ApiSubmitResponse>) -> Result<KeyAction, ApiKeyValidationError> {
    ///         Ok(KeyAction::Store)
    ///     }
    ///     fn needs_prompt(&self, _existing_key: Option<&str>) -> bool { false }
    ///     fn render(&self) -> Result<FormSchema, CapabilityError> { Err(CapabilityError::ProviderDropped) }
    /// }
    ///
    /// let arc = Arc::new(DummyProvider);
    /// let weak: Weak<DummyProvider> = Arc::downgrade(&arc);
    /// let capability = ApiKeyCapability::new(weak);
    /// let values: Vec<ApiSubmitResponse> = vec![];
    /// let res = capability.on_provided(&values);
    /// assert!(matches!(res, Ok(KeyAction::Store)));
    /// ```
    fn on_provided(&self, values: &Vec<ApiSubmitResponse>) -> Result<KeyAction, ApiKeyValidationError> {
        match self.inner() {
            Ok(p) => p.on_provided(values),
            Err(_) => Err(ApiKeyValidationError::ProviderError),
        }
    }
    /// Notify the underlying provider that the user rejected or cancelled API-key entry.
    ///
    /// If the underlying provider has been dropped, this method does nothing.
    fn on_rejected(&self) {
        if let Ok(p) = self.inner() {
            p.on_rejected();
        }
    }
    /// Determines whether the UI should prompt the user for an API key.
    ///
    /// Returns `true` if the underlying provider indicates a prompt is required for the given
    /// `existing_key`, `false` if the provider is unavailable or indicates no prompt is needed.
    ///
    fn needs_prompt(&self, existing_key: Option<&str>) -> bool {
        match self.inner() {
            Ok(p) => p.needs_prompt(existing_key),
            Err(_) => false,
        }
    }
    /// Retrieve the UI form schema used to collect an API key from the underlying provider.
    ///
    /// # Returns
    ///
    /// `Ok(FormSchema)` with the schema produced by the underlying provider when available, or `Err(CapabilityError)` if the capability cannot produce a schema (for example, because the provider has been dropped).
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a provider that returns a simple FormSchema, wrap it in Arc/Weak and call render.
    /// struct Dummy;
    /// impl RequiresApiKey for Dummy {
    ///     fn on_provided(&self, _values: &Vec<ApiSubmitResponse>) -> Result<KeyAction, ApiKeyValidationError> { Ok(KeyAction::DontStore) }
    ///     fn needs_prompt(&self, _existing_key: Option<&str>) -> bool { true }
    ///     fn render(&self) -> Result<FormSchema, CapabilityError> { Ok(FormSchema::default()) }
    /// }
    ///
    /// let arc = std::sync::Arc::new(Dummy);
    /// let weak = std::sync::Arc::downgrade(&arc);
    /// let cap = ApiKeyCapability::new(weak);
    /// let schema = cap.render().expect("provider available");
    /// ```
    fn render(&self) -> Result<FormSchema, CapabilityError> {
        match self.inner() {
            Ok(p) => p.render(),
            Err(e) => Err(e),
        }
    }
}