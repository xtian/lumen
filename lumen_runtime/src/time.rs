use core::convert::{TryFrom, TryInto};

use num_bigint::BigInt;
use num_traits::Zero;

use liblumen_alloc::erts::exception::runtime::Exception;
use liblumen_alloc::erts::exception::system::Alloc;
use liblumen_alloc::erts::term::{atom_unchecked, Term, TypedTerm};
use liblumen_alloc::{badarg, Process};

pub mod datetime;
pub mod monotonic;
pub mod system;

// Must be at least a `u64` because `u32` is only ~49 days (`(1 << 32)`)
pub type Milliseconds = u64;
pub type Source = fn() -> Milliseconds;

// private
const MILLISECONDS_PER_SECOND: u64 = 1_000;
const MICROSECONDS_PER_MILLISECOND: u64 = 1_000;
const NANOSECONDS_PER_MICROSECOND: u64 = 1_000;
const NANOSECONDS_PER_MILLISECONDS: u64 =
    NANOSECONDS_PER_MICROSECOND * MICROSECONDS_PER_MILLISECOND;

pub fn convert_milliseconds(milliseconds: Milliseconds, unit: Unit) -> BigInt {
    match unit {
        Unit::Second => (milliseconds / MILLISECONDS_PER_SECOND).into(),
        Unit::Millisecond => milliseconds.into(),
        Unit::Microsecond => (milliseconds * MICROSECONDS_PER_MILLISECOND).into(),
        Unit::Nanosecond => (milliseconds * NANOSECONDS_PER_MILLISECONDS).into(),
        _ => convert(
            (milliseconds * NANOSECONDS_PER_MILLISECONDS).into(),
            Unit::Nanosecond,
            unit,
        ),
    }
}

pub fn convert(time: BigInt, from_unit: Unit, to_unit: Unit) -> BigInt {
    if from_unit == to_unit {
        time
    } else {
        let from_hertz = from_unit.hertz();
        let to_hertz = to_unit.hertz();

        if from_hertz <= to_hertz {
            time * ((to_hertz / from_hertz) as i32)
        } else {
            // mimic behavior of erts_napi_convert_time_unit, so that rounding is the same
            let denominator: BigInt = (from_hertz / to_hertz).into();
            let zero: BigInt = Zero::zero();

            if zero <= time {
                time / denominator
            } else {
                (time - (denominator.clone() - 1)) / denominator
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(Debug))]
pub enum Unit {
    Hertz(usize),
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
    Native,
    PerformanceCounter,
}

impl Unit {
    const MILLISECOND_HERTZ: usize = 1_000;

    pub fn hertz(&self) -> usize {
        match self {
            Unit::Hertz(hertz) => *hertz,
            Unit::Second => 1,
            Unit::Millisecond => Self::MILLISECOND_HERTZ,
            Unit::Microsecond => 1_000_000,
            Unit::Nanosecond => 1_000_000_000,
            // As a side-channel protection browsers limit most counters to 1 millisecond resolution
            Unit::Native => Self::MILLISECOND_HERTZ,
            Unit::PerformanceCounter => Self::MILLISECOND_HERTZ,
        }
    }

    pub fn to_term(&self, process: &Process) -> Result<Term, Alloc> {
        match self {
            Unit::Hertz(hertz) => process.integer(*hertz),
            Unit::Second => Ok(atom_unchecked("second")),
            Unit::Millisecond => Ok(atom_unchecked("millisecond")),
            Unit::Microsecond => Ok(atom_unchecked("microsecond")),
            Unit::Nanosecond => Ok(atom_unchecked("nanosecond")),
            Unit::Native => Ok(atom_unchecked("native")),
            Unit::PerformanceCounter => Ok(atom_unchecked("perf_counter")),
        }
    }
}

impl TryFrom<Term> for Unit {
    type Error = Exception;

    fn try_from(term: Term) -> Result<Unit, Exception> {
        match term.to_typed_term().unwrap() {
            TypedTerm::SmallInteger(small_integer) => {
                let hertz: usize = small_integer.try_into()?;

                if 0 < hertz {
                    Ok(Unit::Hertz(hertz))
                } else {
                    Err(badarg!())
                }
            }
            TypedTerm::Boxed(boxed) => match boxed.to_typed_term().unwrap() {
                TypedTerm::BigInteger(big_integer) => {
                    let big_integer_usize: usize = big_integer.try_into()?;

                    Ok(Unit::Hertz(big_integer_usize))
                }
                _ => Err(badarg!()),
            },
            TypedTerm::Atom(atom) => {
                let term_string = atom.name();
                let mut result = Err(badarg!());

                for (s, unit) in [
                    ("second", Unit::Second),
                    ("seconds", Unit::Second),
                    ("millisecond", Unit::Millisecond),
                    ("milli_seconds", Unit::Millisecond),
                    ("microsecond", Unit::Microsecond),
                    ("micro_seconds", Unit::Microsecond),
                    ("nanosecond", Unit::Nanosecond),
                    ("nano_seconds", Unit::Nanosecond),
                    ("native", Unit::Native),
                    ("perf_counter", Unit::PerformanceCounter),
                ]
                .iter()
                {
                    if &term_string == s {
                        result = Ok(*unit);
                        break;
                    }
                }

                result
            }
            _ => Err(badarg!()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod unit {
        use super::*;

        use crate::scheduler::with_process;

        #[test]
        fn zero_errors_badarg() {
            with_process(|process| {
                let term: Term = process.integer(0).unwrap();

                let result: Result<Unit, Exception> = term.try_into();

                assert_badarg!(result);
            });
        }
    }
}
