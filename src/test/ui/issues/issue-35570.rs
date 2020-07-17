use std::mem;

trait Trait1<T> {}
trait Trait2<'a> {
    type Ty;
}

fn _ice(param: Box<dyn for<'a> Trait1<<() as Trait2<'a>>::Ty>>) {
    let _e: (usize, usize) = unsafe { mem::transmute(param) };
    //~^ ERROR the trait bound `(): Trait2<'_>` is not satisfied
}

trait Lifetime<'a> {
    type Out;
}
impl<'a> Lifetime<'a> for () {
    type Out = &'a ();
}
fn foo<'a>(x: &'a ()) -> <() as Lifetime<'a>>::Out {
    x
}

fn takes_lifetime(_f: for<'a> fn(&'a ()) -> <() as Lifetime<'a>>::Out) {}

fn main() {
    takes_lifetime(foo);
}
