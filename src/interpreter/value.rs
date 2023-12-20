#[derive(Debug, Default, Clone, Copy)]
pub enum Value {
    Number(f64),
    Bool(bool),
    #[default]
    Nil,
}
