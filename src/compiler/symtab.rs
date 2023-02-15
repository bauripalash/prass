use std::{collections::HashMap, rc::Rc};

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
    pub store: HashMap<String, Symbol>,
    pub numdef: usize,
    pub free_syms : Vec<Symbol>
}

impl Table {
    pub fn new() -> Self {
        Self {
            outer: None,
            store: HashMap::new(),
            numdef: 0,
            free_syms : Vec::new()
        }
    }

    pub fn new_enclosed(outer: Self) -> Self {
        Self {
            outer: Some(Rc::new(outer)),
            store: HashMap::new(),
            numdef: 0,
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

    pub fn define(&mut self, name: String) -> Symbol {
        let mut sm = Symbol {
            name: name.clone(),
            index: self.numdef,
            scope : Scope::default(),
        };

        if self.outer.is_some(){

            sm.scope = Scope::Local;
        }
        self.store.insert(name, sm.clone());
        self.numdef += 1;
        sm
    }
    pub fn define_func(&mut self , name: String) -> Symbol{
        let s = Symbol{ name : name.clone() , index : 0 , scope : Scope::Func };
        self.store.insert(name, s.clone());
        s
    }

    pub fn define_free(&mut self , org : Symbol) -> Symbol {
        self.free_syms.push(org.clone());
        let sm = Symbol{ name : org.name.clone() , index : self.free_syms.len() - 1  , scope : Scope::Free};
        self.store.insert(org.name, sm.clone());
        sm
    }

    pub fn resolve(&mut self, name: String) -> Result<Symbol, bool> {
        let obj = self.store.get(&name);

        if obj.is_none() && self.outer.is_some(){
            let obx = self.get_outer().unwrap().resolve(name.clone());
            if obx.is_err() {
                return Err(false)
            }
            let unwrapped_obx = obx.unwrap() ;
            if unwrapped_obx.scope == Scope::Global{
                return Ok(unwrapped_obx)
            }

            let fr = self.define_free(unwrapped_obx);
            return Ok(fr)
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
mod tests{
    use super::*;
    #[test]
    fn define(){
        let mut tab = Table::new();
        let a = tab.define("a".to_string());
        assert_eq!(a , Symbol{ name :"a".to_string() , scope : Scope::Global , index : 0 });
    }

    #[test]
    fn resolve_free() {
        let mut t = Table::new();
        t.define("a".to_string());
        t.define("b".to_string());
        let mut tt = Table::new_enclosed(t);
        let rs = tt.resolve("a".to_string()).expect("expected to resolve");
        assert_eq!(rs , Symbol{ scope : Scope::Global , index : 0 , name : "a".to_owned() });
        tt.define("c".to_string());
          let rs = tt.resolve("c".to_string()).expect("expected to resolve");
        assert_eq!(rs , Symbol{ scope : Scope::Local , index : 0 , name : "c".to_owned() });
        tt.define_free(Symbol { name: "d".to_string(), scope: Scope::Free, index: 4 });
        let rs = tt.resolve("d".to_string()).expect("expected to resolve");
        assert_eq!(rs , Symbol{ scope : Scope::Free , index : 0 , name : "d".to_owned() });
    }
}
