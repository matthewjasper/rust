// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::{Add, Mul, Neg};

fn sum_of_squares<A, B>(a: A, b: B)
where
    A: Mul + Copy,
    B: Mul + Copy,
    A::Output: Add<B::Output>
{
    let _: A::Output::Output = a * a + b * b;
}

fn minus_square<A>(a: A) -> A::Output::Output
where
    A: Mul + Copy,
    A::Output: Neg,
{
    -(a * a)
}

fn main() {}
