// Check that allow_internal_unstable is not sufficient to allow features in
// min const fns.

#![feature(allow_internal_unstable)]

#[allow_internal_unstable(const_if_match)]
macro_rules! make_fn {
    () => {
        const fn get_number(x: bool) -> usize {
            if x { //~ ERROR loops and conditional expressions are not stable in const fn
                43
            } else {
                2765
            }
        }
    }
}

make_fn!();

fn main() {}
