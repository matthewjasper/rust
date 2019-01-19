// Test that the fake borrows for matches are removed after borrow checking.

// ignore-wasm32-bare

#![feature(nll)]

fn match_guard(x: Option<&&i32>) -> i32 {
    match x {
        Some(0) if true => 0,
        _ => 1,
    }
}

fn main() {
    match_guard(None);
}

// END RUST SOURCE

// START rustc.match_guard.CleanupNonCodegenStatements.before.mir
// bb0: {
//     FakeRead(ForMatchedPlace, _1);
//     _2 = discriminant(_1);
//     _3 = &guard _1;
//      FakeRead(ForFakeBorrowPlace, _1);
//     _4 = &guard ((_1 as Some).0: &'<empty> &'<empty> i32);
//      FakeRead(ForFakeBorrowPlace, ((_1 as Some).0: &'<empty> &'<empty> i32));
//     _5 = &guard (*((_1 as Some).0: &'<empty> &'<empty> i32));
//      FakeRead(ForFakeBorrowPlace, (*((_1 as Some).0: &'<empty> &'<empty> i32)));
//     _6 = &guard (*(*((_1 as Some).0: &'<empty> &'<empty> i32)));
//      FakeRead(ForFakeBorrowPlace, (*(*((_1 as Some).0: &'<empty> &'<empty> i32))));
//     switchInt(move _2) -> [1isize: bb6, otherwise: bb3];
// }
// bb1: {
//     _0 = const 1i32;
//     goto -> bb9;
// }
// bb2: {
//     FakeRead(ForMatchGuard, _3);
//     FakeRead(ForMatchGuard, _4);
//     FakeRead(ForMatchGuard, _5);
//     FakeRead(ForMatchGuard, _6);
//     goto -> bb5;
// }
// bb3: {
//     FakeRead(ForMatchGuard, _3);
//     FakeRead(ForMatchGuard, _4);
//     FakeRead(ForMatchGuard, _5);
//     FakeRead(ForMatchGuard, _6);
//     goto -> bb1;
// }
// bb4: {
//     unreachable;
// }
// bb5: {
//     _3 = &guard _1;
//     _4 = &guard ((_1 as Some).0: &'<empty> &'<empty> i32);
//     _5 = &guard (*((_1 as Some).0: &'<empty> &'<empty> i32));
//     _6 = &guard (*(*((_1 as Some).0: &'<empty> &'<empty> i32)));
//     goto -> bb8;
// }
// bb6: {
//     switchInt((*(*((_1 as Some).0: &'<empty> &'<empty> i32)))) -> [0i32: bb2, otherwise: bb3];
// }
// bb7: {
//     goto -> bb3;
// }
// bb8: {
//     _0 = const 0i32;
//     goto -> bb9;
// }
// bb9: {
//     return;
// }
// bb10: {
//     resume;
// }
// END rustc.match_guard.CleanupNonCodegenStatements.before.mir

// START rustc.match_guard.CleanupNonCodegenStatements.after.mir
// bb0: {
//     nop;
//     _2 = discriminant(_1);
//     nop;
//     nop;
//     nop;
//     nop;
//     nop;
//     nop;
//     nop;
//     nop;
//     switchInt(move _2) -> [1isize: bb6, otherwise: bb3];
// }
// bb1: {
//     _0 = const 1i32;
//     goto -> bb9;
// }
// bb2: {
//     nop;
//     nop;
//     nop;
//     nop;
//     goto -> bb5;
// }
// bb3: {
//     nop;
//     nop;
//     nop;
//     nop;
//     goto -> bb1;
// }
// bb4: {
//     unreachable;
// }
// bb5: {
//     nop;
//     nop;
//     nop;
//     nop;
//     goto -> bb8;
// }
// bb6: {
//     switchInt((*(*((_1 as Some).0: &'<empty> &'<empty> i32)))) -> [0i32: bb2, otherwise: bb3];
// }
// bb7: {
//     goto -> bb3;
// }
// bb8: {
//     _0 = const 0i32;
//     goto -> bb9;
// }
// bb9: {
//     return;
// }
// bb10: {
//     resume;
// }
// END rustc.match_guard.CleanupNonCodegenStatements.after.mir
