use core::ops::{ControlFlow, Deref, DerefMut, FromResidual, Try};
use core::result::Result as StdResult;
use core::{convert::Infallible, fmt::Debug, hint};

impl<T, E> From<StdResult<T, E>> for Maybe<T, E> {
    fn from(result: StdResult<T, E>) -> Self {
        match result {
            StdResult::Ok(x) => O(x),
            StdResult::Err(x) => X(x),
        }
    }
}

// Implement `Try` trait v2 for `?` operator support
impl<T, E> Try for Maybe<T, E> {
    type Output = T; // Success type
    type Residual = Maybe<Infallible, E>; // Residual type for errors

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        O(output)
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            O(val) => ControlFlow::Continue(val),
            X(e) => ControlFlow::Break(X(e)),
        }
    }
}

impl<T, E, F: From<E>> FromResidual<Maybe<Infallible, E>> for Maybe<T, F> {
    #[inline]
    #[track_caller]
    fn from_residual(residual: Maybe<Infallible, E>) -> Self {
        match residual {
            X(e) => X(From::from(e)),
        }
    }
}

impl<T, E, F: From<E>> FromResidual<StdResult<Infallible, E>> for Maybe<T, F> {
    #[inline]
    #[track_caller]
    fn from_residual(residual: StdResult<Infallible, E>) -> Self {
        match residual {
            StdResult::Err(e) => X(From::from(e)),
        }
    }
}

impl<T, F: From<OptNone>> FromResidual<Option<Infallible>> for Maybe<T, F> {
    #[inline]
    #[track_caller]
    fn from_residual(residual: Option<Infallible>) -> Self {
        match residual {
            Option::None => X(From::from(OptNone)),
        }
    }
}

pub enum Maybe<T, E> {
    O(T),
    X(E),
}

use Maybe::*;

use crate::{OptNone, unwrap_failed};

impl<T, E> Maybe<T, E> {
    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, O(_))
    }

    #[inline]
    pub fn is_ok_and(self, f: impl FnOnce(T) -> bool) -> bool {
        match self {
            X(_) => false,
            O(x) => f(x),
        }
    }

    #[inline]
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    #[inline]
    pub fn is_err_and(self, f: impl FnOnce(E) -> bool) -> bool {
        match self {
            O(_) => false,
            X(e) => f(e),
        }
    }

    #[inline]
    pub fn ok(self) -> Option<T> {
        match self {
            O(x) => Some(x),
            X(_) => None,
        }
    }

    #[inline]
    pub fn err(self) -> Option<E> {
        match self {
            O(_) => None,
            X(x) => Some(x),
        }
    }

    #[inline]
    pub const fn as_ref(&self) -> Maybe<&T, &E> {
        match *self {
            O(ref x) => O(x),
            X(ref x) => X(x),
        }
    }

    #[inline]
    pub const fn as_mut(&mut self) -> Maybe<&mut T, &mut E> {
        match *self {
            O(ref mut x) => O(x),
            X(ref mut x) => X(x),
        }
    }

    #[inline]
    pub fn map<U, F: FnOnce(T) -> U>(self, op: F) -> Maybe<U, E> {
        match self {
            O(t) => O(op(t)),
            X(e) => X(e),
        }
    }

    #[inline]
    pub fn map_or<U, F: FnOnce(T) -> U>(self, default: U, f: F) -> U {
        match self {
            O(t) => f(t),
            X(_) => default,
        }
    }

    #[inline]
    pub fn map_or_else<U, D: FnOnce(E) -> U, F: FnOnce(T) -> U>(self, default: D, f: F) -> U {
        match self {
            O(t) => f(t),
            X(e) => default(e),
        }
    }

    #[inline]
    pub fn map_or_default<U, F>(self, f: F) -> U
    where
        U: Default,
        F: FnOnce(T) -> U,
    {
        match self {
            O(t) => f(t),
            X(_) => U::default(),
        }
    }

    #[inline]
    pub fn map_err<F, O: FnOnce(E) -> F>(self, op: O) -> Maybe<T, F> {
        match self {
            O(t) => O(t),
            X(e) => X(op(e)),
        }
    }

    #[inline]
    pub fn inspect<F: FnOnce(&T)>(self, f: F) -> Self {
        if let O(ref t) = self {
            f(t);
        }

        self
    }

    #[inline]
    pub fn inspect_err<F: FnOnce(&E)>(self, f: F) -> Self {
        if let X(ref e) = self {
            f(e);
        }

        self
    }

    #[inline]
    pub fn as_deref(&self) -> Maybe<&T::Target, &E>
    where
        T: Deref,
    {
        self.as_ref().map(|t| t.deref())
    }

    #[inline]
    pub fn as_deref_mut(&mut self) -> Maybe<&mut T::Target, &mut E>
    where
        T: DerefMut,
    {
        self.as_mut().map(|t| t.deref_mut())
    }

    /*#[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.as_ref().ok(),
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            inner: self.as_mut().ok(),
        }
    }*/

    #[track_caller]
    pub fn expect(self, msg: &str) -> T
    where
        E: Debug,
    {
        match self {
            O(t) => t,
            X(e) => unwrap_failed(msg, &e),
        }
    }

    #[track_caller]
    pub fn unwrap(self) -> T
    where
        E: Debug,
    {
        match self {
            O(t) => t,
            X(e) => unwrap_failed("called `Result::unwrap()` on an `Err` value", &e),
        }
    }

    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            O(x) => x,
            X(_) => Default::default(),
        }
    }

    #[inline]
    #[track_caller]
    pub fn expect_err(self, msg: &str) -> E
    where
        T: Debug,
    {
        match self {
            O(t) => unwrap_failed(msg, &t),
            X(e) => e,
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_err(self) -> E
    where
        T: Debug,
    {
        match self {
            O(t) => unwrap_failed("called `Result::unwrap_err()` on an `Ok` value", &t),
            X(e) => e,
        }
    }

    #[inline]
    pub fn into_ok(self) -> T
    where
        E: Into<!>,
    {
        match self {
            O(x) => x,
            X(e) => e.into(),
        }
    }

    #[inline]
    pub fn into_err(self) -> E
    where
        T: Into<!>,
    {
        match self {
            O(x) => x.into(),
            X(e) => e,
        }
    }

    #[inline]
    pub fn and<U>(self, res: Maybe<U, E>) -> Maybe<U, E> {
        match self {
            O(_) => res,
            X(e) => X(e),
        }
    }

    #[inline]
    pub fn and_then<U, F: FnOnce(T) -> Maybe<U, E>>(self, op: F) -> Maybe<U, E> {
        match self {
            O(t) => op(t),
            X(e) => X(e),
        }
    }

    #[inline]
    pub fn or<F>(self, res: Maybe<T, F>) -> Maybe<T, F> {
        match self {
            O(v) => O(v),
            X(_) => res,
        }
    }

    #[inline]
    pub fn or_else<F, O: FnOnce(E) -> Maybe<T, F>>(self, op: O) -> Maybe<T, F> {
        match self {
            O(t) => O(t),
            X(e) => op(e),
        }
    }

    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            O(t) => t,
            X(_) => default,
        }
    }

    #[inline]
    pub fn unwrap_or_else<F: FnOnce(E) -> T>(self, op: F) -> T {
        match self {
            O(t) => t,
            X(e) => op(e),
        }
    }

    /// # Safety
    ///
    /// Calling this method on an [`Err`] is *[undefined behavior]*.
    #[inline]
    #[track_caller]
    pub unsafe fn unwrap_unchecked(self) -> T {
        match self {
            O(t) => t,
            // SAFETY: the safety contract must be upheld by the caller.
            X(_) => unsafe { hint::unreachable_unchecked() },
        }
    }

    /// # Safety
    ///
    /// Calling this method on an [`Ok`] is *[undefined behavior]*.
    #[inline]
    #[track_caller]
    pub unsafe fn unwrap_err_unchecked(self) -> E {
        match self {
            // SAFETY: the safety contract must be upheld by the caller.
            O(_) => unsafe { hint::unreachable_unchecked() },
            X(e) => e,
        }
    }
}

impl<T, E> Maybe<&T, E> {
    #[inline]
    pub const fn copied(self) -> Maybe<T, E>
    where
        T: Copy,
    {
        // FIXME(const-hack): this implementation, which sidesteps using `Result::map` since it's not const
        // ready yet, should be reverted when possible to avoid code repetition
        match self {
            O(&v) => O(v),
            X(e) => X(e),
        }
    }

    #[inline]
    pub fn cloned(self) -> Maybe<T, E>
    where
        T: Clone,
    {
        self.map(|t| t.clone())
    }
}
