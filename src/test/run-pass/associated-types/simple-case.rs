// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

struct A;

// trait B {
//     type C;
// }
// impl B for A {
//     type C = i32;
// }

// FIXME: Improve the error messages when the type can't be infered.
// FIXME: Error if this type isn't implemented.
trait B<T> {
    type C;
}
// FIXME? This shouldn't error?
impl B<i32> for A {
    type C = i32;
}

// trait E {
//     type C;
// }
// impl E for A {
//     type C = i32;
// }



// fn id(x: i32) -> A::C {
//     x
// }

fn local() -> i32 {
    let x: A::C = 4i32;
    x
}

fn main() {
    println!("{}", local());
    // println!("{}", id(8));
}
