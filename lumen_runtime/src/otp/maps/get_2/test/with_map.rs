use super::*;

use liblumen_alloc::badkey;
use liblumen_alloc::erts::term::atom_unchecked;

#[test]
fn without_key_errors_badkey() {
    TestRunner::new(Config::with_source_file(file!()))
        .run(
            &strategy::process()
                .prop_flat_map(|arc_process| {
                    (
                        Just(arc_process.clone()),
                        strategy::term(arc_process.clone()),
                        strategy::term(arc_process),
                    )
                })
                .prop_filter("Key and non-key must be different", |(_, key, non_key)| {
                    key != non_key
                })
                .prop_map(|(arc_process, key, non_key)| {
                    let value = atom_unchecked("value");

                    (
                        arc_process.clone(),
                        non_key,
                        arc_process.map_from_slice(&[(key, value)]).unwrap(),
                    )
                }),
            |(arc_process, key, map)| {
                prop_assert_eq!(
                    native(&arc_process, key, map),
                    Err(badkey!(&arc_process, key))
                );

                Ok(())
            },
        )
        .unwrap();
}

#[test]
fn with_key_returns_value() {
    with_process_arc(|arc_process| {
        TestRunner::new(Config::with_source_file(file!()))
            .run(
                &strategy::term(arc_process.clone()).prop_map(|key| {
                    let value = atom_unchecked("value");

                    (key, arc_process.map_from_slice(&[(key, value)]).unwrap())
                }),
                |(key, map)| {
                    let value = atom_unchecked("value");
                    prop_assert_eq!(native(&arc_process, key, map), Ok(value.into()));

                    Ok(())
                },
            )
            .unwrap();
    });
}
