pub type TypedParam = five_protocol::execute_payload::TypedParam;

pub fn canonical_execute_payload(function_index: u32, params: &[TypedParam]) -> Vec<u8> {
    five_protocol::execute_payload::canonical_execute_payload(function_index, params)
}
