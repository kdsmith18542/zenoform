use std::collections::HashMap;

pub fn derive_seed_hash(world_id: &str, seed: i32) -> String {
    use starknet_crypto::{Felt, poseidon_hash_many};

    let world_bytes = world_id.as_bytes();
    let mut data = Vec::new();

    for chunk in world_bytes.chunks(32) {
        let mut padded = [0u8; 32];
        let len = chunk.len().min(32);
        padded[..len].copy_from_slice(&chunk[..len]);
        data.push(Felt::from_bytes_be(&padded));
    }

    data.push(Felt::from(seed as u64));

    let hash = poseidon_hash_many(&data);
    format!("0x{:x}", hash)
}

pub fn derive_module_hash(module_id: &str, content_hash: &str) -> String {
    use starknet_crypto::{Felt, poseidon_hash_many};

    let id_bytes = module_id.as_bytes();
    let mut data = Vec::new();

    let mut id_padded = [0u8; 32];
    let len = id_bytes.len().min(32);
    id_padded[..len].copy_from_slice(&id_bytes[..len]);
    data.push(Felt::from_bytes_be(&id_padded));

    let mut ch_padded = [0u8; 32];
    let ch_bytes = content_hash.strip_prefix("0x").unwrap_or(content_hash).as_bytes();
    let ch_len = ch_bytes.len().min(32);
    ch_padded[..ch_len].copy_from_slice(&ch_bytes[..ch_len]);
    data.push(Felt::from_bytes_be(&ch_padded));

    let hash = poseidon_hash_many(&data);
    format!("0x{:x}", hash)
}

#[derive(Debug, Clone)]
pub struct ModuleEntry {
    pub module_id: String,
    pub module_hash: String,
    pub content_description: String,
}

#[derive(Debug, Clone, Default)]
pub struct ModuleRegistry {
    modules: HashMap<String, ModuleEntry>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self { modules: HashMap::new() }
    }

    pub fn register(&mut self, module_id: String, content_description: String) -> String {
        let content_hash = format!("0x{}", blake3::hash(content_description.as_bytes()).to_hex());
        let module_hash = derive_module_hash(&module_id, &content_hash);

        self.modules.insert(
            module_id.clone(),
            ModuleEntry { module_id: module_id.clone(), module_hash: module_hash.clone(), content_description },
        );

        module_hash
    }

    pub fn get(&self, module_id: &str) -> Option<&ModuleEntry> {
        self.modules.get(module_id)
    }

    pub fn get_hash(&self, module_id: &str) -> Option<&str> {
        self.modules.get(module_id).map(|e| e.module_hash.as_str())
    }

    pub fn list_modules(&self) -> Vec<&ModuleEntry> {
        self.modules.values().collect()
    }

    pub fn len(&self) -> usize {
        self.modules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    pub fn validate_module(&self, module_id: &str, module_hash: &str) -> bool {
        self.modules.get(module_id).map(|e| e.module_hash == module_hash).unwrap_or(false)
    }
}

pub fn default_registry() -> ModuleRegistry {
    let mut registry = ModuleRegistry::new();

    registry.register(
        "terrain.fixed_noise.v1".to_string(),
        "Fixed-point deterministic terrain generation using value noise and fractal octave layering with Q16.16 arithmetic, 12-biome classification, and 8 resource types. Uses Poseidon commitment hashing for proof-friendly output commitment.".to_string(),
    );

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_hash_deterministic() {
        let h1 = derive_seed_hash("test", 42);
        let h2 = derive_seed_hash("test", 42);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_seed_hash_different_seeds_differ() {
        let h1 = derive_seed_hash("test", 1);
        let h2 = derive_seed_hash("test", 2);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_seed_hash_different_worlds_differ() {
        let h1 = derive_seed_hash("world-a", 42);
        let h2 = derive_seed_hash("world-b", 42);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_module_hash_deterministic() {
        let h1 = derive_module_hash("terrain.v1", "content hash");
        let h2 = derive_module_hash("terrain.v1", "content hash");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_module_registry_register_and_get() {
        let mut reg = ModuleRegistry::new();
        let hash = reg.register("test.module".to_string(), "Test module content".to_string());
        assert!(reg.get("test.module").is_some());
        assert_eq!(reg.get_hash("test.module"), Some(hash.as_str()));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_module_registry_validate() {
        let mut reg = ModuleRegistry::new();
        let hash = reg.register("test.module".to_string(), "content".to_string());
        assert!(reg.validate_module("test.module", &hash));
        assert!(!reg.validate_module("test.module", "wrong"));
        assert!(!reg.validate_module("unknown", "hash"));
    }

    #[test]
    fn test_default_registry_has_terrain() {
        let reg = default_registry();
        assert!(reg.get("terrain.fixed_noise.v1").is_some());
    }
}
