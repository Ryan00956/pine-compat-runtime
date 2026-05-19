//! Syntax layer: source files, spans, diagnostics, lexer, AST, and parser.

mod ast;
mod diagnostic;
mod lexer;
mod parser;
mod source;

pub use ast::{
    BinaryOp, CallArg, DeclMode, Expr, ExprKind, FunctionBody, Literal, Program, Stmt, StmtKind,
    UnaryOp, VersionDecl,
};
pub use diagnostic::{Diagnostic, Severity};
pub use lexer::{Lexed, Token, TokenKind, lex};
pub use parser::{Parse, parse_source};
pub use source::{LineCol, SourceFile, Span};
