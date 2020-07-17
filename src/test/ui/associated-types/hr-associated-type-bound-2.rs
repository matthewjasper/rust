trait X<'a>
where
    for<'b> <Self as X<'b>>::U: Clone,
{
    type U: ?Sized;
    fn f(&self, x: &Self::U) {
        <Self::U>::clone(x);
    }
}

impl X<'_> for u32
//~^ ERROR the trait bound `str: std::clone::Clone` is not satisfied
where
    for<'b> <Self as X<'b>>::U: Clone,
{
    type U = str;
    //~^ ERROR the trait bound `str: std::clone::Clone` is not satisfied
}

fn main() {
    1u32.f("abc");
}
