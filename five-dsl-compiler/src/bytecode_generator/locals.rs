use super::types::FieldInfo;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Default)]
pub struct LocalSymbolTable(HashMap<String, FieldInfo>);

impl Deref for LocalSymbolTable {
    type Target = HashMap<String, FieldInfo>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LocalSymbolTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
