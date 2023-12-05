#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    span: Span<'a>,
    kind: TokenKind,
}

impl<'a> Token<'a> {
    pub fn new(span: Span<'a>, kind: TokenKind) -> Self {
        Self { span, kind }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::Eof
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Span<'a> {
    pub(super) source: &'a str,
    /// Start index of the token
    pub(super) start: usize,
    /// End index of the token (exclusive)
    pub(super) end: usize,
}

impl<'a> Span<'a> {
    pub fn new(source: &'a str, start: usize, end: usize) -> Self {
        Self { source, start, end }
    }

    pub fn slice(&self) -> &str {
        &self.source[self.start..self.end]
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    /* Punctuation. */
    LParen,
    RParen,
    LBrack,
    RBrack,
    LBrace,
    RBrace,
    Period,
    Semicolon,
    Comma,


    Let,
    Mut,
    Func,
    Class,
    Identifier,

    /* Literals. */
    String,
    Number,
    True,
    False,
    Nil,

    /* Arithmetic operators. */
    StarStar,
    Star,
    Slash,
    Percent,
    Plus,
    Minus,

    /* Assignment operators. */
    Equal,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PercentEqual,

    /* Comparison operators. */
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    GreaterEqual,
    Greater,

    /* Logical operators. */
    Or,
    And,

    This,
    Return,

    /* Control flow. */
    For,
    While,
    If,
    Else,

    Eof,
}
