// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

fn main() {}

struct A;
struct G<T>(T);

trait B {
    type C;
}

trait E<T> {
    type D;
}

// Should be E0118, not E0391
// impl A::C {}

impl B for A {
    type C = i32;
}

impl B for () {
    type C = A::C;
}

// impl G<<A as B>::C> {
//     fn foo() {}
// }

// impl E<A::C> for i8 {
//     type D = <()>::C;
// }


struct X(A::C);

enum Y {
    V(A::C),
}

fn f(x: X) -> i32 {
    x.0
}

fn g(y: Y) -> i32 {
    match y {
        Y::V(v) => v,
    }
}


