#[cfg(test)]
use std::convert::TryInto;
use std::sync::Arc;

#[cfg(test)]
use liblumen_alloc::erts::exception::runtime;
use liblumen_alloc::erts::exception::system::Alloc;
#[cfg(not(test))]
use liblumen_alloc::erts::exception::Exception;
use liblumen_alloc::erts::process::code;
use liblumen_alloc::erts::process::code::stack::frame::{Frame, Placement};
use liblumen_alloc::erts::process::Process;
#[cfg(test)]
use liblumen_alloc::erts::term::TypedTerm;
use liblumen_alloc::erts::term::{Atom, Term};
use liblumen_alloc::ModuleFunctionArity;

#[cfg(test)]
use crate::otp::erlang;

pub fn place_frame_with_arguments(
    process: &Process,
    placement: Placement,
    module: Term,
    function: Term,
    arguments: Term,
) -> Result<(), Alloc> {
    process.stack_push(arguments)?;
    process.stack_push(function)?;
    process.stack_push(module)?;
    process.place_frame(frame(), placement);

    Ok(())
}

// Private

/// Treats all MFAs as undefined.
#[cfg(not(test))]
fn code(arc_process: &Arc<Process>) -> code::Result {
    // arguments are consumed, but unused
    let module = arc_process.stack_pop().unwrap();
    let function = arc_process.stack_pop().unwrap();
    let arguments = arc_process.stack_pop().unwrap();
    arc_process.reduce();

    match liblumen_alloc::undef!(arc_process, module, function, arguments) {
        Exception::Runtime(runtime_exception) => {
            arc_process.exception(runtime_exception);

            Ok(())
        }
        Exception::System(system_exception) => Err(system_exception),
    }
}

#[cfg(test)]
pub fn code(arc_process: &Arc<Process>) -> code::Result {
    let module = arc_process.stack_pop().unwrap();
    let function = arc_process.stack_pop().unwrap();
    let argument_list = arc_process.stack_pop().unwrap();

    let mut argument_vec: Vec<Term> = Vec::new();

    match argument_list.to_typed_term().unwrap() {
        TypedTerm::Nil => (),
        TypedTerm::List(argument_cons) => {
            for result in argument_cons.into_iter() {
                let element = result.unwrap();

                argument_vec.push(element);
            }
        }
        _ => panic!("{:?} is not an argument list", argument_list),
    }

    let arity = argument_vec.len();

    let module_atom: Atom = module.try_into().unwrap();
    let function_atom: Atom = function.try_into().unwrap();

    match module_atom.name() {
        "erlang" => match function_atom.name() {
            "+" => match arity {
                1 => {
                    erlang::number_or_badarith_1::place_frame_with_arguments(
                        arc_process,
                        Placement::Replace,
                        argument_vec[0],
                    )?;

                    Process::call_code(arc_process)
                }
                _ => undef(arc_process, module, function, argument_list),
            },
            "self" => match arity {
                0 => {
                    erlang::self_0::place_frame_with_arguments(arc_process, Placement::Replace)?;

                    Process::call_code(arc_process)
                }
                _ => undef(arc_process, module, function, argument_list),
            },
            _ => undef(arc_process, module, function, argument_list),
        },
        _ => undef(arc_process, module, function, argument_list),
    }
}

fn frame() -> Frame {
    Frame::new(module_function_arity(), code)
}

fn function() -> Atom {
    Atom::try_from_str("apply").unwrap()
}

pub(crate) fn module_function_arity() -> Arc<ModuleFunctionArity> {
    Arc::new(ModuleFunctionArity {
        module: super::module(),
        function: function(),
        arity: 3,
    })
}

#[cfg(test)]
fn undef(
    arc_process: &Arc<Process>,
    module: Term,
    function: Term,
    arguments: Term,
) -> code::Result {
    arc_process.reduce();
    let exception = liblumen_alloc::undef!(arc_process, module, function, arguments);
    let runtime_exception: runtime::Exception = exception.try_into().unwrap();
    arc_process.exception(runtime_exception);

    Ok(())
}
