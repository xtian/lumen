use proptest::prop_assert_eq;
use proptest::strategy::{Just, Strategy};
use proptest::test_runner::{Config, TestRunner};

use liblumen_alloc::erts::term::atom_unchecked;

use crate::otp::erlang::put_2::native;
use crate::scheduler::with_process_arc;
use crate::test::strategy;

#[test]
fn without_key_returns_undefined_for_previous_value() {
    with_process_arc(|arc_process| {
        TestRunner::new(Config::with_source_file(file!()))
            .run(
                &(
                    strategy::term(arc_process.clone()),
                    strategy::term(arc_process.clone()),
                ),
                |(key, value)| {
                    arc_process.erase_entries().unwrap();

                    prop_assert_eq!(
                        native(&arc_process, key, value),
                        Ok(atom_unchecked("undefined"))
                    );

                    prop_assert_eq!(arc_process.get_value_from_key(key), value);

                    Ok(())
                },
            )
            .unwrap();
    });
}

#[test]
fn with_key_returns_previous_value() {
    TestRunner::new(Config::with_source_file(file!()))
        .run(
            &strategy::process().prop_flat_map(|arc_process| {
                (
                    Just(arc_process.clone()),
                    strategy::term(arc_process.clone()),
                    strategy::term(arc_process.clone()),
                    strategy::term(arc_process),
                )
            }),
            |(arc_process, key, old_value, new_value)| {
                arc_process.erase_entries().unwrap();

                arc_process.put(key, old_value).unwrap();

                prop_assert_eq!(native(&arc_process, key, new_value), Ok(old_value));

                prop_assert_eq!(arc_process.get_value_from_key(key), new_value);

                Ok(())
            },
        )
        .unwrap();
}
