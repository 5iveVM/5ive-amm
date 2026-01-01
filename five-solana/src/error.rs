use pinocchio::program_error::ProgramError;

/// Safety documentation for error handling
///
/// All error conversions maintain the invariant that VM errors
/// are faithfully mapped to unique program error codes.
/// This ensures debuggability and prevents error code collisions.

/// Program-specific errors
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FIVEError {
    /// Invalid account data provided
    InvalidAccountData = 6000,
    /// Version history reached its limit
    VersionHistoryFull = 6001,
}

impl From<FIVEError> for ProgramError {
    #[inline(always)]
    fn from(err: FIVEError) -> Self {
        ProgramError::Custom(err as u32)
    }
}

/// Error for when program operations are attempted before initialization
#[inline(always)]
pub fn program_not_initialized_error() -> ProgramError {
    ProgramError::Custom(1022)
}

/// Error for when initialization is attempted on an already initialized program
#[allow(dead_code)]
#[inline(always)]
pub fn program_already_initialized_error() -> ProgramError {
    ProgramError::Custom(1023)
}
