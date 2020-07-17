//! Trait Resolution. See the [rustc dev guide] for more information on how this works.
//!
//! [rustc dev guide]: https://rustc-dev-guide.rust-lang.org/traits/resolution.html

#[allow(dead_code)]
pub mod auto_trait;
mod chalk_fulfill;
pub mod codegen;
mod coherence;
mod engine;
pub mod error_reporting;
mod fulfill;
pub mod misc;
mod object_safety;
mod on_unimplemented;
mod project;
pub mod query;
mod select;
mod specialize;
mod structural_match;
mod util;
pub mod wf;

use crate::infer::{InferCtxt, TyCtxtInferExt};
use crate::traits::query::evaluate_obligation::InferCtxtExt as _;
use rustc_hir as hir;
use rustc_hir::def_id::DefId;
use rustc_middle::traits::Reveal;
use rustc_middle::ty::fold::TypeFoldable;
use rustc_middle::ty::subst::{InternalSubsts, SubstsRef};
use rustc_middle::ty::{
    self, GenericParamDefKind, ParamEnv, ToPredicate, Ty, TyCtxt, WithConstness,
};
use rustc_span::Span;

use std::fmt::Debug;

pub use self::FulfillmentErrorCode::*;
pub use self::ImplSource::*;
pub use self::ObligationCauseCode::*;
pub use self::SelectionError::*;

pub use self::coherence::{add_placeholder_note, orphan_check, overlapping_impls};
pub use self::coherence::{OrphanCheckErr, OverlapResult};
pub use self::engine::TraitEngineExt;
pub use self::fulfill::{FulfillmentContext, PendingPredicateObligation};
pub use self::object_safety::astconv_object_safety_violations;
pub use self::object_safety::is_vtable_safe_method;
pub use self::object_safety::MethodViolationCode;
pub use self::object_safety::ObjectSafetyViolation;
pub use self::on_unimplemented::{OnUnimplementedDirective, OnUnimplementedNote};
pub use self::project::{normalize, normalize_projection_type};
pub use self::select::{EvaluationCache, SelectionCache, SelectionContext};
pub use self::select::{EvaluationResult, IntercrateAmbiguityCause, OverflowError};
pub use self::specialize::specialization_graph::FutureCompatOverlapError;
pub use self::specialize::specialization_graph::FutureCompatOverlapErrorKind;
pub use self::specialize::{specialization_graph, translate_substs, OverlapError};
pub use self::structural_match::search_for_structural_match_violation;
pub use self::structural_match::NonStructuralMatchTy;
pub use self::util::subst_assoc_item_bound;
pub use self::util::{elaborate_predicates, elaborate_trait_ref, elaborate_trait_refs};
pub use self::util::{expand_trait_aliases, TraitAliasExpander};
pub use self::util::{
    get_vtable_index_of_object_method, impl_item_is_final, predicate_for_trait_def, upcast_choices,
};
pub use self::util::{
    supertrait_def_ids, supertraits, transitive_bounds, SupertraitDefIds, Supertraits,
};

pub use self::chalk_fulfill::FulfillmentContext as ChalkFulfillmentContext;

pub use rustc_infer::traits::*;

/// Whether to skip the leak check, as part of a future compatibility warning step.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SkipLeakCheck {
    Yes,
    No,
}

impl SkipLeakCheck {
    fn is_yes(self) -> bool {
        self == SkipLeakCheck::Yes
    }
}

/// The "default" for skip-leak-check corresponds to the current
/// behavior (do not skip the leak check) -- not the behavior we are
/// transitioning into.
impl Default for SkipLeakCheck {
    fn default() -> Self {
        SkipLeakCheck::No
    }
}

/// The mode that trait queries run in.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TraitQueryMode {
    // Standard/un-canonicalized queries get accurate
    // spans etc. passed in and hence can do reasonable
    // error reporting on their own.
    Standard,
    // Canonicalized queries get dummy spans and hence
    // must generally propagate errors to
    // pre-canonicalization callsites.
    Canonical,
}

/// Creates predicate obligations from the generic bounds.
pub fn predicates_for_generics<'tcx>(
    cause: ObligationCause<'tcx>,
    param_env: ty::ParamEnv<'tcx>,
    generic_bounds: ty::InstantiatedPredicates<'tcx>,
) -> impl Iterator<Item = PredicateObligation<'tcx>> {
    util::predicates_for_generics(cause, 0, param_env, generic_bounds)
}

/// Determines whether the type `ty` is known to meet `bound` and
/// returns true if so. Returns false if `ty` either does not meet
/// `bound` or is not known to meet bound (note that this is
/// conservative towards *no impl*, which is the opposite of the
/// `evaluate` methods).
pub fn type_known_to_meet_bound_modulo_regions<'a, 'tcx>(
    infcx: &InferCtxt<'a, 'tcx>,
    param_env: ty::ParamEnv<'tcx>,
    ty: Ty<'tcx>,
    def_id: DefId,
    span: Span,
) -> bool {
    debug!(
        "type_known_to_meet_bound_modulo_regions(ty={:?}, bound={:?})",
        ty,
        infcx.tcx.def_path_str(def_id)
    );

    let trait_ref = ty::TraitRef { def_id, substs: infcx.tcx.mk_substs_trait(ty, &[]) };
    let obligation = Obligation {
        param_env,
        cause: ObligationCause::misc(span, hir::CRATE_HIR_ID),
        recursion_depth: 0,
        predicate: trait_ref.without_const().to_predicate(infcx.tcx),
    };

    let result = infcx.predicate_must_hold_modulo_regions(&obligation);
    debug!(
        "type_known_to_meet_ty={:?} bound={} => {:?}",
        ty,
        infcx.tcx.def_path_str(def_id),
        result
    );

    if result && ty.has_infer_types_or_consts() {
        // Because of inference "guessing", selection can sometimes claim
        // to succeed while the success requires a guess. To ensure
        // this function's result remains infallible, we must confirm
        // that guess. While imperfect, I believe this is sound.

        // The handling of regions in this area of the code is terrible,
        // see issue #29149. We should be able to improve on this with
        // NLL.
        let mut fulfill_cx = FulfillmentContext::new_ignoring_regions();

        // We can use a dummy node-id here because we won't pay any mind
        // to region obligations that arise (there shouldn't really be any
        // anyhow).
        let cause = ObligationCause::misc(span, hir::CRATE_HIR_ID);

        fulfill_cx.register_bound(infcx, param_env, ty, def_id, cause);

        // Note: we only assume something is `Copy` if we can
        // *definitively* show that it implements `Copy`. Otherwise,
        // assume it is move; linear is always ok.
        match fulfill_cx.select_all_or_error(infcx) {
            Ok(()) => {
                debug!(
                    "type_known_to_meet_bound_modulo_regions: ty={:?} bound={} success",
                    ty,
                    infcx.tcx.def_path_str(def_id)
                );
                true
            }
            Err(e) => {
                debug!(
                    "type_known_to_meet_bound_modulo_regions: ty={:?} bound={} errors={:?}",
                    ty,
                    infcx.tcx.def_path_str(def_id),
                    e
                );
                false
            }
        }
    } else {
        result
    }
}

/// Elaborates a list of predicates to form a parameter environment.
pub fn mk_param_env<'tcx>(
    tcx: TyCtxt<'tcx>,
    predicates: &[ty::Predicate<'tcx>],
    item_id: Option<DefId>,
) -> ty::ParamEnv<'tcx> {
    debug!("mk_param_env(predicates={:?}, item_id={:?})", predicates, item_id);

    let elaborated_predicates = tcx.mk_predicates(
        util::elaborate_predicates(tcx, predicates.iter().copied())
            .map(|obligation| obligation.predicate),
    );

    debug!("mk_param_env: elaborated_predicates={:?}", elaborated_predicates);

    ty::ParamEnv::new(elaborated_predicates, Reveal::UserFacing, item_id)
}

pub fn fully_normalize<'a, 'tcx, T>(
    infcx: &InferCtxt<'a, 'tcx>,
    mut fulfill_cx: FulfillmentContext<'tcx>,
    cause: ObligationCause<'tcx>,
    param_env: ty::ParamEnv<'tcx>,
    value: &T,
) -> Result<T, Vec<FulfillmentError<'tcx>>>
where
    T: TypeFoldable<'tcx>,
{
    debug!("fully_normalize_with_fulfillcx(value={:?})", value);
    let selcx = &mut SelectionContext::new(infcx);
    let Normalized { value: normalized_value, obligations } =
        project::normalize(selcx, param_env, cause, value);
    debug!(
        "fully_normalize: normalized_value={:?} obligations={:?}",
        normalized_value, obligations
    );
    for obligation in obligations {
        fulfill_cx.register_predicate_obligation(selcx.infcx(), obligation);
    }

    debug!("fully_normalize: select_all_or_error start");
    fulfill_cx.select_all_or_error(infcx)?;
    debug!("fully_normalize: select_all_or_error complete");
    let resolved_value = infcx.resolve_vars_if_possible(&normalized_value);
    debug!("fully_normalize: resolved_value={:?}", resolved_value);
    Ok(resolved_value)
}

/// Normalizes the predicates and checks whether they hold in an empty
/// environment. If this returns false, then either normalize
/// encountered an error or one of the predicates did not hold. Used
/// when creating vtables to check for unsatisfiable methods.
pub fn normalize_and_test_predicates<'tcx>(
    tcx: TyCtxt<'tcx>,
    predicates: Vec<ty::Predicate<'tcx>>,
) -> bool {
    debug!("normalize_and_test_predicates(predicates={:?})", predicates);

    let result = tcx.infer_ctxt().enter(|infcx| {
        let param_env = ty::ParamEnv::reveal_all();
        let mut selcx = SelectionContext::new(&infcx);
        let mut fulfill_cx = FulfillmentContext::new();
        let cause = ObligationCause::dummy();
        let Normalized { value: predicates, obligations } =
            normalize(&mut selcx, param_env, cause.clone(), &predicates);
        for obligation in obligations {
            fulfill_cx.register_predicate_obligation(&infcx, obligation);
        }
        for predicate in predicates {
            let obligation = Obligation::new(cause.clone(), param_env, predicate);
            fulfill_cx.register_predicate_obligation(&infcx, obligation);
        }

        fulfill_cx.select_all_or_error(&infcx).is_ok()
    });
    debug!("normalize_and_test_predicates(predicates={:?}) = {:?}", predicates, result);
    result
}

fn substitute_normalize_and_test_predicates<'tcx>(
    tcx: TyCtxt<'tcx>,
    key: (DefId, SubstsRef<'tcx>),
) -> bool {
    debug!("substitute_normalize_and_test_predicates(key={:?})", key);

    let predicates = tcx.predicates_of(key.0).instantiate(tcx, key.1).predicates;
    let result = normalize_and_test_predicates(tcx, predicates);

    debug!("substitute_normalize_and_test_predicates(key={:?}) = {:?}", key, result);
    result
}

/// Given a trait `trait_ref`, iterates the vtable entries
/// that come from `trait_ref`, including its supertraits.
#[inline] // FIXME(#35870): avoid closures being unexported due to `impl Trait`.
fn vtable_methods<'tcx>(
    tcx: TyCtxt<'tcx>,
    trait_ref: ty::PolyTraitRef<'tcx>,
) -> &'tcx [Option<(DefId, SubstsRef<'tcx>)>] {
    debug!("vtable_methods({:?})", trait_ref);

    tcx.arena.alloc_from_iter(supertraits(tcx, trait_ref).flat_map(move |trait_ref| {
        let trait_methods = tcx
            .associated_items(trait_ref.def_id())
            .in_definition_order()
            .filter(|item| item.kind == ty::AssocKind::Fn);

        // Now list each method's DefId and InternalSubsts (for within its trait).
        // If the method can never be called from this object, produce None.
        trait_methods.map(move |trait_method| {
            debug!("vtable_methods: trait_method={:?}", trait_method);
            let def_id = trait_method.def_id;

            // Some methods cannot be called on an object; skip those.
            if !is_vtable_safe_method(tcx, trait_ref.def_id(), &trait_method) {
                debug!("vtable_methods: not vtable safe");
                return None;
            }

            // The method may have some early-bound lifetimes; add regions for those.
            let substs = trait_ref.map_bound(|trait_ref| {
                InternalSubsts::for_item(tcx, def_id, |param, _| match param.kind {
                    GenericParamDefKind::Lifetime => tcx.lifetimes.re_erased.into(),
                    GenericParamDefKind::Type { .. } | GenericParamDefKind::Const => {
                        trait_ref.substs[param.index as usize]
                    }
                })
            });

            // The trait type may have higher-ranked lifetimes in it;
            // erase them if they appear, so that we get the type
            // at some particular call site.
            let substs =
                tcx.normalize_erasing_late_bound_regions(ty::ParamEnv::reveal_all(), &substs);

            // It's possible that the method relies on where-clauses that
            // do not hold for this particular set of type parameters.
            // Note that this method could then never be called, so we
            // do not want to try and codegen it, in that case (see #23435).
            let predicates = tcx.predicates_of(def_id).instantiate_own(tcx, substs);
            if !normalize_and_test_predicates(tcx, predicates.predicates) {
                debug!("vtable_methods: predicates do not hold");
                return None;
            }

            Some((def_id, substs))
        })
    }))
}

/// Check whether a `ty` implements given trait(trait_def_id).
///
/// NOTE: Always return `false` for a type which needs inference.
fn type_implements_trait<'tcx>(
    tcx: TyCtxt<'tcx>,
    key: (
        DefId,    // trait_def_id,
        Ty<'tcx>, // type
        SubstsRef<'tcx>,
        ParamEnv<'tcx>,
    ),
) -> bool {
    let (trait_def_id, ty, params, param_env) = key;

    debug!(
        "type_implements_trait: trait_def_id={:?}, type={:?}, params={:?}, param_env={:?}",
        trait_def_id, ty, params, param_env
    );

    let trait_ref = ty::TraitRef { def_id: trait_def_id, substs: tcx.mk_substs_trait(ty, params) };

    let obligation = Obligation {
        cause: ObligationCause::dummy(),
        param_env,
        recursion_depth: 0,
        predicate: trait_ref.without_const().to_predicate(tcx),
    };
    tcx.infer_ctxt().enter(|infcx| infcx.predicate_must_hold_modulo_regions(&obligation))
}

pub fn provide(providers: &mut ty::query::Providers<'_>) {
    object_safety::provide(providers);
    structural_match::provide(providers);
    *providers = ty::query::Providers {
        specialization_graph_of: specialize::specialization_graph_provider,
        specializes: specialize::specializes,
        codegen_fulfill_obligation: codegen::codegen_fulfill_obligation,
        vtable_methods,
        substitute_normalize_and_test_predicates,
        type_implements_trait,
        ..*providers
    };
}
