use std::{any::Any, sync::Arc};

pub trait Capability: Any + Send + Sync {
    /// String discriminator. Prefer lowercase, dot-seperated names
    /// example: `vmm.game.installs_mod_loader`
    fn id(&self) -> &'static str;

    /// Used for typed downcasting helpers.
    fn as_any(&self) -> &dyn Any;
}

/// Helper to avoid manual downcast_ref
pub trait CapabilityCastExt {
    fn get<T: Capability + 'static>(&self) -> Option<&T>;
}

impl CapabilityCastExt for dyn Capability {
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
