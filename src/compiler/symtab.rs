use std::{collections::HashMap, rc::Rc, cell::Ref};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Scope {
    #[default]
    Global,
    Local,
    Free,
    Func,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub scope: Scope,
    pub index: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Table {
    pub outer: Option<Rc<Table>>,
    pub store: HashMap<String, Rc<Symbol>>,
    pub numdef: usize,
    pub free_syms: Vec<Rc<Symbol>>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            outer: None,
            store: HashMap::new(),
            numdef: 0,
            free_syms: Vec::new(),
        }
    }

    pub fn new_enclosed(outer: Ref<Self>) -> Self {
        Self {
            outer: Some(Rc::new(outer.clone())),
            store: HashMap::new(),
            numdef: 0,
            free_syms: Vec::new(),
        }
    }

    pub fn new_enclosed_noref(outer : Rc<Self>) -> Self{
            Self{
                outer : Some(outer),
                store : HashMap::new(),
                numdef : 0,
                free_syms : Vec::new()
            }
    }

    

    pub fn get_outer(&self) -> Option<Self> {
        if let Some(o) = &self.outer {
            let x = o.as_ref();
            return Some(x.clone());
        }
        None
    }

    pub fn get_outer_no_check(&self) -> Self{
        self.outer.as_ref().unwrap().as_ref().clone()
    }

    pub fn define(&mut self, name: &str) -> Rc<Symbol> {
        let sm = Rc::new(Symbol {
            name: name.to_string(),
            index: self.numdef,
            scope: if self.outer.is_some() { Scope::Local } else { Scope::default() } ,
        });

        self.store.insert(name.to_string() , sm.clone());
        self.numdef += 1;
        sm
    }
    pub fn define_func(&mut self, name: String) -> Rc<Symbol> {
        let s = Rc::new(Symbol {
            name: name.clone(),
            index: 0,
            scope: Scope::Func,
        });
        self.store.insert(name, s.clone());
        s
    }

    pub fn define_free(&mut self, org: Rc<Symbol>) -> Rc<Symbol> {
        self.free_syms.push(org.clone());
        let sm = Rc::new(Symbol {
            name: org.name.clone(),
            index: self.free_syms.len() - 1,
            scope: Scope::Free,
        });
        self.store.insert(org.name.clone(), sm.clone());
        sm
        
    }

    pub fn resolve(&mut self, name: String) -> Result<Rc<Symbol>, bool> {
        let obj = self.store.get(&name);

        if obj.is_none() && self.outer.is_some() {
            let obx = self.get_outer().unwrap().resolve(name.clone());
            if obx.is_err() {
                return Err(false);
            }
            let unwrapped_obx = obx.unwrap();
            if unwrapped_obx.scope == Scope::Global {
                return Ok(unwrapped_obx);
            }

            let fr = self.define_free(unwrapped_obx);
            return Ok(fr);
        }

        //        if let Some(x) = obj{
        //            Ok(x.clone())
        //        }else{
        //            Err(false)
        //        }

        obj.map_or(Err(false), |x| Ok(x.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn define() {
        let mut tab = Table::new();
        let a = tab.define("a");
        assert_eq!(
            a,
            Symbol {
                name: "a".to_string(),
                scope: Scope::Global,
                index: 0
            }.into()
        );
    }

    #[test]
    fn resolve_free() {
        let mut t = Table::new();
        t.define("a");
        t.define("b");
        let mut tt = Table::new_enclosed_noref(t.into());
        let rs = tt.resolve("a".to_string()).expect("expected to resolve");
        assert_eq!(
            rs,
            Symbol {
                scope: Scope::Global,
                index: 0,
                name: "a".to_owned()
            }.into()
        );
        tt.define("c");
        let rs = tt.resolve("c".to_string()).expect("expected to resolve");
        assert_eq!(
            rs,
            Symbol {
                scope: Scope::Local,
                index: 0,
                name: "c".to_owned()
            }.into()
        );
        tt.define_free(Symbol {
            name: "d".to_string(),
            scope: Scope::Free,
            index: 4,
        }.into());
        let rs = tt.resolve("d".to_string()).expect("expected to resolve");
        assert_eq!(
            rs,
            Symbol {
                scope: Scope::Free,
                index: 0,
                name: "d".to_owned()
            }.into()
        );
    }
}
