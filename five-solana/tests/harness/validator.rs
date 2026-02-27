use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use five::instructions::{DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION};
use five::state::{FIVEVMState, ScriptAccountHeader};
use five_dsl_compiler::DslCompiler;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcSimulateTransactionConfig, RpcTransactionConfig};
use solana_program::system_instruction;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction},
    message::Message,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signature, Signer},
    system_program,
    transaction::Transaction,
};
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::UiTransactionEncoding;

use super::addresses::{canonical_execute_fee_header, fee_vault_pda, fee_vault_shard0_pda};
use super::compile::{load_or_compile_bytecode, maybe_write_generated_v};
use super::fixtures::{canonical_execute_payload, TypedParam};

#[derive(Debug, Deserialize)]
pub struct RuntimeFixture {
    pub name: String,
    pub bytecode_path: String,
    pub permissions: u8,
    #[serde(default)]
    pub skip_deploy: bool,
    pub authority: AuthorityFixture,
    pub vm_state_name: String,
    pub script_name: String,
    #[serde(default)]
    pub vm_fees: Option<FeeFixture>,
    #[serde(default)]
    pub extra_accounts: Vec<AccountFixture>,
    #[serde(default)]
    pub steps: Vec<StepFixture>,
}

#[derive(Debug, Deserialize)]
pub struct AuthorityFixture {
    pub name: String,
    #[serde(default = "default_authority_lamports")]
    pub lamports: u64,
}

#[derive(Debug, Deserialize)]
pub struct FeeFixture {
    #[serde(alias = "deploy_fee_bps")]
    pub deploy_fee_lamports: u32,
    #[serde(alias = "execute_fee_bps")]
    pub execute_fee_lamports: u32,
}

#[derive(Debug, Deserialize)]
pub struct AccountFixture {
    pub name: String,
    #[serde(default)]
    pub key_seed: Option<u8>,
    #[serde(default)]
    pub pubkey: Option<String>,
    pub owner: AccountOwner,
    #[serde(default)]
    pub lamports: u64,
    #[serde(default)]
    pub data_len: usize,
    #[serde(default)]
    pub is_signer: bool,
    #[serde(default)]
    pub is_writable: bool,
    #[serde(default)]
    pub executable: bool,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AccountOwner {
    Program,
    System,
    Authority,
    SelfAccount,
    SplTokenProgram,
    AnchorTokenProgram,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StepFixture {
    pub name: String,
    pub function_index: u32,
    #[serde(default)]
    pub extras: Vec<String>,
    #[serde(default)]
    pub params: Vec<ParamFixture>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParamFixture {
    AccountRef { account: String },
    U8 { value: u8 },
    U64 { value: u64 },
    Bool { value: bool },
    String { value: String },
    PubkeyAccount { account: String },
    AccountIndex { value: u8 },
}

pub struct RuntimeAccount {
    pub pubkey: Pubkey,
    pub signer: Option<Keypair>,
    pub owner: Pubkey,
    pub lamports: u64,
    pub data_len: usize,
    pub is_signer: bool,
    pub is_writable: bool,
    pub executable: bool,
}

#[derive(Clone, Debug)]
pub struct TxCuResult {
    pub signature: Signature,
    pub units_consumed: u64,
}

#[derive(Debug, Serialize)]
pub struct StepRunResult {
    pub name: String,
    pub signature: String,
    pub units: u64,
    pub success: bool,
}

#[derive(Debug, Serialize)]
pub struct ScenarioRunResult {
    pub name: String,
    pub deploy_signature: Option<String>,
    pub deploy_units: u64,
    pub step_results: Vec<StepRunResult>,
    pub total_units: u64,
    pub elapsed_ms: u128,
}

#[derive(Debug, Serialize)]
pub struct ValidatorRunReport {
    pub network: String,
    pub rpc_url: String,
    pub program_id: String,
    pub commitment: String,
    pub commit_sha: String,
    pub cu_mode: String,
    pub started_unix_ms: u128,
    pub completed_unix_ms: u128,
    pub scenarios: Vec<ScenarioRunResult>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Network {
    Localnet,
    Devnet,
}

impl Network {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Localnet => "localnet",
            Self::Devnet => "devnet",
        }
    }

    pub fn from_env() -> Result<Self, String> {
        let raw = std::env::var("FIVE_CU_NETWORK").unwrap_or_else(|_| "localnet".to_string());
        match raw.as_str() {
            "localnet" => Ok(Self::Localnet),
            "devnet" => Ok(Self::Devnet),
            _ => Err(format!(
                "invalid FIVE_CU_NETWORK `{}` (expected localnet|devnet)",
                raw
            )),
        }
    }
}

fn cu_fee_bypass_enabled() -> bool {
    matches!(
        std::env::var("FIVE_CU_BYPASS_FEES")
            .ok()
            .map(|v| v.to_ascii_lowercase()),
        Some(v) if v == "1" || v == "true" || v == "yes" || v == "on"
    )
}

fn cu_fee_shard_index() -> u8 {
    if cu_fee_bypass_enabled() {
        assert!(
            cfg!(feature = "cu-bypass-fees"),
            "FIVE_CU_BYPASS_FEES=1 requires cargo feature `cu-bypass-fees`"
        );
        five::instructions::fees::FEE_BYPASS_SHARD_INDEX
    } else {
        0
    }
}

pub struct ValidatorHarness {
    pub rpc: RpcClient,
    pub network: Network,
    pub rpc_url: String,
    pub payer: Keypair,
    pub program_id: Pubkey,
    pending_signers: BTreeMap<String, Keypair>,
}

impl ValidatorHarness {
    pub fn from_env() -> Result<Self, String> {
        let network = Network::from_env()?;
        let rpc_url = std::env::var("FIVE_CU_RPC_URL").unwrap_or_else(|_| match network {
            Network::Localnet => "http://127.0.0.1:8899".to_string(),
            Network::Devnet => "https://api.devnet.solana.com".to_string(),
        });

        let payer_path = std::env::var("FIVE_CU_PAYER_KEYPAIR").map_err(|_| {
            "missing FIVE_CU_PAYER_KEYPAIR (path to payer keypair json)".to_string()
        })?;
        let payer = read_keypair_file(&payer_path)
            .map_err(|e| format!("failed reading payer keypair {}: {}", payer_path, e))?;

        let program_id_raw = std::env::var("FIVE_CU_PROGRAM_ID").map_err(|_| {
            "missing FIVE_CU_PROGRAM_ID (pre-deployed Five VM program id)".to_string()
        })?;
        let program_id = Pubkey::from_str(&program_id_raw)
            .map_err(|e| format!("invalid FIVE_CU_PROGRAM_ID `{}`: {}", program_id_raw, e))?;

        let rpc = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());

        rpc.get_version()
            .map_err(|e| format!("rpc connectivity check failed for {}: {}", rpc_url, e))?;

        let program = rpc
            .get_account(&program_id)
            .map_err(|e| format!("failed fetching program account {}: {}", program_id, e))?;
        if !program.executable {
            return Err(format!(
                "program account {} exists but is not executable",
                program_id
            ));
        }

        let mut out = Self {
            rpc,
            network,
            rpc_url,
            payer,
            program_id,
            pending_signers: BTreeMap::new(),
        };

        out.ensure_balance()?;
        Ok(out)
    }

    fn ensure_balance(&mut self) -> Result<(), String> {
        let balance = self
            .rpc
            .get_balance(&self.payer.pubkey())
            .map_err(|e| format!("failed reading payer balance: {}", e))?;

        let minimum = 2 * LAMPORTS_PER_SOL;
        if balance >= minimum {
            return Ok(());
        }

        if self.network == Network::Localnet {
            let sig = self
                .rpc
                .request_airdrop(&self.payer.pubkey(), 20 * LAMPORTS_PER_SOL)
                .map_err(|e| format!("localnet airdrop request failed: {}", e))?;
            self.rpc
                .confirm_transaction(&sig)
                .map_err(|e| format!("localnet airdrop confirmation failed: {}", e))?;
            return Ok(());
        }

        Err(format!(
            "insufficient payer balance on devnet: {} lamports (need at least {})",
            balance, minimum
        ))
    }

    pub fn create_program_owned_account(
        &self,
        space: usize,
        lamports: u64,
        owner: Pubkey,
    ) -> Result<Keypair, String> {
        let kp = Keypair::new();
        let ix = system_instruction::create_account(
            &self.payer.pubkey(),
            &kp.pubkey(),
            lamports,
            space as u64,
            &owner,
        );
        self.send_ixs("create_account", vec![ix], vec![&kp], None)?;
        Ok(kp)
    }

    pub fn ensure_vm_state(&self) -> Result<Pubkey, String> {
        let (vm_state, bump) = Pubkey::find_program_address(&[b"vm_state"], &self.program_id);

        // Canonical PDA already exists and is initialized.
        if let Ok(existing) = self.rpc.get_account(&vm_state) {
            if existing.owner == self.program_id && existing.data.len() >= FIVEVMState::LEN {
                if let Ok(state) = FIVEVMState::from_account_data(&existing.data) {
                    if state.is_initialized() {
                        return Ok(vm_state);
                    }
                }
            }
        }

        // Initialize canonical VM state PDA.
        let init_ix = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(vm_state, false),
                AccountMeta::new_readonly(self.payer.pubkey(), true),
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: vec![0u8, bump],
        };
        self.send_ixs("initialize_vm_state", vec![init_ix], vec![], None)?;
        self.set_vm_fees(vm_state, 0, 0)?;
        Ok(vm_state)
    }

    pub fn ensure_fee_vault_shard(
        &self,
        vm_state_pubkey: Pubkey,
        shard_index: u8,
    ) -> Result<Pubkey, String> {
        let (fee_vault, bump) = fee_vault_pda(&self.program_id, shard_index);
        let init_ix = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(vm_state_pubkey, false),
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(fee_vault, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: vec![11u8, shard_index, bump],
        };
        self.send_ixs("initialize_fee_vault", vec![init_ix], vec![], None)?;
        Ok(fee_vault)
    }

    pub fn set_vm_fees(
        &self,
        vm_state_pubkey: Pubkey,
        deploy_fee_lamports: u32,
        execute_fee_lamports: u32,
    ) -> Result<(), String> {
        let mut data = Vec::with_capacity(9);
        data.push(6);
        data.extend_from_slice(&deploy_fee_lamports.to_le_bytes());
        data.extend_from_slice(&execute_fee_lamports.to_le_bytes());
        let ix = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(vm_state_pubkey, false),
                AccountMeta::new_readonly(self.payer.pubkey(), true),
            ],
            data,
        };
        self.send_ixs("set_vm_fees", vec![ix], vec![], None)?;
        Ok(())
    }

    pub fn rent_exempt(&self, space: usize) -> Result<u64, String> {
        self.rpc
            .get_minimum_balance_for_rent_exemption(space)
            .map_err(|e| format!("rent-exempt query failed: {}", e))
    }

    pub fn send_ixs(
        &self,
        label: &str,
        mut ixs: Vec<Instruction>,
        extra_signers: Vec<&Keypair>,
        cu_limit: Option<u32>,
    ) -> Result<TxCuResult, String> {
        if let Some(limit) = cu_limit {
            ixs.insert(0, ComputeBudgetInstruction::set_compute_unit_limit(limit));
        }

        let recent = self
            .rpc
            .get_latest_blockhash()
            .map_err(|e| format!("{}: latest blockhash failed: {}", label, e))?;
        let msg = Message::new(&ixs, Some(&self.payer.pubkey()));
        let mut tx = Transaction::new_unsigned(msg);

        let mut all_signers: Vec<&Keypair> = Vec::with_capacity(1 + extra_signers.len());
        all_signers.push(&self.payer);
        all_signers.extend(extra_signers);

        tx.try_sign(&all_signers, recent)
            .map_err(|e| format!("{}: signing failed: {}", label, e))?;

        let sim = self
            .rpc
            .simulate_transaction_with_config(
                &tx,
                RpcSimulateTransactionConfig {
                    sig_verify: false,
                    commitment: Some(CommitmentConfig::confirmed()),
                    ..RpcSimulateTransactionConfig::default()
                },
            )
            .map_err(|e| format!("{}: simulate failed: {}", label, e))?;
        if sim.value.err.is_some() {
            let logs = sim.value.logs.unwrap_or_default().join(" | ");
            return Err(format!(
                "{}: simulation err={:?} logs=[{}]",
                label, sim.value.err, logs
            ));
        }

        let sig = self
            .rpc
            .send_and_confirm_transaction_with_spinner(&tx)
            .map_err(|e| format!("{}: send/confirm failed: {}", label, e))?;

        let tx_info = self
            .rpc
            .get_transaction_with_config(
                &sig,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::Json),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: Some(0),
                },
            )
            .map_err(|e| format!("{}: fetch tx {} failed: {}", label, sig, e))?;

        let meta = tx_info
            .transaction
            .meta
            .ok_or_else(|| format!("{}: missing transaction meta for {}", label, sig))?;

        if meta.err.is_some() {
            return Err(format!("{}: tx {} meta.err={:?}", label, sig, meta.err));
        }

        let units = match meta.compute_units_consumed {
            OptionSerializer::Some(u) => u,
            OptionSerializer::None | OptionSerializer::Skip => {
                let logs: Option<&[String]> = match &meta.log_messages {
                    OptionSerializer::Some(lines) => Some(lines.as_slice()),
                    OptionSerializer::None | OptionSerializer::Skip => None,
                };
                parse_cu_from_logs(logs).ok_or_else(|| {
                    format!("{}: unable to determine compute units for {}", label, sig)
                })?
            }
        };

        Ok(TxCuResult {
            signature: sig,
            units_consumed: units,
        })
    }

    pub fn write_report(
        &self,
        scenarios: Vec<ScenarioRunResult>,
        explicit_path: Option<PathBuf>,
    ) -> Result<PathBuf, String> {
        let started = now_unix_ms();
        let completed = now_unix_ms();
        let report = ValidatorRunReport {
            network: self.network.as_str().to_string(),
            rpc_url: self.rpc_url.clone(),
            program_id: self.program_id.to_string(),
            commitment: "confirmed".to_string(),
            commit_sha: std::env::var("FIVE_CU_COMMIT_SHA")
                .unwrap_or_else(|_| "unknown".to_string()),
            cu_mode: std::env::var("FIVE_CU_MODE").unwrap_or_else(|_| "parity".to_string()),
            started_unix_ms: started,
            completed_unix_ms: completed,
            scenarios,
        };

        let out_path = explicit_path.unwrap_or_else(|| {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
            root.join("five-solana/tests/benchmarks/validator-runs")
                .join(format!(
                    "{}-{}-cu.json",
                    self.network.as_str(),
                    now_unix_ms()
                ))
        });

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed creating report dir {}: {}", parent.display(), e))?;
        }

        let raw = serde_json::to_string_pretty(&report)
            .map_err(|e| format!("failed serializing report: {}", e))?;
        fs::write(&out_path, raw)
            .map_err(|e| format!("failed writing report {}: {}", out_path.display(), e))?;

        Ok(out_path)
    }
}

fn parse_cu_from_logs(logs: Option<&[String]>) -> Option<u64> {
    let logs = logs?;
    for line in logs {
        if !line.contains("consumed") {
            continue;
        }
        let mut parts = line.split_whitespace();
        while let Some(tok) = parts.next() {
            if tok == "consumed" {
                if let Some(next) = parts.next() {
                    if let Ok(v) = next.parse::<u64>() {
                        return Some(v);
                    }
                }
            }
        }
    }
    None
}

fn step_signers<'a>(
    h: &'a ValidatorHarness,
    accounts: &'a BTreeMap<String, RuntimeAccount>,
    authority_name: &str,
    extras: &[String],
) -> Vec<&'a Keypair> {
    extras
        .iter()
        .filter_map(|name| {
            if name == authority_name {
                return Some(&h.payer);
            }
            accounts
                .get(name)
                .and_then(|a| if a.is_signer { a.signer.as_ref() } else { None })
        })
        .collect()
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis()
}

fn default_authority_lamports() -> u64 {
    200_000
}

pub fn load_fixture(path: &Path) -> RuntimeFixture {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed reading fixture {}: {}", path.display(), e));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("failed parsing fixture {}: {}", path.display(), e))
}

pub fn resolve_bytecode_path(
    repo_root: &Path,
    fixture_path: &Path,
    configured_path: &str,
) -> PathBuf {
    let configured = PathBuf::from(configured_path);
    if configured.is_absolute() {
        return configured;
    }

    let rel_to_fixture = fixture_path
        .parent()
        .map(|parent| parent.join(&configured))
        .unwrap_or_else(|| configured.clone());
    if rel_to_fixture.exists() {
        return rel_to_fixture;
    }

    repo_root.join(configured)
}

pub fn build_deploy_instruction(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
    script_name: &str,
    vm_state_name: &str,
    owner_name: &str,
    bytecode: &[u8],
    permissions: u8,
    metadata: &[u8],
) -> Instruction {
    let fee_shard_index = cu_fee_shard_index();
    let (fee_vault_pubkey, _fee_vault_bump) = fee_vault_shard0_pda(&program_id);
    let mut data = Vec::with_capacity(10 + metadata.len() + bytecode.len());
    data.push(DEPLOY_INSTRUCTION);
    data.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
    data.push(permissions);
    data.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
    data.extend_from_slice(metadata);
    data.extend_from_slice(bytecode);
    data.push(fee_shard_index);

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(accounts[script_name].pubkey, false),
            AccountMeta::new(accounts[vm_state_name].pubkey, false),
            AccountMeta::new_readonly(accounts[owner_name].pubkey, true),
            AccountMeta::new(fee_vault_pubkey, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}

fn build_init_large_instruction(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
    script_name: &str,
    owner_name: &str,
    vm_state_name: &str,
    expected_size: usize,
    chunk: &[u8],
) -> Instruction {
    let (fee_vault_pubkey, _fee_vault_bump) = fee_vault_shard0_pda(&program_id);
    let mut data = Vec::with_capacity(1 + 4 + chunk.len());
    data.push(4);
    data.extend_from_slice(&(expected_size as u32).to_le_bytes());
    data.extend_from_slice(chunk);

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(accounts[script_name].pubkey, false),
            AccountMeta::new_readonly(accounts[owner_name].pubkey, true),
            AccountMeta::new(accounts[vm_state_name].pubkey, false),
            AccountMeta::new(fee_vault_pubkey, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}

fn build_append_instruction(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
    script_name: &str,
    owner_name: &str,
    vm_state_name: &str,
    chunk: &[u8],
) -> Instruction {
    let (fee_vault_pubkey, _fee_vault_bump) = fee_vault_shard0_pda(&program_id);
    let mut data = Vec::with_capacity(1 + chunk.len());
    data.push(5);
    data.extend_from_slice(chunk);
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(accounts[script_name].pubkey, false),
            AccountMeta::new_readonly(accounts[owner_name].pubkey, true),
            AccountMeta::new(accounts[vm_state_name].pubkey, false),
            AccountMeta::new(fee_vault_pubkey, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}

fn build_finalize_upload_instruction(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
    script_name: &str,
    owner_name: &str,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(accounts[script_name].pubkey, false),
            AccountMeta::new_readonly(accounts[owner_name].pubkey, true),
        ],
        data: vec![7],
    }
}

fn deploy_script_with_chunk_fallback(
    h: &ValidatorHarness,
    accounts: &BTreeMap<String, RuntimeAccount>,
    script_name: &str,
    vm_state_name: &str,
    owner_name: &str,
    bytecode: &[u8],
    permissions: u8,
    metadata: &[u8],
    label_prefix: &str,
) -> Result<TxCuResult, String> {
    let direct = h.send_ixs(
        &format!("{}_direct", label_prefix),
        vec![build_deploy_instruction(
            h.program_id,
            accounts,
            script_name,
            vm_state_name,
            owner_name,
            bytecode,
            permissions,
            metadata,
        )],
        vec![],
        Some(1_400_000),
    );
    if let Ok(ok) = direct {
        return Ok(ok);
    }

    let err = direct
        .err()
        .unwrap_or_else(|| "direct deploy failed".to_string());
    if !err.contains("too large") {
        return Err(err);
    }

    const INIT_CHUNK_SIZE: usize = 512;
    const APPEND_CHUNK_SIZE: usize = 850;

    let first = bytecode.len().min(INIT_CHUNK_SIZE);
    let init = {
        let owner_signers: Vec<&Keypair> = if owner_name == "payer" {
            vec![&h.payer]
        } else {
            accounts
                .get(owner_name)
                .and_then(|a| a.signer.as_ref())
                .map(|signer| vec![signer])
                .unwrap_or_default()
        };
        h.send_ixs(
            &format!("{}_init_large", label_prefix),
            vec![build_init_large_instruction(
                h.program_id,
                accounts,
                script_name,
                owner_name,
                vm_state_name,
                bytecode.len(),
                &bytecode[..first],
            )],
            owner_signers,
            Some(1_400_000),
        )
    }?;

    let mut total = init.units_consumed;
    let mut tail_sig = init.signature;

    let mut offset = first;
    while offset < bytecode.len() {
        let end = (offset + APPEND_CHUNK_SIZE).min(bytecode.len());
        let append = {
            let owner_signers: Vec<&Keypair> = if owner_name == "payer" {
                vec![&h.payer]
            } else {
                accounts
                    .get(owner_name)
                    .and_then(|a| a.signer.as_ref())
                    .map(|signer| vec![signer])
                    .unwrap_or_default()
            };
            h.send_ixs(
                &format!("{}_append", label_prefix),
                vec![build_append_instruction(
                    h.program_id,
                    accounts,
                    script_name,
                    owner_name,
                    vm_state_name,
                    &bytecode[offset..end],
                )],
                owner_signers,
                Some(1_400_000),
            )
        }?;
        total = total.saturating_add(append.units_consumed);
        tail_sig = append.signature;
        offset = end;
    }

    let finalize = {
        let owner_signers: Vec<&Keypair> = if owner_name == "payer" {
            vec![&h.payer]
        } else {
            accounts
                .get(owner_name)
                .and_then(|a| a.signer.as_ref())
                .map(|signer| vec![signer])
                .unwrap_or_default()
        };
        h.send_ixs(
            &format!("{}_finalize", label_prefix),
            vec![build_finalize_upload_instruction(
                h.program_id,
                accounts,
                script_name,
                owner_name,
            )],
            owner_signers,
            Some(1_400_000),
        )
    }?;
    total = total.saturating_add(finalize.units_consumed);
    tail_sig = finalize.signature;

    Ok(TxCuResult {
        signature: tail_sig,
        units_consumed: total,
    })
}

pub fn build_execute_instruction_with_extras(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
    script_name: &str,
    vm_state_name: &str,
    extras: &[String],
    payload: Vec<u8>,
) -> Instruction {
    let fee_shard_index = cu_fee_shard_index();
    let (fee_vault_pubkey, _fee_vault_bump) = fee_vault_shard0_pda(&program_id);
    let mut data = Vec::with_capacity(1 + payload.len());
    data.push(EXECUTE_INSTRUCTION);
    data.extend_from_slice(&canonical_execute_fee_header(fee_shard_index));
    data.extend_from_slice(&payload);

    let mut metas = vec![
        AccountMeta::new(accounts[script_name].pubkey, false),
        AccountMeta::new(accounts[vm_state_name].pubkey, false),
    ];

    for name in extras {
        let a = &accounts[name];
        let is_external_script = name != script_name && name.ends_with("_script");
        metas.push(AccountMeta {
            pubkey: a.pubkey,
            is_signer: a.is_signer,
            is_writable: if is_external_script {
                false
            } else {
                a.is_writable
            },
        });
    }

    let payer = accounts
        .get("payer")
        .filter(|a| a.is_signer && a.is_writable)
        .or_else(|| {
            extras
                .iter()
                .filter_map(|name| accounts.get(name))
                .find(|a| a.is_signer && a.is_writable)
        })
        .or_else(|| {
            accounts
                .get("owner")
                .filter(|a| a.is_signer && a.is_writable)
        })
        .or_else(|| accounts.values().find(|a| a.is_signer && a.is_writable))
        .expect("missing signer+writable payer account for execute");
    metas.push(AccountMeta::new(payer.pubkey, true));
    metas.push(AccountMeta::new(fee_vault_pubkey, false));
    metas.push(AccountMeta::new_readonly(system_program::id(), false));

    Instruction {
        program_id,
        accounts: metas,
        data,
    }
}

pub fn build_payload(accounts: &BTreeMap<String, RuntimeAccount>, step: &StepFixture) -> Vec<u8> {
    let mut params = Vec::with_capacity(step.params.len());
    for param in &step.params {
        match param {
            ParamFixture::AccountRef { account } => {
                let idx = step
                    .extras
                    .iter()
                    .position(|n| n == account)
                    .unwrap_or_else(|| {
                        panic!(
                            "account `{}` missing from extras {:?}",
                            account, step.extras
                        )
                    });
                params.push(TypedParam::Account((idx as u8) + 1));
            }
            ParamFixture::U8 { value } => params.push(TypedParam::U8(*value)),
            ParamFixture::U64 { value } => params.push(TypedParam::U64(*value)),
            ParamFixture::Bool { value } => params.push(TypedParam::Bool(*value)),
            ParamFixture::String { value } => params.push(TypedParam::String(value.clone())),
            ParamFixture::PubkeyAccount { account } => {
                params.push(TypedParam::Pubkey(accounts[account].pubkey.to_bytes()));
            }
            ParamFixture::AccountIndex { value } => params.push(TypedParam::Account(*value)),
        }
    }

    canonical_execute_payload(step.function_index, &params)
}

pub fn setup_accounts_for_fixture(
    h: &mut ValidatorHarness,
    fixture: &RuntimeFixture,
    vm_state_pubkey: Pubkey,
) -> Result<BTreeMap<String, RuntimeAccount>, String> {
    let mut out = BTreeMap::<String, RuntimeAccount>::new();
    let (fee_vault_pubkey, _fee_vault_bump) = fee_vault_shard0_pda(&h.program_id);

    out.insert(
        fixture.authority.name.clone(),
        RuntimeAccount {
            pubkey: h.payer.pubkey(),
            signer: None,
            owner: system_program::id(),
            lamports: fixture.authority.lamports,
            data_len: 0,
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );

    out.insert(
        fixture.vm_state_name.clone(),
        RuntimeAccount {
            pubkey: vm_state_pubkey,
            signer: None,
            owner: h.program_id,
            lamports: h.rent_exempt(FIVEVMState::LEN)?,
            data_len: FIVEVMState::LEN,
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    out.insert(
        "fee_vault".to_string(),
        RuntimeAccount {
            pubkey: fee_vault_pubkey,
            signer: None,
            owner: h.program_id,
            lamports: 0,
            data_len: 0,
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    out.insert(
        "system_program".to_string(),
        RuntimeAccount {
            pubkey: system_program::id(),
            signer: None,
            owner: system_program::id(),
            lamports: 0,
            data_len: 0,
            is_signer: false,
            is_writable: false,
            executable: true,
        },
    );

    for account in &fixture.extra_accounts {
        let configured_pubkey = match &account.pubkey {
            Some(value) => Pubkey::from_str(value)
                .map_err(|e| format!("invalid pubkey in fixture for {}: {}", account.name, e))?,
            None => Keypair::new().pubkey(),
        };
        let pubkey = if account.executable {
            match account.owner {
                AccountOwner::System => system_program::id(),
                AccountOwner::SplTokenProgram => spl_token::id(),
                AccountOwner::AnchorTokenProgram => configured_pubkey,
                _ => {
                    if account.name == "system_program" {
                        system_program::id()
                    } else {
                        configured_pubkey
                    }
                }
            }
        } else {
            configured_pubkey
        };

        if !account.executable {
            let owner = match account.owner {
                AccountOwner::Program => h.program_id,
                AccountOwner::System => system_program::id(),
                AccountOwner::Authority => h.payer.pubkey(),
                AccountOwner::SelfAccount => pubkey,
                AccountOwner::SplTokenProgram => spl_token::id(),
                AccountOwner::AnchorTokenProgram => Pubkey::default(),
            };

            let lamports = if account.lamports > 0 {
                account.lamports
            } else {
                h.rent_exempt(account.data_len.max(0))?
            };

            let maybe_new = if account.pubkey.is_none() {
                Some(Keypair::new())
            } else {
                None
            };
            let target_pubkey = maybe_new.as_ref().map(|kp| kp.pubkey()).unwrap_or(pubkey);

            if maybe_new.is_some() {
                let create_ix = system_instruction::create_account(
                    &h.payer.pubkey(),
                    &target_pubkey,
                    lamports,
                    account.data_len as u64,
                    &owner,
                );
                let signer = maybe_new.as_ref().expect("checked");
                h.send_ixs(
                    "create_fixture_account",
                    vec![create_ix],
                    vec![signer],
                    None,
                )?;
                h.pending_signers
                    .insert(account.name.clone(), signer.insecure_clone());
            }

            out.insert(
                account.name.clone(),
                RuntimeAccount {
                    pubkey: target_pubkey,
                    signer: maybe_new,
                    owner,
                    lamports,
                    data_len: account.data_len,
                    is_signer: account.is_signer,
                    is_writable: account.is_writable,
                    executable: false,
                },
            );
        } else {
            out.insert(
                account.name.clone(),
                RuntimeAccount {
                    pubkey,
                    signer: None,
                    owner: system_program::id(),
                    lamports: account.lamports,
                    data_len: account.data_len,
                    is_signer: account.is_signer,
                    is_writable: account.is_writable,
                    executable: true,
                },
            );
        }
    }

    Ok(out)
}

pub fn filter_steps<'a>(steps: &'a [StepFixture], names: &[&str]) -> Vec<StepFixture> {
    let wanted: HashSet<&str> = names.iter().copied().collect();
    steps
        .iter()
        .filter(|s| wanted.contains(s.name.as_str()))
        .cloned()
        .collect()
}

pub fn run_fixture_scenario(
    h: &mut ValidatorHarness,
    repo_root: &Path,
    fixture_path: &Path,
    scenario_name: &str,
    step_filter: Option<&[&str]>,
) -> Result<ScenarioRunResult, String> {
    let fixture = load_fixture(fixture_path);
    if fixture.skip_deploy {
        return Err(format!("fixture {} has skip_deploy=true", fixture.name));
    }

    let now = Instant::now();
    let vm_state = h.ensure_vm_state()?;
    let mut accounts = setup_accounts_for_fixture(h, &fixture, vm_state)?;
    let fee_vault = h.ensure_fee_vault_shard(vm_state, 0)?;
    if let Some(existing) = accounts.get_mut("fee_vault") {
        existing.pubkey = fee_vault;
    }
    if let Some(fees) = &fixture.vm_fees {
        if cu_fee_bypass_enabled() {
            h.set_vm_fees(vm_state, 0, 0)?;
        } else {
            h.set_vm_fees(
                vm_state,
                fees.deploy_fee_lamports,
                fees.execute_fee_lamports,
            )?;
        }
    } else {
        h.set_vm_fees(vm_state, 0, 0)?;
    }

    let bytecode_path = resolve_bytecode_path(repo_root, fixture_path, &fixture.bytecode_path);
    let bytecode = load_or_compile_bytecode(&bytecode_path)
        .map_err(|e| format!("failed loading bytecode {}: {}", bytecode_path.display(), e))?;

    let script = h.create_program_owned_account(
        ScriptAccountHeader::LEN + bytecode.len(),
        h.rent_exempt(ScriptAccountHeader::LEN + bytecode.len())?,
        h.program_id,
    )?;

    accounts.insert(
        fixture.script_name.clone(),
        RuntimeAccount {
            pubkey: script.pubkey(),
            signer: Some(script),
            owner: h.program_id,
            lamports: 0,
            data_len: ScriptAccountHeader::LEN + bytecode.len(),
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let deploy = deploy_script_with_chunk_fallback(
        h,
        &accounts,
        &fixture.script_name,
        &fixture.vm_state_name,
        &fixture.authority.name,
        &bytecode,
        fixture.permissions,
        &[],
        "fixture_deploy",
    )?;

    let steps = if let Some(filter) = step_filter {
        filter_steps(&fixture.steps, filter)
    } else {
        fixture.steps.clone()
    };

    let mut step_results = Vec::with_capacity(steps.len());
    let mut total = deploy.units_consumed;

    for step in steps {
        let payload = build_payload(&accounts, &step);
        let ix = build_execute_instruction_with_extras(
            h.program_id,
            &accounts,
            &fixture.script_name,
            &fixture.vm_state_name,
            &step.extras,
            payload,
        );

        let extra_signers = step_signers(h, &accounts, &fixture.authority.name, &step.extras);

        let tx = h.send_ixs("fixture_execute", vec![ix], extra_signers, Some(1_400_000))?;
        total = total.saturating_add(tx.units_consumed);
        println!(
            "BPF_CU validator step={} signature={} units={} success=true",
            step.name, tx.signature, tx.units_consumed
        );
        step_results.push(StepRunResult {
            name: step.name,
            signature: tx.signature.to_string(),
            units: tx.units_consumed,
            success: true,
        });
    }

    Ok(ScenarioRunResult {
        name: scenario_name.to_string(),
        deploy_signature: Some(deploy.signature.to_string()),
        deploy_units: deploy.units_consumed,
        step_results,
        total_units: total,
        elapsed_ms: now.elapsed().as_millis(),
    })
}

pub fn run_external_non_cpi(
    h: &mut ValidatorHarness,
    repo_root: &Path,
) -> Result<ScenarioRunResult, String> {
    let fixture = repo_root.join("five-templates/token/runtime-fixtures/init_mint.json");
    let setup_steps = [
        "init_mint",
        "init_token_account_user1",
        "init_token_account_user2",
        "init_token_account_user3",
        "mint_to_user1",
        "mint_to_user2",
        "mint_to_user3",
    ];
    let setup = run_fixture_scenario(
        h,
        repo_root,
        &fixture,
        "external_non_cpi_setup",
        Some(&setup_steps),
    )?;

    let token_script = setup
        .deploy_signature
        .clone()
        .ok_or_else(|| "missing deploy signature for setup".to_string())?;
    let _ = token_script;

    // We re-run fixture fresh to get account map with known names and token script pubkey.
    let parsed = load_fixture(&fixture);
    let vm_state = h.ensure_vm_state()?;
    let mut accounts = setup_accounts_for_fixture(h, &parsed, vm_state)?;
    let token_bytecode_path = resolve_bytecode_path(repo_root, &fixture, &parsed.bytecode_path);
    let token_bytecode = load_or_compile_bytecode(&token_bytecode_path)
        .map_err(|e| format!("failed reading token bytecode: {}", e))?;
    let token_script_kp = h.create_program_owned_account(
        ScriptAccountHeader::LEN + token_bytecode.len(),
        h.rent_exempt(ScriptAccountHeader::LEN + token_bytecode.len())?,
        h.program_id,
    )?;
    accounts.insert(
        parsed.script_name.clone(),
        RuntimeAccount {
            pubkey: token_script_kp.pubkey(),
            signer: Some(token_script_kp),
            owner: h.program_id,
            lamports: 0,
            data_len: ScriptAccountHeader::LEN + token_bytecode.len(),
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let deploy_token = deploy_script_with_chunk_fallback(
        h,
        &accounts,
        &parsed.script_name,
        &parsed.vm_state_name,
        &parsed.authority.name,
        &token_bytecode,
        parsed.permissions,
        &[],
        "external_non_cpi_deploy_token",
    )?;

    let init_steps = filter_steps(&parsed.steps, &setup_steps);
    for step in init_steps {
        let ix = build_execute_instruction_with_extras(
            h.program_id,
            &accounts,
            &parsed.script_name,
            &parsed.vm_state_name,
            &step.extras,
            build_payload(&accounts, &step),
        );
        let extra_signers = step_signers(h, &accounts, &parsed.authority.name, &step.extras);
        h.send_ixs(
            "external_non_cpi_setup_step",
            vec![ix],
            extra_signers,
            Some(1_400_000),
        )?;
    }
    // Alias imported callee account with `_script` suffix so execute meta builder keeps it read-only.
    let token_script_pubkey = accounts[&parsed.script_name].pubkey;
    accounts.insert(
        "token_script".to_string(),
        RuntimeAccount {
            pubkey: token_script_pubkey,
            signer: None,
            owner: h.program_id,
            lamports: 0,
            data_len: 0,
            is_signer: false,
            is_writable: false,
            executable: false,
        },
    );

    let token_import = bs58::encode(accounts[&parsed.script_name].pubkey.to_bytes()).into_string();
    let caller_source = format!(
        r#"
        use "{token_import}"::{{transfer}};

        pub fn call_transfer(
            source_account: account @mut,
            destination_account: account @mut,
            owner: account @mut,
            ext0: account
        ) {{
            transfer(source_account, destination_account, owner, 50);
        }}
    "#
    );
    maybe_write_generated_v(
        repo_root,
        "generated/validator-external-transfer-caller.v",
        &caller_source,
    );
    let caller_bytecode = DslCompiler::compile_dsl(&caller_source)
        .map_err(|e| format!("caller compile failed: {}", e))?;

    let caller_script = h.create_program_owned_account(
        ScriptAccountHeader::LEN + caller_bytecode.len(),
        h.rent_exempt(ScriptAccountHeader::LEN + caller_bytecode.len())?,
        h.program_id,
    )?;
    accounts.insert(
        "caller_script".to_string(),
        RuntimeAccount {
            pubkey: caller_script.pubkey(),
            signer: Some(caller_script),
            owner: h.program_id,
            lamports: 0,
            data_len: ScriptAccountHeader::LEN + caller_bytecode.len(),
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let deploy_caller = deploy_script_with_chunk_fallback(
        h,
        &accounts,
        "caller_script",
        &parsed.vm_state_name,
        &parsed.authority.name,
        &caller_bytecode,
        0,
        &[],
        "external_non_cpi_deploy_caller",
    )?;

    let execute_step = StepFixture {
        name: "external_transfer_non_cpi".to_string(),
        function_index: 0,
        extras: vec![
            "user2_token".to_string(),
            "user3_token".to_string(),
            "user2".to_string(),
            "token_script".to_string(),
        ],
        params: vec![
            ParamFixture::AccountRef {
                account: "user2_token".to_string(),
            },
            ParamFixture::AccountRef {
                account: "user3_token".to_string(),
            },
            ParamFixture::AccountRef {
                account: "user2".to_string(),
            },
            ParamFixture::AccountRef {
                account: "token_script".to_string(),
            },
        ],
    };

    let execute_signers = step_signers(h, &accounts, &parsed.authority.name, &execute_step.extras);
    let execute = h.send_ixs(
        "external_non_cpi_execute",
        vec![build_execute_instruction_with_extras(
            h.program_id,
            &accounts,
            "caller_script",
            &parsed.vm_state_name,
            &execute_step.extras,
            build_payload(&accounts, &execute_step),
        )],
        execute_signers,
        Some(1_400_000),
    )?;

    let total = deploy_token
        .units_consumed
        .saturating_add(deploy_caller.units_consumed)
        .saturating_add(execute.units_consumed);

    Ok(ScenarioRunResult {
        name: "external_non_cpi".to_string(),
        deploy_signature: Some(deploy_token.signature.to_string()),
        deploy_units: deploy_token
            .units_consumed
            .saturating_add(deploy_caller.units_consumed),
        step_results: vec![StepRunResult {
            name: "external_transfer_non_cpi".to_string(),
            signature: execute.signature.to_string(),
            units: execute.units_consumed,
            success: true,
        }],
        total_units: total,
        elapsed_ms: setup.elapsed_ms,
    })
}

pub fn run_external_burst_non_cpi(
    h: &mut ValidatorHarness,
    repo_root: &Path,
) -> Result<ScenarioRunResult, String> {
    let fixture = repo_root.join("five-templates/token/runtime-fixtures/init_mint.json");
    let parsed = load_fixture(&fixture);
    let vm_state = h.ensure_vm_state()?;
    let mut accounts = setup_accounts_for_fixture(h, &parsed, vm_state)?;

    let token_bytecode_path = resolve_bytecode_path(repo_root, &fixture, &parsed.bytecode_path);
    let token_bytecode = load_or_compile_bytecode(&token_bytecode_path)
        .map_err(|e| format!("failed reading token bytecode: {}", e))?;
    let token_script = h.create_program_owned_account(
        ScriptAccountHeader::LEN + token_bytecode.len(),
        h.rent_exempt(ScriptAccountHeader::LEN + token_bytecode.len())?,
        h.program_id,
    )?;
    accounts.insert(
        parsed.script_name.clone(),
        RuntimeAccount {
            pubkey: token_script.pubkey(),
            signer: Some(token_script),
            owner: h.program_id,
            lamports: 0,
            data_len: ScriptAccountHeader::LEN + token_bytecode.len(),
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let deploy_token = deploy_script_with_chunk_fallback(
        h,
        &accounts,
        &parsed.script_name,
        &parsed.vm_state_name,
        &parsed.authority.name,
        &token_bytecode,
        parsed.permissions,
        &[],
        "external_burst_deploy_token",
    )?;

    let setup_steps = [
        "init_mint",
        "init_token_account_user1",
        "init_token_account_user2",
        "mint_to_user1",
    ];
    for step in filter_steps(&parsed.steps, &setup_steps) {
        let ix = build_execute_instruction_with_extras(
            h.program_id,
            &accounts,
            &parsed.script_name,
            &parsed.vm_state_name,
            &step.extras,
            build_payload(&accounts, &step),
        );
        let extra_signers = step_signers(h, &accounts, &parsed.authority.name, &step.extras);
        h.send_ixs(
            "external_burst_setup_step",
            vec![ix],
            extra_signers,
            Some(1_400_000),
        )?;
    }
    let token_script_pubkey = accounts[&parsed.script_name].pubkey;
    accounts.insert(
        "token_script".to_string(),
        RuntimeAccount {
            pubkey: token_script_pubkey,
            signer: None,
            owner: h.program_id,
            lamports: 0,
            data_len: 0,
            is_signer: false,
            is_writable: false,
            executable: false,
        },
    );

    let token_import = bs58::encode(accounts[&parsed.script_name].pubkey.to_bytes()).into_string();
    let caller_source = format!(
        r#"
        use "{token_import}"::{{transfer}};

        pub fn burst_transfer(
            s: account @mut,
            d: account @mut,
            owner: account @mut,
            ext0: account
        ) {{
            transfer(s, d, owner, 10);
            transfer(s, d, owner, 20);
            transfer(s, d, owner, 30);
            transfer(s, d, owner, 40);
        }}
    "#
    );
    maybe_write_generated_v(
        repo_root,
        "generated/validator-external-burst-caller.v",
        &caller_source,
    );
    let caller_bytecode = DslCompiler::compile_dsl(&caller_source)
        .map_err(|e| format!("caller compile failed: {}", e))?;

    let caller_script = h.create_program_owned_account(
        ScriptAccountHeader::LEN + caller_bytecode.len(),
        h.rent_exempt(ScriptAccountHeader::LEN + caller_bytecode.len())?,
        h.program_id,
    )?;
    accounts.insert(
        "caller_script".to_string(),
        RuntimeAccount {
            pubkey: caller_script.pubkey(),
            signer: Some(caller_script),
            owner: h.program_id,
            lamports: 0,
            data_len: ScriptAccountHeader::LEN + caller_bytecode.len(),
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let deploy_caller = deploy_script_with_chunk_fallback(
        h,
        &accounts,
        "caller_script",
        &parsed.vm_state_name,
        &parsed.authority.name,
        &caller_bytecode,
        0,
        &[],
        "external_burst_deploy_caller",
    )?;

    let execute_step = StepFixture {
        name: "external_transfer_burst_non_cpi".to_string(),
        function_index: 0,
        extras: vec![
            "user1_token".to_string(),
            "user2_token".to_string(),
            "user1".to_string(),
            "token_script".to_string(),
        ],
        params: vec![
            ParamFixture::AccountRef {
                account: "user1_token".to_string(),
            },
            ParamFixture::AccountRef {
                account: "user2_token".to_string(),
            },
            ParamFixture::AccountRef {
                account: "user1".to_string(),
            },
            ParamFixture::AccountRef {
                account: "token_script".to_string(),
            },
        ],
    };

    let execute_signers = step_signers(h, &accounts, &parsed.authority.name, &execute_step.extras);
    let execute = h.send_ixs(
        "external_burst_execute",
        vec![build_execute_instruction_with_extras(
            h.program_id,
            &accounts,
            "caller_script",
            &parsed.vm_state_name,
            &execute_step.extras,
            build_payload(&accounts, &execute_step),
        )],
        execute_signers,
        Some(1_400_000),
    )?;

    let total = deploy_token
        .units_consumed
        .saturating_add(deploy_caller.units_consumed)
        .saturating_add(execute.units_consumed);

    Ok(ScenarioRunResult {
        name: "external_burst_non_cpi".to_string(),
        deploy_signature: Some(deploy_token.signature.to_string()),
        deploy_units: deploy_token
            .units_consumed
            .saturating_add(deploy_caller.units_consumed),
        step_results: vec![StepRunResult {
            name: "external_transfer_burst_non_cpi".to_string(),
            signature: execute.signature.to_string(),
            units: execute.units_consumed,
            success: true,
        }],
        total_units: total,
        elapsed_ms: 0,
    })
}

pub fn run_external_interface_mapping_non_cpi(
    h: &mut ValidatorHarness,
    repo_root: &Path,
) -> Result<ScenarioRunResult, String> {
    let now = Instant::now();
    let vm_state = h.ensure_vm_state()?;
    let mut accounts = BTreeMap::<String, RuntimeAccount>::new();

    accounts.insert(
        "owner".to_string(),
        RuntimeAccount {
            pubkey: h.payer.pubkey(),
            signer: None,
            owner: system_program::id(),
            lamports: 0,
            data_len: 0,
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );

    accounts.insert(
        "vm_state".to_string(),
        RuntimeAccount {
            pubkey: vm_state,
            signer: None,
            owner: h.program_id,
            lamports: 0,
            data_len: FIVEVMState::LEN,
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let callee_source = r#"
        pub fn transfer_checked(
            source_account: account @mut,
            destination_account: account @mut,
            owner: account @mut,
            amount: u64
        ) {
        }
    "#;
    maybe_write_generated_v(
        repo_root,
        "generated/validator-interface-callee.v",
        callee_source,
    );
    let callee_bytecode = DslCompiler::compile_dsl(callee_source)
        .map_err(|e| format!("callee compile failed: {}", e))?;

    let callee_metadata = encode_export_metadata_for_test(
        &["transfer_checked"],
        &[("TokenOps", &[("transfer", "transfer_checked")])],
    );

    let callee_script = h.create_program_owned_account(
        ScriptAccountHeader::LEN + callee_metadata.len() + callee_bytecode.len(),
        h.rent_exempt(ScriptAccountHeader::LEN + callee_metadata.len() + callee_bytecode.len())?,
        h.program_id,
    )?;
    let callee_pubkey = callee_script.pubkey();
    accounts.insert(
        "callee_script".to_string(),
        RuntimeAccount {
            pubkey: callee_pubkey,
            signer: Some(callee_script),
            owner: h.program_id,
            lamports: 0,
            data_len: ScriptAccountHeader::LEN + callee_metadata.len() + callee_bytecode.len(),
            is_signer: false,
            is_writable: false,
            executable: false,
        },
    );

    let lock_guard = scoped_lockfile_guard(
        repo_root,
        lockfile_with_exports(
            &bs58::encode(callee_pubkey.to_bytes()).into_string(),
            &[("transfer_checked", "transfer_checked")],
            &[("TokenOps", &[("transfer", "transfer_checked")])],
        ),
    );

    let caller_source = format!(
        r#"
        use "{}"::{{interface TokenOps}};

        pub fn call_interface(
            source_account: account @mut,
            destination_account: account @mut,
            owner: account @mut,
            TokenOps: account
        ) {{
            TokenOps.transfer(source_account, destination_account, owner, 50);
        }}
    "#,
        bs58::encode(callee_pubkey.to_bytes()).into_string()
    );
    maybe_write_generated_v(
        repo_root,
        "generated/validator-interface-caller.v",
        &caller_source,
    );
    let caller_bytecode = DslCompiler::compile_dsl(&caller_source)
        .map_err(|e| format!("caller compile failed via lockfile mapping: {}", e))?;
    drop(lock_guard);

    let caller_script = h.create_program_owned_account(
        ScriptAccountHeader::LEN + caller_bytecode.len(),
        h.rent_exempt(ScriptAccountHeader::LEN + caller_bytecode.len())?,
        h.program_id,
    )?;
    accounts.insert(
        "caller_script".to_string(),
        RuntimeAccount {
            pubkey: caller_script.pubkey(),
            signer: Some(caller_script),
            owner: h.program_id,
            lamports: 0,
            data_len: ScriptAccountHeader::LEN + caller_bytecode.len(),
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    for name in ["source_account", "destination_account"] {
        let kp = h.create_program_owned_account(64, h.rent_exempt(64)?, h.program_id)?;
        accounts.insert(
            name.to_string(),
            RuntimeAccount {
                pubkey: kp.pubkey(),
                signer: Some(kp),
                owner: h.program_id,
                lamports: 0,
                data_len: 64,
                is_signer: false,
                is_writable: true,
                executable: false,
            },
        );
    }

    let deploy_callee = deploy_script_with_chunk_fallback(
        h,
        &accounts,
        "callee_script",
        "vm_state",
        "owner",
        &callee_bytecode,
        0,
        &callee_metadata,
        "external_interface_deploy_callee",
    )?;

    let deploy_caller = deploy_script_with_chunk_fallback(
        h,
        &accounts,
        "caller_script",
        "vm_state",
        "owner",
        &caller_bytecode,
        0,
        &[],
        "external_interface_deploy_caller",
    )?;

    let step = StepFixture {
        name: "external_interface_mapping_non_cpi".to_string(),
        function_index: 0,
        extras: vec![
            "source_account".to_string(),
            "destination_account".to_string(),
            "owner".to_string(),
            "callee_script".to_string(),
        ],
        params: vec![
            ParamFixture::AccountRef {
                account: "source_account".to_string(),
            },
            ParamFixture::AccountRef {
                account: "destination_account".to_string(),
            },
            ParamFixture::AccountRef {
                account: "owner".to_string(),
            },
            ParamFixture::AccountRef {
                account: "callee_script".to_string(),
            },
        ],
    };

    let execute_signers = step_signers(h, &accounts, "owner", &step.extras);
    let execute = h.send_ixs(
        "external_interface_execute",
        vec![build_execute_instruction_with_extras(
            h.program_id,
            &accounts,
            "caller_script",
            "vm_state",
            &step.extras,
            build_payload(&accounts, &step),
        )],
        execute_signers,
        Some(1_400_000),
    )?;

    let total = deploy_callee
        .units_consumed
        .saturating_add(deploy_caller.units_consumed)
        .saturating_add(execute.units_consumed);

    Ok(ScenarioRunResult {
        name: "external_interface_mapping_non_cpi".to_string(),
        deploy_signature: Some(deploy_callee.signature.to_string()),
        deploy_units: deploy_callee
            .units_consumed
            .saturating_add(deploy_caller.units_consumed),
        step_results: vec![StepRunResult {
            name: step.name,
            signature: execute.signature.to_string(),
            units: execute.units_consumed,
            success: true,
        }],
        total_units: total,
        elapsed_ms: now.elapsed().as_millis(),
    })
}

fn encode_export_metadata_for_test(
    methods: &[&str],
    interfaces: &[(&str, &[(&str, &str)])],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"5EXP");
    out.push(1);
    out.push(methods.len().min(255) as u8);
    for method in methods.iter().take(255) {
        let bytes = method.as_bytes();
        out.push(bytes.len().min(255) as u8);
        out.extend_from_slice(&bytes[..bytes.len().min(255)]);
    }
    out.push(interfaces.len().min(255) as u8);
    for (iface_name, method_map) in interfaces.iter().take(255) {
        let iface_bytes = iface_name.as_bytes();
        out.push(iface_bytes.len().min(255) as u8);
        out.extend_from_slice(&iface_bytes[..iface_bytes.len().min(255)]);
        out.push(method_map.len().min(255) as u8);
        for (method, callee) in method_map.iter().take(255) {
            let method_bytes = method.as_bytes();
            out.push(method_bytes.len().min(255) as u8);
            out.extend_from_slice(&method_bytes[..method_bytes.len().min(255)]);
            let callee_bytes = callee.as_bytes();
            out.push(callee_bytes.len().min(255) as u8);
            out.extend_from_slice(&callee_bytes[..callee_bytes.len().min(255)]);
        }
    }
    out
}

fn lockfile_with_exports(
    address: &str,
    methods: &[(&str, &str)],
    interfaces: &[(&str, &[(&str, &str)])],
) -> String {
    let mut out = String::from(
        "version = 1\n\n[[packages]]\nname = \"validator-interface-lib\"\nversion = \"0.0.0\"\n",
    );
    out.push_str(&format!(
        "address = \"{}\"\nbytecode_hash = \"deadbeef\"\ndeployed_at = \"2026-01-01T00:00:00Z\"\n\n[packages.exports]\n",
        address
    ));
    let method_list = methods
        .iter()
        .map(|(name, _)| format!("\"{}\"", name))
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!("methods = [{}]\n", method_list));
    for (iface_name, method_map) in interfaces {
        out.push_str("\n[[packages.exports.interfaces]]\n");
        out.push_str(&format!("name = \"{}\"\n", iface_name));
        let mapping = method_map
            .iter()
            .map(|(method, callee)| format!("\"{}\" = \"{}\"", method, callee))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("method_map = {{ {} }}\n", mapping));
    }
    out
}

struct LockfileGuard {
    path: PathBuf,
    previous: Option<Vec<u8>>,
}

impl Drop for LockfileGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(bytes) => {
                let _ = fs::write(&self.path, bytes);
            }
            None => {
                let _ = fs::remove_file(&self.path);
            }
        }
    }
}

fn scoped_lockfile_guard(repo_root: &Path, content: String) -> LockfileGuard {
    let path = std::env::current_dir()
        .unwrap_or_else(|_| repo_root.to_path_buf())
        .join("five.lock");
    let previous = fs::read(&path).ok();
    fs::write(&path, content.as_bytes())
        .expect("failed to write temporary five.lock for validator test");
    LockfileGuard { path, previous }
}
