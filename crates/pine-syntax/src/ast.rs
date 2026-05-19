use crate::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub version: Option<VersionDecl>,
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionDecl {
    pub version: u16,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Expr(Expr),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: FunctionBody,
    },
    Decl {
        mode: DeclMode,
        name: String,
        value: Expr,
    },
    Reassign {
        name: String,
        value: Expr,
    },
    TupleDecl {
        names: Vec<String>,
        value: Expr,
    },
    Unsupported {
        feature: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    Expr(Expr),
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclMode {
    Normal,
    Var,
    Varip,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Literal(Literal),
    Identifier(String),
    QualifiedName(Vec<String>),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    Tuple(Vec<Expr>),
    Call {
        callee: Box<Expr>,
        args: Vec<CallArg>,
    },
    History {
        expr: Box<Expr>,
        offset: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallArg {
    pub name: Option<String>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    ColorHex(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Gt,
    Gte,
    Lt,
    Lte,
    And,
    Or,
}
