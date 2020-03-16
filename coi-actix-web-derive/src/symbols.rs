use std::fmt::{self, Display};
use syn::{Ident, Path};

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

pub const CRATE: Symbol = Symbol("crate");

impl PartialEq<Symbol> for Ident {
    fn eq(&self, sym: &Symbol) -> bool {
        self == sym.0
    }
}

impl<'a> PartialEq<Symbol> for &'a Ident {
    fn eq(&self, sym: &Symbol) -> bool {
        *self == sym.0
    }
}

impl PartialEq<Symbol> for Path {
    fn eq(&self, sym: &Symbol) -> bool {
        self.is_ident(sym.0)
    }
}

impl<'a> PartialEq<Symbol> for &'a Path {
    fn eq(&self, sym: &Symbol) -> bool {
        self.is_ident(sym.0)
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0)
    }
}
