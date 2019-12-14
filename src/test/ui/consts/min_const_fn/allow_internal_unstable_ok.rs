// check-pass

#![feature(allow_internal_unstable)]

#[allow_internal_unstable(const_if_match)]
macro_rules! make_fn {
    () => {
        #[allow_internal_unstable(const_if_match)]
        const fn get_number(x: bool) -> usize {
            if x {
                43
            } else {
                2765
            }
        }
    }
}

make_fn!();

fn main() {}
