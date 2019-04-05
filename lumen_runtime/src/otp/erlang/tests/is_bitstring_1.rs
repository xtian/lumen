use super::*;

use std::sync::{Arc, RwLock};

use num_traits::Num;

use crate::environment::{self, Environment};
use crate::process::IntoProcess;

#[test]
fn with_atom_is_false() {
    let term = Term::str_to_atom("atom", DoNotCare).unwrap();

    assert_eq!(erlang::is_bitstring_1(term), false.into());
}

#[test]
fn with_local_reference_is_false() {
    with_process(|mut process| {
        let term = Term::local_reference(&mut process);

        assert_eq!(erlang::is_bitstring_1(term), false.into());
    });
}

#[test]
fn with_empty_list_is_false() {
    let term = Term::EMPTY_LIST;

    assert_eq!(erlang::is_bitstring_1(term), false.into());
}

#[test]
fn with_list_is_false() {
    with_process(|mut process| {
        let head_term = Term::str_to_atom("head", DoNotCare).unwrap();
        let term = Term::cons(head_term, Term::EMPTY_LIST, &mut process);

        assert_eq!(erlang::is_bitstring_1(term), false.into());
    });
}

#[test]
fn with_small_integer_is_false() {
    let environment_rw_lock: Arc<RwLock<Environment>> = Default::default();
    let process_rw_lock = environment::process(Arc::clone(&environment_rw_lock));
    let mut process = process_rw_lock.write().unwrap();
    let term = 0.into_process(&mut process);

    assert_eq!(erlang::is_bitstring_1(term), false.into());
}

#[test]
fn with_big_integer_is_false() {
    with_process(|mut process| {
        let term = <BigInt as Num>::from_str_radix("576460752303423489", 10)
            .unwrap()
            .into_process(&mut process);

        assert_eq!(erlang::is_bitstring_1(term), false.into());
    });
}

#[test]
fn with_float_is_false() {
    with_process(|mut process| {
        let term = 1.0.into_process(&mut process);

        assert_eq!(erlang::is_bitstring_1(term), false.into());
    });
}

#[test]
fn with_local_pid_is_false() {
    let term = Term::local_pid(0, 0).unwrap();

    assert_eq!(erlang::is_bitstring_1(term), false.into());
}

#[test]
fn with_external_pid_is_false() {
    with_process(|mut process| {
        let term = Term::external_pid(1, 0, 0, &mut process).unwrap();

        assert_eq!(erlang::is_bitstring_1(term), false.into());
    });
}

#[test]
fn with_tuple_is_false() {
    with_process(|mut process| {
        let term = Term::slice_to_tuple(&[], &mut process);

        assert_eq!(erlang::is_bitstring_1(term), false.into());
    });
}

#[test]
fn with_map_is_false() {
    with_process(|mut process| {
        let term = Term::slice_to_map(&[], &mut process);

        assert_eq!(erlang::is_bitstring_1(term), false.into());
    });
}

#[test]
fn with_heap_binary_is_true() {
    with_process(|mut process| {
        let term = Term::slice_to_binary(&[], &mut process);

        assert_eq!(erlang::is_bitstring_1(term), true.into());
    });
}

#[test]
fn with_subbinary_with_bit_count_is_true() {
    with_process(|mut process| {
        let original =
            Term::slice_to_binary(&[0b0000_00001, 0b1111_1110, 0b1010_1011], &mut process);
        let term = Term::subbinary(original, 0, 7, 2, 1, &mut process);

        assert_eq!(erlang::is_bitstring_1(term), true.into());
    });
}

#[test]
fn with_subbinary_without_bit_count_is_true() {
    with_process(|mut process| {
        let original =
            Term::slice_to_binary(&[0b0000_00001, 0b1111_1110, 0b1010_1011], &mut process);
        let term = Term::subbinary(original, 0, 7, 2, 0, &mut process);

        assert_eq!(erlang::is_bitstring_1(term), true.into());
    });
}