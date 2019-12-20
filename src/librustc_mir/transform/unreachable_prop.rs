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
        let is_asm_stmt = |stmt: &Statement<'_>| match stmt.kind {
            StatementKind::InlineAsm(..) => true,
            _ => false,
        };

        for (bb, bb_data) in traversal::postorder(body) {
            let terminator = bb_data.terminator();

            if let TerminatorKind::Unreachable = terminator.kind {
                unreachable_blocks.insert(bb);
            } else {
                // HACK: If the block contains any asm statement the optimization is not attempted.
                // This is a temporal solution that handles possibly diverging asm statements.
                if bb_data.statements.iter().any(|stmt| is_asm_stmt(stmt)) {
                    continue;
                }

                let is_unreachable = |succ: BasicBlock| unreachable_blocks.contains(&succ);
                let terminator_kind_opt = remove_successors(&terminator.kind, is_unreachable);

                if let Some(terminator_kind) = terminator_kind_opt {
                    if let TerminatorKind::Unreachable = terminator_kind {
                        unreachable_blocks.insert(bb);
                    }
                    replacements.insert(bb, terminator_kind);
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

fn remove_successors<F>(
    terminator_kind: &TerminatorKind<'tcx>,
    predicate: F
) -> Option<TerminatorKind<'tcx>>
    where F: Fn(BasicBlock) -> bool
{
    match *terminator_kind {
        TerminatorKind::Goto { target } if predicate(target) =>{
            Some(TerminatorKind::Unreachable)
        },
        TerminatorKind::SwitchInt { ref discr, switch_ty, ref values, ref targets } => {
            let original_targets_len = targets.len();
            let (otherwise, targets) = targets.split_last().unwrap();
            let retained = values.iter().zip(targets.iter()).filter(|(_, &t)| !predicate(t)).
                collect::<Vec<_>>();
            let mut values = retained.iter().map(|&(v, _)| *v).collect::<Vec<_>>();
            let mut targets = retained.iter().map(|&(_, d)| *d).collect::<Vec<_>>();

            if !predicate(*otherwise) {
                targets.push(*otherwise);
            } else {
                values.truncate(values.len() - 1);
            }

            let retained_targets_len = targets.len();

            if targets.is_empty() {
                Some(TerminatorKind::Unreachable)
            } else if targets.len() == 1 {
                Some(TerminatorKind::Goto { target: targets[0] })
            } else if original_targets_len != retained_targets_len {
                Some(TerminatorKind::SwitchInt {
                    discr: discr.clone(),
                    switch_ty,
                    values: Cow::from(values),
                    targets
                })
            } else {
                None
            }

        },
        _ => None
    }
}
