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
    const C: usize;
}

trait E<T> {
    const D: usize;
}

// Should be E0118, not E0391
// impl [(); A::C] {}

impl B for A {
    const C: usize = 4;
}

impl B for () {
    const C: usize = A::C;
}

impl G<[u16; A::C]> {
    fn foo() {}
}

impl E<[i16; A::C]> for i8 {
    const D: usize = <()>::C;
}


struct X([String; A::C]);

enum Y {
    V([i64; A::C]),
}

fn f(x: X) -> [String; 4] {
    x.0
}

fn g(y: Y) -> [i64; 4] {
    match y {
        Y::V(v) => v,
    }
}


