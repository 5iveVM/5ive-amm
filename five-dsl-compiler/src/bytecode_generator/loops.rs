use super::types::LoopContext;

#[derive(Default)]
pub struct LoopStack {
    frames: Vec<LoopContext>,
}

impl LoopStack {
    #[allow(dead_code)]
    pub fn push(&mut self, ctx: LoopContext) {
        self.frames.push(ctx);
    }

    #[allow(dead_code)]
    pub fn pop(&mut self) -> Option<LoopContext> {
        self.frames.pop()
    }

    #[allow(dead_code)]
    pub fn last_mut(&mut self) -> Option<&mut LoopContext> {
        self.frames.last_mut()
    }
}

impl Clone for LoopStack {
    fn clone(&self) -> Self {
        Self {
            frames: self.frames.clone(),
        }
    }
}
