#[cfg(test)]
mod test;

use liblumen_alloc::erts::exception;
use liblumen_alloc::erts::process::Process;

use lumen_runtime_macros::native_implemented_function;

use crate::time::{system, Unit::Native};

#[native_implemented_function(system_time/0)]
pub fn native(process: &Process) -> exception::Result {
    let big_int = system::time(Native);

    Ok(process.integer(big_int)?)
}
