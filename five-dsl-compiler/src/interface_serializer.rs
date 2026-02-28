//! Interface CPI instruction-data serialization helpers (CPI-only).
//!
//! This module is intentionally scoped to interface/CPI usage so that
//! the core Five VM bytecode serialization remains unchanged.

use crate::type_checker::InterfaceSerializer;
use crate::TypeNode;
use five_protocol::Value;
use five_vm_mito::error::VMError;

/// Serialize discriminator bytes + arguments according to the requested serializer.
pub fn serialize_instruction_data(
    serializer: &InterfaceSerializer,
    discriminator_bytes: &[u8],
    param_types: &[TypeNode],
    args: &[Value],
    string_table: Option<&[Vec<u8>]>,
) -> Result<Vec<u8>, VMError> {
    if param_types.len() != args.len() {
        return Err(VMError::InvalidParameterCount);
    }

    let mut out = Vec::new();
    out.extend_from_slice(discriminator_bytes);

    match serializer {
        InterfaceSerializer::Raw => {
            // Raw mode: args must already be encoded as bytes in a single Value::Array(u8)
            // This keeps the legacy "caller provides bytes" behavior.
            if args.len() != 1 {
                return Err(VMError::InvalidParameterCount);
            }
            if let Value::Array(bytes_id) = args[0] {
                // In production this would look up the temp buffer entry.
                // For offline serialization tests, treat the array id as a placeholder error.
                let _ = bytes_id;
                return Err(VMError::InvalidOperation);
            } else {
                return Err(VMError::InvalidOperation);
            }
        }
        InterfaceSerializer::Borsh => {
            for (ty, arg) in param_types.iter().zip(args.iter()) {
                borsh_encode_value(&mut out, ty, arg, string_table)?;
            }
        }
        InterfaceSerializer::Bincode => {
            for (ty, arg) in param_types.iter().zip(args.iter()) {
                bincode_encode_value(&mut out, ty, arg, string_table)?;
            }
        }
    }

    Ok(out)
}

fn borsh_encode_value(
    buf: &mut Vec<u8>,
    ty: &TypeNode,
    value: &Value,
    string_table: Option<&[Vec<u8>]>,
) -> Result<(), VMError> {
    match (ty, value) {
        (TypeNode::Primitive(name), Value::U64(v)) if name == "u64" => {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        (TypeNode::Primitive(name), Value::U64(v)) if name == "u32" => {
            if *v > u32::MAX as u64 {
                return Err(VMError::InvalidOperation);
            }
            buf.extend_from_slice(&(*v as u32).to_le_bytes());
        }
        (TypeNode::Primitive(name), Value::U64(v)) if name == "u16" => {
            if *v > u16::MAX as u64 {
                return Err(VMError::InvalidOperation);
            }
            buf.extend_from_slice(&(*v as u16).to_le_bytes());
        }
        (TypeNode::Primitive(name), Value::U8(v)) if name == "u8" => buf.push(*v),
        (TypeNode::Primitive(name), Value::Bool(v)) if name == "bool" => {
            buf.push(if *v { 1 } else { 0 })
        }
        (TypeNode::Primitive(name), Value::String(idx)) if name == "string" => {
            let table = string_table.ok_or(VMError::InvalidOperation)?;
            let bytes = table.get(*idx as usize).ok_or(VMError::InvalidOperation)?;
            let len = bytes.len() as u32;
            buf.extend_from_slice(&len.to_le_bytes());
            buf.extend_from_slice(bytes);
        }
        (TypeNode::Primitive(name), Value::Pubkey(pk)) if name == "pubkey" => {
            buf.extend_from_slice(pk);
        }
        _ => return Err(VMError::InvalidOperation),
    }
    Ok(())
}

fn bincode_encode_value(
    buf: &mut Vec<u8>,
    ty: &TypeNode,
    value: &Value,
    string_table: Option<&[Vec<u8>]>,
) -> Result<(), VMError> {
    match (ty, value) {
        (TypeNode::Primitive(name), Value::U64(v)) if name == "u64" => {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        (TypeNode::Primitive(name), Value::U64(v)) if name == "u32" => {
            if *v > u32::MAX as u64 {
                return Err(VMError::InvalidOperation);
            }
            buf.extend_from_slice(&(*v as u32).to_le_bytes());
        }
        (TypeNode::Primitive(name), Value::U64(v)) if name == "u16" => {
            if *v > u16::MAX as u64 {
                return Err(VMError::InvalidOperation);
            }
            buf.extend_from_slice(&(*v as u16).to_le_bytes());
        }
        (TypeNode::Primitive(name), Value::U8(v)) if name == "u8" => buf.push(*v),
        (TypeNode::Primitive(name), Value::Bool(v)) if name == "bool" => {
            buf.push(if *v { 1 } else { 0 })
        }
        (TypeNode::Primitive(name), Value::String(idx)) if name == "string" => {
            let table = string_table.ok_or(VMError::InvalidOperation)?;
            let bytes = table.get(*idx as usize).ok_or(VMError::InvalidOperation)?;
            let len = bytes.len() as u32;
            buf.extend_from_slice(&len.to_le_bytes());
            buf.extend_from_slice(bytes);
        }
        (TypeNode::Primitive(name), Value::Pubkey(pk)) if name == "pubkey" => {
            buf.extend_from_slice(pk);
        }
        _ => return Err(VMError::InvalidOperation),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borsh_serializes_discriminator_and_args() {
        let discriminator = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let params = vec![
            TypeNode::Primitive("u64".to_string()),
            TypeNode::Primitive("pubkey".to_string()),
        ];
        let args = vec![Value::U64(42), Value::Pubkey([9u8; 32])];

        let out = serialize_instruction_data(
            &InterfaceSerializer::Borsh,
            &discriminator,
            &params,
            &args,
            None,
        )
        .unwrap();

        // discriminator + u64 + pubkey
        assert_eq!(out.len(), 8 + 8 + 32);
        assert_eq!(&out[0..8], &discriminator[..]);
        assert_eq!(&out[8..16], &42u64.to_le_bytes());
        assert_eq!(&out[16..48], &[9u8; 32]);
    }

    #[test]
    fn bincode_serializes_discriminator_and_args() {
        let discriminator = vec![0xAA];
        let params = vec![TypeNode::Primitive("u32".to_string())];
        let args = vec![Value::U64(0x11223344)];

        let out = serialize_instruction_data(
            &InterfaceSerializer::Bincode,
            &discriminator,
            &params,
            &args,
            None,
        )
        .unwrap();

        assert_eq!(out[0], 0xAA);
        assert_eq!(&out[1..5], &0x11223344u32.to_le_bytes());
    }

    #[test]
    fn spl_token_mint_to_serialization_matches_layout() {
        // SPL Token mint_to: discriminator (7) + amount (u64 LE).
        let discriminator = vec![7u8];
        let params = vec![TypeNode::Primitive("u64".to_string())];
        let args = vec![Value::U64(1_000)];

        let out = serialize_instruction_data(
            &InterfaceSerializer::Bincode,
            &discriminator,
            &params,
            &args,
            None,
        )
        .unwrap();

        let mut expected = vec![7u8];
        expected.extend_from_slice(&1_000u64.to_le_bytes());
        assert_eq!(out, expected);
    }

    #[test]
    fn anchor_borsh_serialization_with_discriminator_bytes() {
        let discriminator = vec![0xA1, 0xB2, 0xC3, 0xD4, 0xE5, 0xF6, 0x07, 0x18];
        let params = vec![
            TypeNode::Primitive("u64".to_string()),
            TypeNode::Primitive("pubkey".to_string()),
        ];
        let args = vec![Value::U64(500), Value::Pubkey([0xAB; 32])];

        let out = serialize_instruction_data(
            &InterfaceSerializer::Borsh,
            &discriminator,
            &params,
            &args,
            None,
        )
        .unwrap();

        let mut expected = discriminator.clone();
        expected.extend_from_slice(&500u64.to_le_bytes());
        expected.extend_from_slice(&[0xAB; 32]);
        assert_eq!(out, expected);
    }

    #[test]
    fn borsh_serializes_string_from_table() {
        let discriminator = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let params = vec![TypeNode::Primitive("string".to_string())];
        let args = vec![Value::String(0)];
        let string_table = vec![b"vault".to_vec()];

        let out = serialize_instruction_data(
            &InterfaceSerializer::Borsh,
            &discriminator,
            &params,
            &args,
            Some(&string_table),
        )
        .unwrap();

        let mut expected = discriminator.clone();
        expected.extend_from_slice(&(5u32.to_le_bytes()));
        expected.extend_from_slice(b"vault");
        assert_eq!(out, expected);
    }
}
