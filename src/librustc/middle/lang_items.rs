//! Detecting language items.
//!
//! Language items are items that represent concepts intrinsic to the language
//! itself. Examples are:
//!
//! * Traits that specify "kinds"; e.g., `Sync`, `Send`.
//! * Traits that represent operators; e.g., `Add`, `Sub`, `Index`.
//! * Functions called by the compiler itself.

pub use rustc_hir::lang_item::*;

use crate::middle::cstore::ExternCrate;
use crate::middle::weak_lang_items;
use crate::ty::{self, TyCtxt};

use rustc_data_structures::fx::FxHashMap;
use rustc_errors::struct_span_err;
use rustc_hir as hir;
use rustc_hir::def_id::DefId;
use rustc_hir::itemlikevisit::ItemLikeVisitor;
use rustc_span::symbol::Symbol;
use rustc_span::Span;
use syntax::ast;

/// Traverses and collects all the lang items in all crates.
pub fn collect<'tcx>(tcx: TyCtxt<'tcx>) -> LanguageItems {
    // Initialize the collector.
    let mut collector = LanguageItemCollector::new(tcx);

    // Collect lang items in other crates.
    for &cnum in tcx.crates().iter() {
        for &(def_id, item_index) in tcx.defined_lang_items(cnum).iter() {
            collector.collect_item(item_index, def_id);
        }
    }

    // Collect lang items in this crate.
    tcx.hir().krate().visit_all_item_likes(&mut collector);

    // Extract out the found lang items.
    let LanguageItemCollector { mut items, .. } = collector;

    // Find all required but not-yet-defined lang items.
    weak_lang_items::check_crate(tcx, &mut items);

    items
}

struct LanguageItemCollector<'tcx> {
    items: LanguageItems,
    tcx: TyCtxt<'tcx>,
    /// A mapping from the name of the lang item to its order and the form it must be of.
    item_refs: FxHashMap<&'static str, (usize, Location)>,
}

impl LanguageItemCollector<'_> {
    fn new(tcx: TyCtxt<'tcx>) -> LanguageItemCollector<'tcx> {
        LanguageItemCollector { tcx, items: LanguageItems::new(), item_refs: LangItem::table() }
    }

    fn collect_item(&mut self, item_index: usize, item_def_id: DefId) {
        let item_place = &mut self.items.items[item_index];

        // Check for duplicates.
        if let Some(original_def_id) = *item_place {
            if original_def_id != item_def_id {
                let name = LangItem::from_u32(item_index as u32).unwrap().name();
                let mut err = match self.tcx.hir().span_if_local(item_def_id) {
                    Some(span) => struct_span_err!(
                        self.tcx.sess,
                        span,
                        E0152,
                        "found duplicate lang item `{}`",
                        name
                    ),
                    None => match self.tcx.extern_crate(item_def_id) {
                        Some(ExternCrate { dependency_of, .. }) => {
                            self.tcx.sess.struct_err(&format!(
                                "duplicate lang item in crate `{}` (which `{}` depends on): `{}`.",
                                self.tcx.crate_name(item_def_id.krate),
                                self.tcx.crate_name(*dependency_of),
                                name
                            ))
                        }
                        _ => self.tcx.sess.struct_err(&format!(
                            "duplicate lang item in crate `{}`: `{}`.",
                            self.tcx.crate_name(item_def_id.krate),
                            name
                        )),
                    },
                };
                if let Some(span) = self.tcx.hir().span_if_local(original_def_id) {
                    err.span_note(span, "first defined here");
                } else {
                    match self.tcx.extern_crate(original_def_id) {
                        Some(ExternCrate { dependency_of, .. }) => {
                            err.note(&format!(
                                "first defined in crate `{}` (which `{}` depends on)",
                                self.tcx.crate_name(original_def_id.krate),
                                self.tcx.crate_name(*dependency_of)
                            ));
                        }
                        _ => {
                            err.note(&format!(
                                "first defined in crate `{}`.",
                                self.tcx.crate_name(original_def_id.krate)
                            ));
                        }
                    }
                }
                err.emit();
            }
        }

        // Matched.
        *item_place = Some(item_def_id);
    }

    /// Emits a diagnostic error when `value` is an unknown lang item.
    fn unknown_lang_item(&self, value: Symbol, span: Span) {
        struct_span_err!(
            self.tcx.sess,
            span,
            E0522,
            "definition of an unknown language item: `{}`",
            value
        )
        .span_label(span, format!("definition of unknown language item `{}`", value))
        .emit();
    }

    /// Known lang item with attribute on incorrect target.
    fn wrong_location(&self, value: Symbol, span: Span, expected: Location, actual: Location) {
        struct_span_err!(
            self.tcx.sess,
            span,
            E0718,
            "`{}` language item must be applied to a {}",
            value,
            expected,
        )
        .span_label(
            span,
            format!("attribute should be applied to a {}, not a {}", expected, actual),
        )
        .emit();
    }

    /// The item form does not admit language items.
    fn lang_items_not_allowed_here(&self, span: Span) {
        struct_span_err!(self.tcx.sess, span, E0719, "item cannot be a language item").emit();
    }

    /// Visit an item-like object and possibly collect it as a lang item.
    /// It has to have at least a sequence of attributes to scan for ´#[lang(..)]`.
    /// The `DefId` that is collected is determined by the given `HirId`.
    /// Finally, the `locator` determines the `Location` of the `HirId`
    /// or returns `None` should the location be invalid.
    fn visit_item_like(
        &mut self,
        attrs: &[ast::Attribute],
        hir_id: hir::HirId,
        locator: impl FnOnce() -> Option<Location>,
    ) {
        if let Some((value, span)) = extract(attrs) {
            if let Some(actual) = locator() {
                match self.item_refs.get(&*value.as_str()).cloned() {
                    Some((item_index, expected)) if actual == expected => {
                        // Known lang item with attribute on correct location.
                        let def_id = self.tcx.hir().local_def_id(hir_id);
                        self.collect_item(item_index, def_id);
                    }
                    Some((_, expected)) => self.wrong_location(value, span, expected, actual),
                    _ => self.unknown_lang_item(value, span),
                }
            } else {
                self.lang_items_not_allowed_here(span);
            }
        }
    }
}

impl ItemLikeVisitor<'_> for LanguageItemCollector<'_> {
    fn visit_item(&mut self, item: &hir::Item<'_>) {
        use hir::ItemKind::*;
        self.visit_item_like(&item.attrs, item.hir_id, || {
            Some(match item.kind {
                Enum(..) => Location::Enum,
                Fn(..) => Location::Fn,
                Impl { .. } => Location::Impl,
                Static(..) => Location::Static,
                Struct(..) => Location::Struct,
                Trait(..) => Location::Trait,
                Union(..) => Location::Union,
                _ => return None,
            })
        });

        if let Enum(ref def, _) = item.kind {
            for variant in def.variants {
                self.visit_item_like(&variant.attrs, variant.id, || Some(Location::Variant));
            }
        }
    }

    fn visit_impl_item(&mut self, item: &hir::ImplItem<'_>) {
        use hir::ImplItemKind::*;
        self.visit_item_like(&item.attrs, item.hir_id, || {
            Some(match item.kind {
                Method(..) => Location::Method,
                _ => return None,
            })
        });
    }

    fn visit_trait_item(&mut self, item: &hir::TraitItem<'_>) {
        use hir::TraitItemKind::*;
        self.visit_item_like(&item.attrs, item.hir_id, || {
            Some(match item.kind {
                Method(..) => Location::Method,
                _ => return None,
            })
        });
    }
}

impl<'tcx> TyCtxt<'tcx> {
    /// Returns the kind of closure that `id`, which is one of the `Fn*` traits, corresponds to.
    /// If `id` is not one of the `Fn*` traits, `None` is returned.
    pub fn fn_trait_kind(self, id: DefId) -> Option<ty::ClosureKind> {
        let lang_items = self.lang_items();
        match Some(id) {
            x if x == lang_items.fn_trait() => Some(ty::ClosureKind::Fn),
            x if x == lang_items.fn_mut_trait() => Some(ty::ClosureKind::FnMut),
            x if x == lang_items.fn_once_trait() => Some(ty::ClosureKind::FnOnce),
            _ => None,
        }
    }
    /// Returns the `DefId` for a given `LangItem`.
    /// If not found, fatally aborts compilation.
    pub fn require_lang_item(self, lang_item: LangItem, span: Option<Span>) -> DefId {
        self.lang_items().require(lang_item).unwrap_or_else(|msg| {
            if let Some(span) = span {
                self.sess.span_fatal(span, &msg)
            } else {
                self.sess.fatal(&msg)
            }
        })
    }
}
