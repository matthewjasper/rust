trait X<'a, T>
where
    for<'b> (T,): X<'b, T>,
    for<'b> <(T,) as X<'b, T>>::U: Clone,
{
    type U: ?Sized;
    fn f(x: &<(T,) as X<'_, T>>::U) {
        <<(T,) as X<'_, T>>::U>::clone(x);
    }
}

impl<S, T> X<'_, T> for (S,) {
    type U = str;
    //~^ ERROR overflow evaluating the requirement `T: std::marker::Sized`
    //~| ERROR the trait bound `str: std::clone::Clone` is not satisfied
    //~| ERROR overflow evaluating the requirement `(T,): X<\'_, T>`
}

pub fn main() {
    <(i32,) as X<i32>>::f("abc");
}
