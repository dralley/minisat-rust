use std::{fmt, ops};
pub use self::index_map::*;

pub mod assignment;
pub mod clause;
mod index_map;
pub mod util;


#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub struct Var(usize);

impl Var {
    #[inline]
    pub fn lit(&self, sign: bool) -> Lit {
        Lit((self.0 << 1) | (sign as usize))
    }

    #[inline]
    pub fn pos_lit(&self) -> Lit {
        Lit(self.0 << 1)
    }

    #[inline]
    pub fn neg_lit(&self) -> Lit {
        Lit((self.0 << 1) | 1)
    }
}

impl fmt::Debug for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "x{}", self.0)
    }
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub struct Lit(usize);

impl Lit {
    #[inline]
    pub fn sign(&self) -> bool {
        (self.0 & 1) != 0
    }

    #[inline]
    pub fn var(&self) -> Var {
        Var(self.0 >> 1)
    }

    #[inline]
    pub fn abstraction(&self) -> u32 {
        1 << ((self.0 >> 1) & 31)
    }
}

impl ops::Not for Lit {
    type Output = Lit;

    #[inline]
    fn not(self) -> Lit {
        Lit(self.0 ^ 1)
    }
}

impl fmt::Debug for Lit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.sign() {
            write!(f, "¬")?;
        }
        write!(f, "{:?}", self.var())
    }
}
