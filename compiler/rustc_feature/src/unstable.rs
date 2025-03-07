//! List of the unstable feature gates.

use super::{to_nonzero, Feature};

use rustc_data_structures::fx::FxHashSet;
use rustc_span::edition::Edition;
use rustc_span::symbol::{sym, Symbol};
use rustc_span::Span;

pub struct UnstableFeature {
    pub feature: Feature,
    pub set_enabled: fn(&mut Features),
}

#[derive(PartialEq)]
enum FeatureStatus {
    Default,
    Incomplete,
    Internal,
}

macro_rules! status_to_enum {
    (unstable) => {
        FeatureStatus::Default
    };
    (incomplete) => {
        FeatureStatus::Incomplete
    };
    (internal) => {
        FeatureStatus::Internal
    };
}

macro_rules! declare_features {
    ($(
        $(#[doc = $doc:tt])* ($status:ident, $feature:ident, $ver:expr, $issue:expr, $edition:expr),
    )+) => {
        /// Unstable language features that are being implemented or being
        /// considered for acceptance (stabilization) or removal.
        pub const UNSTABLE_FEATURES: &[UnstableFeature] = &[
            $(UnstableFeature {
                feature: Feature {
                    name: sym::$feature,
                    since: $ver,
                    issue: to_nonzero($issue),
                    edition: $edition,
                },
                // Sets this feature's corresponding bool within `features`.
                set_enabled: |features| features.$feature = true,
            }),+
        ];

        /// A set of features to be used by later passes.
        #[derive(Clone, Default, Debug)]
        pub struct Features {
            /// `#![feature]` attrs for language features, for error reporting.
            pub declared_lang_features: Vec<(Symbol, Span, Option<Symbol>)>,
            /// `#![feature]` attrs for non-language (library) features.
            pub declared_lib_features: Vec<(Symbol, Span)>,
            /// `declared_lang_features` + `declared_lib_features`.
            pub declared_features: FxHashSet<Symbol>,
            /// Active state of individual features (unstable only).
            $(
                $(#[doc = $doc])*
                pub $feature: bool
            ),+
        }

        impl Features {
            pub fn set_declared_lang_feature(
                &mut self,
                symbol: Symbol,
                span: Span,
                since: Option<Symbol>
            ) {
                self.declared_lang_features.push((symbol, span, since));
                self.declared_features.insert(symbol);
            }

            pub fn set_declared_lib_feature(&mut self, symbol: Symbol, span: Span) {
                self.declared_lib_features.push((symbol, span));
                self.declared_features.insert(symbol);
            }

            pub fn walk_feature_fields(&self, mut f: impl FnMut(&str, bool)) {
                $(f(stringify!($feature), self.$feature);)+
            }

            /// Is the given feature explicitly declared, i.e. named in a
            /// `#![feature(...)]` within the code?
            pub fn declared(&self, feature: Symbol) -> bool {
                self.declared_features.contains(&feature)
            }

            /// Is the given feature active, i.e. declared or automatically
            /// enabled due to the edition?
            ///
            /// Panics if the symbol doesn't correspond to a declared feature.
            pub fn active(&self, feature: Symbol) -> bool {
                match feature {
                    $( sym::$feature => self.$feature, )*

                    _ => panic!("`{}` was not listed in `declare_features`", feature),
                }
            }

            /// Some features are known to be incomplete and using them is likely to have
            /// unanticipated results, such as compiler crashes. We warn the user about these
            /// to alert them.
            pub fn incomplete(&self, feature: Symbol) -> bool {
                match feature {
                    $(
                        sym::$feature => status_to_enum!($status) == FeatureStatus::Incomplete,
                    )*
                    // Accepted/removed features aren't in this file but are never incomplete.
                    _ if self.declared_features.contains(&feature) => false,
                    _ => panic!("`{}` was not listed in `declare_features`", feature),
                }
            }

            /// Some features are internal to the compiler and standard library and should not
            /// be used in normal projects. We warn the user about these to alert them.
            pub fn internal(&self, feature: Symbol) -> bool {
                match feature {
                    $(
                        sym::$feature => status_to_enum!($status) == FeatureStatus::Internal,
                    )*
                    // Accepted/removed features aren't in this file but are never internal
                    // (a removed feature might have been internal, but that's now irrelevant).
                    _ if self.declared_features.contains(&feature) => false,
                    _ => panic!("`{}` was not listed in `declare_features`", feature),
                }
            }
        }
    };
}

// See https://rustc-dev-guide.rust-lang.org/feature-gates.html#feature-gates for more
// documentation about handling feature gates.
//
// If you change this, please modify `src/doc/unstable-book` as well.
//
// Don't ever remove anything from this list; move them to `accepted.rs` if
// accepted or `removed.rs` if removed.
//
// The version numbers here correspond to the version in which the current status
// was set.
//
// Note that the features are grouped into internal/user-facing and then
// sorted by version inside those groups. This is enforced with tidy.
//
// N.B., `tools/tidy/src/features.rs` parses this information directly out of the
// source, so take care when modifying it.

#[rustfmt::skip]
declare_features! (
    // -------------------------------------------------------------------------
    // feature-group-start: internal feature gates (no tracking issue)
    // -------------------------------------------------------------------------
    // no-tracking-issue-start

    /// Allows using the `unadjusted` ABI; perma-unstable.
    (unstable, abi_unadjusted, "1.16.0", None, None),
    /// Allows using the `vectorcall` ABI.
    (unstable, abi_vectorcall, "1.7.0", None, None),
    /// Allows using `#![needs_allocator]`, an implementation detail of `#[global_allocator]`.
    (internal, allocator_internals, "1.20.0", None, None),
    /// Allows using `#[allow_internal_unsafe]`. This is an
    /// attribute on `macro_rules!` and can't use the attribute handling
    /// below (it has to be checked before expansion possibly makes
    /// macros disappear).
    (internal, allow_internal_unsafe, "1.0.0", None, None),
    /// Allows using `#[allow_internal_unstable]`. This is an
    /// attribute on `macro_rules!` and can't use the attribute handling
    /// below (it has to be checked before expansion possibly makes
    /// macros disappear).
    (internal, allow_internal_unstable, "1.0.0", None, None),
    /// Allows using anonymous lifetimes in argument-position impl-trait.
    (unstable, anonymous_lifetime_in_impl_trait, "1.63.0", None, None),
    /// Allows identifying the `compiler_builtins` crate.
    (internal, compiler_builtins, "1.13.0", None, None),
    /// Allows writing custom MIR
    (internal, custom_mir, "1.65.0", None, None),
    /// Outputs useful `assert!` messages
    (unstable, generic_assert, "1.63.0", None, None),
    /// Allows using the `rust-intrinsic`'s "ABI".
    (internal, intrinsics, "1.0.0", None, None),
    /// Allows using `#[lang = ".."]` attribute for linking items to special compiler logic.
    (internal, lang_items, "1.0.0", None, None),
    /// Allows `#[link(..., cfg(..))]`; perma-unstable per #37406
    (unstable, link_cfg, "1.14.0", None, None),
    /// Allows the `multiple_supertrait_upcastable` lint.
    (unstable, multiple_supertrait_upcastable, "1.69.0", None, None),
    /// Allow negative trait bounds. This is an internal-only feature for testing the trait solver!
    (incomplete, negative_bounds, "1.71.0", None, None),
    /// Allows using `#[omit_gdb_pretty_printer_section]`.
    (internal, omit_gdb_pretty_printer_section, "1.5.0", None, None),
    /// Allows using `#[prelude_import]` on glob `use` items.
    (internal, prelude_import, "1.2.0", None, None),
    /// Used to identify crates that contain the profiler runtime.
    (internal, profiler_runtime, "1.18.0", None, None),
    /// Allows using `rustc_*` attributes (RFC 572).
    (internal, rustc_attrs, "1.0.0", None, None),
    /// Allows using the `#[stable]` and `#[unstable]` attributes.
    (internal, staged_api, "1.0.0", None, None),
    /// Added for testing E0705; perma-unstable.
    (internal, test_2018_feature, "1.31.0", None, Some(Edition::Edition2018)),
    /// Added for testing unstable lints; perma-unstable.
    (internal, test_unstable_lint, "1.60.0", None, None),
    /// Allows non-`unsafe` —and thus, unsound— access to `Pin` constructions.
    /// Marked `internal` since perma-unstable and unsound.
    (internal, unsafe_pin_internals, "1.60.0", None, None),
    /// Use for stable + negative coherence and strict coherence depending on trait's
    /// rustc_strict_coherence value.
    (unstable, with_negative_coherence, "1.60.0", None, None),
    // !!!!    !!!!    !!!!    !!!!   !!!!    !!!!    !!!!    !!!!    !!!!    !!!!    !!!!
    // Features are listed in alphabetical order. Tidy will fail if you don't keep it this way.
    // !!!!    !!!!    !!!!    !!!!   !!!!    !!!!    !!!!    !!!!    !!!!    !!!!    !!!!

    // no-tracking-issue-end
    // -------------------------------------------------------------------------
    // feature-group-end: internal feature gates (no tracking issue)
    // -------------------------------------------------------------------------

    // -------------------------------------------------------------------------
    // feature-group-start: internal feature gates
    // -------------------------------------------------------------------------

    /// Allows features specific to auto traits.
    /// Renamed from `optin_builtin_traits`.
    (unstable, auto_traits, "1.50.0", Some(13231), None),
    /// Allows using `box` in patterns (RFC 469).
    (unstable, box_patterns, "1.0.0", Some(29641), None),
    /// Allows `#[doc(notable_trait)]`.
    /// Renamed from `doc_spotlight`.
    (unstable, doc_notable_trait, "1.52.0", Some(45040), None),
    /// Allows using the `may_dangle` attribute (RFC 1327).
    (unstable, dropck_eyepatch, "1.10.0", Some(34761), None),
    /// Allows using the `#[fundamental]` attribute.
    (unstable, fundamental, "1.0.0", Some(29635), None),
    /// Allows using `#[link_name="llvm.*"]`.
    (internal, link_llvm_intrinsics, "1.0.0", Some(29602), None),
    /// Allows using the `#[linkage = ".."]` attribute.
    (unstable, linkage, "1.0.0", Some(29603), None),
    /// Allows declaring with `#![needs_panic_runtime]` that a panic runtime is needed.
    (internal, needs_panic_runtime, "1.10.0", Some(32837), None),
    /// Allows using the `#![panic_runtime]` attribute.
    (internal, panic_runtime, "1.10.0", Some(32837), None),
    /// Allows `extern "platform-intrinsic" { ... }`.
    (internal, platform_intrinsics, "1.4.0", Some(27731), None),
    /// Allows using `#[rustc_allow_const_fn_unstable]`.
    /// This is an attribute on `const fn` for the same
    /// purpose as `#[allow_internal_unstable]`.
    (internal, rustc_allow_const_fn_unstable, "1.49.0", Some(69399), None),
    /// Allows using compiler's own crates.
    (unstable, rustc_private, "1.0.0", Some(27812), None),
    /// Allows using internal rustdoc features like `doc(keyword)`.
    (internal, rustdoc_internals, "1.58.0", Some(90418), None),
    /// Allows using the `rustdoc::missing_doc_code_examples` lint
    (unstable, rustdoc_missing_doc_code_examples, "1.31.0", Some(101730), None),
    /// Allows using `#[start]` on a function indicating that it is the program entrypoint.
    (unstable, start, "1.0.0", Some(29633), None),
    /// Allows using `#[structural_match]` which indicates that a type is structurally matchable.
    /// FIXME: Subsumed by trait `StructuralPartialEq`, cannot move to removed until a library
    /// feature with the same name exists.
    (unstable, structural_match, "1.8.0", Some(31434), None),
    /// Allows using the `rust-call` ABI.
    (unstable, unboxed_closures, "1.0.0", Some(29625), None),
    // !!!!    !!!!    !!!!    !!!!   !!!!    !!!!    !!!!    !!!!    !!!!    !!!!    !!!!
    // Features are listed in alphabetical order. Tidy will fail if you don't keep it this way.
    // !!!!    !!!!    !!!!    !!!!   !!!!    !!!!    !!!!    !!!!    !!!!    !!!!    !!!!

    // -------------------------------------------------------------------------
    // feature-group-end: internal feature gates
    // -------------------------------------------------------------------------

    // -------------------------------------------------------------------------
    // feature-group-start: actual feature gates (target features)
    // -------------------------------------------------------------------------

    // FIXME: Document these and merge with the list below.

    // Unstable `#[target_feature]` directives.
    (unstable, aarch64_ver_target_feature, "1.27.0", Some(44839), None),
    (unstable, arm_target_feature, "1.27.0", Some(44839), None),
    (unstable, avx512_target_feature, "1.27.0", Some(44839), None),
    (unstable, bpf_target_feature, "1.54.0", Some(44839), None),
    (unstable, csky_target_feature, "1.73.0", Some(44839), None),
    (unstable, ermsb_target_feature, "1.49.0", Some(44839), None),
    (unstable, hexagon_target_feature, "1.27.0", Some(44839), None),
    (unstable, mips_target_feature, "1.27.0", Some(44839), None),
    (unstable, powerpc_target_feature, "1.27.0", Some(44839), None),
    (unstable, riscv_target_feature, "1.45.0", Some(44839), None),
    (unstable, rtm_target_feature, "1.35.0", Some(44839), None),
    (unstable, sse4a_target_feature, "1.27.0", Some(44839), None),
    (unstable, tbm_target_feature, "1.27.0", Some(44839), None),
    (unstable, wasm_target_feature, "1.30.0", Some(44839), None),
    // !!!!    !!!!    !!!!    !!!!   !!!!    !!!!    !!!!    !!!!    !!!!    !!!!    !!!!
    // Features are listed in alphabetical order. Tidy will fail if you don't keep it this way.
    // !!!!    !!!!    !!!!    !!!!   !!!!    !!!!    !!!!    !!!!    !!!!    !!!!    !!!!

    // -------------------------------------------------------------------------
    // feature-group-end: actual feature gates (target features)
    // -------------------------------------------------------------------------

    // -------------------------------------------------------------------------
    // feature-group-start: actual feature gates
    // -------------------------------------------------------------------------

    /// Allows using the `amdgpu-kernel` ABI.
    (unstable, abi_amdgpu_kernel, "1.29.0", Some(51575), None),
    /// Allows `extern "avr-interrupt" fn()` and `extern "avr-non-blocking-interrupt" fn()`.
    (unstable, abi_avr_interrupt, "1.45.0", Some(69664), None),
    /// Allows `extern "C-cmse-nonsecure-call" fn()`.
    (unstable, abi_c_cmse_nonsecure_call, "1.51.0", Some(81391), None),
    /// Allows `extern "msp430-interrupt" fn()`.
    (unstable, abi_msp430_interrupt, "1.16.0", Some(38487), None),
    /// Allows `extern "ptx-*" fn()`.
    (unstable, abi_ptx, "1.15.0", Some(38788), None),
    /// Allows `extern "riscv-interrupt-m" fn()` and `extern "riscv-interrupt-s" fn()`.
    (unstable, abi_riscv_interrupt, "1.73.0", Some(111889), None),
    /// Allows `extern "x86-interrupt" fn()`.
    (unstable, abi_x86_interrupt, "1.17.0", Some(40180), None),
    /// Allows additional const parameter types, such as `&'static str` or user defined types
    (incomplete, adt_const_params, "1.56.0", Some(95174), None),
    /// Allows defining an `#[alloc_error_handler]`.
    (unstable, alloc_error_handler, "1.29.0", Some(51540), None),
    /// Allows trait methods with arbitrary self types.
    (unstable, arbitrary_self_types, "1.23.0", Some(44874), None),
    /// Allows using `const` operands in inline assembly.
    (unstable, asm_const, "1.58.0", Some(93332), None),
    /// Enables experimental inline assembly support for additional architectures.
    (unstable, asm_experimental_arch, "1.58.0", Some(93335), None),
    /// Allows the `may_unwind` option in inline assembly.
    (unstable, asm_unwind, "1.58.0", Some(93334), None),
    /// Allows users to enforce equality of associated constants `TraitImpl<AssocConst=3>`.
    (unstable, associated_const_equality, "1.58.0", Some(92827), None),
    /// Allows the user of associated type bounds.
    (unstable, associated_type_bounds, "1.34.0", Some(52662), None),
    /// Allows associated type defaults.
    (unstable, associated_type_defaults, "1.2.0", Some(29661), None),
    /// Allows `async || body` closures.
    (unstable, async_closure, "1.37.0", Some(62290), None),
    /// Allows `#[track_caller]` on async functions.
    (unstable, async_fn_track_caller, "1.73.0", Some(110011), None),
    /// Allows builtin # foo() syntax
    (unstable, builtin_syntax, "1.71.0", Some(110680), None),
    /// Allows `c"foo"` literals.
    (unstable, c_str_literals, "1.71.0", Some(105723), None),
    /// Treat `extern "C"` function as nounwind.
    (unstable, c_unwind, "1.52.0", Some(74990), None),
    /// Allows using C-variadics.
    (unstable, c_variadic, "1.34.0", Some(44930), None),
    /// Allows the use of `#[cfg(overflow_checks)` to check if integer overflow behaviour.
    (unstable, cfg_overflow_checks, "1.71.0", Some(111466), None),
    /// Provides the relocation model information as cfg entry
    (unstable, cfg_relocation_model, "1.73.0", Some(114929), None),
    /// Allows the use of `#[cfg(sanitize = "option")]`; set when -Zsanitizer is used.
    (unstable, cfg_sanitize, "1.41.0", Some(39699), None),
    /// Allows `cfg(target_abi = "...")`.
    (unstable, cfg_target_abi, "1.55.0", Some(80970), None),
    /// Allows `cfg(target(abi = "..."))`.
    (unstable, cfg_target_compact, "1.63.0", Some(96901), None),
    /// Allows `cfg(target_has_atomic_load_store = "...")`.
    (unstable, cfg_target_has_atomic, "1.60.0", Some(94039), None),
    /// Allows `cfg(target_has_atomic_equal_alignment = "...")`.
    (unstable, cfg_target_has_atomic_equal_alignment, "1.60.0", Some(93822), None),
    /// Allows `cfg(target_thread_local)`.
    (unstable, cfg_target_thread_local, "1.7.0", Some(29594), None),
    /// Allow conditional compilation depending on rust version
    (unstable, cfg_version, "1.45.0", Some(64796), None),
    /// Allows to use the `#[cfi_encoding = ""]` attribute.
    (unstable, cfi_encoding, "1.71.0", Some(89653), None),
    /// Allows `for<...>` on closures and coroutines.
    (unstable, closure_lifetime_binder, "1.64.0", Some(97362), None),
    /// Allows `#[track_caller]` on closures and coroutines.
    (unstable, closure_track_caller, "1.57.0", Some(87417), None),
    /// Allows to use the `#[cmse_nonsecure_entry]` attribute.
    (unstable, cmse_nonsecure_entry, "1.48.0", Some(75835), None),
    /// Allows use of the `#[collapse_debuginfo]` attribute.
    (unstable, collapse_debuginfo, "1.65.0", Some(100758), None),
    /// Allows `async {}` expressions in const contexts.
    (unstable, const_async_blocks, "1.53.0", Some(85368), None),
    /// Allows `const || {}` closures in const contexts.
    (incomplete, const_closures, "1.68.0", Some(106003), None),
    /// Allows the definition of `const extern fn` and `const unsafe extern fn`.
    (unstable, const_extern_fn, "1.40.0", Some(64926), None),
    /// Allows basic arithmetic on floating point types in a `const fn`.
    (unstable, const_fn_floating_point_arithmetic, "1.48.0", Some(57241), None),
    /// Allows `for _ in _` loops in const contexts.
    (unstable, const_for, "1.56.0", Some(87575), None),
    /// Allows using `&mut` in constant functions.
    (unstable, const_mut_refs, "1.41.0", Some(57349), None),
    /// Be more precise when looking for live drops in a const context.
    (unstable, const_precise_live_drops, "1.46.0", Some(73255), None),
    /// Allows references to types with interior mutability within constants
    (unstable, const_refs_to_cell, "1.51.0", Some(80384), None),
    /// Allows `impl const Trait for T` syntax.
    (unstable, const_trait_impl, "1.42.0", Some(67792), None),
    /// Allows the `?` operator in const contexts.
    (unstable, const_try, "1.56.0", Some(74935), None),
    /// Allows coroutines to be cloned.
    (unstable, coroutine_clone, "1.65.0", Some(95360), None),
    /// Allows defining coroutines.
    (unstable, coroutines, "1.21.0", Some(43122), None),
    /// Allows function attribute `#[coverage(on/off)]`, to control coverage
    /// instrumentation of that function.
    (unstable, coverage_attribute, "1.74.0", Some(84605), None),
    /// Allows users to provide classes for fenced code block using `class:classname`.
    (unstable, custom_code_classes_in_docs, "1.74.0", Some(79483), None),
    /// Allows non-builtin attributes in inner attribute position.
    (unstable, custom_inner_attributes, "1.30.0", Some(54726), None),
    /// Allows custom test frameworks with `#![test_runner]` and `#[test_case]`.
    (unstable, custom_test_frameworks, "1.30.0", Some(50297), None),
    /// Allows declarative macros 2.0 (`macro`).
    (unstable, decl_macro, "1.17.0", Some(39412), None),
    /// Allows default type parameters to influence type inference.
    (unstable, default_type_parameter_fallback, "1.3.0", Some(27336), None),
    /// Allows using `#[deprecated_safe]` to deprecate the safeness of a function or trait
    (unstable, deprecated_safe, "1.61.0", Some(94978), None),
    /// Allows having using `suggestion` in the `#[deprecated]` attribute.
    (unstable, deprecated_suggestion, "1.61.0", Some(94785), None),
    /// Allows using the `#[diagnostic]` attribute tool namespace
    (unstable, diagnostic_namespace, "1.73.0", Some(111996), None),
    /// Controls errors in trait implementations.
    (unstable, do_not_recommend, "1.67.0", Some(51992), None),
    /// Tells rustdoc to automatically generate `#[doc(cfg(...))]`.
    (unstable, doc_auto_cfg, "1.58.0", Some(43781), None),
    /// Allows `#[doc(cfg(...))]`.
    (unstable, doc_cfg, "1.21.0", Some(43781), None),
    /// Allows `#[doc(cfg_hide(...))]`.
    (unstable, doc_cfg_hide, "1.57.0", Some(43781), None),
    /// Allows `#[doc(masked)]`.
    (unstable, doc_masked, "1.21.0", Some(44027), None),
    /// Allows `dyn* Trait` objects.
    (incomplete, dyn_star, "1.65.0", Some(102425), None),
    // Uses generic effect parameters for ~const bounds
    (unstable, effects, "1.72.0", Some(102090), None),
    /// Allows `X..Y` patterns.
    (unstable, exclusive_range_pattern, "1.11.0", Some(37854), None),
    /// Allows exhaustive pattern matching on types that contain uninhabited types.
    (unstable, exhaustive_patterns, "1.13.0", Some(51085), None),
    /// Allows explicit tail calls via `become` expression.
    (incomplete, explicit_tail_calls, "1.72.0", Some(112788), None),
    /// Allows using `efiapi`, `sysv64` and `win64` as calling convention
    /// for functions with varargs.
    (unstable, extended_varargs_abi_support, "1.65.0", Some(100189), None),
    /// Allows defining `extern type`s.
    (unstable, extern_types, "1.23.0", Some(43467), None),
    /// Allows the use of `#[ffi_const]` on foreign functions.
    (unstable, ffi_const, "1.45.0", Some(58328), None),
    /// Allows the use of `#[ffi_pure]` on foreign functions.
    (unstable, ffi_pure, "1.45.0", Some(58329), None),
    /// Allows using `#[ffi_returns_twice]` on foreign functions.
    (unstable, ffi_returns_twice, "1.34.0", Some(58314), None),
    /// Allows using `#[repr(align(...))]` on function items
    (unstable, fn_align, "1.53.0", Some(82232), None),
    /// Infer generic args for both consts and types.
    (unstable, generic_arg_infer, "1.55.0", Some(85077), None),
    /// An extension to the `generic_associated_types` feature, allowing incomplete features.
    (incomplete, generic_associated_types_extended, "1.61.0", Some(95451), None),
    /// Allows non-trivial generic constants which have to have wfness manually propagated to callers
    (incomplete, generic_const_exprs, "1.56.0", Some(76560), None),
    /// Allows generic parameters and where-clauses on free & associated const items.
    (incomplete, generic_const_items, "1.73.0", Some(113521), None),
    /// Allows using `..=X` as a patterns in slices.
    (unstable, half_open_range_patterns_in_slices, "1.66.0", Some(67264), None),
    /// Allows `if let` guard in match arms.
    (unstable, if_let_guard, "1.47.0", Some(51114), None),
    /// Allows `impl Trait` to be used inside associated types (RFC 2515).
    (unstable, impl_trait_in_assoc_type, "1.70.0", Some(63063), None),
    /// Allows `impl Trait` as output type in `Fn` traits in return position of functions.
    (unstable, impl_trait_in_fn_trait_return, "1.64.0", Some(99697), None),
    /// Allows using imported `main` function
    (unstable, imported_main, "1.53.0", Some(28937), None),
    /// Allows associated types in inherent impls.
    (incomplete, inherent_associated_types, "1.52.0", Some(8995), None),
    /// Allow anonymous constants from an inline `const` block
    (unstable, inline_const, "1.49.0", Some(76001), None),
    /// Allow anonymous constants from an inline `const` block in pattern position
    (incomplete, inline_const_pat, "1.58.0", Some(76001), None),
    /// Allows using `pointer` and `reference` in intra-doc links
    (unstable, intra_doc_pointers, "1.51.0", Some(80896), None),
    // Allows setting the threshold for the `large_assignments` lint.
    (unstable, large_assignments, "1.52.0", Some(83518), None),
    /// Allow to have type alias types for inter-crate use.
    (incomplete, lazy_type_alias, "1.72.0", Some(112792), None),
    /// Allows `if/while p && let q = r && ...` chains.
    (unstable, let_chains, "1.37.0", Some(53667), None),
    /// Allows using `reason` in lint attributes and the `#[expect(lint)]` lint check.
    (unstable, lint_reasons, "1.31.0", Some(54503), None),
    /// Give access to additional metadata about declarative macro meta-variables.
    (unstable, macro_metavar_expr, "1.61.0", Some(83527), None),
    /// Allows `#[marker]` on certain traits allowing overlapping implementations.
    (unstable, marker_trait_attr, "1.30.0", Some(29864), None),
    /// A minimal, sound subset of specialization intended to be used by the
    /// standard library until the soundness issues with specialization
    /// are fixed.
    (unstable, min_specialization, "1.7.0", Some(31844), None),
    /// Allows qualified paths in struct expressions, struct patterns and tuple struct patterns.
    (unstable, more_qualified_paths, "1.54.0", Some(86935), None),
    /// Allows the `#[must_not_suspend]` attribute.
    (unstable, must_not_suspend, "1.57.0", Some(83310), None),
    /// Allows using `#[naked]` on functions.
    (unstable, naked_functions, "1.9.0", Some(32408), None),
    /// Allows specifying the as-needed link modifier
    (unstable, native_link_modifiers_as_needed, "1.53.0", Some(81490), None),
    /// Allow negative trait implementations.
    (unstable, negative_impls, "1.44.0", Some(68318), None),
    /// Allows the `!` type. Does not imply 'exhaustive_patterns' (below) any more.
    (unstable, never_type, "1.13.0", Some(35121), None),
    /// Allows diverging expressions to fall back to `!` rather than `()`.
    (unstable, never_type_fallback, "1.41.0", Some(65992), None),
    /// Allows `#![no_core]`.
    (unstable, no_core, "1.3.0", Some(29639), None),
    /// Allows the use of `no_sanitize` attribute.
    (unstable, no_sanitize, "1.42.0", Some(39699), None),
    /// Allows using the `non_exhaustive_omitted_patterns` lint.
    (unstable, non_exhaustive_omitted_patterns_lint, "1.57.0", Some(89554), None),
    /// Allows `for<T>` binders in where-clauses
    (incomplete, non_lifetime_binders, "1.69.0", Some(108185), None),
    /// Allows making `dyn Trait` well-formed even if `Trait` is not object safe.
    /// In that case, `dyn Trait: Trait` does not hold. Moreover, coercions and
    /// casts in safe Rust to `dyn Trait` for such a `Trait` is also forbidden.
    (unstable, object_safe_for_dispatch, "1.40.0", Some(43561), None),
    /// Allows using `#[optimize(X)]`.
    (unstable, optimize_attribute, "1.34.0", Some(54882), None),
    /// Allows using `#![plugin(myplugin)]`.
    (unstable, plugin, "1.0.0", Some(29597), None),
    /// Allows exhaustive integer pattern matching on `usize` and `isize`.
    (unstable, precise_pointer_size_matching, "1.32.0", Some(56354), None),
    /// Allows macro attributes on expressions, statements and non-inline modules.
    (unstable, proc_macro_hygiene, "1.30.0", Some(54727), None),
    /// Allows `&raw const $place_expr` and `&raw mut $place_expr` expressions.
    (unstable, raw_ref_op, "1.41.0", Some(64490), None),
    /// Allows using the `#[register_tool]` attribute.
    (unstable, register_tool, "1.41.0", Some(66079), None),
    /// Allows the `#[repr(i128)]` attribute for enums.
    (incomplete, repr128, "1.16.0", Some(56071), None),
    /// Allows `repr(simd)` and importing the various simd intrinsics.
    (unstable, repr_simd, "1.4.0", Some(27731), None),
    /// Allows bounding the return type of AFIT/RPITIT.
    (incomplete, return_type_notation, "1.70.0", Some(109417), None),
    /// Allows `extern "rust-cold"`.
    (unstable, rust_cold_cc, "1.63.0", Some(97544), None),
    /// Allows the use of SIMD types in functions declared in `extern` blocks.
    (unstable, simd_ffi, "1.0.0", Some(27731), None),
    /// Allows specialization of implementations (RFC 1210).
    (incomplete, specialization, "1.7.0", Some(31844), None),
    /// Allows attributes on expressions and non-item statements.
    (unstable, stmt_expr_attributes, "1.6.0", Some(15701), None),
    /// Allows lints part of the strict provenance effort.
    (unstable, strict_provenance, "1.61.0", Some(95228), None),
    /// Allows string patterns to dereference values to match them.
    (unstable, string_deref_patterns, "1.67.0", Some(87121), None),
    /// Allows the use of `#[target_feature]` on safe functions.
    (unstable, target_feature_11, "1.45.0", Some(69098), None),
    /// Allows using `#[thread_local]` on `static` items.
    (unstable, thread_local, "1.0.0", Some(29594), None),
    /// Allows defining `trait X = A + B;` alias items.
    (unstable, trait_alias, "1.24.0", Some(41517), None),
    /// Allows dyn upcasting trait objects via supertraits.
    /// Dyn upcasting is casting, e.g., `dyn Foo -> dyn Bar` where `Foo: Bar`.
    (unstable, trait_upcasting, "1.56.0", Some(65991), None),
    /// Allows for transmuting between arrays with sizes that contain generic consts.
    (unstable, transmute_generic_consts, "1.70.0", Some(109929), None),
    /// Allows #[repr(transparent)] on unions (RFC 2645).
    (unstable, transparent_unions, "1.37.0", Some(60405), None),
    /// Allows inconsistent bounds in where clauses.
    (unstable, trivial_bounds, "1.28.0", Some(48214), None),
    /// Allows using `try {...}` expressions.
    (unstable, try_blocks, "1.29.0", Some(31436), None),
    /// Allows `impl Trait` to be used inside type aliases (RFC 2515).
    (unstable, type_alias_impl_trait, "1.38.0", Some(63063), None),
    /// Allows the use of type ascription in expressions.
    (unstable, type_ascription, "1.6.0", Some(23416), None),
    /// Allows creation of instances of a struct by moving fields that have
    /// not changed from prior instances of the same struct (RFC #2528)
    (unstable, type_changing_struct_update, "1.58.0", Some(86555), None),
    /// Allows using type privacy lints (`private_interfaces`, `private_bounds`, `unnameable_types`).
    (unstable, type_privacy_lints, "1.72.0", Some(48054), None),
    /// Enables rustc to generate code that instructs libstd to NOT ignore SIGPIPE.
    (unstable, unix_sigpipe, "1.65.0", Some(97889), None),
    /// Allows unnamed fields of struct and union type
    (incomplete, unnamed_fields, "1.74.0", Some(49804), None),
    /// Allows unsized fn parameters.
    (unstable, unsized_fn_params, "1.49.0", Some(48055), None),
    /// Allows unsized rvalues at arguments and parameters.
    (incomplete, unsized_locals, "1.30.0", Some(48055), None),
    /// Allows unsized tuple coercion.
    (unstable, unsized_tuple_coercion, "1.20.0", Some(42877), None),
    /// Allows using the `#[used(linker)]` (or `#[used(compiler)]`) attribute.
    (unstable, used_with_arg, "1.60.0", Some(93798), None),
    /// Allows `extern "wasm" fn`
    (unstable, wasm_abi, "1.53.0", Some(83788), None),
    /// Allows `do yeet` expressions
    (unstable, yeet_expr, "1.62.0", Some(96373), None),
    // !!!!    !!!!    !!!!    !!!!   !!!!    !!!!    !!!!    !!!!    !!!!    !!!!    !!!!
    // Features are listed in alphabetical order. Tidy will fail if you don't keep it this way.
    // !!!!    !!!!    !!!!    !!!!   !!!!    !!!!    !!!!    !!!!    !!!!    !!!!    !!!!

    // -------------------------------------------------------------------------
    // feature-group-end: actual feature gates
    // -------------------------------------------------------------------------
);

/// Some features are not allowed to be used together at the same time, if
/// the two are present, produce an error.
///
/// Currently empty, but we will probably need this again in the future,
/// so let's keep it in for now.
pub const INCOMPATIBLE_FEATURES: &[(Symbol, Symbol)] = &[];
