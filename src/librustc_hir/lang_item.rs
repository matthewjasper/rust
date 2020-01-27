//! Language items.
//!
//! Language items are items that represent concepts intrinsic to the language
//! itself. Examples are:
//!
//! * Traits that specify "kinds"; e.g., `Sync`, `Send`.
//! * Traits that represent operators; e.g., `Add`, `Sub`, `Index`.
//! * Functions called by the compiler itself.

pub use self::LangItem::*;

use crate::def_id::DefId;
use rustc_data_structures::fx::FxHashMap;
use rustc_macros::HashStable_Generic;
use rustc_span::symbol::{sym, Symbol};
use rustc_span::Span;
use syntax::ast;

use std::fmt;

/// All the locations that a `#[lang = "<name>"]` attribute is valid on.
/// The variant names correspond to the variants in `DefKind`.
#[derive(Copy, Clone, PartialEq)]
pub enum Location {
    Enum,
    Fn,
    Impl,
    Method,
    Static,
    Struct,
    Trait,
    Union,
    Variant,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Enum => "enum",
            Self::Fn => "function",
            Self::Impl => "implementation",
            Self::Method => "method",
            Self::Static => "static item",
            Self::Struct => "struct",
            Self::Trait => "trait",
            Self::Union => "union",
            Self::Variant => "variant",
        })
    }
}

macro_rules! enum_from_u32 {
    ($(#[$attr:meta])* pub enum $name:ident {
        $($variant:ident = $e:expr,)*
    }) => {
        $(#[$attr])*
        pub enum $name {
            $($variant = $e),*
        }

        impl $name {
            pub fn from_u32(u: u32) -> Option<$name> {
                $(if u == $name::$variant as u32 {
                    return Some($name::$variant)
                })*
                None
            }
        }
    };
    ($(#[$attr:meta])* pub enum $name:ident {
        $($variant:ident,)*
    }) => {
        $(#[$attr])*
        pub enum $name {
            $($variant,)*
        }

        impl $name {
            pub fn from_u32(u: u32) -> Option<$name> {
                $(if u == $name::$variant as u32 {
                    return Some($name::$variant)
                })*
                None
            }
        }
    }
}

// The actual lang items defined come at the end of this file in one handy table.
// So you probably just want to nip down to the end.
macro_rules! language_item_table {
    (
        $( $variant:ident, $name:expr, $method:ident, $location:path; )*
    ) => {

enum_from_u32! {
    /// A representation of all the valid language items in Rust.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    #[derive(HashStable_Generic, RustcEncodable, RustcDecodable)]
    pub enum LangItem {
        $($variant,)*
    }
}

impl LangItem {
    /// Returns the `name` in `#[lang = "$name"]`.
    /// For example, `LangItem::EqTraitLangItem`,
    /// that is `#[lang = "eq"]` would result in `"eq"`.
    pub fn name(self) -> &'static str {
        match self {
            $( $variant => $name, )*
        }
    }

    /// Returns the `Location` where the lang item may be attached.
    pub fn location(self) -> Location {
        match self {
            $( $variant => $location, )*
        }
    }

    /// Get a mapping from the name of the lang item to its index and
    /// the form it must be of.
    pub fn table() -> FxHashMap<&'static str, (usize, Location)> {
        let mut item_refs = FxHashMap::default();

        $( item_refs.insert($name, ($variant as usize, $location)); )*

        item_refs
    }
}

#[derive(HashStable_Generic, Debug)]
pub struct LanguageItems {
    /// Mappings from lang items to their possibly found `DefId`s.
    /// The index corresponds to the order in `LangItem`.
    pub items: Vec<Option<DefId>>,
    /// Lang items that were not found during collection.
    pub missing: Vec<LangItem>,
}

impl LanguageItems {
    /// Construct an empty collection of lang items and no missing ones.
    pub fn new() -> Self {
        fn init_none(_: LangItem) -> Option<DefId> { None }

        Self {
            items: vec![$(init_none($variant)),*],
            missing: Vec::new(),
        }
    }

    /// Returns the mappings to the possibly found `DefId`s for each lang item.
    pub fn items(&self) -> &[Option<DefId>] {
        &*self.items
    }

    /// Requires that a given `LangItem` was bound and returns the corresponding `DefId`.
    /// If it wasn't bound, e.g. due to a missing `#[lang = "<it.name()>"]`,
    /// returns an error message as a string.
    pub fn require(&self, it: LangItem) -> Result<DefId, String> {
        self.items[it as usize].ok_or_else(|| format!("requires `{}` lang_item", it.name()))
    }

    $(
        /// Returns the corresponding `DefId` for the lang item
        #[doc = $name]
        /// if it exists.
        #[allow(dead_code)]
        pub fn $method(&self) -> Option<DefId> {
            self.items[$variant as usize]
        }
    )*
}

/// Extracts the first `lang = "$name"` out of a list of attributes.
/// The attributes `#[panic_handler]` and `#[alloc_error_handler]`
/// are also extracted out when found.
pub fn extract(attrs: &[ast::Attribute]) -> Option<(Symbol, Span)> {
    attrs.iter().find_map(|attr| {
        Some(match attr {
            _ if attr.check_name(sym::lang) => (attr.value_str()?, attr.span),
            _ if attr.check_name(sym::panic_handler) => (sym::panic_impl, attr.span),
            _ if attr.check_name(sym::alloc_error_handler) => (sym::oom, attr.span),
            _ => return None,
        })
    })
}

// End of the macro
    }
}

language_item_table! {
//  Variant name,                Name,                 Method name,             Location;
    BoolImplItem,                "bool",               bool_impl,               Location::Impl;
    CharImplItem,                "char",               char_impl,               Location::Impl;
    StrImplItem,                 "str",                str_impl,                Location::Impl;
    SliceImplItem,               "slice",              slice_impl,              Location::Impl;
    SliceU8ImplItem,             "slice_u8",           slice_u8_impl,           Location::Impl;
    StrAllocImplItem,            "str_alloc",          str_alloc_impl,          Location::Impl;
    SliceAllocImplItem,          "slice_alloc",        slice_alloc_impl,        Location::Impl;
    SliceU8AllocImplItem,        "slice_u8_alloc",     slice_u8_alloc_impl,     Location::Impl;
    ConstPtrImplItem,            "const_ptr",          const_ptr_impl,          Location::Impl;
    MutPtrImplItem,              "mut_ptr",            mut_ptr_impl,            Location::Impl;
    I8ImplItem,                  "i8",                 i8_impl,                 Location::Impl;
    I16ImplItem,                 "i16",                i16_impl,                Location::Impl;
    I32ImplItem,                 "i32",                i32_impl,                Location::Impl;
    I64ImplItem,                 "i64",                i64_impl,                Location::Impl;
    I128ImplItem,                "i128",               i128_impl,               Location::Impl;
    IsizeImplItem,               "isize",              isize_impl,              Location::Impl;
    U8ImplItem,                  "u8",                 u8_impl,                 Location::Impl;
    U16ImplItem,                 "u16",                u16_impl,                Location::Impl;
    U32ImplItem,                 "u32",                u32_impl,                Location::Impl;
    U64ImplItem,                 "u64",                u64_impl,                Location::Impl;
    U128ImplItem,                "u128",               u128_impl,               Location::Impl;
    UsizeImplItem,               "usize",              usize_impl,              Location::Impl;
    F32ImplItem,                 "f32",                f32_impl,                Location::Impl;
    F64ImplItem,                 "f64",                f64_impl,                Location::Impl;
    F32RuntimeImplItem,          "f32_runtime",        f32_runtime_impl,        Location::Impl;
    F64RuntimeImplItem,          "f64_runtime",        f64_runtime_impl,        Location::Impl;

    SizedTraitLangItem,          "sized",              sized_trait,             Location::Trait;
    // trait injected by #[derive(PartialEq)], (i.e. "Partial EQ").
    StructuralPeqTraitLangItem,  "structural_peq",     structural_peq_trait,    Location::Trait;
    // trait injected by #[derive(Eq)], (i.e. "Total EQ"; no, I will not apologize).
    StructuralTeqTraitLangItem,  "structural_teq",     structural_teq_trait,    Location::Trait;
    UnsizeTraitLangItem,         "unsize",             unsize_trait,            Location::Trait;
    CopyTraitLangItem,           "copy",               copy_trait,              Location::Trait;
    CloneTraitLangItem,          "clone",              clone_trait,             Location::Trait;
    SyncTraitLangItem,           "sync",               sync_trait,              Location::Trait;
    FreezeTraitLangItem,         "freeze",             freeze_trait,            Location::Trait;

    DropTraitLangItem,           "drop",               drop_trait,              Location::Trait;

    CoerceUnsizedTraitLangItem,  "coerce_unsized",     coerce_unsized_trait,    Location::Trait;
    DispatchFromDynTraitLangItem,"dispatch_from_dyn",  dispatch_from_dyn_trait, Location::Trait;

    AddTraitLangItem,            "add",                add_trait,               Location::Trait;
    SubTraitLangItem,            "sub",                sub_trait,               Location::Trait;
    MulTraitLangItem,            "mul",                mul_trait,               Location::Trait;
    DivTraitLangItem,            "div",                div_trait,               Location::Trait;
    RemTraitLangItem,            "rem",                rem_trait,               Location::Trait;
    NegTraitLangItem,            "neg",                neg_trait,               Location::Trait;
    NotTraitLangItem,            "not",                not_trait,               Location::Trait;
    BitXorTraitLangItem,         "bitxor",             bitxor_trait,            Location::Trait;
    BitAndTraitLangItem,         "bitand",             bitand_trait,            Location::Trait;
    BitOrTraitLangItem,          "bitor",              bitor_trait,             Location::Trait;
    ShlTraitLangItem,            "shl",                shl_trait,               Location::Trait;
    ShrTraitLangItem,            "shr",                shr_trait,               Location::Trait;
    AddAssignTraitLangItem,      "add_assign",         add_assign_trait,        Location::Trait;
    SubAssignTraitLangItem,      "sub_assign",         sub_assign_trait,        Location::Trait;
    MulAssignTraitLangItem,      "mul_assign",         mul_assign_trait,        Location::Trait;
    DivAssignTraitLangItem,      "div_assign",         div_assign_trait,        Location::Trait;
    RemAssignTraitLangItem,      "rem_assign",         rem_assign_trait,        Location::Trait;
    BitXorAssignTraitLangItem,   "bitxor_assign",      bitxor_assign_trait,     Location::Trait;
    BitAndAssignTraitLangItem,   "bitand_assign",      bitand_assign_trait,     Location::Trait;
    BitOrAssignTraitLangItem,    "bitor_assign",       bitor_assign_trait,      Location::Trait;
    ShlAssignTraitLangItem,      "shl_assign",         shl_assign_trait,        Location::Trait;
    ShrAssignTraitLangItem,      "shr_assign",         shr_assign_trait,        Location::Trait;
    IndexTraitLangItem,          "index",              index_trait,             Location::Trait;
    IndexMutTraitLangItem,       "index_mut",          index_mut_trait,         Location::Trait;

    UnsafeCellTypeLangItem,      "unsafe_cell",        unsafe_cell_type,        Location::Struct;
    VaListTypeLangItem,          "va_list",            va_list,                 Location::Struct;

    DerefTraitLangItem,          "deref",              deref_trait,             Location::Trait;
    DerefMutTraitLangItem,       "deref_mut",          deref_mut_trait,         Location::Trait;
    ReceiverTraitLangItem,       "receiver",           receiver_trait,          Location::Trait;

    FnTraitLangItem,             "fn",                 fn_trait,                Location::Trait;
    FnMutTraitLangItem,          "fn_mut",             fn_mut_trait,            Location::Trait;
    FnOnceTraitLangItem,         "fn_once",            fn_once_trait,           Location::Trait;

    FutureTraitLangItem,         "future_trait",       future_trait,            Location::Trait;
    GeneratorStateLangItem,      "generator_state",    gen_state,               Location::Enum;
    GeneratorTraitLangItem,      "generator",          gen_trait,               Location::Trait;
    UnpinTraitLangItem,          "unpin",              unpin_trait,             Location::Trait;
    PinTypeLangItem,             "pin",                pin_type,                Location::Struct;

    EqTraitLangItem,             "eq",                 eq_trait,                Location::Trait;
    PartialOrdTraitLangItem,     "partial_ord",        partial_ord_trait,       Location::Trait;
    OrdTraitLangItem,            "ord",                ord_trait,               Location::Trait;

    // A number of panic-related lang items. The `panic` item corresponds to
    // divide-by-zero and various panic cases with `match`. The
    // `panic_bounds_check` item is for indexing arrays.
    //
    // The `begin_unwind` lang item has a predefined symbol name and is sort of
    // a "weak lang item" in the sense that a crate is not required to have it
    // defined to use it, but a final product is required to define it
    // somewhere. Additionally, there are restrictions on crates that use a weak
    // lang item, but do not have it defined.
    PanicFnLangItem,             "panic",              panic_fn,                Location::Fn;
    PanicBoundsCheckFnLangItem,  "panic_bounds_check", panic_bounds_check_fn,   Location::Fn;
    PanicInfoLangItem,           "panic_info",         panic_info,              Location::Struct;
    PanicLocationLangItem,       "panic_location",     panic_location,          Location::Struct;
    PanicImplLangItem,           "panic_impl",         panic_impl,              Location::Fn;
    // Libstd panic entry point. Necessary for const eval to be able to catch it
    BeginPanicFnLangItem,        "begin_panic",        begin_panic_fn,          Location::Fn;

    ExchangeMallocFnLangItem,    "exchange_malloc",    exchange_malloc_fn,      Location::Fn;
    BoxFreeFnLangItem,           "box_free",           box_free_fn,             Location::Fn;
    DropInPlaceFnLangItem,       "drop_in_place",      drop_in_place_fn,        Location::Fn;
    OomLangItem,                 "oom",                oom,                     Location::Fn;
    AllocLayoutLangItem,         "alloc_layout",       alloc_layout,            Location::Struct;

    StartFnLangItem,             "start",              start_fn,                Location::Fn;

    EhPersonalityLangItem,       "eh_personality",     eh_personality,          Location::Fn;
    EhUnwindResumeLangItem,      "eh_unwind_resume",   eh_unwind_resume,        Location::Fn;
    EhCatchTypeinfoLangItem,     "eh_catch_typeinfo",  eh_catch_typeinfo,       Location::Static;

    OwnedBoxLangItem,            "owned_box",          owned_box,               Location::Struct;

    PhantomDataItem,             "phantom_data",       phantom_data,            Location::Struct;

    ManuallyDropItem,            "manually_drop",      manually_drop,           Location::Struct;

    MaybeUninitLangItem,         "maybe_uninit",       maybe_uninit,            Location::Union;

    // Align offset for stride != 1; must not panic.
    AlignOffsetLangItem,         "align_offset",       align_offset_fn,         Location::Fn;

    TerminationTraitLangItem,    "termination",        termination,             Location::Trait;

    Arc,                         "arc",                arc,                     Location::Struct;
    Rc,                          "rc",                 rc,                      Location::Struct;

    // Things used in lowering.

    // Range expressions:
    RangeInclusiveNew,           "range_inclusive",    range_inclusive_fn,      Location::Method;
    Range,                       "range",              range,                   Location::Struct;
    RangeFrom,                   "range_from",         range_from,              Location::Struct;
    RangeTo,                     "range_to",           range_to,                Location::Struct;
    RangeFull,                   "range_full",         range_full,              Location::Struct;
    RangeToInclusive,            "range_to_inclusive", range_to_inclusive,      Location::Struct;

    // For loops:
    IntoIter,                    "into_iter",          into_iter,               Location::Method;
    IterNext,                    "iter_next",          iter_next,               Location::Method;
    OptionSome,                  "option_some",        option_some,             Location::Variant;
    OptionNone,                  "option_none",        option_none,             Location::Variant;

    // `?` and `try { }`
    TryFromOk,                   "try_from_ok",        try_from_ok,             Location::Method;
    TryFromError,                "try_from_error",     try_from_error,          Location::Method;
    TryIntoResult,               "try_into_result",    try_into_result,         Location::Method;
    FromMethod,                  "from_method",        from_method,             Location::Method;
    ResultOk,                    "result_ok",          result_ok,               Location::Variant;
    ResultErr,                   "result_err",         result_err,              Location::Variant;

    // `async` (also uses FutureTraitLangItem)
    FromGenerator,               "from_generator",     from_generator,          Location::Fn;

    // `.await`
    PollWithTlsContext,          "poll_with_context",  poll_with_context,       Location::Fn;
    PinNewUnchecked,             "pin_new_unchecked",  pin_new_unchecked,       Location::Method;
    PollReady,                   "poll_ready",         poll_ready,              Location::Variant;
    PollPending,                 "poll_pending",       poll_pending,            Location::Variant;
}
