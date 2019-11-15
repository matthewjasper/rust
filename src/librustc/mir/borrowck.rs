/// Extra information produced by MIR construction that is used by Borrowck

use rustc_index::vec::IndexVec;
use rustc_macros::HashStable;
use syntax_pos::Span;

use crate::ty;
use crate::hir::ImplicitSelfKind;
use super::{Local, Place};

pub type ExtraLocalInfo<'tcx> = IndexVec<Local, LocalInfo<'tcx>>;

/// Extra information on LocalDecls that is used by borrowck.
#[derive(Debug, Clone, HashStable)]
pub enum LocalInfo<'tcx> {
    /// A user declared local variable
    Local {
        /// Is variable bound via `x`, `mut x`, `ref x`, or `ref mut x`?
        binding_mode: ty::BindingMode,
        /// Place of the RHS of the =, or the subject of the `match` where this
        /// variable is initialized. None in the case of `let PATTERN;`.
        /// Some((None, ..)) in the case of and `let [mut] x = ...` because
        /// (a) the right-hand side isn't evaluated as a place expression.
        /// (b) it gives a way to separate this case from the remaining cases
        ///     for diagnostics.
        opt_match_place: Option<(Option<Place<'tcx>>, Span)>,
        /// If an explicit type was provided for this variable binding,
        /// this holds the source Span of that type.
        // FIXME(matthewjasper) use `id` for this.
        opt_ty_info: Option<Span>,
        /// The span of the pattern in which this variable was bound.
        pat_span: Span,
        /// If true then the user visible local with the above `ItemLocalId` is
        /// represented by dereferencing this Local.
        ref_for_guard: bool,
    },
    /// `self` without a type annotation.
    ImplicitSelf {
        kind: ImplicitSelfKind,
    },
    /// A temporary value, an unnamed function parameter, or the return place.
    Other,
}

impl LocalInfo<'_> {
    #[inline]
    pub fn is_user_variable(&self) -> bool {
        match self {
            LocalInfo::Local { .. } | LocalInfo::ImplicitSelf { .. } => true,
            LocalInfo::Other => false,
        }
    }

    /// Returns `true` only if local is a binding that can itself be
    /// made mutable via the addition of the `mut` keyword, namely
    /// something like the occurrences of `x` in:
    /// - `fn foo(x: Type) { ... }`,
    /// - `let x = ...`,
    /// - or `match ... { C(x) => ... }`
    pub fn can_be_made_mutable(&self) -> bool {
        match self {
            LocalInfo::Local { binding_mode: ty::BindingMode::BindByValue(_), .. }
            | LocalInfo::ImplicitSelf { kind: ImplicitSelfKind::Imm } => true,

            _ => false,
        }
    }

    /// Returns `true` if local is definitely not a `ref ident` or
    /// `ref mut ident` binding. (Such bindings cannot be made into
    /// mutable bindings, but the inverse does not necessarily hold).
    pub fn is_nonref_binding(&self) -> bool {
        match self {
            LocalInfo::Local { binding_mode: ty::BindingMode::BindByValue(_), .. }
            | LocalInfo::ImplicitSelf { .. } => true,

            _ => false,
        }
    }
    /// Returns `true` if this is a reference to a variable bound in a `match`
    /// expression that is used to access said variable for the guard of the
    /// match arm.
    pub fn is_ref_for_guard(&self) -> bool {
        match *self {
            LocalInfo::Local { ref_for_guard, .. } => ref_for_guard,
            _ => false,
        }
    }
}
