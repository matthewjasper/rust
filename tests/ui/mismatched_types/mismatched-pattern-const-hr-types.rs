// Regression test for ICE from #144312

#[allow(dead_code)]
struct Inv<'a>(*mut &'a ());

type F1 = for<'a> fn(Inv<'a>);
type F2 = fn(Inv<'static>);

trait Trait {
    type Assoc: PartialEq;
}
impl Trait for F1 {
    type Assoc = i32;
}
impl Trait for F2 {
    //~^ WARN conflicting implementations of trait `Trait` for type `for<'a> fn(Inv<'a>)` [coherence_leak_check]
    //~| WARN the behavior may change in a future release
    type Assoc = i64;
}

#[derive(PartialEq)]
struct InvTy<T: Trait>(<T as Trait>::Assoc);

const A: InvTy<F1> = InvTy(1i32);
const B: InvTy<F2> = InvTy(1i64);
const N: i64 = 1;

pub fn main() {
    if let A = B { //~ ERROR mismatched types
    } else {
        panic!();
    };
    if let B = A { //~ ERROR mismatched types
    } else {
        panic!();
    };
}
