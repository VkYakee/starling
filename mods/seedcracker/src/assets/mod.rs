//! Asset management for SeedCracker - handles translations and data files

use std::collections::HashMap;
use std::sync::OnceLock;

/// Asset manager for loading and caching translations
pub struct AssetManager {
    translations: HashMap<String, HashMap<String, String>>,
}

impl AssetManager {
    pub fn new() -> Self {
        let mut translations = HashMap::new();
        
        // Load default English translations
        let mut en_us = HashMap::new();
        en_us.insert("seedcracker.title".to_string(), "SeedCracker".to_string());
        en_us.insert("seedcracker.collecting".to_string(), "Collecting Data".to_string());
        en_us.insert("seedcracker.solving".to_string(), "Solving...".to_string());
        en_us.insert("seedcracker.cracked".to_string(), "Seed Cracked!".to_string());
        en_us.insert("seedcracker.structures".to_string(), "Structures".to_string());
        en_us.insert("seedcracker.biomes".to_string(), "Biomes".to_string());
        en_us.insert("seedcracker.bedrock".to_string(), "Bedrock".to_string());
        en_us.insert("seedcracker.pillars".to_string(), "End Pillars".to_string());
        en_us.insert("seedcracker.hashed_seed".to_string(), "Hashed Seed".to_string());
        en_us.insert("seedcracker.candidates".to_string(), "Candidates".to_string());
        en_us.insert("seedcracker.status".to_string(), "Status".to_string());
        translations.insert("en_us".to_string(), en_us);
        
        // Spanish translations
        let mut es_es = HashMap::new();
        es_es.insert("seedcracker.title".to_string(), "SeedCracker".to_string());
        es_es.insert("seedcracker.collecting".to_string(), "Recopilando Datos".to_string());
        es_es.insert("seedcracker.solving".to_string(), "Resolviendo...".to_string());
        es_es.insert("seedcracker.cracked".to_string(), "¡Semilla Descifrada!".to_string());
        es_es.insert("seedcracker.structures".to_string(), "Estructuras".to_string());
        es_es.insert("seedcracker.biomes".to_string(), "Biomas".to_string());
        es_es.insert("seedcracker.bedrock".to_string(), "Bedrock".to_string());
        es_es.insert("seedcracker.pillars".to_string(), "Pilares del End".to_string());
        es_es.insert("seedcracker.hashed_seed".to_string(), "Semilla Hash".to_string());
        es_es.insert("seedcracker.candidates".to_string(), "Candidatos".to_string());
        es_es.insert("seedcracker.status".to_string(), "Estado".to_string());
        translations.insert("es_es".to_string(), es_es);
        
        Self { translations }
    }
    
    /// Get a translation for the given key and language
    pub fn get(&self, lang: &str, key: &str) -> Option<&str> {
        self.translations.get(lang)?.get(key).map(|s| s.as_str())
    }
    
    /// Get translation with fallback to English
    pub fn get_or_fallback(&self, lang: &str, key: &str) -> &str {
        self.get(lang, key).or_else(|| self.get("en_us", key)).unwrap_or(key)
    }
}

static ASSET_MANAGER: OnceLock<AssetManager> = OnceLock::new();

/// Get the global asset manager instance
pub fn ASSET_MANAGER() -> &'static AssetManager {
    ASSET_MANAGER.get_or_init(AssetManager::new)
}

/// Translate a key using the global asset manager
pub fn tr(lang: &str, key: &str) -> &str {
    ASSET_MANAGER().get_or_fallback(lang, key)
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translations() {
        let mgr = AssetManager::new();
        assert_eq!(mgr.get_or_fallback("en_us", "seedcracker.title"), "SeedCracker");
        assert_eq!(mgr.get_or_fallback("es_es", "seedcracker.title"), "SeedCracker");
        assert_eq!(mgr.get_or_fallback("en_us", "unknown_key"), "unknown_key");
    }
}
