//! Five File Format Implementation
//!
//! This module handles the .five file format which contains both bytecode and ABI
//! in a compact binary format for efficient cross-contract imports.

use crate::bytecode_generator::types::{ABIField, ABIFunction, ABIParameter, FIVEABI};
use five_vm_mito::error::{Result, VMError};
use std::io::{Cursor, Read};

/// Magic header for .five files
const FIVE_MAGIC: &[u8; 4] = b"FIVE";
const FIVE_VERSION: u8 = 0x01;

/// Type IDs for compact ABI encoding
const TYPE_VOID: u8 = 0;
const TYPE_U64: u8 = 1;
const TYPE_BOOL: u8 = 2;
const TYPE_ACCOUNT: u8 = 3;
const TYPE_STRING: u8 = 4;
const TYPE_PUBKEY: u8 = 5;
const TYPE_U8: u8 = 6;
const TYPE_I64: u8 = 7;
const TYPE_U128: u8 = 8;

/// Maximum name lengths to ensure compact encoding
const MAX_PROGRAM_NAME_LEN: usize = 32;
const MAX_FUNCTION_NAME_LEN: usize = 16;
const MAX_FIELD_NAME_LEN: usize = 16;
const MAX_PARAM_NAME_LEN: usize = 16;
const MAX_FUNCTIONS: usize = 64;
const MAX_FIELDS: usize = 64;

/// Attribute bit flags for parameters
const ATTR_SIGNER: u8 = 0x01;
const ATTR_MUT: u8 = 0x02;
const ATTR_INIT: u8 = 0x04;

/// A complete .five file containing ABI and bytecode
#[derive(Debug, Clone)]
pub struct FiveFile {
    pub abi: FIVEABI,
    pub bytecode: Vec<u8>,
}

impl FiveFile {
    /// Create a new .five file
    pub fn new(abi: FIVEABI, bytecode: Vec<u8>) -> Self {
        Self { abi, bytecode }
    }

    /// Load a .five file from disk
    pub fn load(path: &str) -> Result<Self> {
        let data = std::fs::read(path).map_err(|_| VMError::InvalidOperation)?;
        Self::from_bytes(&data)
    }

    /// Save a .five file to disk
    pub fn save(&self, path: &str) -> Result<()> {
        let data = self.to_bytes()?;
        std::fs::write(path, data).map_err(|_| VMError::InvalidOperation)?;
        Ok(())
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);

        // Read and validate magic header
        let mut magic = [0u8; 4];
        cursor
            .read_exact(&mut magic)
            .map_err(|_| VMError::InvalidScript)?;
        if &magic != FIVE_MAGIC {
            return Err(VMError::InvalidScript);
        }

        // Read version
        let mut version = [0u8; 1];
        cursor
            .read_exact(&mut version)
            .map_err(|_| VMError::InvalidScript)?;
        if version[0] != FIVE_VERSION {
            return Err(VMError::InvalidScript);
        }

        // Read section lengths
        let abi_length = read_u32(&mut cursor)?;
        let bytecode_length = read_u32(&mut cursor)?;

        // Read ABI section
        let mut abi_data = vec![0u8; abi_length as usize];
        cursor
            .read_exact(&mut abi_data)
            .map_err(|_| VMError::InvalidScript)?;
        let abi = Self::deserialize_abi(&abi_data)?;

        // Read bytecode section
        let mut bytecode = vec![0u8; bytecode_length as usize];
        cursor
            .read_exact(&mut bytecode)
            .map_err(|_| VMError::InvalidScript)?;

        Ok(FiveFile { abi, bytecode })
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut result = Vec::new();

        // Write magic header
        result.extend_from_slice(FIVE_MAGIC);

        // Write version
        result.push(FIVE_VERSION);

        // Serialize ABI
        let abi_data = self.serialize_abi()?;

        // Write section lengths
        result.extend_from_slice(&(abi_data.len() as u32).to_le_bytes());
        result.extend_from_slice(&(self.bytecode.len() as u32).to_le_bytes());

        // Write sections
        result.extend_from_slice(&abi_data);
        result.extend_from_slice(&self.bytecode);

        Ok(result)
    }

    /// Serialize ABI to compact binary format
    fn serialize_abi(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();

        // Program header with name length validation
        let name_bytes = self.abi.program_name.as_bytes();
        if name_bytes.len() > MAX_PROGRAM_NAME_LEN {
            return Err(VMError::InvalidScript);
        }
        data.push(name_bytes.len() as u8);
        data.extend_from_slice(name_bytes);

        if self.abi.functions.len() > MAX_FUNCTIONS {
            return Err(VMError::InvalidScript);
        }
        if self.abi.fields.len() > MAX_FIELDS {
            return Err(VMError::InvalidScript);
        }
        data.push(self.abi.functions.len() as u8);
        data.push(self.abi.fields.len() as u8);

        // Function entries with name length validation
        for function in &self.abi.functions {
            let name_bytes = function.name.as_bytes();
            if name_bytes.len() > MAX_FUNCTION_NAME_LEN {
                return Err(VMError::InvalidScript);
            }
            data.push(name_bytes.len() as u8);
            data.extend_from_slice(name_bytes);
            data.push(function.index);

            // Use VLE encoding for bytecode offse

            let bytecode_offset = function.bytecode_offset;
            write_vle(&mut data, bytecode_offset);

            data.push(function.parameters.len() as u8);
            data.push(if function.is_public { 1 } else { 0 });
            data.push(Self::type_to_id(
                function.return_type.as_deref().unwrap_or("void"),
            )?);

            // Function parameters with name length validation
            for param in &function.parameters {
                let param_name_bytes = param.name.as_bytes();
                if param_name_bytes.len() > MAX_PARAM_NAME_LEN {
                    return Err(VMError::InvalidScript);
                }
                data.push(param_name_bytes.len() as u8);
                data.extend_from_slice(param_name_bytes);
                data.push(Self::type_to_id(&param.param_type)?);
                data.push(if param.is_account { 1 } else { 0 });

                // Encode attributes as bit flags
                let mut attrs = 0u8;
                if param.attributes.contains(&"signer".to_string()) {
                    attrs |= ATTR_SIGNER;
                }
                if param.attributes.contains(&"mut".to_string()) {
                    attrs |= ATTR_MUT;
                }
                if param.attributes.contains(&"init".to_string()) {
                    attrs |= ATTR_INIT;
                }
                data.push(attrs);
            }
        }

        // Field entries with name length validation
        for field in &self.abi.fields {
            let name_bytes = field.name.as_bytes();
            if name_bytes.len() > MAX_FIELD_NAME_LEN {
                return Err(VMError::InvalidScript);
            }
            data.push(name_bytes.len() as u8);
            data.extend_from_slice(name_bytes);

            // Use VLE encoding for memory offset

            let memory_offset = field.memory_offset;
            write_vle(&mut data, memory_offset);

            data.push(Self::type_to_id(&field.field_type)?);
            data.push(if field.is_mutable { 1 } else { 0 });
        }

        Ok(data)
    }

    /// Deserialize ABI from compact binary format
    fn deserialize_abi(data: &[u8]) -> Result<FIVEABI> {
        let mut cursor = Cursor::new(data);

        // Read program header
        let name_len = read_u8(&mut cursor)? as usize;
        let program_name = read_string(&mut cursor, name_len)?;
        let function_count = read_u8(&mut cursor)? as usize;
        if function_count > MAX_FUNCTIONS {
            return Err(VMError::InvalidScript);
        }
        let field_count = read_u8(&mut cursor)? as usize;
        if field_count > MAX_FIELDS {
            return Err(VMError::InvalidScript);
        }

        // Read functions
        let mut functions = Vec::with_capacity(function_count);
        for _ in 0..function_count {
            let name_len = read_u8(&mut cursor)? as usize;
            let name = read_string(&mut cursor, name_len)?;
            let index = read_u8(&mut cursor)?;
            let bytecode_offset = read_vle(&mut cursor)?;
            let param_count = read_u8(&mut cursor)? as usize;
            let is_public = read_u8(&mut cursor)? != 0;
            let return_type_id = read_u8(&mut cursor)?;

            // Read parameters
            let mut parameters = Vec::with_capacity(param_count);
            for _ in 0..param_count {
                let param_name_len = read_u8(&mut cursor)? as usize;
                let param_name = read_string(&mut cursor, param_name_len)?;
                let type_id = read_u8(&mut cursor)?;
                let is_account = read_u8(&mut cursor)? != 0;
                let attrs_byte = read_u8(&mut cursor)?;

                // Decode attribute flags
                let mut attributes = Vec::new();
                if (attrs_byte & ATTR_SIGNER) != 0 {
                    attributes.push("signer".to_string());
                }
                if (attrs_byte & ATTR_MUT) != 0 {
                    attributes.push("mut".to_string());
                }
                if (attrs_byte & ATTR_INIT) != 0 {
                    attributes.push("init".to_string());
                }

                parameters.push(ABIParameter {
                    name: param_name,
                    param_type: Self::id_to_type(type_id)?,
                    is_account,
                    attributes,
                });
            }

            functions.push(ABIFunction {
                name,
                index,
                parameters,
                return_type: if return_type_id == TYPE_VOID {
                    None
                } else {
                    Some(Self::id_to_type(return_type_id)?)
                },
                is_public,
                bytecode_offset,
            });
        }

        // Read fields
        let mut fields = Vec::with_capacity(field_count);
        for _ in 0..field_count {
            let name_len = read_u8(&mut cursor)? as usize;
            let name = read_string(&mut cursor, name_len)?;
            let memory_offset = read_vle(&mut cursor)?;
            let type_id = read_u8(&mut cursor)?;
            let is_mutable = read_u8(&mut cursor)? != 0;

            fields.push(ABIField {
                name,
                field_type: Self::id_to_type(type_id)?,
                is_mutable,
                memory_offset,
            });
        }

        Ok(FIVEABI {
            program_name,
            functions,
            fields,
            version: "1.0".to_string(),
        })
    }

    /// Convert type string to type ID
    fn type_to_id(type_str: &str) -> Result<u8> {
        match type_str {
            "void" => Ok(TYPE_VOID),
            "u64" => Ok(TYPE_U64),
            "bool" => Ok(TYPE_BOOL),
            "Account" => Ok(TYPE_ACCOUNT),
            "String" => Ok(TYPE_STRING),
            "Pubkey" => Ok(TYPE_PUBKEY),
            "u8" => Ok(TYPE_U8),
            "i64" => Ok(TYPE_I64),
            "u128" => Ok(TYPE_U128),
            _ => Err(VMError::InvalidScript),
        }
    }

    /// Convert type ID to type string
    fn id_to_type(type_id: u8) -> Result<String> {
        match type_id {
            TYPE_VOID => Ok("void".to_string()),
            TYPE_U64 => Ok("u64".to_string()),
            TYPE_BOOL => Ok("bool".to_string()),
            TYPE_ACCOUNT => Ok("Account".to_string()),
            TYPE_STRING => Ok("String".to_string()),
            TYPE_PUBKEY => Ok("Pubkey".to_string()),
            TYPE_U8 => Ok("u8".to_string()),
            TYPE_I64 => Ok("i64".to_string()),
            TYPE_U128 => Ok("u128".to_string()),
            _ => Err(VMError::InvalidScript),
        }
    }
}

// Helper functions for reading binary data
fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| VMError::InvalidScript)?;
    Ok(buf[0])
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| VMError::InvalidScript)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_string(cursor: &mut Cursor<&[u8]>, len: usize) -> Result<String> {
    let mut buf = vec![0u8; len];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| VMError::InvalidScript)?;
    String::from_utf8(buf).map_err(|_| VMError::InvalidScript)
}

/// Write variable-length encoded integer
fn write_vle(data: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80; // Set continuation bit
        }
        data.push(byte);
        if value == 0 {
            break;
        }
    }
}

/// Read variable-length encoded integer
fn read_vle(cursor: &mut Cursor<&[u8]>) -> Result<u64> {
    let mut result = 0u64;
    let mut shift = 0;

    loop {
        let byte = read_u8(cursor)?;
        result |= ((byte & 0x7F) as u64) << shift;

        if (byte & 0x80) == 0 {
            break;
        }

        shift += 7;
        if shift >= 64 {
            return Err(VMError::InvalidScript);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode_generator::types::{ABIField, ABIFunction, ABIParameter};

    #[test]
    fn test_five_file_roundtrip() {
        let abi = FIVEABI {
            program_name: "test_contract".to_string(),
            functions: vec![ABIFunction {
                name: "transfer".to_string(),
                index: 0,
                parameters: vec![ABIParameter {
                    name: "amount".to_string(),
                    param_type: "u64".to_string(),
                    is_account: false,
                    attributes: vec![],
                }],
                return_type: Some("bool".to_string()),
                is_public: true,
                bytecode_offset: 2,
            }],
            fields: vec![ABIField {
                name: "balance".to_string(),
                field_type: "u64".to_string(),
                is_mutable: true,
                memory_offset: 8,
            }],
            version: "1.0".to_string(),
        };

        let bytecode = vec![0x01, 0x02, 0x03, 0x04];
        let five_file = FiveFile::new(abi.clone(), bytecode.clone());

        // Serialize and deserialize
        let bytes = five_file.to_bytes().unwrap();
        let loaded_file = FiveFile::from_bytes(&bytes).unwrap();

        // Verify
        assert_eq!(loaded_file.abi.program_name, abi.program_name);
        assert_eq!(loaded_file.abi.functions.len(), abi.functions.len());
        assert_eq!(loaded_file.abi.fields.len(), abi.fields.len());
        assert_eq!(loaded_file.bytecode, bytecode);
        assert_eq!(loaded_file.abi.functions[0].bytecode_offset, 2);
        assert_eq!(loaded_file.abi.fields[0].memory_offset, 8);
    }

    #[test]
    fn test_function_count_limit() {
        let oversized = (MAX_FUNCTIONS + 1) as u8;
        let abi_data = vec![0, oversized, 0];
        let mut data = Vec::new();
        data.extend_from_slice(FIVE_MAGIC);
        data.push(FIVE_VERSION);
        data.extend_from_slice(&(abi_data.len() as u32).to_le_bytes());
        data.extend_from_slice(&(0u32).to_le_bytes());
        data.extend_from_slice(&abi_data);
        assert!(FiveFile::from_bytes(&data).is_err());
    }

    #[test]
    fn test_field_count_limit() {
        let oversized = (MAX_FIELDS + 1) as u8;
        let abi_data = vec![0, 0, oversized];
        let mut data = Vec::new();
        data.extend_from_slice(FIVE_MAGIC);
        data.push(FIVE_VERSION);
        data.extend_from_slice(&(abi_data.len() as u32).to_le_bytes());
        data.extend_from_slice(&(0u32).to_le_bytes());
        data.extend_from_slice(&abi_data);
        assert!(FiveFile::from_bytes(&data).is_err());
    }

    #[test]
    fn test_unknown_type_errors() {
        let abi = FIVEABI {
            program_name: "test_contract".to_string(),
            functions: vec![ABIFunction {
                name: "do_something".to_string(),
                index: 0,
                parameters: vec![ABIParameter {
                    name: "x".to_string(),
                    param_type: "mystery".to_string(),
                    is_account: false,
                    attributes: vec![],
                }],
                return_type: None,
                is_public: true,
                bytecode_offset: 0,
            }],
            fields: vec![],
            version: "1.0".to_string(),
        };

        let file = FiveFile::new(abi, vec![]);
        assert!(file.to_bytes().is_err());
    }

    #[test]
    fn test_load_io_error() {
        let result = FiveFile::load("nonexistent_file.five");
        assert!(matches!(result, Err(VMError::InvalidOperation)));
    }

    #[test]
    fn test_save_io_error() {
        let abi = FIVEABI {
            program_name: "x".to_string(),
            functions: vec![],
            fields: vec![],
            version: "1.0".to_string(),
        };
        let file = FiveFile::new(abi, vec![]);
        let result = file.save("missing_dir/output.five");
        assert!(matches!(result, Err(VMError::InvalidOperation)));
    }
}
