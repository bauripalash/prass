use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use super::Object;

static DEFKEY: &str = "__default";

#[derive(Debug, Clone, PartialOrd, Ord)]
pub struct Env {
    env: BTreeMap<String, Object>,
    outer: Option<Rc<Env>>,
}

impl PartialEq for Env {
    fn eq(&self, other: &Self) -> bool {
        let k: Vec<String> = self.env.keys().cloned().collect();
        let ko: Vec<String> = other.env.keys().cloned().collect();

        if k != ko {
            return false;
        }

        let v: Vec<Object> = self.env.values().cloned().collect();
        let vo: Vec<Object> = other.env.values().cloned().collect();

        if v != vo {
            return false;
        }

        true
    }
}

impl Eq for Env {}
impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}
impl Env {
    pub const fn new() -> Self {
        Self {
            env: BTreeMap::new(),
            outer: None,
        }
    }

    pub fn set_val(&mut self, key: String, value: Object) -> Option<Object> {
        self.env.insert(key, value)
    }

    pub fn get_val(&self, key: String) -> Option<&Object> {
        let mut v = self.env.get(&key);

        if v.is_none() {
            if let Some(out) = &self.outer {
                v = out.get_val(key)
            }
        }

        v
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envmap {
    envs: HashMap<String, Env>,
}

impl Default for Envmap {
    fn default() -> Self {
        Self::new()
    }
}

impl Envmap {
    pub fn new() -> Self {
        let envs: HashMap<String, Env> = HashMap::from([(DEFKEY.to_string(), Env::new())]);

        Self { envs }
    }

    pub fn get_default(&mut self) -> Result<&mut Env, bool> {
        if let Some(e) = self.envs.get_mut(&DEFKEY.to_string()) {
            return Ok(e);
        }
        Err(false)
    }

    pub fn get_env(&mut self, en: String) -> Result<&mut Env, bool> {
        if let Some(e) = self.envs.get_mut(&en) {
            return Ok(e);
        }
        Err(false)
    }

    pub fn get_from_default(&mut self, key: String) -> Result<&Object, Object> {
        if let Ok(e) = self.get_default() {
            if let Some(v) = e.env.get(&key) {
                return Ok(v);
            }
        }

        Err(Object::Null)
    }

    pub fn get_from(&mut self, env: String, key: String) -> Result<&Object, Object> {
        if let Ok(e) = self.get_env(env) {
            if let Some(v) = e.env.get(&key) {
                return Ok(v);
            }
        }

        Err(Object::Null)
    }

    pub fn set_to_default(&mut self, key: String, value: Object) -> Option<Object> {
        if let Ok(x) = self.get_default() {
            return x.set_val(key, value);
        }

        None
    }

    pub fn set_to(&mut self, env: String, key: String, value: Object) -> Option<Object> {
        if let Ok(x) = self.get_env(env) {
            return x.set_val(key, value);
        }

        None
    }
}
