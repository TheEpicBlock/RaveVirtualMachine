
pub enum JavaType {
    Reference,
    Integer,
    Byte,
    Char,
    Void
}

pub enum Literal {
    NullReference,
    Integer(u32),
}