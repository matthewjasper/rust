// compile-flags: -Z borrowck=mir

fn guard() -> bool {
    false
}

fn guard2(_:i32) -> bool {
    true
}

// no_mangle to make sure this gets instantiated even in an executable.
#[no_mangle]
pub fn full_tested_match() {
    let _ = match Some(42) {
        Some(x) if guard() => (1, x),
        Some(y) => (2, y),
        None => (3, 3),
    };
}

// no_mangle to make sure this gets instantiated even in an executable.
#[no_mangle]
pub fn full_tested_match2() {
    let _ = match Some(42) {
        Some(x) if guard() => (1, x),
        None => (3, 3),
        Some(y) => (2, y),
    };
}

fn main() {
    let _ = match Some(1) {
        Some(_w) if guard() => 1,
        _x => 2,
        Some(y) if guard2(y) => 3,
        _z => 4,
    };
}

// END RUST SOURCE
//
// START rustc.full_tested_match.QualifyAndPromoteConstants.after.mir
//  bb0: {
//      ...
//      _2 = std::option::Option<i32>::Some(const 42i32,);
//      FakeRead(ForMatchedPlace, _2);
//      _6 = discriminant(_2);
//      _8 = &guard (promoted[2]: std::option::Option<i32>);
//      FakeRead(ForFakeBorrowPlace, _2);
//      switchInt(move _6) -> [0isize: bb5, 1isize: bb3, otherwise: bb9];
//  }
//  bb1: {
//      resume;
//  }
//  bb2: {
//      _1 = (const 3i32, const 3i32);
//      goto -> bb13;
//  }
//  bb3: {
//      FakeRead(ForMatchGuard, _8);
//      falseEdges -> [real: bb7, imaginary: bb4];
//  }
//  bb4: {
//      FakeRead(ForMatchGuard, _8);
//      falseEdges -> [real: bb8, imaginary: bb5];
//  }
//  bb5: {
//      FakeRead(ForMatchGuard, _8);
//      falseEdges -> [real: bb2, imaginary: bb6];
//  }
//  bb6: {
//      unreachable;
//  }
//  bb7: {
//      StorageLive(_4);
//      _4 = &(((promoted[1]: std::option::Option<i32>) as Some).0: i32);
//      _8 = &guard (promoted[0]: std::option::Option<i32>);
//      StorageLive(_7);
//      _7 = const guard() -> [return: bb10, unwind: bb1];
//  }
//  bb8: {
//      StorageLive(_5);
//      _5 = ((_2 as Some).0: i32);
//      StorageLive(_10);
//      _10 = _5;
//      _1 = (const 2i32, move _10);
//      StorageDead(_10);
//      goto -> bb13;
//  }
//  bb9: {
//      unreachable;
//  }
//  bb10: {
//      switchInt(move _7) -> [false: bb11, otherwise: bb12];
//  }
//  bb11: {
//      falseEdges -> [real: bb4, imaginary: bb4];
//  }
//  bb12: {
//      StorageLive(_3);
//      _3 = ((_2 as Some).0: i32);
//      StorageLive(_9);
//      _9 = _3;
//      _1 = (const 1i32, move _9);
//      StorageDead(_9);
//      goto -> bb13;
//  }
//  bb13: {
//      ...
//      return;
//  }
// END rustc.full_tested_match.QualifyAndPromoteConstants.after.mir
//
// START rustc.full_tested_match2.QualifyAndPromoteConstants.before.mir
//  bb0: {
//      ...
//      _2 = std::option::Option<i32>::Some(const 42i32,);
//      FakeRead(ForMatchedPlace, _2);
//      _6 = discriminant(_2);
//      _8 = &guard _2;
//      FakeRead(ForFakeBorrowPlace, _2);
//      switchInt(move _6) -> [0isize: bb4, 1isize: bb3, otherwise: bb9];
//  }
//  bb1: {
//      resume;
//  }
//  bb2: {
//      _1 = (const 3i32, const 3i32);
//      goto -> bb13;
//  }
//  bb3: {
//      FakeRead(ForMatchGuard, _8);
//      falseEdges -> [real: bb7, imaginary: bb4];
//  }
//  bb4: {
//      FakeRead(ForMatchGuard, _8);
//      falseEdges -> [real: bb2, imaginary: bb5];
//  }
//  bb5: {
//      FakeRead(ForMatchGuard, _8);
//      falseEdges -> [real: bb8, imaginary: bb6];
//  }
//  bb6: {
//      unreachable;
//  }
//  bb7: {
//      StorageLive(_4);
//      _4 = &((_2 as Some).0: i32);
//      _8 = &guard _2;
//      StorageLive(_7);
//      _7 = const guard() -> [return: bb10, unwind: bb1];
//  }
//  bb8: {
//      StorageLive(_5);
//      _5 = ((_2 as Some).0: i32);
//      StorageLive(_10);
//      _10 = _5;
//      _1 = (const 2i32, move _10);
//      StorageDead(_10);
//      goto -> bb13;
//  }
//  bb9: {
//      unreachable;
//  }
//  bb10: {
//      switchInt(move _7) -> [false: bb11, otherwise: bb12];
//  }
//  bb11: {
//      falseEdges -> [real: bb5, imaginary: bb4];
//  }
//  bb12: {
//      StorageLive(_3);
//      _3 = ((_2 as Some).0: i32);
//      StorageLive(_9);
//      _9 = _3;
//      _1 = (const 1i32, move _9);
//      StorageDead(_9);
//      goto -> bb13;
//  }
//  bb13: {
//      ...
//      return;
//  }
// END rustc.full_tested_match2.QualifyAndPromoteConstants.before.mir
//
// START rustc.main.QualifyAndPromoteConstants.before.mir
// bb0: {
//     ...
//     _2 = std::option::Option<i32>::Some(const 1i32,);
//     FakeRead(ForMatchedPlace, _2);
//    _9 = discriminant(_2);
//    _14 = &guard _2;
//    FakeRead(ForFakeBorrowPlace, _2);
//    switchInt(move _9) -> [1isize: bb2, otherwise: bb3];
// }
// bb1: {
//     resume;
// }
// bb2: {
//     FakeRead(ForMatchGuard, _14);
//     falseEdges -> [real: bb7, imaginary: bb3];
// }
// bb3: {
//     FakeRead(ForMatchGuard, _14);
//     falseEdges -> [real: bb8, imaginary: bb4];
// }
// bb4: {
//     FakeRead(ForMatchGuard, _14);
//     falseEdges -> [real: bb9, imaginary: bb5];
// }
// bb5: {
//     FakeRead(ForMatchGuard, _14);
//     falseEdges -> [real: bb10, imaginary: bb6];
// }
// bb6: {
//     unreachable;
// }
// bb7: {
//     StorageLive(_4);
//     _4 = &((_2 as Some).0: i32);
//     _14 = &guard _2;
//     StorageLive(_10);
//     _10 = const guard() -> [return: bb11, unwind: bb1];
// }
// bb8: {
//     StorageLive(_5);
//     _5 = _2;
//     _1 = const 2i32;
//     goto -> bb17;
// }
// bb9: {
//     StorageLive(_7);
//     _7 = &((_2 as Some).0: i32);
//     _14 = &guard _2;
//     StorageLive(_12);
//     StorageLive(_13);
//     _13 = (*_7);
//     _12 = const guard2(move _13) -> [return: bb14, unwind: bb1];
// }
// bb10: {
//     StorageLive(_8);
//     _8 = _2;
//     _1 = const 4i32;
//     goto -> bb17;
// }
// bb11: {
//     switchInt(move _10) -> [false: bb12, otherwise: bb13];
// }
// bb12: {
//     falseEdges -> [real: bb3, imaginary: bb3];
// }
// bb13: {
//     StorageLive(_3);
//     _3 = ((_2 as Some).0: i32);
//     _1 = const 1i32;
//     goto -> bb17;
// }
// bb14: {
//     StorageDead(_13);
//     switchInt(move _12) -> [false: bb15, otherwise: bb16];
// }
// bb15: {
//     falseEdges -> [real: bb5, imaginary: bb5];
// }
// bb16: {
//     StorageLive(_6);
//     _6 = ((_2 as Some).0: i32);
//     _1 = const 3i32;
//     goto -> bb17;
// }
// bb17: {
//     ...
//     return;
// }
// END rustc.main.QualifyAndPromoteConstants.before.mir
