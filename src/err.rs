use alloc::{
    boxed::{Box, ThinBox},
    vec,
    vec::Vec,
};
use core::{convert::identity, error::Error, intrinsics::const_eval_select};
use core::fmt::{self, Debug, Display};
use core::panic::Location;

pub trait IF: Debug + Display + Error + 'static {}
impl<T: Debug + Display + Error + 'static> IF for T {}

pub struct Traced {
    pub loc: Vec<Location<'static>>,
    pub org: ThinBox<dyn IF>,
}
impl Debug for Traced {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}
impl Display for Traced {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}\n", &*self.org))?;

        for x in self.loc.iter().rev() {
            f.write_fmt(format_args!("{}:{}:{}\n", x.file(), x.line(), x.column()))?;
        }

        return fmt::Result::Ok(());
    }
}
impl Error for Traced {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.org)
    }
}

impl const From<Traced> for Traced {
    #[track_caller]
    fn from(x: Self) -> Self {
        #[track_caller]
        fn rt(mut v: Vec<Location<'_>>) -> Vec<Location<'_>> {
            v.push(*Location::caller());
            v
        }
        let v = const_eval_select((x.loc,), identity, rt);
        Self { loc: v, org: x.org }
    }
}

auto trait NegErr {}
impl<T: ?Sized> NegErr for Box<T> {}
impl !NegErr for Traced {}

impl<T: NegErr + IF + 'static> From<T> for Traced {
    #[track_caller]
    fn from(x: T) -> Self {
        Self {
            loc: vec![*Location::caller()],
            org: ThinBox::new_unsize(x),
        }
    }
}
