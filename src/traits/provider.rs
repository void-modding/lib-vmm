use crate::capabilities::base::{Capability, CapabilityRef};

pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str;

    /// A list of capabilities that providers have.
    fn capabilities(&self) -> &[CapabilityRef];

    /// Helper to fetch by 'id' string.
    fn find_capability(&self, id: &str) -> Option<&dyn Capability> {
        self.capabilities()
            .iter()
            .map(|o| o.as_ref())
            .find(|o| o.id() == id)
    }

    /// Helper to get a concrete type
    fn get<T: Capability + 'static>(&self) -> Option<&T>
    where
        Self: Sized,
    {
        self.capabilities()
            .iter()
            .find_map(|o| o.as_ref().as_any().downcast_ref::<T>())
    }
}
