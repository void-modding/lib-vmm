use std::{any::Any, sync::Arc};

use crate::capabilities::api_key_capability::RequiresApiKey;

pub trait Capability: Any + Send + Sync {
    /// String discriminator. Prefer lowercase, dot-seperated names
    /// example: `vmm.game.installs_mod_loader`
    fn id(&self) -> &'static str;

    /// Used for typed downcasting helpers.
    fn as_any(&self) -> &dyn Any;

    /// Provide access to a `RequiresApiKey` capability when the implementation exposes it.
///
/// Implementations that represent a capability requiring an API key should override this
/// method to return `Some(&self)` referencing their `RequiresApiKey` implementation.
/// The default implementation returns `None`.
///
/// # Examples
///
/// ```
/// use std::any::Any;
///
/// struct Dummy;
///
/// impl crate::capabilities::base::Capability for Dummy {
///     fn id(&self) -> &'static str { "dummy" }
///     fn as_any(&self) -> &dyn Any { self }
/// }
///
/// let d = Dummy;
/// assert!(d.as_requires_api_key().is_none());
/// ```
///
/// # Returns
///
/// `Some(&dyn crate::capabilities::api_key_capability::RequiresApiKey)` if the capability exposes
/// `RequiresApiKey`, `None` otherwise.
fn as_requires_api_key(&self) -> Option<&dyn RequiresApiKey> { None }
}

/// Helper to avoid manual downcast_ref
pub trait CapabilityCastExt {
    fn get<T: Capability + 'static>(&self) -> Option<&T>;
}

impl CapabilityCastExt for dyn Capability {
    /// Attempts to downcast the capability to the concrete type `T`.
    ///
    /// Returns `Some(&T)` if the underlying capability is of type `T`, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use crate::capabilities::base::{Capability, CapabilityCastExt, capability};
    ///
    /// struct Dummy;
    /// capability!(Dummy, "example.dummy");
    ///
    /// let cap: Arc<dyn Capability> = Arc::new(Dummy);
    /// let got = (&*cap as &dyn Capability).get::<Dummy>();
    /// assert!(got.is_some());
    /// ```
    fn get<T: Capability + 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

/// Macro to reduce boilerplate when declaring a capability with a fixed id.
#[macro_export]
macro_rules! capability {
    ($ty:ty, $id:expr) => {
        impl Capability for $ty {
            fn id(&self) -> &'static str { $id }
            fn as_any(&self) -> &dyn std::any::Any { self }
        }
    };
}

/// Container type for shared ownership
pub type CapabilityRef = Arc<dyn Capability>;