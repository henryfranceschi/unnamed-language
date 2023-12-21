#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum Value {
    Number(f64),
    Bool(bool),
    #[default]
    Nil,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match *self {
            Value::Number(_) => true,
            Value::Bool(b) => b,
            Value::Nil => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use Value::*;
        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            None
        } else {
            match (self, other) {
                (Number(a), Number(b)) => a.partial_cmp(b),
                (Bool(_), Bool(_)) => None,
                (Nil, Nil) => None,
                _ => unreachable!(),
            }
        }
    }
}
