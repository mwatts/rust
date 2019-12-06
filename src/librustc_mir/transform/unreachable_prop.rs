//! A pass that propagates the unreachable terminator of a block to its predecessors
//! when all of their successors are unreachable. This is achieved through a
//! post-order traversal of the blocks.

use rustc::ty::TyCtxt;
use rustc::mir::*;
use rustc_data_structures::fx::{FxHashSet, FxHashMap};
use std::borrow::Cow;
use crate::transform::{MirPass, MirSource};
use crate::transform::simplify;

pub struct UnreachablePropagation;

impl MirPass<'_> for UnreachablePropagation {
    fn run_pass<'tcx>(&self, _tcx: TyCtxt<'tcx>, _: MirSource<'tcx>, body: &mut BodyCache<'tcx>) {
        let mut unreachable_blocks = FxHashSet::default();
        let mut replacements = FxHashMap::default();

        for (bb, bb_data) in traversal::postorder(body) {
            let terminator = bb_data.terminator();

            if let TerminatorKind::Unreachable = terminator.kind {
                unreachable_blocks.insert(bb);
            } else {
                let is_unreachable_pred = |succ: &'_ BasicBlock| unreachable_blocks.contains(succ);
                let mut mut_terminator = terminator.clone();

                if remove_successors(&mut mut_terminator, is_unreachable_pred) {
                    replacements.insert(bb, mut_terminator.kind.clone());
                    if let TerminatorKind::Unreachable = mut_terminator.kind {
                        unreachable_blocks.insert(bb);
                    }
                }
            }
        }

        let replaced = !replacements.is_empty();
        for (bb, terminator_kind) in replacements {
            body.basic_blocks_mut()[bb].terminator_mut().kind = terminator_kind;
        }

        if replaced {
            simplify::remove_dead_blocks(body);
        }
    }
}

fn remove_successors<F>(terminator: &mut Terminator<'_>, pred: F) -> bool
    where F: Fn(&BasicBlock) -> bool
{
    let mut removed_successor = false;

    match &terminator.kind {
        TerminatorKind::Goto { target } => {
            if pred(target) {
                terminator.kind = TerminatorKind::Unreachable;
                removed_successor = true;
            }
        },
        TerminatorKind::SwitchInt { discr, switch_ty, values, targets } => {
            let original_targets_len = targets.len();
            let (otherwise, targets) = targets.split_last().unwrap();
            let retained = values.iter().zip(targets.iter()).filter(|(_, t)| !pred(t)).
                collect::<Vec<_>>();
            let mut values = retained.iter().map(|&(v, _)| *v).collect::<Vec<_>>();
            let mut targets = retained.iter().map(|&(_, d)| *d).collect::<Vec<_>>();

            if !pred(otherwise) {
                targets.push(*otherwise);
            }

            let retained_targets_len = targets.len();

            if targets.is_empty() {
                terminator.kind = TerminatorKind::Unreachable;
            } else if targets.len() == 1 {
                terminator.kind = TerminatorKind::Goto { target: targets[0] }
            } else {
                if values.len() == retained_targets_len {
                    values.truncate(retained_targets_len - 1);
                }

                terminator.kind = TerminatorKind::SwitchInt {
                    discr: discr.clone(),
                    switch_ty,
                    values: Cow::from(values),
                    targets
                }
            }

            removed_successor = original_targets_len != retained_targets_len;
        },
        _ => {}
    }

    removed_successor
}
