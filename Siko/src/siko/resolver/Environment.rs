use std::collections::BTreeMap;

use crate::siko::hir::Function::{ValueKind, Variable};

pub struct Environment<'a> {
    values: BTreeMap<String, ValueKind>,
    parent: Option<&'a Environment<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Environment<'a> {
        Environment {
            values: BTreeMap::new(),
            parent: None,
        }
    }

    pub fn child(parent: &'a Environment<'a>) -> Environment<'a> {
        Environment {
            values: BTreeMap::new(),
            parent: Some(parent),
        }
    }

    pub fn addArg(&mut self, arg: String, index: i64) {
        self.values.insert(arg.clone(), ValueKind::Arg(arg, index));
    }

    pub fn addValue(&mut self, old: String, new: Variable) {
        //println!("Added value {}", new);
        self.values.insert(old.clone(), ValueKind::Value(new));
    }

    pub fn addTmpValue(&mut self, name: Variable) {
        //println!("Added value {}", new);
        self.values.insert(name.value.clone(), ValueKind::Value(name));
    }

    pub fn resolve(&self, value: &String) -> Option<ValueKind> {
        match self.values.get(value) {
            Some(v) => Some(v.clone()),
            None => {
                if let Some(parent) = self.parent {
                    return parent.resolve(value);
                } else {
                    None
                }
            }
        }
    }
}
