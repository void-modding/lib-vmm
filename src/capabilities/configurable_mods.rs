use std::sync::{Arc, Weak};

use crate::capabilities::{base::Capability, builder::CapabilityError, form::{FormResponse, FormSchema}, ids};


pub trait ConfigurableModsBehavior: Send + Sync {
    fn get_configurable(&self, mod_id: &str) -> Option<FormSchema>;
    fn apply_configuration(&self, mod_id: &str, response: Vec<FormResponse>) -> ();
}

pub struct ConfigurableModsCapability<T: ConfigurableModsBehavior + Send + Sync + 'static>(Weak<T>);

impl<T: ConfigurableModsBehavior + Send + Sync + 'static> ConfigurableModsCapability<T> {
    pub fn new(inner: Weak<T>) -> Self {
        Self(inner)
    }

    pub fn inner(&self) -> Result<Arc<T>, CapabilityError> {
        self.upgrade().ok_or(CapabilityError::ProviderDropped)
    }

    fn upgrade(&self) -> Option<Arc<T>> {
        self.0.upgrade()
    }
}

impl <T: ConfigurableModsBehavior + Send + Sync + 'static> Capability for ConfigurableModsCapability<T> {
    fn id(&self) -> &'static str { ids::CONFIGURABLE_MODS }

    fn as_any(&self) -> &dyn std::any::Any { self }

    fn as_configurable_mods(&self) -> Option<&dyn ConfigurableModsBehavior> { Some(self) }
}

impl<T: ConfigurableModsBehavior + Send + Sync + 'static> ConfigurableModsBehavior for ConfigurableModsCapability<T> {
    fn get_configurable(&self, mod_id: &str) -> Option<FormSchema> {
        match self.inner() {
            Ok(p) => p.get_configurable(mod_id),
            Err(_) => None
        }
    }

    fn apply_configuration(&self, mod_id: &str, response: Vec<FormResponse>) -> () {
        match self.inner() {
            Ok(p) => p.apply_configuration(mod_id, response),
            Err(_) => ()
        }
    }
}
