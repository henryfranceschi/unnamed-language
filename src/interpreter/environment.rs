use std::collections::HashMap;

use super::value::Value;

#[derive(Debug, Default)]
pub struct Environment {
    parent: Option<Box<Environment>>,
    map: HashMap<String, Value>,
}

impl Environment {
    pub fn define(&mut self, name: &str, value: Value) {
        self.map.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.find(name).copied()
    }

    pub fn set(&mut self, name: &str, mut value: Value) -> Option<Value> {
        std::mem::swap(self.find_mut(name)?, &mut value);

        Some(value)
    }

    fn find(&self, name: &str) -> Option<&Value> {
        self.map
            .get(name)
            .or_else(|| self.parent.as_ref()?.find(name))
    }

    fn find_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.map
            .get_mut(name)
            .or_else(|| self.parent.as_mut()?.find_mut(name))
    }

    pub fn push(&mut self) {
        let parent = std::mem::take(self);
        self.parent.replace(Box::new(parent));
    }

    pub fn pop(&mut self) {
        let parent = self.parent.take().expect("pop called on root environment");
        let _ = std::mem::replace(self, *parent);
    }
}
