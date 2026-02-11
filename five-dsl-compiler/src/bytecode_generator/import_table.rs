/// Specification for an imported Five bytecode account
#[derive(Debug, Clone, PartialEq)]
pub enum ImportSpec {
    /// Direct Five bytecode account address (as base58 string)
    /// Validation happens at runtime in the VM
    Address(String),
    /// PDA seeds for deriving Five bytecode account address
    PdaSeeds(Vec<Vec<u8>>),
}

/// Single entry in the import table
#[derive(Debug, Clone)]
pub struct ImportEntry {
    pub spec: ImportSpec,
    pub function_name: String,
}

/// Import verification table for bytecode accounts
/// Stores both direct addresses and PDA seeds for Five bytecode imports
#[derive(Debug, Clone)]
pub struct ImportTable {
    entries: Vec<ImportEntry>,
}

impl ImportTable {
    /// Create a new empty import table
    pub fn new() -> Self {
        ImportTable {
            entries: Vec::new(),
        }
    }

    /// Add an import by direct account address
    /// Address is stored as base58 string; validation happens at runtime in the VM
    pub fn add_import_by_address(&mut self, address: &str, function_name: String) {
        self.entries.push(ImportEntry {
            spec: ImportSpec::Address(address.to_string()),
            function_name,
        });
    }

    /// Add an import by PDA seeds
    pub fn add_import_by_seeds(&mut self, seeds: Vec<Vec<u8>>, function_name: String) {
        self.entries.push(ImportEntry {
            spec: ImportSpec::PdaSeeds(seeds),
            function_name,
        });
    }

    /// Serialize import table to bytecode format
    /// Format:
    /// [import_count: u8]
    /// For each import:
    ///   [import_type: u8]  (0 = address, 1 = PDA seeds)
    ///   If address:
    ///     [pubkey: 32 bytes]
    ///   If PDA seeds:
    ///     [seed_count: u8]
    ///     For each seed:
    ///       [seed_len: u8]
    ///       [seed_bytes: variable]
    ///   [function_name_len: u8]
    ///   [function_name: variable]
    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();

        // Import count (max 255)
        bytes.push(self.entries.len() as u8);

        // Each import entry
        for entry in &self.entries {
            match &entry.spec {
                ImportSpec::Address(pubkey) => {
                    // Type: 0 = address
                    bytes.push(0);
                    // Address (32 bytes, base58 decoded)
                    let decoded = bs58::decode(pubkey)
                        .into_vec()
                        .map_err(|_| format!("Invalid base58 pubkey in import: {}", pubkey))?;
                    if decoded.len() != 32 {
                        return Err(format!(
                            "Invalid pubkey length in import (expected 32 bytes, got {}): {}",
                            decoded.len(),
                            pubkey
                        ));
                    }
                    bytes.extend_from_slice(&decoded);
                }
                ImportSpec::PdaSeeds(seeds) => {
                    // Type: 1 = PDA seeds
                    bytes.push(1);
                    // Seed count (max 255)
                    bytes.push(seeds.len() as u8);
                    // Each seed
                    for seed in seeds {
                        // Seed length
                        bytes.push(seed.len() as u8);
                        // Seed bytes
                        bytes.extend_from_slice(seed);
                    }
                }
            }

            // Function name
            bytes.push(entry.function_name.len() as u8);
            bytes.extend_from_slice(entry.function_name.as_bytes());
        }

        Ok(bytes)
    }

    /// Check if the table is empty (no imports)
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get number of imports
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Get all entries (for testing)
    pub fn entries(&self) -> &[ImportEntry] {
        &self.entries
    }
}

impl Default for ImportTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_import_table() {
        let table = ImportTable::new();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);

        let serialized = table.serialize().expect("serialize");
        assert_eq!(serialized.len(), 1);
        assert_eq!(serialized[0], 0); // import_count = 0
    }

    #[test]
    fn test_add_import_by_address() {
        let mut table = ImportTable::new();
        let address = "11111111111111111111111111111111"; // System program address

        table.add_import_by_address(address, "test_func".to_string());
        assert_eq!(table.len(), 1);
        assert!(!table.is_empty());
    }

    #[test]
    fn test_add_import_by_address_multiple() {
        let mut table = ImportTable::new();
        let address = "11111111111111111111111111111111";

        table.add_import_by_address(address, "test_func1".to_string());
        table.add_import_by_address(address, "test_func2".to_string());

        assert_eq!(table.len(), 2);
        assert!(!table.is_empty());
    }

    #[test]
    fn test_add_import_by_seeds() {
        let mut table = ImportTable::new();
        let seeds = vec![
            b"vault".to_vec(),
            b"user".to_vec(),
        ];

        table.add_import_by_seeds(seeds, "pda_func".to_string());
        assert_eq!(table.len(), 1);
        assert!(!table.is_empty());
    }

    #[test]
    fn test_serialize_address_import() {
        let mut table = ImportTable::new();
        let address = "11111111111111111111111111111111";
        table.add_import_by_address(address, "transfer".to_string());

        let serialized = table.serialize().expect("serialize");
        assert!(!serialized.is_empty());
        assert_eq!(serialized[0], 1); // import_count = 1
        assert_eq!(serialized[1], 0); // import_type = 0 (address)

        // Next 32 bytes should be the address string
        // Then name length and name
        assert_eq!(serialized[34], "transfer".len() as u8);
    }

    #[test]
    fn test_serialize_pda_seeds_import() {
        let mut table = ImportTable::new();
        let seeds = vec![
            b"vault".to_vec(),
            b"user123".to_vec(),
        ];

        table.add_import_by_seeds(seeds.clone(), "vault_func".to_string());

        let serialized = table.serialize().expect("serialize");
        assert_eq!(serialized[0], 1); // import_count = 1
        assert_eq!(serialized[1], 1); // import_type = 1 (PDA seeds)
        assert_eq!(serialized[2], 2); // seed_count = 2

        // First seed: length + data
        assert_eq!(serialized[3], 5); // "vault".len() = 5
        assert_eq!(&serialized[4..9], b"vault");

        // Second seed: length + data
        assert_eq!(serialized[9], 7); // "user123".len() = 7
        assert_eq!(&serialized[10..17], b"user123");
    }

    #[test]
    fn test_multiple_mixed_imports() {
        let mut table = ImportTable::new();

        // Add address import
        let addr1 = "11111111111111111111111111111111";
        table.add_import_by_address(addr1, "func1".to_string());

        // Add PDA import
        let seeds = vec![b"seed1".to_vec(), b"seed2".to_vec()];
        table.add_import_by_seeds(seeds, "func2".to_string());

        // Add another address import
        let addr2 = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
        table.add_import_by_address(addr2, "func3".to_string());

        assert_eq!(table.len(), 3);

        let serialized = table.serialize().expect("serialize");
        assert_eq!(serialized[0], 3); // import_count = 3
    }

    #[test]
    fn test_serialize_roundtrip() {
        let mut table = ImportTable::new();
        let address = "11111111111111111111111111111111";
        table.add_import_by_address(address, "myfunction".to_string());

        let serialized = table.serialize().expect("serialize");

        // Verify structure
        assert_eq!(serialized[0], 1); // 1 import
        assert_eq!(serialized[1], 0); // type = address
        // [2..34] = 32-byte address string
        assert_eq!(serialized[34], 10); // function name length
        assert_eq!(&serialized[35..45], b"myfunction");
    }

    #[test]
    fn test_max_seeds() {
        let mut table = ImportTable::new();

        // Create 4 seeds (max we support in VM verification)
        let seeds = vec![
            b"s1".to_vec(),
            b"s2".to_vec(),
            b"s3".to_vec(),
            b"s4".to_vec(),
        ];

        table.add_import_by_seeds(seeds, "func".to_string());
        assert_eq!(table.len(), 1);

        let serialized = table.serialize().expect("serialize");
        assert_eq!(serialized[2], 4); // seed_count = 4
    }

    #[test]
    fn test_long_function_name() {
        let mut table = ImportTable::new();
        let address = "11111111111111111111111111111111";
        let long_name = "this_is_a_very_long_function_name_for_testing".to_string();

        table.add_import_by_address(address, long_name.clone());

        let serialized = table.serialize().expect("serialize");
        let name_len = serialized[34] as usize;
        assert_eq!(name_len, long_name.len());
        assert_eq!(&serialized[35..35 + name_len], long_name.as_bytes());
    }

    #[test]
    fn test_long_seed_value() {
        let mut table = ImportTable::new();

        // Create a seed with many bytes
        let long_seed = vec![0x42u8; 100];
        let seeds = vec![long_seed];

        table.add_import_by_seeds(seeds, "func".to_string());

        let serialized = table.serialize().expect("serialize");
        assert_eq!(serialized[2], 1); // 1 seed
        assert_eq!(serialized[3], 100); // seed length = 100
    }

    #[test]
    fn test_duplicate_imports_allowed() {
        let mut table = ImportTable::new();
        let address = "11111111111111111111111111111111";

        // Same address with different function names should be allowed
        table.add_import_by_address(address, "func1".to_string());
        table.add_import_by_address(address, "func2".to_string());

        assert_eq!(table.len(), 2);
    }

    #[test]
    fn test_default_creates_empty_table() {
        let table = ImportTable::default();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }
}
