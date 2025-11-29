use crate::capabilities::base::{Capability, CapabilityRef};

pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str;

    /// A list of capabilities that providers have.
    fn capabilities(&self) -> &[CapabilityRef];

    /// Finds a capability by its string identifier.
    ///
    /// Searches this provider's capabilities for the first capability whose `id()` equals the given `id`.
    ///
    /// # Returns
    ///
    /// `Some(&dyn Capability)` if a capability with the given id exists, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// // assuming `provider` implements Provider
    /// if let Some(cap) = provider.find_capability("network") {
    ///     assert_eq!(cap.id(), "network");
    /// }
    /// ```
    fn find_capability(&self, id: &str) -> Option<&dyn Capability> {
        self.capabilities()
            .iter()
            .map(|o| o.as_ref())
            .find(|o| o.id() == id)
    }

    /// Returns a reference to a concrete capability of type `T` from the provider's capability list if present.
    ///
    /// The method searches the provider's capabilities and attempts to downcast each capability to `T`.
    ///
    /// # Returns
    ///
    /// `Some(&T)` if a capability of type `T` is found, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Given a value `provider` implementing `Provider`:
    /// let cap: Option<&MyCapabilityType> = provider.get::<MyCapabilityType>();
    /// if let Some(c) = cap {
    ///     // use `c`
    /// }
    /// ```
    fn get<T: Capability + 'static>(&self) -> Option<&T>
    where
        Self: Sized,
    {
        self.capabilities()
            .iter()
            .find_map(|o| o.as_ref().as_any().downcast_ref::<T>())
    }
}