use crate::{
    BinaryOp, CallArg, DeclMode, Diagnostic, Expr, ExprKind, FunctionBody, Lexed, Literal, Program,
    SourceFile, Span, Stmt, StmtKind, Token, TokenKind, UnaryOp, VersionDecl, lex,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Parse {
    pub program: Program,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse_source(source: &SourceFile) -> Parse {
    let lexed = lex(source);
    Parser::new(lexed).parse()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    fn new(lexed: Lexed) -> Self {
        Self {
            tokens: lexed.tokens,
            pos: 0,
            diagnostics: lexed.diagnostics,
        }
    }

    fn parse(mut self) -> Parse {
        let version = self.parse_optional_version();
        let mut statements = Vec::new();

        while !self.at(TokenKind::Eof) {
            self.skip_newlines();
            if self.at(TokenKind::Eof) {
                break;
            }

            match self.parse_stmt() {
                Some(statement) => statements.push(statement),
                None => self.recover_stmt(),
            }
            self.skip_newlines();
        }

        Parse {
            program: Program {
                version,
                statements,
            },
            diagnostics: self.diagnostics,
        }
    }

    fn parse_optional_version(&mut self) -> Option<VersionDecl> {
        self.skip_newlines();
        match self.current().kind {
            TokenKind::VersionDirective(version) => {
                let span = self.current().span;
                self.bump();
                Some(VersionDecl { version, span })
            }
            _ => None,
        }
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        if self.at(TokenKind::Import) {
            return Some(self.parse_unsupported_line("import"));
        }

        if self.at(TokenKind::If) {
            return self.parse_if_stmt();
        }

        if self.at(TokenKind::For) {
            return self.parse_for_stmt();
        }

        if self.at(TokenKind::Break) {
            let span = self.expect(TokenKind::Break, "expected `break`")?;
            return Some(Stmt {
                span,
                kind: StmtKind::Break,
            });
        }

        if self.at(TokenKind::Continue) {
            let span = self.expect(TokenKind::Continue, "expected `continue`")?;
            return Some(Stmt {
                span,
                kind: StmtKind::Continue,
            });
        }

        if self.at(TokenKind::LBracket) {
            return self.parse_tuple_decl();
        }

        let mode = if self.at(TokenKind::Var) {
            self.bump();
            Some(DeclMode::Var)
        } else if self.at(TokenKind::Varip) {
            self.bump();
            Some(DeclMode::Varip)
        } else {
            None
        };

        if let TokenKind::Identifier(name) = self.current().kind.clone() {
            let start = self.current().span;
            if self.looks_like_function_decl() {
                return self.parse_function_decl();
            }

            if self.nth_at(1, TokenKind::Eq) || mode.is_some() {
                self.bump();
                self.expect(TokenKind::Eq, "expected `=` in variable declaration")?;
                let value = self.parse_expr(0)?;
                return Some(Stmt {
                    span: start.merge(value.span),
                    kind: StmtKind::Decl {
                        mode: mode.unwrap_or(DeclMode::Normal),
                        name,
                        value,
                    },
                });
            }

            if self.nth_at(1, TokenKind::ColonEq) {
                self.bump();
                self.bump();
                let value = self.parse_expr(0)?;
                return Some(Stmt {
                    span: start.merge(value.span),
                    kind: StmtKind::Reassign { name, value },
                });
            }
        } else if mode.is_some() {
            self.error_here("E_PARSE_DECL", "expected identifier after declaration mode");
            return None;
        }

        let expr = self.parse_expr(0)?;
        Some(Stmt {
            span: expr.span,
            kind: StmtKind::Expr(expr),
        })
    }

    fn parse_tuple_decl(&mut self) -> Option<Stmt> {
        let start = self.expect(TokenKind::LBracket, "expected `[`")?;
        let mut names = Vec::new();

        loop {
            match self.current().kind.clone() {
                TokenKind::Identifier(name) => {
                    names.push(name);
                    self.bump();
                }
                _ => {
                    self.error_here("E_PARSE_DECL", "expected identifier in tuple declaration");
                    return None;
                }
            }

            if self.at(TokenKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }

        self.expect(TokenKind::RBracket, "expected `]` after tuple declaration")?;
        self.expect(TokenKind::Eq, "expected `=` in tuple declaration")?;
        let value = self.parse_expr(0)?;

        Some(Stmt {
            span: start.merge(value.span),
            kind: StmtKind::TupleDecl { names, value },
        })
    }

    fn parse_if_stmt(&mut self) -> Option<Stmt> {
        let start = self.expect(TokenKind::If, "expected `if`")?;
        let condition = self.parse_expr(0)?;
        self.expect(TokenKind::Newline, "expected newline after `if` condition")?;
        let then_branch = self.parse_indented_block()?;
        let mut span = then_branch
            .last()
            .map_or(condition.span, |statement| statement.span);

        let else_branch = if self.at(TokenKind::Else) {
            self.bump();
            if self.at(TokenKind::If) {
                let nested_if = self.parse_if_stmt()?;
                span = nested_if.span;
                vec![nested_if]
            } else {
                self.expect(TokenKind::Newline, "expected newline after `else`")?;
                let branch = self.parse_indented_block()?;
                span = branch.last().map_or(span, |statement| statement.span);
                branch
            }
        } else {
            Vec::new()
        };

        Some(Stmt {
            span: start.merge(span),
            kind: StmtKind::If {
                condition,
                then_branch,
                else_branch,
            },
        })
    }

    fn parse_for_stmt(&mut self) -> Option<Stmt> {
        let start = self.expect(TokenKind::For, "expected `for`")?;
        let counter = match self.current().kind.clone() {
            TokenKind::Identifier(name) => {
                self.bump();
                name
            }
            _ => {
                self.error_here("E_PARSE_FOR", "expected loop counter name");
                return None;
            }
        };
        self.expect(TokenKind::Eq, "expected `=` after loop counter")?;
        let from = self.parse_expr(0)?;
        self.expect(TokenKind::To, "expected `to` in for loop")?;
        let to = self.parse_expr(0)?;
        let step = if self.at(TokenKind::By) {
            self.bump();
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        self.expect(TokenKind::Newline, "expected newline after `for` range")?;
        let body = self.parse_indented_block()?;
        let span = body.last().map_or(
            step.as_ref().map_or(to.span, |step| step.span),
            |statement| statement.span,
        );

        Some(Stmt {
            span: start.merge(span),
            kind: StmtKind::For {
                counter,
                from,
                to,
                step,
                body,
            },
        })
    }

    fn parse_function_decl(&mut self) -> Option<Stmt> {
        let start = self.current().span;
        let TokenKind::Identifier(name) = self.current().kind.clone() else {
            return None;
        };
        self.bump();
        self.expect(TokenKind::LParen, "expected `(` after function name")?;
        let mut params = Vec::new();

        if !self.at(TokenKind::RParen) {
            loop {
                match self.current().kind.clone() {
                    TokenKind::Identifier(param) => {
                        params.push(param);
                        self.bump();
                    }
                    _ => {
                        self.error_here("E_PARSE_FUNCTION", "expected parameter name");
                        return None;
                    }
                }

                if self.at(TokenKind::Comma) {
                    self.bump();
                    continue;
                }
                break;
            }
        }

        self.expect(TokenKind::RParen, "expected `)` after function parameters")?;
        self.expect(TokenKind::Arrow, "expected `=>` in function declaration")?;
        let (body, span) = if self.at(TokenKind::Newline) {
            self.bump();
            let block = self.parse_indented_block()?;
            let span = block.last().map_or(start, |statement| statement.span);
            (FunctionBody::Block(block), span)
        } else {
            let body = self.parse_expr(0)?;
            let span = body.span;
            (FunctionBody::Expr(body), span)
        };

        Some(Stmt {
            span: start.merge(span),
            kind: StmtKind::Function { name, params, body },
        })
    }

    fn parse_indented_block(&mut self) -> Option<Vec<Stmt>> {
        self.expect(TokenKind::Indent, "expected indented block")?;
        let mut statements = Vec::new();

        loop {
            self.skip_newlines();
            if self.at(TokenKind::Dedent) {
                self.bump();
                break;
            }
            if self.at(TokenKind::Eof) {
                self.error_here("E_PARSE_BLOCK", "expected dedent before end of file");
                break;
            }

            match self.parse_stmt() {
                Some(statement) => statements.push(statement),
                None => self.recover_stmt(),
            }
            self.skip_newlines();
        }

        if statements.is_empty() {
            self.error_here("E_PARSE_BLOCK", "expected statement in block");
            return None;
        }

        Some(statements)
    }

    fn parse_unsupported_line(&mut self, feature: &str) -> Stmt {
        let start = self.current().span;
        self.bump();
        while !self.at(TokenKind::Newline) && !self.at(TokenKind::Eof) {
            self.bump();
        }

        Stmt {
            span: start.merge(self.previous().span),
            kind: StmtKind::Unsupported {
                feature: feature.to_owned(),
            },
        }
    }

    fn looks_like_function_decl(&self) -> bool {
        if !self.nth_at(1, TokenKind::LParen) {
            return false;
        }

        let mut depth = 0_u32;
        let mut index = self.pos + 1;
        while let Some(token) = self.tokens.get(index) {
            match token.kind {
                TokenKind::Newline | TokenKind::Eof => return false,
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return self.tokens.get(index + 1).is_some_and(|next| {
                            std::mem::discriminant(&next.kind)
                                == std::mem::discriminant(&TokenKind::Arrow)
                        });
                    }
                }
                _ => {}
            }
            index += 1;
        }

        false
    }

    fn parse_expr(&mut self, min_bp: u8) -> Option<Expr> {
        let mut left = self.parse_prefix()?;

        loop {
            if self.at(TokenKind::LParen) {
                left = self.finish_call(left)?;
                continue;
            }
            if self.at(TokenKind::LBracket) {
                left = self.finish_history(left)?;
                continue;
            }
            if self.at(TokenKind::Question) {
                let (_, right_bp) = (1, 0);
                if right_bp < min_bp {
                    break;
                }
                self.bump();
                let then_expr = self.parse_expr(0)?;
                self.expect(TokenKind::Colon, "expected `:` in ternary expression")?;
                let else_expr = self.parse_expr(0)?;
                let span = left.span.merge(else_expr.span);
                left = Expr {
                    span,
                    kind: ExprKind::Ternary {
                        condition: Box::new(left),
                        then_expr: Box::new(then_expr),
                        else_expr: Box::new(else_expr),
                    },
                };
                continue;
            }

            let Some((op, left_bp, right_bp)) = self.current_binary_op() else {
                break;
            };
            if left_bp < min_bp {
                break;
            }

            self.bump();
            let right = self.parse_expr(right_bp)?;
            let span = left.span.merge(right.span);
            left = Expr {
                span,
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            };
        }

        Some(left)
    }

    fn parse_prefix(&mut self) -> Option<Expr> {
        let token = self.current().clone();
        match token.kind {
            TokenKind::Int(value) => {
                self.bump();
                Some(Expr {
                    span: token.span,
                    kind: ExprKind::Literal(Literal::Int(value)),
                })
            }
            TokenKind::Float(value) => {
                self.bump();
                Some(Expr {
                    span: token.span,
                    kind: ExprKind::Literal(Literal::Float(value)),
                })
            }
            TokenKind::String(value) => {
                self.bump();
                Some(Expr {
                    span: token.span,
                    kind: ExprKind::Literal(Literal::String(value)),
                })
            }
            TokenKind::ColorHex(value) => {
                self.bump();
                Some(Expr {
                    span: token.span,
                    kind: ExprKind::Literal(Literal::ColorHex(value)),
                })
            }
            TokenKind::True | TokenKind::False => {
                self.bump();
                Some(Expr {
                    span: token.span,
                    kind: ExprKind::Literal(Literal::Bool(matches!(token.kind, TokenKind::True))),
                })
            }
            TokenKind::Identifier(name) => {
                self.bump();
                self.parse_identifier_tail(name, token.span)
            }
            TokenKind::Plus | TokenKind::Minus | TokenKind::Not => {
                self.bump();
                let op = match token.kind {
                    TokenKind::Plus => UnaryOp::Plus,
                    TokenKind::Minus => UnaryOp::Minus,
                    TokenKind::Not => UnaryOp::Not,
                    _ => unreachable!(),
                };
                let expr = self.parse_expr(12)?;
                Some(Expr {
                    span: token.span.merge(expr.span),
                    kind: ExprKind::Unary {
                        op,
                        expr: Box::new(expr),
                    },
                })
            }
            TokenKind::LParen => {
                self.bump();
                let expr = self.parse_expr(0)?;
                self.expect(TokenKind::RParen, "expected `)`")?;
                Some(expr)
            }
            TokenKind::LBracket => self.parse_tuple_expr(),
            _ => {
                self.error_here("E_PARSE_EXPR", "expected expression");
                None
            }
        }
    }

    fn parse_tuple_expr(&mut self) -> Option<Expr> {
        let start = self.expect(TokenKind::LBracket, "expected `[`")?;
        let mut items = Vec::new();

        if !self.at(TokenKind::RBracket) {
            loop {
                items.push(self.parse_expr(0)?);
                if self.at(TokenKind::Comma) {
                    self.bump();
                    continue;
                }
                break;
            }
        }

        let end = self.expect(TokenKind::RBracket, "expected `]` after tuple expression")?;
        Some(Expr {
            span: start.merge(end),
            kind: ExprKind::Tuple(items),
        })
    }

    fn parse_identifier_tail(&mut self, first: String, first_span: Span) -> Option<Expr> {
        let mut parts = vec![first];
        let mut span = first_span;

        while self.at(TokenKind::Dot) {
            self.bump();
            match self.current().kind.clone() {
                TokenKind::Identifier(part) => {
                    span = span.merge(self.current().span);
                    parts.push(part);
                    self.bump();
                }
                _ => {
                    self.error_here("E_PARSE_NAME", "expected identifier after `.`");
                    return None;
                }
            }
        }

        Some(Expr {
            span,
            kind: if parts.len() == 1 {
                ExprKind::Identifier(parts.pop().expect("parts has one item"))
            } else {
                ExprKind::QualifiedName(parts)
            },
        })
    }

    fn finish_call(&mut self, callee: Expr) -> Option<Expr> {
        let start = callee.span;
        self.expect(TokenKind::LParen, "expected `(`")?;
        let mut args = Vec::new();

        if !self.at(TokenKind::RParen) {
            loop {
                let arg_start = self.current().span;
                let name = if let TokenKind::Identifier(name) = self.current().kind.clone() {
                    if self.nth_at(1, TokenKind::Eq) {
                        self.bump();
                        self.bump();
                        Some(name)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let value = self.parse_expr(0)?;
                args.push(CallArg {
                    span: arg_start.merge(value.span),
                    name,
                    value,
                });

                if self.at(TokenKind::Comma) {
                    self.bump();
                    continue;
                }
                break;
            }
        }

        let end = self.expect(TokenKind::RParen, "expected `)` after arguments")?;
        Some(Expr {
            span: start.merge(end),
            kind: ExprKind::Call {
                callee: Box::new(callee),
                args,
            },
        })
    }

    fn finish_history(&mut self, expr: Expr) -> Option<Expr> {
        self.expect(TokenKind::LBracket, "expected `[`")?;
        let offset = self.parse_expr(0)?;
        let end = self.expect(TokenKind::RBracket, "expected `]` after history offset")?;
        Some(Expr {
            span: expr.span.merge(end),
            kind: ExprKind::History {
                expr: Box::new(expr),
                offset: Box::new(offset),
            },
        })
    }

    fn current_binary_op(&self) -> Option<(BinaryOp, u8, u8)> {
        match self.current().kind {
            TokenKind::Or => Some((BinaryOp::Or, 2, 3)),
            TokenKind::And => Some((BinaryOp::And, 4, 5)),
            TokenKind::EqEq => Some((BinaryOp::Eq, 6, 7)),
            TokenKind::BangEq => Some((BinaryOp::NotEq, 6, 7)),
            TokenKind::Gt => Some((BinaryOp::Gt, 8, 9)),
            TokenKind::Gte => Some((BinaryOp::Gte, 8, 9)),
            TokenKind::Lt => Some((BinaryOp::Lt, 8, 9)),
            TokenKind::Lte => Some((BinaryOp::Lte, 8, 9)),
            TokenKind::Plus => Some((BinaryOp::Add, 10, 11)),
            TokenKind::Minus => Some((BinaryOp::Sub, 10, 11)),
            TokenKind::Star => Some((BinaryOp::Mul, 12, 13)),
            TokenKind::Slash => Some((BinaryOp::Div, 12, 13)),
            TokenKind::Percent => Some((BinaryOp::Mod, 12, 13)),
            _ => None,
        }
    }

    fn skip_newlines(&mut self) {
        while self.at(TokenKind::Newline) {
            self.bump();
        }
    }

    fn recover_stmt(&mut self) {
        while !self.at(TokenKind::Newline) && !self.at(TokenKind::Eof) {
            self.bump();
        }
    }

    fn expect(&mut self, expected: TokenKind, message: &str) -> Option<Span> {
        if self.at(expected) {
            let span = self.current().span;
            self.bump();
            Some(span)
        } else {
            self.error_here("E_PARSE_EXPECTED", message);
            None
        }
    }

    fn error_here(&mut self, code: &str, message: &str) {
        self.diagnostics
            .push(Diagnostic::error(code, message, self.current().span));
    }

    fn at(&self, expected: TokenKind) -> bool {
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(&expected)
    }

    fn nth_at(&self, offset: usize, expected: TokenKind) -> bool {
        self.tokens.get(self.pos + offset).is_some_and(|token| {
            std::mem::discriminant(&token.kind) == std::mem::discriminant(&expected)
        })
    }

    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.pos.saturating_sub(1)]
    }

    fn bump(&mut self) {
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Parse {
        parse_source(&SourceFile::new("test.pine", text))
    }

    #[test]
    fn parses_version_and_indicator() {
        let parsed = parse("//@version=5\nindicator(\"Demo\", overlay=true)\n");

        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(
            parsed.program.version.map(|version| version.version),
            Some(5)
        );
        assert_eq!(parsed.program.statements.len(), 1);
    }

    #[test]
    fn parses_declaration_with_history_call() {
        let parsed = parse("x = ta.sma(close, 20)[1]\n");

        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.program.statements.len(), 1);
        assert!(matches!(
            parsed.program.statements[0].kind,
            StmtKind::Decl { .. }
        ));
    }

    #[test]
    fn parses_reassignment() {
        let parsed = parse("x := x + 1\n");

        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert!(matches!(
            parsed.program.statements[0].kind,
            StmtKind::Reassign { .. }
        ));
    }

    #[test]
    fn recovers_after_bad_declaration() {
        let parsed = parse("x =\ny = close\n");

        assert_eq!(parsed.program.statements.len(), 1);
        assert!(!parsed.diagnostics.is_empty());
        assert!(matches!(
            parsed.program.statements[0].kind,
            StmtKind::Decl { .. }
        ));
    }

    #[test]
    fn parses_tuple_declaration() {
        let parsed = parse("[macd, signal, hist] = ta.macd(close, 12, 26, 9)\n");

        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert!(matches!(
            parsed.program.statements[0].kind,
            StmtKind::TupleDecl { .. }
        ));
    }

    #[test]
    fn parses_tuple_expression() {
        let parsed = parse("x = [close, open]\n");

        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert!(matches!(
            parsed.program.statements[0].kind,
            StmtKind::Decl { .. }
        ));
    }

    #[test]
    fn parses_import_as_unsupported_statement() {
        let parsed = parse("import user/library/1\nindicator(\"Demo\")\n");

        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.program.statements.len(), 2);
        assert!(matches!(
            parsed.program.statements[0].kind,
            StmtKind::Unsupported { ref feature } if feature == "import"
        ));
    }

    #[test]
    fn parses_if_statement() {
        let parsed = parse("if close > open\n    plot(close)\nelse\n    plot(open)\n");

        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.program.statements.len(), 1);
        let StmtKind::If {
            then_branch,
            else_branch,
            ..
        } = &parsed.program.statements[0].kind
        else {
            panic!("expected if statement");
        };
        assert_eq!(then_branch.len(), 1);
        assert_eq!(else_branch.len(), 1);
    }

    #[test]
    fn parses_function_declaration() {
        let parsed = parse("double(x) => x * 2\nplot(close)\n");

        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.program.statements.len(), 2);
        let StmtKind::Function { name, params, .. } = &parsed.program.statements[0].kind else {
            panic!("expected function statement");
        };
        assert_eq!(name, "double");
        assert_eq!(params, &vec!["x".to_owned()]);
    }

    #[test]
    fn parses_block_function_declaration() {
        let parsed = parse("double(x) =>\n    y = x * 2\n    y\nplot(close)\n");

        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.program.statements.len(), 2);
        let StmtKind::Function { body, .. } = &parsed.program.statements[0].kind else {
            panic!("expected function statement");
        };
        let FunctionBody::Block(statements) = body else {
            panic!("expected block function body");
        };
        assert_eq!(statements.len(), 2);
    }

    #[test]
    fn parses_for_statement() {
        let parsed = parse("for i = 0 to 10\n    plot(close)\n");

        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.program.statements.len(), 1);
        let StmtKind::For {
            counter,
            from,
            to,
            step,
            body,
        } = &parsed.program.statements[0].kind
        else {
            panic!("expected for statement");
        };
        assert_eq!(counter, "i");
        assert!(matches!(from.kind, ExprKind::Literal(Literal::Int(0))));
        assert!(matches!(to.kind, ExprKind::Literal(Literal::Int(10))));
        assert!(step.is_none());
        assert_eq!(body.len(), 1);
    }

    #[test]
    fn parses_for_statement_with_step() {
        let parsed = parse("for i = 0 to 10 by 2\n    plot(close)\n");

        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.program.statements.len(), 1);
        let StmtKind::For { step, .. } = &parsed.program.statements[0].kind else {
            panic!("expected for statement");
        };
        let Some(step) = step else {
            panic!("expected for step");
        };
        assert!(matches!(step.kind, ExprKind::Literal(Literal::Int(2))));
    }

    #[test]
    fn parses_loop_control_statements() {
        let parsed = parse("for i = 0 to 10\n    if i == 2\n        break\n    continue\n");

        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let StmtKind::For { body, .. } = &parsed.program.statements[0].kind else {
            panic!("expected for statement");
        };
        let StmtKind::If { then_branch, .. } = &body[0].kind else {
            panic!("expected if statement");
        };
        assert!(matches!(then_branch[0].kind, StmtKind::Break));
        assert!(matches!(body[1].kind, StmtKind::Continue));
    }
}
