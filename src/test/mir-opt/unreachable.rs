use std::mem::transmute;

enum Empty {}

fn main() {
    unsafe {
        let _x: Empty = transmute(());
        let mut _y;

        if true {
            _y = 21;
        } else {
            _y = 42;
        }

        match _x {}
    }
}

// END RUST SOURCE
// START rustc.main.UnreachablePropagation.before.mir
//  bb2: {
//      ...
//      switchInt(_5) -> [false: bb4, otherwise: bb3];
//  }
// END rustc.main.UnreachablePropagation.before.mir
// START rustc.main.UnreachablePropagation.after.mir
//  bb2: {
//      ...
//      unreachable;
//  }
// END rustc.main.UnreachablePropagation.after.mir
