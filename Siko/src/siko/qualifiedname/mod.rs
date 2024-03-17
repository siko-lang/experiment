use std::fmt::{Debug, Display};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum QualifiedName {
    Module(String),
    Instance(Box<QualifiedName>, u64),
    Item(Box<QualifiedName>, String),
}

impl QualifiedName {
    pub fn add(&self, item: String) -> QualifiedName {
        QualifiedName::Item(Box::new(self.clone()), item)
    }
}

impl Debug for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            QualifiedName::Module(i) => write!(f, "{}", i),
            QualifiedName::Instance(p, i) => write!(f, "{}/{}", p, i),
            QualifiedName::Item(p, i) => write!(f, "{}.{}", p, i),
        }
    }
}
