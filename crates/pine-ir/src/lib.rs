//! Host-independent intermediate representation scaffolding.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeriesId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CallSiteId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarSlotId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Qualifier {
    Const,
    Input,
    Simple,
    Series,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Int,
    Float,
    Bool,
    String,
    Color,
    Plot,
    HLine,
    Tuple,
    Na,
    Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PineType {
    pub qualifier: Qualifier,
    pub kind: ValueKind,
}

impl PineType {
    #[must_use]
    pub const fn new(qualifier: Qualifier, kind: ValueKind) -> Self {
        Self { qualifier, kind }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirProgram {
    pub symbols: Vec<HirSymbol>,
    pub statements: Vec<HirStmt>,
    pub next_series_id: u32,
    pub next_call_site_id: u32,
    pub next_var_slot_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirSymbol {
    pub id: SymbolId,
    pub name: String,
    pub pine_type: PineType,
    pub series_id: Option<SeriesId>,
    pub var_slot_id: Option<VarSlotId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirStmt {
    pub kind: HirStmtKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirStmtKind {
    Expr(HirExpr),
    If {
        condition: HirExpr,
        then_branch: Vec<HirStmt>,
        else_branch: Vec<HirStmt>,
    },
    For {
        counter: SymbolId,
        from: HirExpr,
        to: HirExpr,
        step: Option<HirExpr>,
        body: Vec<HirStmt>,
    },
    Decl {
        symbol: SymbolId,
        value: HirExpr,
    },
    Reassign {
        symbol: SymbolId,
        value: HirExpr,
    },
    TupleDecl {
        symbols: Vec<SymbolId>,
        value: HirExpr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirExpr {
    pub kind: HirExprKind,
    pub pine_type: PineType,
    pub series_id: Option<SeriesId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirExprKind {
    Literal(HirLiteral),
    Symbol(SymbolId),
    Builtin(String),
    Unary {
        op: HirUnaryOp,
        expr: Box<HirExpr>,
    },
    Binary {
        op: HirBinaryOp,
        left: Box<HirExpr>,
        right: Box<HirExpr>,
    },
    Ternary {
        condition: Box<HirExpr>,
        then_expr: Box<HirExpr>,
        else_expr: Box<HirExpr>,
    },
    Tuple(Vec<HirExpr>),
    Block {
        statements: Vec<HirStmt>,
        result: Box<HirExpr>,
    },
    Call {
        callee: String,
        call_site_id: CallSiteId,
        args: Vec<HirCallArg>,
    },
    History {
        expr: Box<HirExpr>,
        offset: u32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirCallArg {
    pub name: Option<String>,
    pub value: HirExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirLiteral {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    ColorHex(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirUnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirBinaryOp {
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
