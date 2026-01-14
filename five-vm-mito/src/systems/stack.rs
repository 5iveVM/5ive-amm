use five_protocol::ValueRef;
use crate::error::{CompactResult, VMErrorCode};
use crate::STACK_SIZE;

/// Manages the operand stack for the VM.
pub struct StackManager<'a> {
    pub stack: &'a mut [ValueRef],
    pub sp: u8,
    pub registers: &'a mut [ValueRef],
}

impl<'a> StackManager<'a> {
    #[inline(always)]
    pub fn new(stack: &'a mut [ValueRef], registers: &'a mut [ValueRef]) -> Self {
        Self { stack, sp: 0, registers }
    }

    #[inline(always)]
    pub fn push(&mut self, value: ValueRef) -> CompactResult<()> {
        if self.sp as usize >= STACK_SIZE {
            return Err(VMErrorCode::StackOverflow);
        }
        // Safety: sp check above ensures we are within bounds of STACK_SIZE
        // and STACK_SIZE is checked to be within slice bounds by StackStorage construction
        self.stack[self.sp as usize] = value;
        self.sp += 1;
        Ok(())
    }

    #[inline(always)]
    pub fn pop(&mut self) -> CompactResult<ValueRef> {
        if self.sp == 0 {
            return Err(VMErrorCode::StackUnderflow);
        }
        self.sp -= 1;
        Ok(self.stack[self.sp as usize])
    }

    #[inline(always)]
    pub fn peek(&self) -> CompactResult<ValueRef> {
        if self.sp == 0 {
            return Err(VMErrorCode::StackUnderflow);
        }
        Ok(self.stack[self.sp as usize - 1])
    }

    #[inline(always)]
    pub fn dup(&mut self) -> CompactResult<()> {
        let value = self.peek()?;
        self.push(value)
    }

    #[inline(always)]
    pub fn swap(&mut self) -> CompactResult<()> {
        if self.sp < 2 {
            return Err(VMErrorCode::StackUnderflow);
        }
        let idx = self.sp as usize;
        self.stack.swap(idx - 1, idx - 2);
        Ok(())
    }

    #[inline(always)]
    pub fn pick(&mut self, depth: u8) -> CompactResult<()> {
        if depth >= self.sp {
            return Err(VMErrorCode::StackUnderflow);
        }
        let idx = self.sp as usize - 1 - depth as usize;
        let value = self.stack[idx];
        self.push(value)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.sp as usize
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.sp == 0
    }

    #[inline(always)]
    pub fn get_register(&self, index: u8) -> CompactResult<ValueRef> {
        if index >= 8 {
            return Err(VMErrorCode::InvalidRegister);
        }
        Ok(self.registers[index as usize])
    }

    #[inline(always)]
    pub fn set_register(&mut self, index: u8, value: ValueRef) -> CompactResult<()> {
        if index >= 8 {
            return Err(VMErrorCode::InvalidRegister);
        }
        self.registers[index as usize] = value;
        Ok(())
    }
}
