//! HIR datatypes. See the [rustc guide] for more info.
//!
//! [rustc guide]: https://rust-lang.github.io/rustc-guide/hir.html

pub mod check_attr;
pub mod exports;
pub mod map;
pub mod upvars;

use crate::ty::query::Providers;
use crate::ty::TyCtxt;
use rustc_hir as hir;
use rustc_hir::def_id::DefId;
use rustc_hir::lang_item::LangItem;
use rustc_span::Span;

pub fn provide(providers: &mut Providers<'_>) {
    check_attr::provide(providers);
    map::provide(providers);
    upvars::provide(providers);
}

impl hir::RequireLangItem for TyCtxt<'_> {
    fn require_lang_item(self, lang_item: LangItem, span: Span) -> DefId {
        self.require_lang_item(lang_item, Some(span))
    }
}
