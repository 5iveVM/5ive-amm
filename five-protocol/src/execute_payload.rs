use alloc::string::String;
use alloc::vec::Vec;

/// Canonical typed parameter representation for execute payload encoding.
#[derive(Clone, Debug, PartialEq)]
pub enum TypedParam {
    U8(u8),
    U64(u64),
    Bool(bool),
    Pubkey([u8; 32]),
    String(String),
    Account(u8),
}

/// Builder for canonical execute payloads:
/// [function_index:u32 LE][param_count:u32 LE][typed params...]
#[derive(Default, Clone, Debug)]
pub struct ExecutePayloadBuilder {
    function_index: u32,
    params: Vec<TypedParam>,
}

impl ExecutePayloadBuilder {
    #[inline]
    pub fn new(function_index: u32) -> Self {
        Self {
            function_index,
            params: Vec::new(),
        }
    }

    #[inline]
    pub fn with_params(mut self, params: &[TypedParam]) -> Self {
        self.params.extend_from_slice(params);
        self
    }

    #[inline]
    pub fn push_param(&mut self, param: TypedParam) -> &mut Self {
        self.params.push(param);
        self
    }

    #[inline]
    pub fn build(&self) -> Vec<u8> {
        canonical_execute_payload(self.function_index, &self.params)
    }
}

#[inline]
pub fn canonical_execute_payload(function_index: u32, params: &[TypedParam]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&function_index.to_le_bytes());
    out.extend_from_slice(&(params.len() as u32).to_le_bytes());

    for param in params {
        encode_param(&mut out, param);
    }

    out
}

#[inline]
pub fn encode_param(out: &mut Vec<u8>, param: &TypedParam) {
    match param {
        TypedParam::U8(v) => {
            out.push(crate::types::U8);
            out.extend_from_slice(&(*v as u32).to_le_bytes());
        }
        TypedParam::U64(v) => {
            out.push(crate::types::U64);
            out.extend_from_slice(&v.to_le_bytes());
        }
        TypedParam::Bool(v) => {
            out.push(crate::types::BOOL);
            let encoded = if *v { 1u32 } else { 0u32 };
            out.extend_from_slice(&encoded.to_le_bytes());
        }
        TypedParam::Pubkey(pk) => {
            out.push(crate::types::PUBKEY);
            out.extend_from_slice(pk);
        }
        TypedParam::String(s) => {
            out.push(crate::types::STRING);
            out.extend_from_slice(&(s.len() as u32).to_le_bytes());
            out.extend_from_slice(s.as_bytes());
        }
        TypedParam::Account(idx) => {
            out.push(crate::types::ACCOUNT);
            out.extend_from_slice(&(*idx as u32).to_le_bytes());
        }
    }
}
