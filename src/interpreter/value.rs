#[derive(Debug, Default)]
pub enum Value {
    Number(f64),
    Bool(bool),
    #[default]
    Nil,
}
