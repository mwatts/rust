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
//      bb7: {
//          ...
//          goto -> bb9;
//      }
//      bb8: {
//          ...
//          goto -> bb9;
//      }
//      bb9: {
//          ...
//          unreachable;
//      }
//  }
// END rustc.main.UnreachablePropagation.before.mir
// START rustc.main.UnreachablePropagation.after.mir
//      bb7: {
//          ...
//          unreachable;
//      }
//      bb8: {
//          ...
//          unreachable;
//      }
//  }
// END rustc.main.UnreachablePropagation.after.mir
