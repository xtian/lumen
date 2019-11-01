use liblumen_alloc::erts::exception::Exception;
use liblumen_alloc::{Process, Term};

use super::{decode_vec_term, term, u32};

pub fn decode<'a>(
    process: &Process,
    safe: bool,
    bytes: &'a [u8],
) -> Result<(Term, &'a [u8]), Exception> {
    let (len_32, after_len_bytes) = u32::decode(bytes)?;
    let (element_vec, after_elements_bytes) =
        decode_vec_term(process, safe, after_len_bytes, len_32 as usize)?;
    let (tail, after_tail_bytes) = term::decode_tagged(process, safe, after_elements_bytes)?;

    let list = process.improper_list_from_slice(&element_vec, tail)?;

    Ok((list, after_tail_bytes))
}
