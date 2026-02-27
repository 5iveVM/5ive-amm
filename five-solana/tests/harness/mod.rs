//! Test harness contract:
//! - `RuntimeHarness` is for fast unit/encoding/negative-shape tests only.
//! - `ProgramTest`-backed tests are the source of truth for runtime behavior.
//! - `ValidatorHarness` is for RPC/integration/perf and network-gated checks.
//! - If behavior can be covered in BPF, keep it in BPF and trim in-process duplicates.

#[cfg(feature = "legacy-runtime-harness")]
use std::collections::BTreeMap;

#[cfg(feature = "legacy-runtime-harness")]
use five::{
    instructions::{deploy, execute},
    state::{FIVEVMState, ScriptAccountHeader},
};
#[cfg(feature = "legacy-runtime-harness")]
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

pub mod addresses;
pub mod compile;
pub mod fixtures;
pub mod instruction_builders;
pub mod perf;
#[cfg(feature = "validator-harness")]
pub mod validator;

#[cfg(feature = "legacy-runtime-harness")]
#[derive(Clone, Debug)]
pub struct AccountSeed {
    pub key: Pubkey,
    pub owner: Pubkey,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub is_signer: bool,
    pub is_writable: bool,
    pub executable: bool,
}

#[cfg(feature = "legacy-runtime-harness")]
#[derive(Clone, Debug)]
pub struct AccountSnapshot {
    pub key: Pubkey,
    pub owner: Pubkey,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub is_signer: bool,
    pub is_writable: bool,
    pub executable: bool,
}

#[cfg(feature = "legacy-runtime-harness")]
#[derive(Clone, Debug)]
pub struct TxResult {
    pub success: bool,
    pub error: Option<ProgramError>,
    pub logs: Vec<String>,
    pub compute_units: Option<u64>,
}

#[cfg(feature = "legacy-runtime-harness")]
impl TxResult {
    pub fn ok() -> Self {
        Self {
            success: true,
            error: None,
            logs: Vec::new(),
            compute_units: None,
        }
    }

    pub fn err(error: ProgramError) -> Self {
        Self {
            success: false,
            error: Some(error),
            logs: Vec::new(),
            compute_units: None,
        }
    }
}

#[cfg(feature = "legacy-runtime-harness")]
#[derive(Clone, Debug)]
pub enum ExpectedOutcome {
    Success,
    ProgramError(ProgramError),
}

#[cfg(feature = "legacy-runtime-harness")]
#[derive(Clone, Debug)]
pub struct ScriptFixture {
    pub bytecode: Vec<u8>,
    pub permissions: u8,
    pub execute_payload: Vec<u8>,
    pub initial_accounts: Vec<(String, AccountSeed)>,
    pub expectation: ExpectedOutcome,
}

#[cfg(feature = "legacy-runtime-harness")]
#[derive(Clone, Debug)]
struct HarnessAccount {
    key: Pubkey,
    owner: Pubkey,
    lamports: u64,
    data: Vec<u8>,
    is_signer: bool,
    is_writable: bool,
    executable: bool,
}

#[cfg(feature = "legacy-runtime-harness")]
#[derive(Debug)]
pub struct RuntimeHarness {
    pub program_id: Pubkey,
    accounts: Vec<HarnessAccount>,
    index_by_name: BTreeMap<String, usize>,
    last_logs: Vec<String>,
}

#[cfg(feature = "legacy-runtime-harness")]
impl RuntimeHarness {
    const MAX_ACCOUNT_DATA_LEN: usize = 10 * 1024 * 1024;
    const OFFCHAIN_DATA_HEADROOM: usize = 16 * 1024;
    pub fn start(program_id: Pubkey) -> Self {
        Self {
            program_id,
            accounts: Vec::new(),
            index_by_name: BTreeMap::new(),
            last_logs: Vec::new(),
        }
    }

    pub fn add_account(&mut self, name: &str, seed: AccountSeed) {
        let idx = self.accounts.len();
        self.accounts.push(HarnessAccount {
            key: seed.key,
            owner: seed.owner,
            lamports: seed.lamports,
            data: seed.data,
            is_signer: seed.is_signer,
            is_writable: seed.is_writable,
            executable: seed.executable,
        });
        self.index_by_name.insert(name.to_string(), idx);
    }

    pub fn init_vm_state(&mut self, vm_state_name: &str, authority_name: &str) {
        let vm_idx = self.idx(vm_state_name);
        let authority_idx = self.idx(authority_name);

        let authority = self.accounts[authority_idx].key;
        let vm_state_data = &mut self.accounts[vm_idx].data;
        let vm_state = FIVEVMState::from_account_data_mut(vm_state_data)
            .expect("vm_state account must be allocated with FIVEVMState::LEN bytes");
        vm_state.initialize(authority, 0);
        vm_state.deploy_fee_lamports = 0;
        vm_state.execute_fee_lamports = 0;
    }

    pub fn set_vm_fees(
        &mut self,
        vm_state_name: &str,
        deploy_fee_lamports: u32,
        execute_fee_lamports: u32,
    ) {
        let vm_idx = self.idx(vm_state_name);
        let vm_state_data = &mut self.accounts[vm_idx].data;
        let vm_state = FIVEVMState::from_account_data_mut(vm_state_data)
            .expect("vm_state account must be allocated with FIVEVMState::LEN bytes");
        vm_state.deploy_fee_lamports = deploy_fee_lamports;
        vm_state.execute_fee_lamports = execute_fee_lamports;
    }

    pub fn deploy_script(
        &mut self,
        script_name: &str,
        vm_state_name: &str,
        owner_name: &str,
        bytecode: &[u8],
        permissions: u8,
        admin_name: Option<&str>,
    ) -> TxResult {
        let mut account_names = vec![script_name, vm_state_name, owner_name];
        if let Some(admin) = admin_name {
            account_names.push(admin);
        }
        if !self.index_by_name.contains_key("fee_vault") {
            let (fee_vault, _bump) = five_vm_mito::utils::find_program_address_offchain(
                &[b"\xFFfive_vm_fee_vault_v1", &[0u8]],
                &self.program_id,
            )
            .expect("derive fee_vault shard 0");
            self.add_account(
                "fee_vault",
                AccountSeed {
                    key: fee_vault,
                    owner: self.program_id,
                    lamports: 0,
                    data: Vec::new(),
                    is_signer: false,
                    is_writable: true,
                    executable: false,
                },
            );
        }
        if !self.index_by_name.contains_key("system_program") {
            let system_program = Pubkey::default();
            self.add_account(
                "system_program",
                AccountSeed {
                    key: system_program,
                    owner: system_program,
                    lamports: 0,
                    data: Vec::new(),
                    is_signer: false,
                    is_writable: false,
                    executable: false,
                },
            );
        }
        account_names.push("fee_vault");
        account_names.push("system_program");

        let program_id = self.program_id;
        let result = self.with_account_infos(&account_names, |accounts| {
            deploy(&program_id, accounts, bytecode, &[], permissions, 0)
        });

        match result {
            Ok(()) => TxResult::ok(),
            Err(e) => TxResult::err(e),
        }
    }

    pub fn execute_script(
        &mut self,
        script_name: &str,
        vm_state_name: &str,
        extra_account_names: &[&str],
        payload: &[u8],
    ) -> TxResult {
        let mut account_names = vec![script_name, vm_state_name];
        let canonical_extras = self.canonicalize_execute_extras(extra_account_names);
        let extra_refs: Vec<&str> = canonical_extras.iter().map(|s| s.as_str()).collect();
        account_names.extend_from_slice(&extra_refs);

        let program_id = self.program_id;
        let result = self.with_account_infos(&account_names, |accounts| {
            execute(&program_id, accounts, payload)
        });

        match result {
            Ok(()) => TxResult::ok(),
            Err(e) => TxResult::err(e),
        }
    }

    pub fn execute_script_raw(
        &mut self,
        script_name: &str,
        vm_state_name: &str,
        extra_account_names: &[&str],
        payload: &[u8],
    ) -> TxResult {
        let mut account_names = vec![script_name, vm_state_name];
        account_names.extend_from_slice(extra_account_names);

        let program_id = self.program_id;
        let result = self.with_account_infos(&account_names, |accounts| {
            execute(&program_id, accounts, payload)
        });

        match result {
            Ok(()) => TxResult::ok(),
            Err(e) => TxResult::err(e),
        }
    }

    fn canonicalize_execute_extras(&self, extra_account_names: &[&str]) -> Vec<String> {
        // Canonical execute account ordering is:
        // [business extras..., payer, fee_vault, system_program].
        let mut extras: Vec<String> = Vec::with_capacity(extra_account_names.len() + 3);

        for name in extra_account_names {
            if *name == "fee_vault" || *name == "system_program" {
                continue;
            }
            extras.push((*name).to_string());
        }

        let payer_candidate = self
            .index_by_name
            .get("payer")
            .and_then(|idx| {
                let acc = &self.accounts[*idx];
                if acc.is_signer && acc.is_writable {
                    Some("payer".to_string())
                } else {
                    None
                }
            })
            .or_else(|| {
                self.index_by_name.get("owner").and_then(|idx| {
                    let acc = &self.accounts[*idx];
                    if acc.is_signer && acc.is_writable {
                        Some("owner".to_string())
                    } else {
                        None
                    }
                })
            })
            .or_else(|| {
                self.index_by_name
                    .iter()
                    .filter_map(|(name, idx)| {
                        let acc = &self.accounts[*idx];
                        if acc.is_signer && acc.is_writable {
                            Some((name.clone(), acc.lamports))
                        } else {
                            None
                        }
                    })
                    .max_by(|(name_a, lamports_a), (name_b, lamports_b)| {
                        lamports_a.cmp(lamports_b).then_with(|| name_b.cmp(name_a))
                    })
                    .map(|(name, _)| name)
            });

        if let Some(payer_name) = payer_candidate {
            // Keep original business account ordering stable so typed account
            // indices in execute payload remain deterministic.
            // Append payer in the canonical fee tail slot even if it duplicates
            // an earlier business account.
            extras.push(payer_name);
        }
        if self.index_by_name.contains_key("fee_vault") {
            extras.push("fee_vault".to_string());
        }
        if self.index_by_name.contains_key("system_program") {
            extras.push("system_program".to_string());
        }
        extras
    }

    pub fn fetch_account(&self, name: &str) -> AccountSnapshot {
        let idx = self.idx(name);
        let acc = &self.accounts[idx];
        AccountSnapshot {
            key: acc.key,
            owner: acc.owner,
            lamports: acc.lamports,
            data: acc.data.clone(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
            executable: acc.executable,
        }
    }

    pub fn set_account_data(&mut self, name: &str, data: Vec<u8>) {
        let idx = self.idx(name);
        self.accounts[idx].data = data;
    }

    pub fn create_script_account_seed(owner: Pubkey, bytecode_len: usize) -> AccountSeed {
        let mut data = vec![0u8; ScriptAccountHeader::LEN + bytecode_len];
        // Keep as zeroed/uninitialized until deploy writes header+bytecode.
        data.fill(0);

        AccountSeed {
            key: unique_pubkey(200),
            owner,
            lamports: 0,
            data,
            is_signer: false,
            is_writable: true,
            executable: false,
        }
    }

    pub fn create_vm_state_seed(owner: Pubkey) -> AccountSeed {
        AccountSeed {
            key: unique_pubkey(201),
            owner,
            lamports: 0,
            data: vec![0u8; FIVEVMState::LEN],
            is_signer: false,
            is_writable: true,
            executable: false,
        }
    }

    pub fn assert_logs(&self, expected_substr: &str) {
        let combined = self.last_logs.join("\n");
        assert!(
            combined.contains(expected_substr),
            "expected logs to contain `{}`, got: {}",
            expected_substr,
            combined
        );
    }

    pub fn run_fixture(
        &mut self,
        fixture: &ScriptFixture,
        script_name: &str,
        vm_state_name: &str,
        owner_name: &str,
    ) -> TxResult {
        for (name, seed) in &fixture.initial_accounts {
            if !self.index_by_name.contains_key(name) {
                self.add_account(name, seed.clone());
            }
        }

        let deploy_result = self.deploy_script(
            script_name,
            vm_state_name,
            owner_name,
            &fixture.bytecode,
            fixture.permissions,
            None,
        );

        if !deploy_result.success {
            return deploy_result;
        }

        // For fixture-driven execution, maintain canonical execute ordering:
        // custom extras first, then payer, fee_vault, system_program tail.
        let mut extras: Vec<String> = self
            .index_by_name
            .keys()
            .filter(|k| {
                let name = k.as_str();
                name != script_name
                    && name != vm_state_name
                    && name != owner_name
                    && name != "fee_vault"
                    && name != "system_program"
            })
            .map(|k| k.to_string())
            .collect();
        extras.sort_unstable();
        if self.index_by_name.contains_key(owner_name) {
            extras.push(owner_name.to_string());
        }
        if self.index_by_name.contains_key("fee_vault") {
            extras.push("fee_vault".to_string());
        }
        if self.index_by_name.contains_key("system_program") {
            extras.push("system_program".to_string());
        }
        let extra_refs: Vec<&str> = extras.iter().map(|s| s.as_str()).collect();

        let execute_result = self.execute_script(
            script_name,
            vm_state_name,
            &extra_refs,
            &fixture.execute_payload,
        );

        match &fixture.expectation {
            ExpectedOutcome::Success => {
                assert!(
                    execute_result.success,
                    "expected fixture execution success, got: {:?}",
                    execute_result.error
                );
            }
            ExpectedOutcome::ProgramError(expected) => {
                assert_eq!(
                    execute_result.error,
                    Some(*expected),
                    "fixture returned unexpected error"
                );
            }
        }

        execute_result
    }

    fn idx(&self, name: &str) -> usize {
        *self
            .index_by_name
            .get(name)
            .unwrap_or_else(|| panic!("unknown harness account: {}", name))
    }

    fn with_account_infos<R>(
        &mut self,
        account_names: &[&str],
        f: impl FnOnce(&[AccountInfo]) -> Result<R, ProgramError>,
    ) -> Result<R, ProgramError> {
        self.last_logs.clear();

        let mut indices = Vec::with_capacity(account_names.len());
        for &name in account_names {
            indices.push(self.idx(name));
        }

        // Snapshot current account state into temporary host-side pinocchio accounts.
        let mut keys = Vec::with_capacity(indices.len());
        let mut owners = Vec::with_capacity(indices.len());
        let mut lamports = Vec::with_capacity(indices.len());
        let mut data = Vec::with_capacity(indices.len());
        let mut signer = Vec::with_capacity(indices.len());
        let mut writable = Vec::with_capacity(indices.len());
        let mut executable = Vec::with_capacity(indices.len());

        for &idx in &indices {
            let acc = &self.accounts[idx];
            keys.push(acc.key);
            owners.push(acc.owner);
            lamports.push(acc.lamports);
            data.push(acc.data.clone());
            signer.push(acc.is_signer);
            writable.push(acc.is_writable);
            executable.push(acc.executable);
        }

        let mut infos = Vec::with_capacity(indices.len());
        for i in 0..indices.len() {
            let logical_len = data[i].len();
            if writable[i] && !executable[i] {
                let reserve_target = logical_len.saturating_add(Self::OFFCHAIN_DATA_HEADROOM);
                if reserve_target > data[i].len() {
                    data[i].resize(reserve_target, 0);
                }
            }

            infos.push(AccountInfo::new(
                &keys[i],
                signer[i],
                writable[i],
                &mut lamports[i],
                data[i].as_mut_slice(),
                &owners[i],
                executable[i],
                0,
            ));

            if data[i].len() != logical_len {
                unsafe {
                    infos[i]
                        .resize_unchecked(logical_len)
                        .map_err(|_| ProgramError::InvalidAccountData)?;
                }
            }
        }

        let out = f(&infos);

        // Copy mutated account state back into the harness store.
        for (local_idx, &global_idx) in indices.iter().enumerate() {
            let info = &infos[local_idx];
            let acc = &mut self.accounts[global_idx];
            let data_len = info.data_len();
            if data_len > Self::MAX_ACCOUNT_DATA_LEN {
                return Err(ProgramError::InvalidAccountData);
            }
            acc.lamports = info.lamports();
            acc.owner = *info.owner();
            acc.data = unsafe { info.borrow_data_unchecked().to_vec() };
        }

        out
    }
}

#[cfg(feature = "legacy-runtime-harness")]
pub fn unique_pubkey(seed: u8) -> Pubkey {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    bytes
}

pub fn script_with_header(public_count: u8, total_count: u8, body: &[u8]) -> Vec<u8> {
    let mut script = vec![
        b'5',
        b'I',
        b'V',
        b'E',
        0x00,
        0x00,
        0x00,
        0x00,
        public_count,
        total_count,
    ];
    script.extend_from_slice(body);
    script
}
