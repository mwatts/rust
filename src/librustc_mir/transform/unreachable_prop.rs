//! A pass that propagates the unreachable terminator of a block to its predecessors
//! when all of their successors are unreachable. This is achieved through a
//! post-order traversal of the blocks.

use rustc::ty::TyCtxt;
use rustc::mir::*;
use rustc_data_structures::fx::FxHashSet;
use crate::transform::{MirPass, MirSource};

pub struct UnreachablePropagation;

impl MirPass<'_> for UnreachablePropagation {
    fn run_pass<'tcx>(&self, _tcx: TyCtxt<'tcx>, _: MirSource<'tcx>, body: &mut Body<'tcx>) {
        let mut unreachable_blocks = FxHashSet::default();

        for (bb, bb_data) in traversal::postorder(body) {
            if let Some(terminator) = &bb_data.terminator {
                if let TerminatorKind::Unreachable = terminator.kind {
                    unreachable_blocks.insert(bb.clone());
                } else {
                    let mut bb_successors = terminator.successors().peekable();

                    if bb_successors.peek().is_some() {
                        if bb_successors.all(|bb| unreachable_blocks.contains(bb)) {
                            unreachable_blocks.insert(bb.clone());
                        }
                    }
                }
            }
        }

        for block in unreachable_blocks {
            body.basic_blocks_mut()[block]
                .terminator.as_mut().unwrap().kind = TerminatorKind::Unreachable;
        }
    }
}