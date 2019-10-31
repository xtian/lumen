use hashbrown::hash_map::HashMap;

use liblumen_core::locks::RwLock;

use liblumen_alloc::erts::process::code::Code;
use liblumen_alloc::erts::term::Atom;

pub fn get(module: Atom, function: Atom, arity: u8) -> Option<Code> {
    RW_LOCK_CODE_BY_ARITY_BY_FUNCTION_BY_MODULE
        .read()
        .get(&module)
        .and_then(|code_by_arity_by_function| {
            code_by_arity_by_function
                .get(&function)
                .and_then(|code_by_arity| code_by_arity.get(&arity).map(|code| *code))
        })
}

pub fn insert(module: Atom, function: Atom, arity: u8, code: Code) {
    RW_LOCK_CODE_BY_ARITY_BY_FUNCTION_BY_MODULE
        .write()
        .entry(module)
        .or_insert_with(Default::default)
        .entry(function)
        .or_insert_with(Default::default)
        .insert(arity, code);
}

lazy_static! {
    static ref RW_LOCK_CODE_BY_ARITY_BY_FUNCTION_BY_MODULE: RwLock<HashMap<Atom, HashMap<Atom, HashMap<u8, Code>>>> =
        Default::default();
}
