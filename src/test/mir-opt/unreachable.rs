enum Empty {}

fn empty() -> Option<Empty> {
        None
}

fn main() {
    if let Some(_x) = empty() {
        let mut _y;

        if true {
            _y = 21;
        } else {
            _y = 42;
        }

        match _x { }
    }
}

// END RUST SOURCE
// START rustc.main.UnreachablePropagation.before.mir
//      bb5: {
//          ...
//          switchInt(_6) -> [false: bb7, otherwise: bb6];
//      }
//      bb6: {
//          ...
//      }
//      bb7: {
//          ...
//      }
//      bb8: {
//          ...
//      }
//      bb9: {
//          ...
//      }
//  }
// END rustc.main.UnreachablePropagation.before.mir
// START rustc.main.UnreachablePropagation.after.mir
//      bb5: {
//          ...
//          unreachable;
//      }
//  }
// END rustc.main.UnreachablePropagation.after.mir
