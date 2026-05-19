use crate::{Diagnostic, SourceFile, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Eof,
    Newline,
    VersionDirective(u16),
    Identifier(String),
    Int(i64),
    Float(f64),
    String(String),
    ColorHex(String),
    True,
    False,
    If,
    Else,
    Indent,
    Dedent,
    For,
    Break,
    Continue,
    Import,
    To,
    By,
    Var,
    Varip,
    And,
    Or,
    Not,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Arrow,
    ColonEq,
    EqEq,
    BangEq,
    Gt,
    Gte,
    Lt,
    Lte,
    Question,
    Colon,
    Dot,
    Comma,
    LParen,
    RParen,
    LBracket,
    RBracket,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lexed {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

#[must_use]
pub fn lex(source: &SourceFile) -> Lexed {
    Lexer::new(source).lex()
}

struct Lexer<'a> {
    source: &'a SourceFile,
    text: &'a str,
    pos: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    line_start: bool,
    indent_stack: Vec<usize>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a SourceFile) -> Self {
        Self {
            source,
            text: source.text(),
            pos: 0,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
            line_start: true,
            indent_stack: vec![0],
        }
    }

    fn lex(mut self) -> Lexed {
        while let Some(mut byte) = self.peek_byte() {
            if self.line_start {
                self.handle_line_start();
                let Some(next_byte) = self.peek_byte() else {
                    break;
                };
                byte = next_byte;
            }

            match byte {
                b' ' | b'\t' | b'\r' => {
                    self.pos += 1;
                }
                b'\n' => {
                    self.single(TokenKind::Newline);
                    self.line_start = true;
                }
                b'0'..=b'9' => self.number(),
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.identifier_or_keyword(),
                b'"' => self.string(),
                b'#' => self.color_hex(),
                b'/' if self.peek_next() == Some(b'/') => self.comment_or_version(),
                b'+' => self.single(TokenKind::Plus),
                b'-' => self.single(TokenKind::Minus),
                b'*' => self.single(TokenKind::Star),
                b'/' => self.single(TokenKind::Slash),
                b'%' => self.single(TokenKind::Percent),
                b'=' if self.peek_next() == Some(b'=') => self.double(TokenKind::EqEq),
                b'=' if self.peek_next() == Some(b'>') => self.double(TokenKind::Arrow),
                b'=' => self.single(TokenKind::Eq),
                b':' if self.peek_next() == Some(b'=') => self.double(TokenKind::ColonEq),
                b':' => self.single(TokenKind::Colon),
                b'!' if self.peek_next() == Some(b'=') => self.double(TokenKind::BangEq),
                b'>' if self.peek_next() == Some(b'=') => self.double(TokenKind::Gte),
                b'>' => self.single(TokenKind::Gt),
                b'<' if self.peek_next() == Some(b'=') => self.double(TokenKind::Lte),
                b'<' => self.single(TokenKind::Lt),
                b'?' => self.single(TokenKind::Question),
                b'.' => self.single(TokenKind::Dot),
                b',' => self.single(TokenKind::Comma),
                b'(' => self.single(TokenKind::LParen),
                b')' => self.single(TokenKind::RParen),
                b'[' => self.single(TokenKind::LBracket),
                b']' => self.single(TokenKind::RBracket),
                _ => self.unexpected_byte(),
            }
        }

        self.close_indents();
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span::new(self.pos, self.pos),
        });

        Lexed {
            tokens: self.tokens,
            diagnostics: self.diagnostics,
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.text.as_bytes().get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.text.as_bytes().get(self.pos + 1).copied()
    }

    fn single(&mut self, kind: TokenKind) {
        let start = self.pos;
        self.pos += 1;
        if !matches!(kind, TokenKind::Newline) {
            self.line_start = false;
        }
        self.tokens.push(Token {
            kind,
            span: Span::new(start, self.pos),
        });
    }

    fn double(&mut self, kind: TokenKind) {
        let start = self.pos;
        self.pos += 2;
        self.line_start = false;
        self.tokens.push(Token {
            kind,
            span: Span::new(start, self.pos),
        });
    }

    fn number(&mut self) {
        let start = self.pos;
        self.consume_while(|byte| byte.is_ascii_digit());

        let mut is_float = false;
        if self.peek_byte() == Some(b'.')
            && self
                .text
                .as_bytes()
                .get(self.pos + 1)
                .is_some_and(u8::is_ascii_digit)
        {
            is_float = true;
            self.pos += 1;
            self.consume_while(|byte| byte.is_ascii_digit());
        }

        let raw = &self.text[start..self.pos];
        let kind = if is_float {
            match raw.parse::<f64>() {
                Ok(value) => TokenKind::Float(value),
                Err(_) => {
                    self.diagnostics.push(Diagnostic::error(
                        "E_LEX_FLOAT",
                        "invalid float literal",
                        Span::new(start, self.pos),
                    ));
                    TokenKind::Float(0.0)
                }
            }
        } else {
            match raw.parse::<i64>() {
                Ok(value) => TokenKind::Int(value),
                Err(_) => {
                    self.diagnostics.push(Diagnostic::error(
                        "E_LEX_INT",
                        "invalid integer literal",
                        Span::new(start, self.pos),
                    ));
                    TokenKind::Int(0)
                }
            }
        };

        self.tokens.push(Token {
            kind,
            span: Span::new(start, self.pos),
        });
        self.line_start = false;
    }

    fn identifier_or_keyword(&mut self) {
        let start = self.pos;
        self.consume_while(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
        let raw = &self.text[start..self.pos];
        let kind = match raw {
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "for" => TokenKind::For,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "import" => TokenKind::Import,
            "to" => TokenKind::To,
            "by" => TokenKind::By,
            "var" => TokenKind::Var,
            "varip" => TokenKind::Varip,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            _ => TokenKind::Identifier(raw.to_owned()),
        };

        self.tokens.push(Token {
            kind,
            span: Span::new(start, self.pos),
        });
        self.line_start = false;
    }

    fn string(&mut self) {
        let start = self.pos;
        self.pos += 1;
        let mut value = String::new();

        while let Some(byte) = self.peek_byte() {
            match byte {
                b'"' => {
                    self.pos += 1;
                    self.tokens.push(Token {
                        kind: TokenKind::String(value),
                        span: Span::new(start, self.pos),
                    });
                    self.line_start = false;
                    return;
                }
                b'\\' => {
                    self.pos += 1;
                    if let Some(escaped) = self.peek_byte() {
                        self.pos += 1;
                        value.push(match escaped {
                            b'n' => '\n',
                            b't' => '\t',
                            b'r' => '\r',
                            b'"' => '"',
                            b'\\' => '\\',
                            other => other as char,
                        });
                    }
                }
                b'\n' => {
                    self.diagnostics.push(Diagnostic::error(
                        "E_LEX_STRING",
                        "unterminated string literal",
                        Span::new(start, self.pos),
                    ));
                    return;
                }
                other => {
                    self.pos += 1;
                    value.push(other as char);
                }
            }
        }

        self.diagnostics.push(Diagnostic::error(
            "E_LEX_STRING",
            "unterminated string literal",
            Span::new(start, self.pos),
        ));
    }

    fn color_hex(&mut self) {
        let start = self.pos;
        self.pos += 1;
        self.consume_while(|byte| byte.is_ascii_hexdigit());
        let raw = &self.text[start..self.pos];

        if matches!(raw.len(), 7 | 9) {
            self.tokens.push(Token {
                kind: TokenKind::ColorHex(raw.to_owned()),
                span: Span::new(start, self.pos),
            });
            self.line_start = false;
        } else {
            self.diagnostics.push(Diagnostic::error(
                "E_LEX_COLOR",
                "expected color literal in #RRGGBB or #RRGGBBAA form",
                Span::new(start, self.pos),
            ));
        }
    }

    fn comment_or_version(&mut self) {
        let start = self.pos;
        let line_end = self.text[start..]
            .find('\n')
            .map_or(self.text.len(), |offset| start + offset);
        let line = &self.text[start..line_end];

        if let Some(raw_version) = line.strip_prefix("//@version=") {
            match raw_version.trim().parse::<u16>() {
                Ok(version) => self.tokens.push(Token {
                    kind: TokenKind::VersionDirective(version),
                    span: Span::new(start, line_end),
                }),
                Err(_) => self.diagnostics.push(Diagnostic::error(
                    "E_LEX_VERSION",
                    "invalid version directive",
                    Span::new(start, line_end),
                )),
            }
        }

        self.pos = line_end;
        self.line_start = false;
    }

    fn unexpected_byte(&mut self) {
        let start = self.pos;
        self.pos += 1;
        let line_col = self.source.line_col(start);
        self.diagnostics.push(Diagnostic::error(
            "E_LEX_CHAR",
            format!(
                "unexpected character at line {}, column {}",
                line_col.line, line_col.column
            ),
            Span::new(start, self.pos),
        ));
    }

    fn consume_while(&mut self, mut predicate: impl FnMut(u8) -> bool) {
        while self.peek_byte().is_some_and(&mut predicate) {
            self.pos += 1;
        }
    }

    fn handle_line_start(&mut self) {
        let start = self.pos;
        let mut indent = 0_usize;

        while let Some(byte) = self.peek_byte() {
            match byte {
                b' ' => {
                    indent += 1;
                    self.pos += 1;
                }
                b'\t' => {
                    indent += 4;
                    self.pos += 1;
                }
                b'\r' => {
                    self.pos += 1;
                }
                _ => break,
            }
        }

        if matches!(self.peek_byte(), Some(b'\n') | None) {
            return;
        }

        if self.peek_byte() == Some(b'/') && self.peek_next() == Some(b'/') {
            return;
        }

        let current = *self
            .indent_stack
            .last()
            .expect("indent stack always contains root indent");
        if indent > current {
            self.indent_stack.push(indent);
            self.tokens.push(Token {
                kind: TokenKind::Indent,
                span: Span::new(start, self.pos),
            });
        } else if indent < current {
            while indent
                < *self
                    .indent_stack
                    .last()
                    .expect("indent stack always contains root indent")
            {
                self.indent_stack.pop();
                self.tokens.push(Token {
                    kind: TokenKind::Dedent,
                    span: Span::new(start, self.pos),
                });
            }

            if indent
                != *self
                    .indent_stack
                    .last()
                    .expect("indent stack always contains root indent")
            {
                self.diagnostics.push(Diagnostic::error(
                    "E_LEX_INDENT",
                    "inconsistent indentation",
                    Span::new(start, self.pos),
                ));
            }
        }

        self.line_start = false;
    }

    fn close_indents(&mut self) {
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.tokens.push(Token {
                kind: TokenKind::Dedent,
                span: Span::new(self.pos, self.pos),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        let source = SourceFile::new("test.pine", source);
        lex(&source)
            .tokens
            .into_iter()
            .map(|token| token.kind)
            .collect()
    }

    #[test]
    fn lexes_version_and_indicator_call() {
        assert_eq!(
            kinds("//@version=5\nindicator(\"Demo\", overlay=true)\n"),
            vec![
                TokenKind::VersionDirective(5),
                TokenKind::Newline,
                TokenKind::Identifier("indicator".to_owned()),
                TokenKind::LParen,
                TokenKind::String("Demo".to_owned()),
                TokenKind::Comma,
                TokenKind::Identifier("overlay".to_owned()),
                TokenKind::Eq,
                TokenKind::True,
                TokenKind::RParen,
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_history_and_namespaces() {
        assert_eq!(
            kinds("x = ta.sma(close, 20)[1]\n"),
            vec![
                TokenKind::Identifier("x".to_owned()),
                TokenKind::Eq,
                TokenKind::Identifier("ta".to_owned()),
                TokenKind::Dot,
                TokenKind::Identifier("sma".to_owned()),
                TokenKind::LParen,
                TokenKind::Identifier("close".to_owned()),
                TokenKind::Comma,
                TokenKind::Int(20),
                TokenKind::RParen,
                TokenKind::LBracket,
                TokenKind::Int(1),
                TokenKind::RBracket,
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_for_range_keywords() {
        assert_eq!(
            kinds("for i = 0 to 10 by 2\n"),
            vec![
                TokenKind::For,
                TokenKind::Identifier("i".to_owned()),
                TokenKind::Eq,
                TokenKind::Int(0),
                TokenKind::To,
                TokenKind::Int(10),
                TokenKind::By,
                TokenKind::Int(2),
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_loop_control_keywords() {
        assert_eq!(
            kinds("break\ncontinue\n"),
            vec![
                TokenKind::Break,
                TokenKind::Newline,
                TokenKind::Continue,
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_indented_blocks() {
        assert_eq!(
            kinds("if close > open\n    plot(close)\nelse\n    plot(open)\n"),
            vec![
                TokenKind::If,
                TokenKind::Identifier("close".to_owned()),
                TokenKind::Gt,
                TokenKind::Identifier("open".to_owned()),
                TokenKind::Newline,
                TokenKind::Indent,
                TokenKind::Identifier("plot".to_owned()),
                TokenKind::LParen,
                TokenKind::Identifier("close".to_owned()),
                TokenKind::RParen,
                TokenKind::Newline,
                TokenKind::Dedent,
                TokenKind::Else,
                TokenKind::Newline,
                TokenKind::Indent,
                TokenKind::Identifier("plot".to_owned()),
                TokenKind::LParen,
                TokenKind::Identifier("open".to_owned()),
                TokenKind::RParen,
                TokenKind::Newline,
                TokenKind::Dedent,
                TokenKind::Eof,
            ]
        );
    }
}
