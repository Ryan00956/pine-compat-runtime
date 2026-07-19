mod declarations;
mod dialect;

use pine_syntax::Program;

pub use declarations::ScriptModeClassification;
pub use dialect::{
    MAX_PINE_LANGUAGE_VERSION, MIN_PINE_LANGUAGE_VERSION, PineDialect, VersionOrigin,
};

pub(crate) use declarations::{LegacyAdmissionFailure, legacy_admission_failure};
pub(crate) use dialect::LanguageSelection;

#[derive(Debug, Clone)]
pub(crate) struct SourcePolicy {
    pub(crate) language: LanguageSelection,
    pub(crate) script_mode: ScriptModeClassification,
    pub(crate) legacy_admission_failure: Option<LegacyAdmissionFailure>,
}

impl SourcePolicy {
    pub(crate) fn from_program_with_implicit(
        program: &Program,
        implicit_dialect: PineDialect,
    ) -> Self {
        let language = LanguageSelection::from_program_with_implicit(program, implicit_dialect);
        let script_mode = declarations::classify_script_mode(program);
        let legacy_admission_failure = language
            .dialect
            .and_then(|dialect| legacy_admission_failure(program, dialect));
        Self {
            language,
            script_mode,
            legacy_admission_failure,
        }
    }
}
