use crate::namespaces::types::VOID;
use crate::namespaces::{
    alerts, arrays, colors, core, drawings, math, outputs, requests, strategy, strings, ta, time,
};
use crate::signature::{BuiltinParam, BuiltinPhase, BuiltinSignature, ReturnSpec};

const EMPTY_PARAMS: &[BuiltinParam] = &[];
const EMPTY_SIGNATURE: BuiltinSignature = BuiltinSignature {
    name: "",
    phase: BuiltinPhase::Later,
    params: EMPTY_PARAMS,
    returns: ReturnSpec::Fixed(VOID),
    variadic: false,
};

const BUILTIN_COUNT: usize = core::SCRIPT_SIGNATURES.len()
    + alerts::SIGNATURES.len()
    + outputs::SIGNATURES.len()
    + requests::SIGNATURES.len()
    + strategy::SIGNATURES.len()
    + drawings::SIGNATURES.len()
    + colors::SIGNATURES.len()
    + strings::SIGNATURES.len()
    + time::SIGNATURES.len()
    + core::CAST_SIGNATURES.len()
    + math::SIGNATURES.len()
    + core::VALUE_SIGNATURES.len()
    + arrays::SIGNATURES.len()
    + ta::SIGNATURES.len();

const PHASE_1_BUILTINS_ARRAY: [BuiltinSignature; BUILTIN_COUNT] = build_phase_1_builtins();

pub const PHASE_1_BUILTINS: &[BuiltinSignature] = &PHASE_1_BUILTINS_ARRAY;

const fn build_phase_1_builtins() -> [BuiltinSignature; BUILTIN_COUNT] {
    let mut builtins = [EMPTY_SIGNATURE; BUILTIN_COUNT];
    let mut index = 0;

    index = copy_signatures(&mut builtins, index, core::SCRIPT_SIGNATURES);
    index = copy_signatures(&mut builtins, index, alerts::SIGNATURES);
    index = copy_signatures(&mut builtins, index, outputs::SIGNATURES);
    index = copy_signatures(&mut builtins, index, requests::SIGNATURES);
    index = copy_signatures(&mut builtins, index, strategy::SIGNATURES);
    index = copy_signatures(&mut builtins, index, drawings::SIGNATURES);
    index = copy_signatures(&mut builtins, index, colors::SIGNATURES);
    index = copy_signatures(&mut builtins, index, strings::SIGNATURES);
    index = copy_signatures(&mut builtins, index, time::SIGNATURES);
    index = copy_signatures(&mut builtins, index, core::CAST_SIGNATURES);
    index = copy_signatures(&mut builtins, index, math::SIGNATURES);
    index = copy_signatures(&mut builtins, index, core::VALUE_SIGNATURES);
    index = copy_signatures(&mut builtins, index, arrays::SIGNATURES);
    let _index = copy_signatures(&mut builtins, index, ta::SIGNATURES);

    builtins
}

const fn copy_signatures<const N: usize>(
    builtins: &mut [BuiltinSignature; N],
    start: usize,
    signatures: &[BuiltinSignature],
) -> usize {
    let mut offset = 0;
    while offset < signatures.len() {
        builtins[start + offset] = signatures[offset];
        offset += 1;
    }
    start + offset
}

#[must_use]
pub fn is_phase_1_builtin(name: &str) -> bool {
    PHASE_1_BUILTINS
        .iter()
        .any(|signature| signature.name == name)
}

#[must_use]
pub fn get_phase_1_builtin(name: &str) -> Option<&'static BuiltinSignature> {
    PHASE_1_BUILTINS
        .iter()
        .find(|signature| signature.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_strategy_declaration_signature() {
        let signature = get_phase_1_builtin("strategy").expect("strategy declaration signature");
        assert_eq!(signature.params[0].name, "title");
        assert_eq!(signature.params[0].accepts, crate::Accepts::ConstString);
        assert_eq!(signature.params[4].name, "initial_capital");
        assert_eq!(signature.params[4].accepts, crate::Accepts::ConstNumeric);
        assert_eq!(signature.params[5].name, "default_qty_type");
        assert_eq!(signature.params[5].accepts, crate::Accepts::ConstString);
        assert_eq!(signature.params[6].name, "default_qty_value");
        assert_eq!(signature.params[6].accepts, crate::Accepts::ConstNumeric);
        assert!(!signature.variadic);
    }

    #[test]
    fn registers_strategy_entry_signature() {
        let signature = get_phase_1_builtin("strategy.entry").expect("strategy.entry signature");
        assert_eq!(signature.params[0].name, "id");
        assert_eq!(signature.params[1].name, "direction");
        assert_eq!(signature.params[2].name, "qty");
        assert!(signature.params[2].optional);
        assert!(!signature.variadic);
    }

    #[test]
    fn registers_strategy_close_signature() {
        let signature = get_phase_1_builtin("strategy.close").expect("strategy.close signature");
        assert_eq!(signature.params[0].name, "id");
        assert!(!signature.variadic);
    }

    #[test]
    fn registers_strategy_exit_signature() {
        let signature = get_phase_1_builtin("strategy.exit").expect("strategy.exit signature");
        assert_eq!(signature.params[0].name, "id");
        assert_eq!(signature.params[1].name, "from_entry");
        assert_eq!(signature.params[2].name, "stop");
        assert!(signature.params[2].optional);
        assert_eq!(
            signature.params[2].accepts,
            crate::Accepts::SeriesOrSimpleNumeric
        );
        assert_eq!(signature.params[3].name, "limit");
        assert!(signature.params[3].optional);
        assert_eq!(
            signature.params[3].accepts,
            crate::Accepts::SeriesOrSimpleNumeric
        );
        assert_eq!(signature.params[4].name, "profit");
        assert!(signature.params[4].optional);
        assert_eq!(
            signature.params[4].accepts,
            crate::Accepts::SeriesOrSimpleNumeric
        );
        assert_eq!(signature.params[5].name, "loss");
        assert!(signature.params[5].optional);
        assert_eq!(
            signature.params[5].accepts,
            crate::Accepts::SeriesOrSimpleNumeric
        );
        assert_eq!(signature.params[6].name, "trail_price");
        assert!(signature.params[6].optional);
        assert_eq!(
            signature.params[6].accepts,
            crate::Accepts::SeriesOrSimpleNumeric
        );
        assert_eq!(signature.params[7].name, "trail_points");
        assert!(signature.params[7].optional);
        assert_eq!(
            signature.params[7].accepts,
            crate::Accepts::SeriesOrSimpleNumeric
        );
        assert_eq!(signature.params[8].name, "trail_offset");
        assert!(signature.params[8].optional);
        assert_eq!(
            signature.params[8].accepts,
            crate::Accepts::SeriesOrSimpleNumeric
        );
        assert!(!signature.variadic);
    }
}
