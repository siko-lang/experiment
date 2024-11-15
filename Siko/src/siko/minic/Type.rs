#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Void,
    Int8,
    Int16,
    Int32,
    Int64,
    Struct(String),
    Ptr(Box<Type>),
    Array(u32, u32),
}

impl Type {
    pub fn isPtr(&self) -> bool {
        match self {
            Type::Ptr(_) => true,
            _ => false,
        }
    }

    pub fn isArray(&self) -> bool {
        match self {
            Type::Array(_, _) => true,
            _ => false,
        }
    }

    pub fn getArraySize(&self) -> u32 {
        match self {
            Type::Array(s, _) => *s,
            _ => unreachable!(),
        }
    }

    pub fn getName(&self) -> Option<String> {
        match self {
            Type::Struct(n) => Some(n.clone()),
            Type::Ptr(p) => p.getName(),
            _ => None,
        }
    }
}
