//! Zero-allocation RPN evaluator for CSA conformance expressions and choice-group constraints.

use super::{
    ValidationMode,
    descriptors::{ChoiceGroup, ConformanceOp, ConformanceResult},
    value::extract_signed,
};
use crate::model::{AttributeId, Cluster};

/// Maximum RPN evaluation stack depth.
const MAX_RPN_STACK: usize = 32;

/// Tagged value stored on the fixed-size RPN evaluation stack.
#[derive(Debug, Clone, Copy)]
enum StackVal {
    Bool(bool),
    Num(Option<i64>),
    Conf(Option<ConformanceResult>),
}

/// Evaluates a [`ConformanceOp`] RPN program using a fixed-size stack.
#[must_use]
pub(crate) fn eval_conformance(
    ops: &[ConformanceOp],
    feature_map: u32,
    cluster: Option<&Cluster>,
) -> ConformanceResult {
    if ops.is_empty() {
        return ConformanceResult::Optional;
    }

    let mut stack = [StackVal::Bool(false); MAX_RPN_STACK];
    let mut sp: usize = 0;

    for &op in ops {
        match op {
            ConformanceOp::PushFeature(mask) => {
                push_stack(&mut stack, &mut sp, StackVal::Bool((feature_map & mask) != 0));
            }
            ConformanceOp::PushAttrPresent(id) => {
                let present = cluster.is_some_and(|c| c.attribute(AttributeId::new(id)).is_some());
                push_stack(&mut stack, &mut sp, StackVal::Bool(present));
            }
            ConformanceOp::PushAttrValue(id) => {
                let num = cluster
                    .and_then(|c| c.attribute(AttributeId::new(id)))
                    .and_then(extract_signed);
                push_stack(&mut stack, &mut sp, StackVal::Num(num));
            }
            ConformanceOp::PushConst(val) => {
                push_stack(&mut stack, &mut sp, StackVal::Num(Some(val)));
            }
            ConformanceOp::PushBool(b) => {
                push_stack(&mut stack, &mut sp, StackVal::Bool(b));
            }
            ConformanceOp::And => {
                if sp >= 2 {
                    let b = pop_bool(&stack, &mut sp);
                    let a = pop_bool(&stack, &mut sp);
                    push_stack(&mut stack, &mut sp, StackVal::Bool(a && b));
                }
            }
            ConformanceOp::Or => {
                if sp >= 2 {
                    let b = pop_bool(&stack, &mut sp);
                    let a = pop_bool(&stack, &mut sp);
                    push_stack(&mut stack, &mut sp, StackVal::Bool(a || b));
                }
            }
            ConformanceOp::Not => {
                if sp >= 1 {
                    let a = pop_bool(&stack, &mut sp);
                    push_stack(&mut stack, &mut sp, StackVal::Bool(!a));
                }
            }
            ConformanceOp::Greater => {
                if sp >= 2 {
                    let rhs = pop_num(&stack, &mut sp);
                    let lhs = pop_num(&stack, &mut sp);
                    let res = matches!((lhs, rhs), (Some(l), Some(r)) if l > r);
                    push_stack(&mut stack, &mut sp, StackVal::Bool(res));
                }
            }
            ConformanceOp::GreaterOrEqual => {
                if sp >= 2 {
                    let rhs = pop_num(&stack, &mut sp);
                    let lhs = pop_num(&stack, &mut sp);
                    let res = matches!((lhs, rhs), (Some(l), Some(r)) if l >= r);
                    push_stack(&mut stack, &mut sp, StackVal::Bool(res));
                }
            }
            ConformanceOp::Mandatory => {
                let cond = pop_bool_or_true(&stack, &mut sp);
                push_stack(
                    &mut stack,
                    &mut sp,
                    StackVal::Conf(cond.then_some(ConformanceResult::Mandatory)),
                );
            }
            ConformanceOp::Optional => {
                let cond = pop_bool_or_true(&stack, &mut sp);
                push_stack(
                    &mut stack,
                    &mut sp,
                    StackVal::Conf(cond.then_some(ConformanceResult::Optional)),
                );
            }
            ConformanceOp::Disallow => {
                let cond = pop_bool_or_true(&stack, &mut sp);
                push_stack(
                    &mut stack,
                    &mut sp,
                    StackVal::Conf(cond.then_some(ConformanceResult::Disallowed)),
                );
            }
            ConformanceOp::Provisional => {
                let cond = pop_bool_or_true(&stack, &mut sp);
                push_stack(
                    &mut stack,
                    &mut sp,
                    StackVal::Conf(cond.then_some(ConformanceResult::Provisional)),
                );
            }
            ConformanceOp::Deprecate => {
                let cond = pop_bool_or_true(&stack, &mut sp);
                push_stack(
                    &mut stack,
                    &mut sp,
                    StackVal::Conf(cond.then_some(ConformanceResult::Deprecated)),
                );
            }
            ConformanceOp::Otherwise => {
                if sp >= 2 {
                    let second = pop_conf(&stack, &mut sp);
                    let first = pop_conf(&stack, &mut sp);
                    push_stack(&mut stack, &mut sp, StackVal::Conf(first.or(second)));
                }
            }
        }
    }

    if let Some(top) = sp.checked_sub(1).and_then(|idx| stack.get(idx)) {
        match top {
            StackVal::Conf(Some(c)) => *c,
            _ => ConformanceResult::Optional,
        }
    } else {
        ConformanceResult::Optional
    }
}

/// Returns `true` when an evaluated conformance status rejects presence under `mode`.
pub(super) const fn is_disallowed(conf: ConformanceResult, mode: ValidationMode) -> bool {
    match conf {
        ConformanceResult::Disallowed => true,
        ConformanceResult::Provisional => matches!(
            mode,
            ValidationMode::FullSnapshot {
                enforce_provisional: true
            }
        ),
        _ => false,
    }
}

/// Evaluates whether the count of present members in `group` satisfies `[min, max]` (or `>= min` when `more`).
pub(super) fn check_choice_group(group: &ChoiceGroup, is_present: impl Fn(u32) -> bool) -> bool {
    let mut count: u8 = 0;
    for &member_id in group.members {
        if is_present(member_id) {
            count = count.saturating_add(1);
        }
    }
    count >= group.min && (group.more || count <= group.max)
}

fn push_stack(stack: &mut [StackVal; MAX_RPN_STACK], sp: &mut usize, val: StackVal) {
    if let Some(slot) = stack.get_mut(*sp) {
        *slot = val;
        *sp = sp.saturating_add(1);
    }
}

fn pop_bool(stack: &[StackVal; MAX_RPN_STACK], sp: &mut usize) -> bool {
    *sp = sp.saturating_sub(1);
    match stack.get(*sp) {
        Some(StackVal::Bool(b)) => *b,
        _ => false,
    }
}

fn pop_num(stack: &[StackVal; MAX_RPN_STACK], sp: &mut usize) -> Option<i64> {
    *sp = sp.saturating_sub(1);
    match stack.get(*sp) {
        Some(StackVal::Num(n)) => *n,
        _ => None,
    }
}

fn pop_conf(stack: &[StackVal; MAX_RPN_STACK], sp: &mut usize) -> Option<ConformanceResult> {
    *sp = sp.saturating_sub(1);
    match stack.get(*sp) {
        Some(StackVal::Conf(c)) => *c,
        _ => None,
    }
}

fn pop_bool_or_true(stack: &[StackVal; MAX_RPN_STACK], sp: &mut usize) -> bool {
    if let Some(idx) = sp.checked_sub(1)
        && let Some(StackVal::Bool(b)) = stack.get(idx).copied()
    {
        *sp = idx;
        return b;
    }
    true
}
