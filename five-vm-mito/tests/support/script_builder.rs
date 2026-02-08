#![allow(dead_code)]

use std::collections::HashMap;

use five_protocol::{
    opcodes::*, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC, MAX_FUNCTIONS,
};

/// Errors returned by [`ScriptBuilder`].
#[derive(Debug, PartialEq, Eq)]
pub enum ScriptBuilderError {
    /// Function name already exists.
    DuplicateFunction(String),
    /// A CALL referenced an unknown function name.
    UnknownFunction(String),
    /// No functions were defined before calling [`ScriptBuilder::build`].
    NoFunctions,
    /// No public functions were defined (at least one is required for entry).
    NoPublicFunctions,
    /// Function count exceeds protocol maximum.
    FunctionLimitExceeded(usize),
    /// Function count cannot be represented in header ( > u8::MAX ).
    FunctionCountOverflow(usize),
    /// Computed function address exceeds `u16::MAX` (VM uses 16-bit addresses).
    AddressOverflow(String),
    /// Jump referenced a label that does not exist within the function.
    UnknownLabel(String),
}

impl core::fmt::Display for ScriptBuilderError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ScriptBuilderError::DuplicateFunction(name) => {
                write!(f, "duplicate function name: {}", name)
            }
            ScriptBuilderError::UnknownFunction(name) => {
                write!(f, "unknown function referenced by CALL: {}", name)
            }
            ScriptBuilderError::NoFunctions => write!(f, "no functions defined"),
            ScriptBuilderError::NoPublicFunctions => write!(f, "no public functions defined"),
            ScriptBuilderError::FunctionLimitExceeded(count) => write!(
                f,
                "function count {} exceeds MAX_FUNCTIONS {}",
                count, MAX_FUNCTIONS
            ),
            ScriptBuilderError::FunctionCountOverflow(count) => {
                write!(f, "function count {} cannot fit in header", count)
            }
            ScriptBuilderError::AddressOverflow(name) => {
                write!(f, "function {} start exceeds u16::MAX", name)
            }
            ScriptBuilderError::UnknownLabel(name) => {
                write!(f, "unknown label referenced by jump: {}", name)
            }
        }
    }
}

impl std::error::Error for ScriptBuilderError {}

/// Visibility of a function within the assembled script.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionVisibility {
    Public,
    Private,
}

#[derive(Debug)]
struct FunctionSpec {
    name: String,
    visibility: FunctionVisibility,
    body: FunctionBody,
}

#[derive(Default, Debug)]
pub struct ScriptBuilder {
    features: u8,
    functions: Vec<FunctionSpec>,
}

impl ScriptBuilder {
    pub fn build_script(build: impl FnOnce(&mut ScriptBuilder)) -> Vec<u8> {
        let mut builder = ScriptBuilder::new();
        build(&mut builder);
        builder.build().expect("script assembly should succeed")
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_features(&mut self, features: u8) -> &mut Self {
        self.features = features;
        self
    }

    pub fn with_features(mut self, features: u8) -> Self {
        self.features = features;
        self
    }

    pub fn public_function<F>(
        &mut self,
        name: impl Into<String>,
        build: F,
    ) -> Result<&mut Self, ScriptBuilderError>
    where
        F: FnOnce(&mut FunctionBuilder),
    {
        self.add_function(name.into(), FunctionVisibility::Public, build)?;
        Ok(self)
    }

    pub fn private_function<F>(
        &mut self,
        name: impl Into<String>,
        build: F,
    ) -> Result<&mut Self, ScriptBuilderError>
    where
        F: FnOnce(&mut FunctionBuilder),
    {
        self.add_function(name.into(), FunctionVisibility::Private, build)?;
        Ok(self)
    }

    fn add_function<F>(
        &mut self,
        name: String,
        visibility: FunctionVisibility,
        build: F,
    ) -> Result<(), ScriptBuilderError>
    where
        F: FnOnce(&mut FunctionBuilder),
    {
        if self.functions.iter().any(|f| f.name == name) {
            return Err(ScriptBuilderError::DuplicateFunction(name));
        }

        let mut builder = FunctionBuilder::default();
        build(&mut builder);
        self.functions.push(FunctionSpec {
            name,
            visibility,
            body: builder.finish(),
        });
        Ok(())
    }

    pub fn build(self) -> Result<Vec<u8>, ScriptBuilderError> {
        if self.functions.is_empty() {
            return Err(ScriptBuilderError::NoFunctions);
        }

        let total_functions = self.functions.len();
        let public_functions = self
            .functions
            .iter()
            .filter(|f| f.visibility == FunctionVisibility::Public)
            .count();

        if public_functions == 0 {
            return Err(ScriptBuilderError::NoPublicFunctions);
        }

        if total_functions > MAX_FUNCTIONS {
            return Err(ScriptBuilderError::FunctionLimitExceeded(total_functions));
        }

        if total_functions > u8::MAX as usize {
            return Err(ScriptBuilderError::FunctionCountOverflow(total_functions));
        }

        let mut function_offsets: HashMap<String, usize> = HashMap::new();
        let total_body_len: usize = self.functions.iter().map(|f| f.body.code.len()).sum();

        let mut script = Vec::with_capacity(FIVE_HEADER_OPTIMIZED_SIZE + total_body_len);
        script.extend_from_slice(&FIVE_MAGIC);
        // Header V3: features(4 bytes LE) + public_function_count(1) + total_function_count(1)
        let features_u32 = self.features as u32;
        script.extend_from_slice(&features_u32.to_le_bytes());
        script.push(public_functions as u8);
        script.push(total_functions as u8);

        let mut cursor = FIVE_HEADER_OPTIMIZED_SIZE;
        for func in &self.functions {
            function_offsets.insert(func.name.clone(), cursor);
            cursor += func.body.code.len();
            script.extend_from_slice(&func.body.code);
        }

        for func in &self.functions {
            let func_start = *function_offsets.get(&func.name).expect("function offset");
            for patch in &func.body.call_patches {
                let target_offset = function_offsets
                    .get(&patch.target)
                    .ok_or_else(|| ScriptBuilderError::UnknownFunction(patch.target.clone()))?;
                if *target_offset > u16::MAX as usize {
                    return Err(ScriptBuilderError::AddressOverflow(patch.target.clone()));
                }
                let absolute_patch = func_start + patch.local_offset;
                let addr_bytes = (*target_offset as u16).to_le_bytes();
                script[absolute_patch] = addr_bytes[0];
                script[absolute_patch + 1] = addr_bytes[1];
            }

            for patch in &func.body.jump_patches {
                let label_offset = func
                    .body
                    .labels
                    .get(&patch.target)
                    .ok_or_else(|| ScriptBuilderError::UnknownLabel(patch.target.clone()))?;
                let absolute_patch = func_start + patch.local_offset;
                let absolute_target = func_start + label_offset;
                let addr_bytes = (absolute_target as u16).to_le_bytes();
                script[absolute_patch] = addr_bytes[0];
                script[absolute_patch + 1] = addr_bytes[1];
            }
        }

        Ok(script)
    }
}

#[derive(Debug)]
struct FunctionBody {
    code: Vec<u8>,
    call_patches: Vec<CallPatch>,
    jump_patches: Vec<JumpPatch>,
    labels: HashMap<String, usize>,
}

impl FunctionBody {
    fn new(
        code: Vec<u8>,
        call_patches: Vec<CallPatch>,
        jump_patches: Vec<JumpPatch>,
        labels: HashMap<String, usize>,
    ) -> Self {
        Self {
            code,
            call_patches,
            jump_patches,
            labels,
        }
    }
}

#[derive(Default, Debug)]
pub struct FunctionBuilder {
    code: Vec<u8>,
    call_patches: Vec<CallPatch>,
    jump_patches: Vec<JumpPatch>,
    labels: HashMap<String, usize>,
}

impl FunctionBuilder {
    pub fn emit(&mut self, byte: u8) -> &mut Self {
        self.code.push(byte);
        self
    }

    pub fn emit_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.code.extend_from_slice(bytes);
        self
    }

    pub fn push_u64(&mut self, value: u64) -> &mut Self {
        self.code.push(PUSH_U64);
        self.code.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn push_u8(&mut self, value: u8) -> &mut Self {
        self.code.push(PUSH_U8);
        self.code.push(value);
        self
    }

    pub fn push_bool(&mut self, value: bool) -> &mut Self {
        self.code.push(PUSH_BOOL);
        self.code.push(value as u8);
        self
    }

    /// Push LOAD_PARAM with the compiler-style 1-based parameter index.
    pub fn load_param(&mut self, index: u8) -> &mut Self {
        self.code.push(LOAD_PARAM);
        self.code.push(index);
        self
    }

    pub fn call(&mut self, target: impl Into<String>, param_count: u8) -> &mut Self {
        self.code.push(CALL);
        self.code.push(param_count);
        let patch_offset = self.code.len();
        self.code.extend_from_slice(&[0xFF, 0xFF]);
        self.call_patches.push(CallPatch {
            target: target.into(),
            local_offset: patch_offset,
        });
        self
    }

    pub fn call_raw(&mut self, param_count: u8, absolute_addr: u16) -> &mut Self {
        self.code.push(CALL);
        self.code.push(param_count);
        self.code.extend_from_slice(&absolute_addr.to_le_bytes());
        self
    }

    pub fn return_value(&mut self) -> &mut Self {
        self.code.push(RETURN_VALUE);
        self
    }

    pub fn ret(&mut self) -> &mut Self {
        self.code.push(RETURN);
        self
    }

    pub fn halt(&mut self) -> &mut Self {
        self.code.push(HALT);
        self
    }

    pub fn code_mut(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    pub fn push_i64(&mut self, value: i64) -> &mut Self {
        self.code.push(PUSH_I64);
        self.code.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn label(&mut self, name: impl Into<String>) -> &mut Self {
        let name = name.into();
        self.labels.insert(name, self.code.len());
        self
    }

    pub fn jump(&mut self, label: impl Into<String>) -> &mut Self {
        self.emit_jump(JUMP, label)
    }

    pub fn jump_if(&mut self, label: impl Into<String>) -> &mut Self {
        self.emit_jump(JUMP_IF, label)
    }

    fn emit_jump(&mut self, opcode: u8, label: impl Into<String>) -> &mut Self {
        self.code.push(opcode);
        let patch_offset = self.code.len();
        self.code.extend_from_slice(&[0xFF, 0xFF]);
        self.jump_patches.push(JumpPatch {
            target: label.into(),
            local_offset: patch_offset,
        });
        self
    }

    fn finish(self) -> FunctionBody {
        FunctionBody::new(self.code, self.call_patches, self.jump_patches, self.labels)
    }
}

#[derive(Debug)]
struct CallPatch {
    target: String,
    local_offset: usize,
}

#[derive(Debug)]
struct JumpPatch {
    target: String,
    local_offset: usize,
}
