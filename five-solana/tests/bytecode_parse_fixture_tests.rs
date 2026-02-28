use five::instructions::{verify_bytecode_content, FIVEInstruction, DEPLOY_INSTRUCTION};
use five_protocol::parser::{parse_code_bounds, parse_instruction_with_features};

#[test]
fn parse_defi_bench_fixture_with_feature_aware_parser() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let bytecode_path = repo_root.join("five-templates/defi-bench/src/defi_bench.bin");
    let bytecode = std::fs::read(&bytecode_path)
        .unwrap_or_else(|e| panic!("failed reading {}: {}", bytecode_path.display(), e));

    let (header, mut offset, code_end) = parse_code_bounds(&bytecode).expect("code bounds");
    while offset < code_end {
        match parse_instruction_with_features(&bytecode, offset, header.features) {
            Ok((_, size)) => offset += size,
            Err(e) => panic!(
                "parse error at offset {} (0x{:X}), opcode=0x{:02X}, features=0x{:08X}, err={:?}",
                offset, offset, bytecode[offset], header.features, e
            ),
        }
    }
}

#[test]
fn verify_defi_bench_fixture_deploy_slice() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let bytecode_path = repo_root.join("five-templates/defi-bench/src/defi_bench.bin");
    let bytecode = std::fs::read(&bytecode_path)
        .unwrap_or_else(|e| panic!("failed reading {}: {}", bytecode_path.display(), e));

    let mut deploy_data = Vec::with_capacity(10 + bytecode.len());
    deploy_data.push(DEPLOY_INSTRUCTION);
    deploy_data.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
    deploy_data.push(0); // permissions
    deploy_data.extend_from_slice(&0u32.to_le_bytes()); // metadata_len
    deploy_data.extend_from_slice(&bytecode);

    let parsed = FIVEInstruction::try_from(deploy_data.as_slice()).expect("deploy decode");
    let FIVEInstruction::Deploy { bytecode, .. } = parsed else {
        panic!("expected deploy instruction")
    };

    verify_bytecode_content(bytecode).expect("verify_bytecode_content should pass");
}

#[test]
fn parse_token_fixture_with_feature_aware_parser() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let bytecode_path = repo_root.join("five-templates/token/src/token.bin");
    let bytecode = std::fs::read(&bytecode_path)
        .unwrap_or_else(|e| panic!("failed reading {}: {}", bytecode_path.display(), e));

    let (header, mut offset, code_end) = parse_code_bounds(&bytecode).expect("code bounds");
    while offset < code_end {
        match parse_instruction_with_features(&bytecode, offset, header.features) {
            Ok((_, size)) => offset += size,
            Err(e) => panic!(
                "token parse error at offset {} (0x{:X}), opcode=0x{:02X}, features=0x{:08X}, err={:?}",
                offset,
                offset,
                bytecode[offset],
                header.features,
                e
            ),
        }
    }
}

#[test]
fn verify_token_fixture_deploy_slice() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let bytecode_path = repo_root.join("five-templates/token/src/token.bin");
    let bytecode = std::fs::read(&bytecode_path)
        .unwrap_or_else(|e| panic!("failed reading {}: {}", bytecode_path.display(), e));

    let mut deploy_data = Vec::with_capacity(10 + bytecode.len());
    deploy_data.push(DEPLOY_INSTRUCTION);
    deploy_data.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
    deploy_data.push(0); // permissions
    deploy_data.extend_from_slice(&0u32.to_le_bytes()); // metadata_len
    deploy_data.extend_from_slice(&bytecode);

    let parsed = FIVEInstruction::try_from(deploy_data.as_slice()).expect("deploy decode");
    let FIVEInstruction::Deploy { bytecode, .. } = parsed else {
        panic!("expected deploy instruction")
    };

    verify_bytecode_content(bytecode).expect("verify_bytecode_content should pass");
}
