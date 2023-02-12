use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Scope {
    #[default]
    Global,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub scope: Scope,
    pub index: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Table {
    pub store: HashMap<String, Symbol>,
    pub numdef: usize,
}

impl Table {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            numdef: 0,
        }
    }

    pub fn define(&mut self, name: String) -> Symbol {
        let sm = Symbol {
            name: name.clone(),
            index: self.numdef,
            scope: Scope::Global,
        };
        self.store.insert(name, sm.clone());
        self.numdef += 1;
        sm
    }

    pub fn resolve(&self, name: String) -> Result<Symbol, bool> {
        self.store
            .get(&name)
            .map_or(Err(false), |n| Ok(n.to_owned()))
    }
}
