use pinocchio::pubkey::Pubkey;

#[derive(Clone, Debug)]
pub enum TypedParam {
    U8(u8),
    U64(u64),
    Bool(bool),
    Pubkey(Pubkey),
    String(String),
    Account(u8),
}

pub fn canonical_execute_payload(function_index: u32, params: &[TypedParam]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&function_index.to_le_bytes());
    out.extend_from_slice(&(params.len() as u32).to_le_bytes());

    for param in params {
        encode_param(&mut out, param);
    }

    out
}

fn encode_param(out: &mut Vec<u8>, param: &TypedParam) {
    match param {
        TypedParam::U8(v) => {
            out.push(five_protocol::types::U8);
            out.extend_from_slice(&(*v as u32).to_le_bytes());
        }
        TypedParam::U64(v) => {
            out.push(five_protocol::types::U64);
            out.extend_from_slice(&v.to_le_bytes());
        }
        TypedParam::Bool(v) => {
            out.push(five_protocol::types::BOOL);
            let encoded = if *v { 1u32 } else { 0u32 };
            out.extend_from_slice(&encoded.to_le_bytes());
        }
        TypedParam::Pubkey(pk) => {
            out.push(five_protocol::types::PUBKEY);
            out.extend_from_slice(pk.as_ref());
        }
        TypedParam::String(s) => {
            out.push(five_protocol::types::STRING);
            out.extend_from_slice(&(s.len() as u32).to_le_bytes());
            out.extend_from_slice(s.as_bytes());
        }
        TypedParam::Account(idx) => {
            out.push(five_protocol::types::ACCOUNT);
            out.extend_from_slice(&(*idx as u32).to_le_bytes());
        }
    }
}
