use crate::namespaces::types::VOID;
use crate::namespaces::{
    arrays, colors, core, drawings, math, outputs, requests, strings, ta, time,
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
    + outputs::SIGNATURES.len()
    + requests::SIGNATURES.len()
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
    index = copy_signatures(&mut builtins, index, outputs::SIGNATURES);
    index = copy_signatures(&mut builtins, index, requests::SIGNATURES);
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
