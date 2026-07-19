use pine_syntax::{Diagnostic, Program, Span};

pub const MIN_PINE_LANGUAGE_VERSION: u16 = 1;
pub const MAX_PINE_LANGUAGE_VERSION: u16 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PineDialect {
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
}

impl PineDialect {
    #[must_use]
    pub const fn from_version(version: u16) -> Option<Self> {
        match version {
            1 => Some(Self::V1),
            2 => Some(Self::V2),
            3 => Some(Self::V3),
            4 => Some(Self::V4),
            5 => Some(Self::V5),
            6 => Some(Self::V6),
            _ => None,
        }
    }

    #[must_use]
    pub const fn version(self) -> u16 {
        match self {
            Self::V1 => 1,
            Self::V2 => 2,
            Self::V3 => 3,
            Self::V4 => 4,
            Self::V5 => 5,
            Self::V6 => 6,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::V1 => "v1",
            Self::V2 => "v2",
            Self::V3 => "v3",
            Self::V4 => "v4",
            Self::V5 => "v5",
            Self::V6 => "v6",
        }
    }

    #[must_use]
    pub const fn is_legacy(self) -> bool {
        matches!(self, Self::V1 | Self::V2 | Self::V3 | Self::V4)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VersionOrigin {
    ExplicitDirective,
    ImplicitV1,
}

impl VersionOrigin {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ExplicitDirective => "explicit",
            Self::ImplicitV1 => "implicit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LanguageSelection {
    pub(crate) raw_version: u16,
    pub(crate) origin: VersionOrigin,
    pub(crate) dialect: Option<PineDialect>,
    pub(crate) span: Span,
}

impl LanguageSelection {
    pub(crate) fn from_program_with_implicit(
        program: &Program,
        implicit_dialect: PineDialect,
    ) -> Self {
        match program.version {
            Some(version) => Self {
                raw_version: version.version,
                origin: VersionOrigin::ExplicitDirective,
                dialect: PineDialect::from_version(version.version),
                span: version.span,
            },
            None => Self {
                raw_version: implicit_dialect.version(),
                origin: if implicit_dialect == PineDialect::V1 {
                    VersionOrigin::ImplicitV1
                } else {
                    VersionOrigin::ExplicitDirective
                },
                dialect: Some(implicit_dialect),
                span: Span::new(0, 0),
            },
        }
    }

    pub(crate) fn unsupported_diagnostic(self) -> Option<Diagnostic> {
        self.dialect.is_none().then(|| {
            Diagnostic::error(
                "E_LANGUAGE_VERSION_UNSUPPORTED",
                format!(
                    "Pine language version {} is unsupported; expected {} through {}",
                    self.raw_version, MIN_PINE_LANGUAGE_VERSION, MAX_PINE_LANGUAGE_VERSION
                ),
                self.span,
            )
        })
    }
}
