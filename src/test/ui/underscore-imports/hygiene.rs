// Make sure that if an underscore imports have the same hygiene considerations
// as other imports.

#![feature(decl_macro)]

mod x {
    pub use std::ops::Deref as _;
}

macro m($y:ident) {
    mod $y {
        crate::n!();
    }
}

macro n() {
    pub use crate::x::*;
}

macro p() {
    use std::ops::DerefMut as _;
}

m!(y);

fn main() {
    use crate::y::*;
    p!();
    (&()).deref();              //~ ERROR no method named `deref`
    (&mut ()).deref_mut();      //~ ERROR no method named `deref_mut`
}
