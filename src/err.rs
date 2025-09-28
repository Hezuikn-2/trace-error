use alloc::{
    boxed::{Box, ThinBox},
    vec,
    vec::Vec,
};
use core::error::Error;
use core::fmt::{self, Debug, Display};
use core::panic::Location;

pub trait IF: Debug + Display + Error + 'static {}
impl<T: Debug + Display + Error + 'static> IF for T {}

pub struct Err {
    pub loc: Vec<Location<'static>>,
    pub org: ThinBox<(dyn IF)>,
}
impl Debug for Err {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}
impl Display for Err {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}\n", &*self.org))?;

        for x in self.loc.iter().rev() {
            f.write_fmt(format_args!("{}:{}:{}\n", x.file(), x.line(), x.column()))?;
        }

        return fmt::Result::Ok(());
    }
}
impl Error for Err {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.org)
    }
}

impl From<Err> for Err {
    #[track_caller]
    fn from(x: Err) -> Err {
        let mut v = x.loc;
        v.push(*Location::caller());
        Err { loc: v, org: x.org }
    }
}

auto trait NegErr {}
impl<T: ?Sized> NegErr for Box<T> {}
impl !NegErr for Err {}

impl<T: NegErr + IF + 'static> From<T> for Err {
    #[track_caller]
    fn from(x: T) -> Err {
        Err {
            loc: vec![*Location::caller()],
            org: ThinBox::new_unsize(x),
        }
    }
}
