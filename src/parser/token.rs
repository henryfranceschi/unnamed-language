use std::ops::Add;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token<'a> {
    span: Span<'a>,
    kind: TokenKind,
}

impl<'a> Token<'a> {
    pub fn new(span: Span<'a>, kind: TokenKind) -> Self {
        Self { span, kind }
    }

    pub fn span(self) -> Span<'a> {
        self.span
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::Eof
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Span<'a> {
    source: &'a str,
    /// Start index of the token.
    start: usize,
    /// End index of the token (exclusive).
    end: usize,
}

impl<'a> Span<'a> {
    pub fn new(source: &'a str, start: usize, end: usize) -> Self {
        Self { source, start, end }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn line_number(&self) -> usize {
        self.source[..self.start]
            .chars()
            .filter(|c| *c == '\n')
            .count()
            .add(1)
    }

    pub fn column_number(&self) -> usize {
        self.source[..self.start]
            .chars()
            .rev()
            .take_while(|c| *c != '\n')
            .count()
            .add(1)
    }

    pub fn slice(&self) -> &'a str {
        &self.source[self.start..self.end]
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    LParen,
    RParen,
    LBrack,
    RBrack,
    LBrace,
    RBrace,
    Period,
    Semicolon,
    Comma,
    Identifier,
    Let,
    Mut,
    Func,
    Class,
    Not,
    Or,
    And,
    For,
    While,
    If,
    Else,
    Return,
    This,
    True,
    False,
    Nil,
    String,
    Number,
    StarStar,
    Star,
    Slash,
    Percent,
    Plus,
    Minus,
    Equal,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PercentEqual,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    GreaterEqual,
    Greater,
    Eof,
}

impl TokenKind {
    pub fn keyword_kind_from_str(s: &str) -> Option<TokenKind> {
        let kind = match s {
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "func" => TokenKind::Func,
            "class" => TokenKind::Class,
            "not" => TokenKind::Not,
            "or" => TokenKind::Or,
            "and" => TokenKind::And,
            "for" => TokenKind::For,
            "while" => TokenKind::While,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "return" => TokenKind::Return,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "nil" => TokenKind::Nil,
            _ => return None,
        };

        Some(kind)
    }
}

#[cfg(test)]
mod tests {
    use super::Span;

    #[test]
    fn span() {
        let src = "let x = 10;\nx *= 2";

        // Span for 'let'.
        let span = Span::new(src, 0, 2);
        assert_eq!(span.line_number(), 1);
        assert_eq!(span.column_number(), 1);

        // Span for 'x' on line 1.
        let span = Span::new(src, 4, 4);
        assert_eq!(span.line_number(), 1);
        assert_eq!(span.column_number(), 5);

        // Span for 'x' on line 2.
        let span = Span::new(src, 12, 12);
        assert_eq!(span.line_number(), 2);
        assert_eq!(span.column_number(), 1);
    }
}
