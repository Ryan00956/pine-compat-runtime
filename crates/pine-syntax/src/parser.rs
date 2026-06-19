use crate::{
    BinaryOp, CallArg, DeclMode, Diagnostic, Expr, ExprKind, FunctionBody, Lexed, Literal, Program,
    SourceFile, Span, Stmt, StmtKind, SwitchArm, Token, TokenKind, UnaryOp, VersionDecl, lex,
};

#[path = "parser_phase_j.rs"]
mod phase_j;

const MAX_EXPR_DEPTH: u32 = 256;

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
    expr_depth: u32,
}

struct ForParts {
    start: Span,
    counter: String,
    from: Expr,
    to: Expr,
    step: Option<Expr>,
    body: Vec<Stmt>,
    span: Span,
}

impl Parser {
    fn new(lexed: Lexed) -> Self {
        Self {
            tokens: lexed.tokens,
            pos: 0,
            diagnostics: lexed.diagnostics,
            expr_depth: 0,
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
            return self.parse_import_decl();
        }

        if let Some(statement) = self.parse_phase_j_decl()? {
            return Some(statement);
        }

        if self.at(TokenKind::If) {
            return self.parse_if_stmt();
        }

        if self.at(TokenKind::For) {
            return self.parse_for_stmt();
        }

        if self.at(TokenKind::While) {
            return self.parse_while_stmt();
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

        if self.at(TokenKind::LBracket) && self.looks_like_tuple_decl() {
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

        if let Some((declared_type, name, start)) = self.try_parse_typed_decl_name() {
            self.expect(TokenKind::Eq, "expected `=` in variable declaration")?;
            let value = self.parse_expr(0)?;
            return Some(Stmt {
                span: start.merge(value.span),
                kind: StmtKind::Decl {
                    mode: mode.unwrap_or(DeclMode::Normal),
                    declared_type: Some(declared_type),
                    name,
                    value,
                },
            });
        }

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
                        declared_type: None,
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

            if self.nth_at(1, TokenKind::Dot)
                && self
                    .tokens
                    .get(self.pos + 2)
                    .is_some_and(|token| matches!(token.kind, TokenKind::Identifier(_)))
                && self.nth_at(3, TokenKind::ColonEq)
            {
                self.bump();
                self.bump();
                let TokenKind::Identifier(field) = self.current().kind.clone() else {
                    self.error_here("E_PARSE_ASSIGN", "expected field name after `.`");
                    return None;
                };
                self.bump();
                self.expect(TokenKind::ColonEq, "expected `:=` in field reassignment")?;
                let value = self.parse_expr(0)?;
                if name == "strategy" {
                    return Some(Stmt {
                        span: start.merge(value.span),
                        kind: StmtKind::Unsupported {
                            feature: "strategy state variable mutation".to_owned(),
                        },
                    });
                }
                return Some(Stmt {
                    span: start.merge(value.span),
                    kind: StmtKind::FieldReassign {
                        receiver: name,
                        field,
                        value,
                    },
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

    fn try_parse_typed_decl_name(&mut self) -> Option<(String, String, Span)> {
        if let Some(parsed) = self.try_parse_array_typed_decl_name() {
            return Some(parsed);
        }
        if let Some(parsed) = self.try_parse_array_type_alias_decl_name() {
            return Some(parsed);
        }

        if let TokenKind::Identifier(type_name) = self.current().kind.clone()
            && self.nth_at(2, TokenKind::Eq)
        {
            let TokenKind::Identifier(name) = self.tokens.get(self.pos + 1)?.kind.clone() else {
                return None;
            };
            let start = self.current().span;
            self.bump();
            self.bump();
            return Some((type_name, name, start));
        }

        if !self.nth_at(1, TokenKind::Dot) || !self.nth_at(4, TokenKind::Eq) {
            return None;
        }

        let TokenKind::Identifier(namespace) = self.current().kind.clone() else {
            return None;
        };
        let TokenKind::Identifier(type_name) = self.tokens.get(self.pos + 2)?.kind.clone() else {
            return None;
        };
        let TokenKind::Identifier(name) = self.tokens.get(self.pos + 3)?.kind.clone() else {
            return None;
        };

        if namespace != "chart" || type_name != "point" {
            return None;
        }

        let start = self.current().span;
        self.bump();
        self.bump();
        self.bump();
        self.bump();
        Some(("chart.point".to_owned(), name, start))
    }

    fn try_parse_array_typed_decl_name(&mut self) -> Option<(String, String, Span)> {
        let TokenKind::Identifier(container) = self.current().kind.clone() else {
            return None;
        };
        if container != "array" || !self.nth_at(1, TokenKind::Lt) {
            return None;
        }

        let start_pos = self.pos;
        let (element_type, name_offset, eq_offset) = if self.nth_at(3, TokenKind::Gt)
            && self.nth_at(5, TokenKind::Eq)
        {
            let TokenKind::Identifier(element_type) = self.tokens.get(self.pos + 2)?.kind.clone()
            else {
                return None;
            };
            (element_type, 4, 5)
        } else if self.nth_at(3, TokenKind::Dot)
            && self.nth_at(5, TokenKind::Gt)
            && self.nth_at(7, TokenKind::Eq)
        {
            let TokenKind::Identifier(namespace) = self.tokens.get(self.pos + 2)?.kind.clone()
            else {
                return None;
            };
            let TokenKind::Identifier(type_name) = self.tokens.get(self.pos + 4)?.kind.clone()
            else {
                return None;
            };
            (format!("{namespace}.{type_name}"), 6, 7)
        } else {
            return None;
        };

        let TokenKind::Identifier(name) = self.tokens.get(self.pos + name_offset)?.kind.clone()
        else {
            return None;
        };

        let start = self.current().span;
        while self.pos < start_pos + eq_offset {
            self.bump();
        }
        Some((format!("array<{element_type}>"), name, start))
    }

    fn try_parse_array_type_alias_decl_name(&mut self) -> Option<(String, String, Span)> {
        let TokenKind::Identifier(first_type_name) = self.current().kind.clone() else {
            return None;
        };

        let (element_type, name_offset, eq_offset) = if self.nth_at(1, TokenKind::LBracket)
            && self.nth_at(2, TokenKind::RBracket)
            && self.nth_at(4, TokenKind::Eq)
        {
            (first_type_name, 3, 4)
        } else if self.nth_at(1, TokenKind::Dot)
            && self.nth_at(3, TokenKind::LBracket)
            && self.nth_at(4, TokenKind::RBracket)
            && self.nth_at(6, TokenKind::Eq)
        {
            let TokenKind::Identifier(type_name) = self.tokens.get(self.pos + 2)?.kind.clone()
            else {
                return None;
            };
            (format!("{first_type_name}.{type_name}"), 5, 6)
        } else {
            return None;
        };

        let TokenKind::Identifier(name) = self.tokens.get(self.pos + name_offset)?.kind.clone()
        else {
            return None;
        };

        let start = self.current().span;
        let start_pos = self.pos;
        while self.pos < start_pos + eq_offset {
            self.bump();
        }
        Some((format!("array<{element_type}>"), name, start))
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
        let parts = self.parse_for_parts()?;

        Some(Stmt {
            span: parts.start.merge(parts.span),
            kind: StmtKind::For {
                counter: parts.counter,
                from: parts.from,
                to: parts.to,
                step: parts.step,
                body: parts.body,
            },
        })
    }

    fn parse_while_stmt(&mut self) -> Option<Stmt> {
        let start = self.expect(TokenKind::While, "expected `while`")?;
        let condition = self.parse_expr(0)?;
        self.expect(
            TokenKind::Newline,
            "expected newline after `while` condition",
        )?;
        let body = self.parse_indented_block()?;
        let span = body
            .last()
            .map_or(condition.span, |statement| statement.span);

        Some(Stmt {
            span: start.merge(span),
            kind: StmtKind::While { condition, body },
        })
    }

    fn parse_for_parts(&mut self) -> Option<ForParts> {
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

        Some(ForParts {
            start,
            counter,
            from,
            to,
            step,
            body,
            span,
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

    fn looks_like_tuple_decl(&self) -> bool {
        if !self.at(TokenKind::LBracket) {
            return false;
        }

        let mut depth = 0_u32;
        let mut index = self.pos;
        while let Some(token) = self.tokens.get(index) {
            match token.kind {
                TokenKind::Newline | TokenKind::Eof => return false,
                TokenKind::LBracket => depth += 1,
                TokenKind::RBracket => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return self
                            .tokens
                            .get(index + 1)
                            .is_some_and(|next| matches!(next.kind, TokenKind::Eq));
                    }
                }
                _ => {}
            }
            index += 1;
        }

        false
    }

    fn parse_expr(&mut self, min_bp: u8) -> Option<Expr> {
        if self.expr_depth >= MAX_EXPR_DEPTH {
            self.error_here("E_PARSE_EXPR_DEPTH", "expression nesting is too deep");
            return None;
        }

        self.expr_depth += 1;
        let result = self.parse_expr_inner(min_bp);
        self.expr_depth -= 1;
        result
    }

    fn parse_expr_inner(&mut self, min_bp: u8) -> Option<Expr> {
        let mut left = self.parse_prefix()?;

        loop {
            if matches!(left.kind, ExprKind::For { .. } | ExprKind::Switch { .. }) {
                break;
            }
            if let Some(template_callee) = self.parse_array_new_template_callee(&left) {
                left = self.finish_call(template_callee)?;
                continue;
            }
            if self.at(TokenKind::LParen) {
                left = self.finish_call(left)?;
                continue;
            }
            if self.at(TokenKind::LBracket) {
                left = self.finish_history(left)?;
                continue;
            }
            if self.at(TokenKind::Question) {
                // The ternary `?:` operator has the lowest binding power and is
                // right-associative, so it only binds at the top of an
                // expression (when `min_bp == 0`).
                const TERNARY_BINDING_POWER: u8 = 0;
                if TERNARY_BINDING_POWER < min_bp {
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
            TokenKind::While => {
                self.error_here(
                    "E_PARSE_WHILE_EXPR",
                    "`while` expressions are not supported; use `while` as a statement",
                );
                None
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
            TokenKind::For => self.parse_for_expr(),
            TokenKind::Switch => self.parse_switch_expr(),
            TokenKind::LBracket => self.parse_tuple_expr(),
            _ => {
                self.error_here("E_PARSE_EXPR", "expected expression");
                None
            }
        }
    }

    fn parse_for_expr(&mut self) -> Option<Expr> {
        let parts = self.parse_for_parts()?;

        Some(Expr {
            span: parts.start.merge(parts.span),
            kind: ExprKind::For {
                counter: parts.counter,
                from: Box::new(parts.from),
                to: Box::new(parts.to),
                step: parts.step.map(Box::new),
                body: parts.body,
            },
        })
    }

    fn parse_switch_expr(&mut self) -> Option<Expr> {
        let start = self.expect(TokenKind::Switch, "expected `switch`")?;
        let selector = if self.at(TokenKind::Newline) {
            None
        } else {
            Some(Box::new(self.parse_expr(0)?))
        };
        self.expect(TokenKind::Newline, "expected newline after `switch`")?;
        self.expect(TokenKind::Indent, "expected indented switch arms")?;
        let mut arms = Vec::new();
        let mut end = selector.as_ref().map_or(start, |selector| selector.span);

        loop {
            self.skip_newlines();
            if self.at(TokenKind::Dedent) {
                self.bump();
                break;
            }
            if self.at(TokenKind::Eof) {
                self.error_here("E_PARSE_SWITCH", "expected dedent before end of switch");
                break;
            }

            let condition = if self.at(TokenKind::Arrow) {
                None
            } else {
                Some(self.parse_expr(0)?)
            };
            self.expect(TokenKind::Arrow, "expected `=>` in switch arm")?;
            if self.at(TokenKind::Newline) {
                self.diagnostics.push(Diagnostic::error(
                    "E_PARSE_SWITCH_BLOCK",
                    "statement-block switch arms are not supported; use expression arms",
                    self.current().span,
                ));
                return None;
            }
            let result = self.parse_expr(0)?;
            end = result.span;
            arms.push(SwitchArm { condition, result });

            if self.at(TokenKind::Newline) {
                self.bump();
            } else if !self.at(TokenKind::Dedent) && !self.at(TokenKind::Eof) {
                self.error_here("E_PARSE_SWITCH", "expected newline after switch arm");
                return None;
            }
        }

        if arms.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                "E_PARSE_SWITCH",
                "expected at least one switch arm",
                start,
            ));
            return None;
        }

        Some(Expr {
            span: start.merge(end),
            kind: ExprKind::Switch { selector, arms },
        })
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

    fn parse_array_new_template_callee(&mut self, callee: &Expr) -> Option<Expr> {
        if !matches!(
            &callee.kind,
            ExprKind::QualifiedName(parts) if parts.as_slice() == ["array", "new"]
        ) {
            return None;
        }

        let (builtin_name, close_offset) = self.parse_supported_array_new_template()?;

        for _ in 0..close_offset {
            self.bump();
        }
        let end = self.current().span;
        self.bump();

        Some(Expr {
            span: callee.span.merge(end),
            kind: ExprKind::Identifier(builtin_name.to_owned()),
        })
    }

    fn parse_supported_array_new_template(&self) -> Option<(&'static str, usize)> {
        if !self.at(TokenKind::Lt) {
            return None;
        }

        if self.nth_at(2, TokenKind::Gt) && self.nth_at(3, TokenKind::LParen) {
            return match &self.tokens.get(self.pos + 1)?.kind {
                TokenKind::Identifier(type_name) if type_name == "float" => {
                    Some(("array.new_float", 2))
                }
                TokenKind::Identifier(type_name) if type_name == "int" => {
                    Some(("array.new_int", 2))
                }
                TokenKind::Identifier(type_name) if type_name == "bool" => {
                    Some(("array.new_bool", 2))
                }
                TokenKind::Identifier(type_name) if type_name == "string" => {
                    Some(("array.new_string", 2))
                }
                TokenKind::Identifier(type_name) if type_name == "color" => {
                    Some(("array.new_color", 2))
                }
                TokenKind::Identifier(type_name) if type_name == "label" => {
                    Some(("array.new_label", 2))
                }
                TokenKind::Identifier(type_name) if type_name == "line" => {
                    Some(("array.new_line", 2))
                }
                TokenKind::Identifier(type_name) if type_name == "linefill" => {
                    Some(("array.new_linefill", 2))
                }
                TokenKind::Identifier(type_name) if type_name == "box" => {
                    Some(("array.new_box", 2))
                }
                TokenKind::Identifier(type_name) if type_name == "table" => {
                    Some(("array.new_table", 2))
                }
                _ => None,
            };
        }

        if self.nth_identifier_is(1, "chart")
            && self.nth_at(2, TokenKind::Dot)
            && self.nth_identifier_is(3, "point")
            && self.nth_at(4, TokenKind::Gt)
            && self.nth_at(5, TokenKind::LParen)
        {
            return Some(("array.new<chart.point>", 4));
        }

        None
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

    fn nth_identifier_is(&self, offset: usize, expected: &str) -> bool {
        self.tokens.get(self.pos + offset).is_some_and(
            |token| matches!(&token.kind, TokenKind::Identifier(name) if name == expected),
        )
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
