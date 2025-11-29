use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::{
    registry::{RegistryError, id::normalize_id,
        model::{GameEntry, ProviderEntry, ProviderSource}},
    traits::{discovery::ModExtendedMetadata, game_provider::{GameMetadata, GameProvider}, mod_provider::ModProvider}};

pub struct ContextBuilder {
    mod_providers: HashMap<String, ProviderEntry>,
    games: HashMap<String, GameEntry>
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            mod_providers: HashMap::new(),
            games: HashMap::new(),
        }
    }

    /// Registers a mod provider under a canonicalised identifier.
    ///
    /// Normalises `id` before insertion. Registration fails if the canonicalised id
    /// begins with `core:` while `source` is not `ProviderSource::Core`, or if a
    /// provider with the same canonical id already exists.
    ///
    /// # Parameters
    ///
    /// - `id`: Identifier to register; it will be canonicalised using `normalize_id`.
    /// - `source`: The origin of the provider; `core:` ids are reserved for
    ///   `ProviderSource::Core`.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success. Returns `Err(RegistryError::ReservedCoreId(id))` if the
    /// canonical id starts with `core:` and `source` is not `Core`, or
    /// `Err(RegistryError::ProviderAlreadyExists(id))` if a provider with the same
    /// canonical id is already registered. Errors from `normalize_id` are propagated.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    ///
    /// let mut builder = ContextBuilder::new();
    /// let provider = Arc::new(MyModProvider::new());
    /// builder.register_mod_provider("example:provider", provider, ProviderSource::External).unwrap();
    /// ```
    pub fn register_mod_provider(&mut self, id: &str, provider: Arc<dyn ModProvider + Send + Sync>, source: ProviderSource) -> Result<(), RegistryError> {
        let id = normalize_id(id)?;
        if id.starts_with("core:") && !matches!(source, ProviderSource::Core) {
            return Err(RegistryError::ReservedCoreId(id),)
        }

        if self.mod_providers.contains_key(&id) {
            return Err(RegistryError::ProviderAlreadyExists(id));
        }

        self.mod_providers.insert(id.clone(), ProviderEntry {
            id,
            source,
            provider
        });

        Ok(())
    }

    /// Registers a game provider using the provider's normalised `id()` as the game's identifier.
    ///
    /// The provider's `id()` is normalised and used as the stored game id. The provider's `mod_provider_id()` is normalised and must refer to an already-registered mod provider. The function inserts a new `GameEntry` linking the game to its required mod provider.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::GameAlreadyExists(id)` if a game with the same id is already registered.
    /// Returns `RegistryError::NotFound(depends_on)` if the required mod provider is not registered.
    /// Propagates any `RegistryError` returned by the id normalisation step.
    ///
    /// # Examples
    ///
    /// ```
    /// let provider: Arc<dyn GameProvider + Send + Sync> = Arc::new(MyGameProvider::new(...));
    /// builder.register_game_provider(provider, ProviderSource::External).unwrap();
    /// ```
    pub fn register_game_provider(&mut self, provider: Arc<dyn GameProvider + Send + Sync>, source: ProviderSource) -> Result<(), RegistryError> {
        let id = normalize_id(provider.id())?;
        if self.games.contains_key(&id) {
            return Err(RegistryError::GameAlreadyExists(id));
        }

        let depends_on = normalize_id(provider.mod_provider_id())?;

        if !self.mod_providers.contains_key(&depends_on) {
            return Err(RegistryError::NotFound(depends_on));
        }

        self.games.insert(id.clone(), GameEntry {
            id,
            source,
            game: provider,
            required_provider_id: depends_on
        });

        Ok(())
    }

    pub fn freeze(self) -> Context {
        Context {
            mod_providers: Arc::new(self.mod_providers),
            game_providers: Arc::new(self.games),
            active_game: Mutex::new(None),
        }
    }
}


pub struct Context {
    mod_providers: Arc<HashMap<String, ProviderEntry>>,
    game_providers: Arc<HashMap<String, GameEntry>>,
    active_game: Mutex<Option<String>>
}


impl Context {
    pub fn get_mod_provider(&self, id: &str) -> Result<Arc<dyn ModProvider>, RegistryError> {
        let id = normalize_id(id)?;
        self.mod_providers
            .get(&id)
            .map(|e| Arc::clone(&e.provider))
            .ok_or_else(|| RegistryError::NotFound(id))
    }

    pub fn get_game_provider(&self, id: &str) -> Result<Arc<dyn GameProvider + 'static>, RegistryError> {
        let id = normalize_id(id)?;
        self.game_providers
            .get(&id)
            .map(|g| Arc::clone(&g.game) as Arc<dyn GameProvider + 'static>)
            .ok_or_else(|| RegistryError::NotFound(id))
    }

    pub fn list_mod_providers(&self) -> Vec<(String, ProviderSource)> {
        self.mod_providers
            .values()
            .map(|e| (e.id.clone(), e.source.clone()))
            .collect()
    }

    pub fn list_games(&self) -> Vec<(String, ProviderSource, String)> {
        self.game_providers
            .values()
            .map(|g| (g.id.clone(), g.source.clone(), g.required_provider_id.clone()))
            .collect()
    }

    pub fn activate_game(&self, id: &str) -> Result<(), RegistryError> {
        let id = normalize_id(id)?;
        if !self.game_providers.contains_key(&id) {
            return Err(RegistryError::NotFound(id));
        }
        let mut active = self.active_game.lock().unwrap();
        println!("Activated game {}", &id);
        *active = Some(id);
        Ok(())
    }

    pub fn active_game(&self) -> Option<String> {
        self.active_game.lock().unwrap().clone()
    }

    pub fn active_game_required_provider(&self) -> Option<String> {
        let active = self.active_game();
        active.and_then(|id| {
            self.game_providers
                .get(&id)
                .map(|g| g.required_provider_id.clone())
        })
    }

    pub fn get_metadata(&self, id: &str) -> Result<GameMetadata, RegistryError> {
        let id = normalize_id(id)?;
        match self.game_providers.get(&id) {
            Some(game_entry) => {
                let metadata = game_entry.game.metadata().clone();
                Ok(metadata)
            }
            None => Err(RegistryError::NotFound(id)),
        }
    }

    pub async fn get_extended_info(&self, id: &str) -> Result<ModExtendedMetadata, RegistryError> {
            let id = normalize_id(id)?;
            let provider = self
                .active_game_required_provider()
                .ok_or_else(|| RegistryError::NotFound("No active game".to_string()))?;

            let provider_entry = self
                .mod_providers
                .get(&provider)
                .ok_or_else(|| RegistryError::NotFound(provider.clone()))?;
            let provider = Arc::clone(&provider_entry.provider);

            Ok(provider.get_extended_mod(&id).await)
    }

    #[cfg(debug_assertions)]
    pub fn debug_dump(&self) {
        println!("Context dump\n ---> Providers");
        for (id, provider) in self.mod_providers.iter() {
            println!("\t{} ({:?})", id, provider.source)
        }
        println!("\n ---> Games");
        for (id, game) in self.game_providers.iter() {
            println!("\t{} ({:?}) -> Depends on {}", id, game.source, game.required_provider_id)
        }
    }

}