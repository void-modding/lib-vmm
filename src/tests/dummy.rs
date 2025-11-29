use std::{fmt::Debug, path::PathBuf, sync::Arc};

use async_trait::async_trait;

use crate::{
    capabilities::{
        api_key_capability::{ApiKeyValidationError, ApiSubmitResponse, KeyAction, RequiresApiKey}, base::CapabilityRef, builder::{CapabilityBuilder, CapabilityError}, form::{Field, FieldType, FormSchema}
    },
    registry::model::ProviderSource,
    traits::{
        discovery::{
            DiscoveryError, DiscoveryMeta, DiscoveryQuery, DiscoveryResult, ModExtendedMetadata,
            ModSummary, PaginationMeta, Tag,
        },
        game_provider::{GameIcon, GameInstallError, GameMetadata, GameProvider},
        mod_provider::{ModDownloadResult, ModProvider},
        provider::Provider,
    },
};


pub struct DummyModProvider {
    id: String,
    caps: Vec<CapabilityRef>
}

impl Debug for DummyModProvider {
    /// Formats the provider for debug output, showing the struct name and its `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::sync::Arc;
    /// // Construct a dummy provider inside the same crate/module for the example.
    /// let provider = Arc::new(crate::tests::dummy::DummyModProvider {
    ///     id: "test-id".to_string(),
    ///     caps: Vec::new(),
    /// });
    /// let s = format!("{:?}", provider);
    /// assert!(s.contains("DummyModProvider"));
    /// assert!(s.contains("id"));
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DummyModProvider")
            .field("id", &self.id)
            .finish()
    }
}

impl DummyModProvider {
    /// Create a DummyModProvider instance initialised with the given identifier and its default capabilities, returned wrapped in an `Arc`.
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = DummyModProvider::new("dummy-1");
    /// assert_eq!(provider.id_str(), "dummy-1");
    /// assert!(!provider.capabilities().is_empty());
    /// ```
    pub fn new(id: &str) -> Arc<Self> {
        Arc::new_cyclic(|weak_self| {
            let caps = CapabilityBuilder::new_from_weak(weak_self.clone())
                .api_key()
                .finish();

            DummyModProvider {
                id: id.to_string(),
                caps,
            }
        })
    }

    /// Return the provider's identifier as a string slice.
    ///
    /// The provider's identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = DummyModProvider::new("abc");
    /// assert_eq!(provider.id_str(), "abc");
    /// ```
    pub fn id_str(&self) -> &str {
        &self.id
    }
}


impl Provider for DummyModProvider {
    /// The provider identifier for this implementation.
///
/// # Returns
///
/// `'dummyModProvider'` — the static provider id for the dummy mod provider.
///
/// # Examples
///
/// ```
/// let provider = crate::tests::dummy::DummyModProvider::new("id");
/// assert_eq!(provider.id(), "dummyModProvider");
/// ```
fn id(&self) -> &'static str { "dummyModProvider" }
    /// Return the capability references exposed by the provider.
///
/// # Returns
///
/// A slice of `CapabilityRef` referencing the provider's capabilities.
///
/// # Examples
///
/// ```no_run
/// let provider = DummyModProvider::new("dummy-id");
/// let caps = provider.capabilities();
/// // `caps` is a slice of capability references available from the provider
/// println!("capabilities: {}", caps.len());
/// ```
fn capabilities(&self) -> &[CapabilityRef] { &self.caps }
}

impl RequiresApiKey for DummyModProvider {
    /// Validate a submitted API key and decide the action to take.
    ///
    /// The method checks the first `ApiSubmitResponse` in `value` and validates its `value` field:
    /// - blank (after trimming) is treated as missing,
    /// - length less than 16 is considered too short,
    /// - otherwise the key is accepted.
    ///
    /// # Returns
    ///
    /// `Ok(KeyAction::Store)` if the first submission contains a non-blank key with at least 16 characters;
    /// `Err(ApiKeyValidationError::Empty)` if no submission is present or the first value is empty after trimming;
    /// `Err(ApiKeyValidationError::TooShort { min_len: 16 })` if the first value contains fewer than 16 characters.
    ///
    /// # Examples
    ///
    /// ```
    /// // Given a provider `p` implementing `RequiresApiKey` and a submit response:
    /// // let res = p.on_provided(&vec![ApiSubmitResponse { value: "0123456789abcdef".into() }]);
    /// // assert_eq!(res.unwrap(), KeyAction::Store);
    /// ```
    fn on_provided(&self, value: &Vec<ApiSubmitResponse>) -> Result<KeyAction, ApiKeyValidationError> {
        let first = value.first().ok_or(ApiKeyValidationError::Empty)?;

        if first.value.trim().is_empty() {
            return Err(ApiKeyValidationError::Empty)
        }
        if first.value.len() < 16 {
            return Err(ApiKeyValidationError::TooShort { min_len: 16 });
        }

        Ok(KeyAction::Store)
    }

    /// Determine whether the API key prompt should be shown.
    ///
    /// The prompt is required when there is no existing key or the existing key is an empty string.
    ///
    /// # Parameters
    ///
    /// - `existing_key`: an optional existing API key string to check.
    ///
    /// # Returns
    ///
    /// `true` if the prompt should be shown (no key or empty string), `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = crate::tests::dummy::DummyModProvider::new("id");
    /// assert!(provider.needs_prompt(None));
    /// assert!(provider.needs_prompt(Some("")));
    /// assert!(!provider.needs_prompt(Some("valid-key")));
    /// ```
    fn needs_prompt(&self, existing_key: Option<&str>) -> bool {
        match existing_key {
            None => true,
            Some(k) if k.is_empty() => true,
            Some(_) => false,
        }
    }

    /// Builds a form schema used to prompt the user for an API key.
    ///
    /// The returned schema has the title "Enter key", description "Description" and a single
    /// password field with id "api_key" and placeholder "Paste key here".
    ///
    /// # Examples
    ///
    /// ```
    /// // Create the provider (returns an Arc) and render its API-key form.
    /// let provider = crate::tests::dummy::DummyModProvider::new("dummy-id");
    /// let schema = provider.render().unwrap();
    /// assert_eq!(schema.title, "Enter key");
    /// assert_eq!(schema.fields[0].id, "api_key");
    /// assert_eq!(schema.fields[0].placeholder.as_deref(), Some("Paste key here"));
    /// ```
    fn render(&self) -> Result<FormSchema, CapabilityError> {
        Ok(FormSchema { title: "Enter key".into(), description: Some("Description".into()), fields: vec![ Field {
            id: "api_key".into(),
            label: "api_key".into(),
            field_type: FieldType::Password,
            regex: None,
            help: None,
            placeholder: Some("Paste key here".into()),
        }] })
    }
}


#[async_trait]
impl ModProvider for DummyModProvider {
    /// Downloads a mod and returns the outcome of the download.
    ///
    /// On success returns `ModDownloadResult::Completed` containing the filesystem path
    /// to the downloaded mod; on failure returns `ModDownloadResult::Failed` with an error message.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures::executor::block_on;
    /// // Assuming `DummyModProvider::new` is in scope and returns `Arc<DummyModProvider>`.
    /// let provider = DummyModProvider::new("provider-id");
    /// let ok = block_on(provider.download_mod("mod1".to_string()));
    /// match ok {
    ///     ModDownloadResult::Completed(path) => assert!(path.ends_with("mod1")),
    ///     ModDownloadResult::Failed(_) => panic!("expected success"),
    /// }
    ///
    /// let fail = block_on(provider.download_mod("fail".to_string()));
    /// match fail {
    ///     ModDownloadResult::Failed(msg) => assert!(msg.contains("bad id")),
    ///     ModDownloadResult::Completed(_) => panic!("expected failure"),
    /// }
    /// ```
    async fn download_mod(&self, mod_id: String) -> ModDownloadResult {
        if mod_id == "fail" {
            ModDownloadResult::Failed("bad id".into())
        } else {
            ModDownloadResult::Completed(PathBuf::from(format!("/tmp/{}", mod_id)))
        }
    }

    /// Produce a discovery result for the given query using dummy data.
    ///
    /// The returned result contains discovery metadata (provider and pagination info,
    /// applied and available tags) and a list of mod summaries that match the query.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures::executor::block_on;
    ///
    /// // construct provider and query (types from the crate under test)
    /// let provider = crate::tests::dummy::DummyModProvider::new("provider-1");
    /// let query = crate::DiscoveryQuery { game_id: "game-x".into(), tags: None };
    ///
    /// let result = block_on(provider.discover(&query)).unwrap();
    /// assert_eq!(result.mods.len(), 1);
    /// assert_eq!(result.meta.provider_id, provider.id_str());
    /// ```
    ///
    /// @returns `Ok` with a `DiscoveryResult` containing metadata and mod summaries, `Err` with a `DiscoveryError` on failure.
    async fn discover(&self, query: &DiscoveryQuery) -> Result<DiscoveryResult, DiscoveryError> {
        let summary = ModSummary {
            id: "mod-1".into(),
            name: "Test Mod".into(),
            description: "Long description".into(),
            short_description: "Short".into(),
            downloads: 42,
            views: 10,
            likes: 5,
            thumbnail_image: "/thumb.png".into(),
            tags: vec!["tag1".into()],
            user_name: "tester".into(),
            user_avatar: "/avatar.png".into(),
        };
        Ok(DiscoveryResult {
            meta: DiscoveryMeta {
                provider_id: self.id_str().to_string(),
                game_id: query.game_id.clone(),
                pagination: PaginationMeta {
                    current: 1,
                    page_size: 10,
                    total_pages: Some(1),
                    total_items: Some(1),
                },
                applied_tags: query.tags.clone().unwrap_or_default(),
                available_tags: Some(vec![Tag {
                    id: "tag1".into(),
                    name: "Tag One".into(),
                }]),
            },
            mods: vec![summary],
        })
    }

    /// Builds extended metadata for a mod identified by `mod_id`.
    ///
    /// The returned `ModExtendedMetadata` contains header and carousel images, a version,
    /// an `installed` flag which is `true` when `mod_id` equals `"installed-mod"`, and a
    /// description string derived from the provided `mod_id`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::sync::Arc;
    /// # // assume a type `DummyModProvider` exists with this async method
    /// # async fn _example(provider: &crate::tests::dummy::DummyModProvider) {
    /// let meta = provider.get_extended_mod("some-mod").await;
    /// assert_eq!(meta.header_image, "/header.png");
    /// assert!(meta.carousel_images.len() >= 1);
    /// assert_eq!(meta.version, "1.0.0");
    /// # }
    /// ```
    async fn get_extended_mod(&self, mod_id: &str) -> ModExtendedMetadata {
        ModExtendedMetadata {
            header_image: "/header.png".into(),
            carousel_images: vec!["/c1.png".into(), "/c2.png".into()],
            version: "1.0.0".into(),
            installed: mod_id == "installed-mod",
            description: format!("Extended meta for {}", mod_id),
        }
    }
}

pub struct DummyGameProvider {
    id: String,
    mod_provider: String,
}

impl DummyGameProvider {
    /// Create a new `DummyGameProvider` with the given game id and associated mod provider id.
    ///
    /// `id` is the game's identifier. `mod_provider` is the id of the `ModProvider` this game delegates to.
    ///
    /// # Returns
    ///
    /// A `DummyGameProvider` instance initialised with the provided identifiers.
    ///
    /// # Examples
    ///
    /// ```
    /// let gp = DummyGameProvider::new("game-1", "mod-provider-x");
    /// assert_eq!(gp.game_id(), "game-1");
    /// assert_eq!(gp.mod_provider_id(), "mod-provider-x");
    /// ```
    pub fn new(id: &str, mod_provider: &str) -> Self {
        Self {
            id: id.to_string(),
            mod_provider: mod_provider.to_string(),
        }
    }
}

impl Provider for DummyGameProvider {
    /// Provider identifier for the dummy game provider.
///
/// # Returns
///
/// The static string slice `"dummy.game"` representing this provider's id.
///
/// # Examples
///
/// ```
/// let provider_id = DummyGameProvider::new("dummy", "mod-provider").id();
/// assert_eq!(provider_id, "dummy.game");
/// ```
fn id(&self) -> &'static str { "dummy.game" }
    /// Get the provider's capability references.
///
/// This game provider exposes no capabilities and therefore returns an empty slice.
///
/// # Examples
///
/// ```
/// let provider = DummyGameProvider::new("game-id", "mod-provider");
/// let caps = provider.capabilities();
/// assert!(caps.is_empty());
/// ```
fn capabilities(&self) -> &[CapabilityRef] { &[] }
}

#[async_trait]
impl GameProvider for DummyGameProvider {
    /// Get the ID of the mod provider associated with this game.
    ///
    /// # Returns
    ///
    /// The mod provider ID as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// let g = DummyGameProvider::new("game", "modprov");
    /// assert_eq!(g.mod_provider_id(), "modprov");
    /// ```
    fn mod_provider_id(&self) -> &str {
        &self.mod_provider
    }

    /// Returns the provider's game identifier.
///
/// # Examples
///
/// ```
/// let g = DummyGameProvider::new("game-1", "mod-prov");
/// assert_eq!(g.game_id(), "game-1");
/// ```
fn game_id(&self) -> &str { &self.id }

    /// Returns the game's metadata for this dummy provider.
    ///
    /// The returned metadata contains the provider's game id, display name, short name,
    /// icon path, and provider source.
    ///
    /// # Examples
    ///
    /// ```
    /// let gp = crate::tests::dummy::DummyGameProvider::new("dummy-game", "dummy-mod-provider");
    /// let meta = gp.metadata();
    /// assert_eq!(meta.id, "dummy-game");
    /// assert_eq!(meta.display_name, "Dummy Game");
    /// assert_eq!(meta.short_name, "DG");
    /// ```
    fn metadata(&self) -> GameMetadata {
        GameMetadata {
            id: self.id.clone(),
            display_name: "Dummy Game".into(),
            short_name: "DG".into(),
            icon: GameIcon::Path("/icon.png".into()),
            provider_source: ProviderSource::Plugin("plugin-x".into()),
        }
    }
    /// Get the provider's external identifier.
    ///
    /// Returns the external identifier string slice associated with this game provider.
    ///
    /// # Examples
    ///
    /// ```
    /// let gp = DummyGameProvider::new("dummy", "dummyModProvider");
    /// assert_eq!(gp.get_external_id(), "external-123");
    /// ```
    fn get_external_id(&self) -> &str {
        "external-123"
    }
    /// Performs a no-op installation for the dummy game provider.
    ///
    /// The provided path is accepted but ignored; the method always succeeds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// let gp = DummyGameProvider::new("dummy-game", "dummy-mod-provider");
    /// let path = PathBuf::from("/tmp/mod.zip");
    /// assert!(gp.install_mod(&path).is_ok());
    /// ```
    fn install_mod(&self, _path: &PathBuf) -> Result<(), GameInstallError> {
        Ok(())
    }
}