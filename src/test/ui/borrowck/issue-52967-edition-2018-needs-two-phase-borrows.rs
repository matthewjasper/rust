// This is a regression test for #52967, where we discovered that in
// the initial deployment of NLL for the 2018 edition, I forgot to
// turn on two-phase-borrows in addition to `-Z borrowck=migrate`.
// The original version of this test used matches, which no longer
// use 2 phase-borrows in MIR borrowck.

// revisions: ast zflags edition
//[zflags]compile-flags: -Z borrowck=migrate -Z two-phase-borrows
//[edition]edition:2018

// run-pass

fn f(_: &mut &i32, _: &i32) {}

fn the_bug(y: i32) {
    let x = &mut &y;
    f(x, *x);
}

fn main() {
    the_bug(2);
}
