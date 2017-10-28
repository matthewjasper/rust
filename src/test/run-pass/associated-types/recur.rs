// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(associated_type_defaults)]
pub trait IntoIterator2 {
    type Item = Self::Item;

    type IntoIter: Iterator<Item=Self::Item>;

    fn into_iter(self) -> <Self>::IntoIter;
}

impl IntoIterator2 for Vec<i32> {
    type Item = i32;
    type IntoIter = <Vec<i32> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(self)
    }
}

fn main() {}
