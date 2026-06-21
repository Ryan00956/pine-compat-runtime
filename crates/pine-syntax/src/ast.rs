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
    Import(ImportDecl),
    Library(LibraryDecl),
    Export(ExportDecl),
    UserType(UserTypeDecl),
    Method(MethodDecl),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
    },
    For {
        counter: String,
        from: Expr,
        to: Expr,
        step: Option<Expr>,
        body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    Function {
        name: String,
        params: Vec<String>,
        body: FunctionBody,
    },
    Decl {
        mode: DeclMode,
        declared_type: Option<String>,
        name: String,
        value: Expr,
    },
    Reassign {
        name: String,
        value: Expr,
    },
    FieldReassign {
        receiver: String,
        field: String,
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
pub struct ImportDecl {
    pub key: String,
    pub key_span: Span,
    pub alias: Option<ImportAlias>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportAlias {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LibraryDecl {
    pub name: Option<String>,
    pub name_span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportDecl {
    pub item: ExportItem,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportItem {
    Function {
        name: String,
        params: Vec<String>,
        body: FunctionBody,
        span: Span,
    },
    Const {
        name: String,
        value: Expr,
        span: Span,
    },
    Unknown {
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserTypeDecl {
    pub name: String,
    pub name_span: Span,
    pub fields: Vec<UserTypeField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserTypeField {
    pub type_name: String,
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub name: String,
    pub name_span: Span,
    pub params: Vec<MethodParam>,
    pub body: FunctionBody,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodParam {
    pub type_name: String,
    pub name: String,
    pub span: Span,
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
    If {
        condition: Box<Expr>,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
    },
    For {
        counter: String,
        from: Box<Expr>,
        to: Box<Expr>,
        step: Option<Box<Expr>>,
        body: Vec<Stmt>,
    },
    Switch {
        selector: Option<Box<Expr>>,
        arms: Vec<SwitchArm>,
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
pub struct SwitchArm {
    pub condition: Option<Expr>,
    pub result: Expr,
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
