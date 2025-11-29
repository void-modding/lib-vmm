use std::path::PathBuf;

use async_trait::async_trait;

use crate::traits::discovery::{DiscoveryError, DiscoveryQuery, DiscoveryResult, ModExtendedMetadata, ModSummary};
use crate::traits::provider::Provider;



/// Note: Currently unimplemented
#[deprecated(since = "0.2.0", note = "Use capabilities instead")]
#[derive(Default, Debug)]
pub struct ModProviderFeatures {
    pub supports_endorsements: bool,
    pub requires_api_token: bool,
    pub mod_multi_file: bool
}

pub enum ModDownloadResult {
    Failed(String),
    InProgress(u8),
    Completed(PathBuf),
    Cancelled,
    CannotComplete(String)
}

#[async_trait]
pub trait ModProvider: Provider + Send + Sync {
    async fn download_mod(&self, mod_id: String) -> ModDownloadResult;
    async fn discover(&self, query: &DiscoveryQuery) -> Result<DiscoveryResult, DiscoveryError>;

    /// Deprecated companion for discovering mods by game identifier.
    ///
    /// This method is deprecated; use `discover` with a `DiscoveryQuery` instead.
    /// The default implementation panics and should not be used.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Deprecated — prefer `discover`.
    /// # async fn example<P: crate::traits::mod_provider::ModProvider>(provider: &P) {
    /// let _mods = provider.discover_mods("game-id".to_string()).await;
    /// # }
    /// ```
    #[deprecated(since = "0.1.0", note = "Use `discover` instead")]
    #[allow(unused_variables)]
    async fn discover_mods(&self, game_id: String) -> Vec<ModSummary> {
        panic!("Do not use deprecated discover_mod function")
    }

    async fn get_extended_mod(&self, mod_id: &str) -> ModExtendedMetadata;

    /// Deprecated method that would expose the provider's feature flags; do not call.
///
/// This method is deprecated in favour of the provider `capabilities` API and its
/// default implementation panics.
///
/// # Returns
///
/// A reference to the provider's `ModProviderFeatures`.
///
/// # Examples
///
/// ```
/// use crate::traits::mod_provider::ModProviderFeatures;
///
/// let features = ModProviderFeatures {
///     supports_endorsements: false,
///     requires_api_token: false,
///     mod_multi_file: false,
/// };
///
/// // You can work with a `&ModProviderFeatures` value as shown:
/// let _ref: &ModProviderFeatures = &features;
/// ```
#[deprecated(since = "0.2.0", note = "Use capabilities instead")]
#[allow(deprecated)]
    fn configure(&self) -> &ModProviderFeatures { panic!("DO NOT USE CONFIGURE()") }

    /// Get the provider's identifier.
    ///
    /// # Returns
    ///
    /// `String` with the provider's identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// struct Dummy;
    /// impl crate::traits::provider::Provider for Dummy { fn id(&self) -> &str { "dummy" } }
    /// impl crate::traits::mod_provider::ModProvider for Dummy {}
    /// let d = Dummy;
    /// assert_eq!(d.register(), "dummy".to_string());
    /// ```
    fn register(&self) -> String {
        self.id().to_string()
    }
}