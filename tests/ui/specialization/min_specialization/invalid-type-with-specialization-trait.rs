//! Regression test for https://github.com/rust-lang/rust/issues/102252
//! The invalid circular where clause shouldn't cause an ICE.

#![feature(min_specialization, rustc_attrs)]

#[rustc_specialization_trait]
pub trait Trait {}

struct Struct
where
    Self: Iterator<Item = <Self as Iterator>::Item>, {}
//~^^^ ERROR overflow evaluating the requirement

impl Trait for Struct {}

fn main() {}
