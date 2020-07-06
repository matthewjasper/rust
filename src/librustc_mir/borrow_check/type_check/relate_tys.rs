use rustc_infer::infer::nll_relate::{TypeRelating, TypeRelatingDelegate};
use rustc_infer::infer::{InferCtxt, InferOk, NLLRegionVariableOrigin};
use rustc_infer::traits::{Obligation, ObligationCause, PredicateObligation};
use rustc_middle::mir::ConstraintCategory;
use rustc_middle::ty::relate::TypeRelation;
use rustc_middle::ty::{self, Const, ToPredicate, Ty};
use rustc_trait_selection::traits::query::Fallible;

use crate::borrow_check::constraints::OutlivesConstraint;
use crate::borrow_check::type_check::{BorrowCheckContext, Locations};

/// Adds sufficient constraints to ensure that `a R b` where `R` depends on `v`:
///
/// - "Covariant" `a <: b`
/// - "Invariant" `a == b`
/// - "Contravariant" `a :> b`
///
/// N.B., the type `a` is permitted to have unresolved inference
/// variables, but not the type `b`.
pub(super) fn relate_types<'tcx>(
    infcx: &InferCtxt<'_, 'tcx>,
    param_env: ty::ParamEnv<'tcx>,
    a: Ty<'tcx>,
    v: ty::Variance,
    b: Ty<'tcx>,
    locations: Locations,
    category: ConstraintCategory,
    borrowck_context: Option<&mut BorrowCheckContext<'_, 'tcx>>,
) -> Fallible<InferOk<'tcx, ()>> {
    debug!("relate_types(a={:?}, v={:?}, b={:?}, locations={:?})", a, v, b, locations);

    let mut relating = TypeRelating::new(
        infcx,
        NllTypeRelatingDelegate::new(infcx, param_env, borrowck_context, locations, category),
        v,
    );
    relating.relate(a, b)?;
    Ok(InferOk { obligations: relating.delegate().nested_obligations, value: () })
}

struct NllTypeRelatingDelegate<'me, 'bccx, 'tcx> {
    infcx: &'me InferCtxt<'me, 'tcx>,
    param_env: ty::ParamEnv<'tcx>,
    borrowck_context: Option<&'me mut BorrowCheckContext<'bccx, 'tcx>>,
    nested_obligations: Vec<PredicateObligation<'tcx>>,

    /// Where (and why) is this relation taking place?
    locations: Locations,

    /// What category do we assign the resulting `'a: 'b` relationships?
    category: ConstraintCategory,
}

impl NllTypeRelatingDelegate<'me, 'bccx, 'tcx> {
    fn new(
        infcx: &'me InferCtxt<'me, 'tcx>,
        param_env: ty::ParamEnv<'tcx>,
        borrowck_context: Option<&'me mut BorrowCheckContext<'bccx, 'tcx>>,
        locations: Locations,
        category: ConstraintCategory,
    ) -> Self {
        Self {
            infcx,
            param_env,
            borrowck_context,
            locations,
            category,
            nested_obligations: Vec::new(),
        }
    }
}

impl TypeRelatingDelegate<'tcx> for NllTypeRelatingDelegate<'_, '_, 'tcx> {
    fn create_next_universe(&mut self) -> ty::UniverseIndex {
        self.infcx.create_next_universe()
    }

    fn next_existential_region_var(&mut self, from_forall: bool) -> ty::Region<'tcx> {
        if self.borrowck_context.is_some() {
            let origin = NLLRegionVariableOrigin::Existential { from_forall };
            self.infcx.next_nll_region_var(origin)
        } else {
            self.infcx.tcx.lifetimes.re_erased
        }
    }

    fn next_placeholder_region(&mut self, placeholder: ty::PlaceholderRegion) -> ty::Region<'tcx> {
        if let Some(borrowck_context) = &mut self.borrowck_context {
            borrowck_context.constraints.placeholder_region(self.infcx, placeholder)
        } else {
            self.infcx.tcx.lifetimes.re_erased
        }
    }

    fn generalize_existential(&mut self, universe: ty::UniverseIndex) -> ty::Region<'tcx> {
        self.infcx.next_nll_region_var_in_universe(
            NLLRegionVariableOrigin::Existential { from_forall: false },
            universe,
        )
    }

    fn push_outlives(&mut self, sup: ty::Region<'tcx>, sub: ty::Region<'tcx>) {
        if let Some(borrowck_context) = &mut self.borrowck_context {
            let sub = borrowck_context.universal_regions.to_region_vid(sub);
            let sup = borrowck_context.universal_regions.to_region_vid(sup);
            borrowck_context.constraints.outlives_constraints.push(OutlivesConstraint {
                sup,
                sub,
                locations: self.locations,
                category: self.category,
            });
        }
    }

    fn param_env(&self) -> ty::ParamEnv<'tcx> {
        self.param_env
    }

    fn const_equate(&mut self, a: &'tcx Const<'tcx>, b: &'tcx Const<'tcx>) {
        self.nested_obligations.push(Obligation {
            cause: ObligationCause::dummy(),
            param_env: self.param_env,
            predicate: ty::PredicateKind::ConstEquate(a, b).to_predicate(self.infcx.tcx),
            recursion_depth: 0,
        });
    }

    fn push_projection_predicate(&mut self, projection_ty: ty::ProjectionTy<'tcx>, ty: Ty<'tcx>) {
        self.nested_obligations.push(Obligation {
            cause: ObligationCause::dummy(),
            param_env: self.param_env,
            predicate: ty::PredicateKind::Projection(ty::Binder::dummy(ty::ProjectionPredicate {
                projection_ty,
                ty,
            }))
            .to_predicate(self.infcx.tcx),
            recursion_depth: 0,
        })
    }

    fn forbid_inference_vars() -> bool {
        true
    }
}
