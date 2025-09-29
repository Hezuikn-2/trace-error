#![allow(internal_features)]
#![feature(auto_traits, negative_impls, min_specialization, const_convert, const_trait_impl, core_intrinsics, const_eval_select)]
#![feature(const_precise_live_drops, never_type, try_trait_v2)]
#![feature(thin_box)]
#![no_std]

extern crate alloc;

use core::result::Result as StdResult;
use core::{
    error::Error,
    fmt::{self, Debug, Display},
};

mod err;
mod maybe;

pub use crate::{
    err::Traced,
    maybe::Maybe::{self, *},
};

pub type R<T> = Maybe<T, Traced>;

unsafe extern "C" {
    pub safe fn link_err() -> !;
}

#[derive(Debug)]
pub struct OptNone;
impl Display for OptNone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("None")
    }
}
impl Error for OptNone {}

#[inline(never)]
#[cold]
#[track_caller]
fn unwrap_failed(msg: &str, error: &dyn fmt::Debug) -> ! {
    panic!("{msg}: {error:?}")
}

pub trait CertainResult {
    type T;
    type E;
    fn certain_ok(self) -> Self::T;
    fn certain_err(self) -> Self::E;
}
impl<T, E> CertainResult for Maybe<T, E> {
    type T = T;
    type E = E;

    #[inline]
    fn certain_ok(self) -> T {
        match self {
            O(t) => t,
            X(_) => link_err(),
        }
    }

    #[inline]
    fn certain_err(self) -> E {
        match self {
            O(_) => link_err(),
            X(e) => e,
        }
    }
}
impl<T, E> CertainResult for StdResult<T, E> {
    type T = T;
    type E = E;

    #[inline]
    fn certain_ok(self) -> T {
        match self {
            StdResult::Ok(t) => t,
            StdResult::Err(_) => link_err(),
        }
    }

    #[inline]
    fn certain_err(self) -> E {
        match self {
            StdResult::Ok(_) => link_err(),
            StdResult::Err(e) => e,
        }
    }
}

#[derive(Debug)]
struct Anyway<T: AsRef<str>>(pub T);
impl<T: AsRef<str>> std::fmt::Display for Anyway<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_ref())
    }
}
impl<T: AsRef<str> + std::fmt::Debug> std::error::Error for Anyway<T> {}

pub macro anyway($($arg:tt)*) {
    Traced::from(Anyway(format!($($arg)*)))
}
