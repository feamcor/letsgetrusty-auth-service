//! Shared boilerplate for the `Arc<dyn Trait>` newtype wrappers that let [`AppState`] stay
//! `Clone` while owning trait objects.
//!
//! Each store/client field in [`AppState`] is a thin newtype around `Arc<dyn SomeTrait>` so the
//! whole state can be cloned per request by bumping reference counts rather than the underlying
//! impl. The wrappers are otherwise identical, so [`arc_dyn_newtype`] generates them.
//!
//! [`AppState`]: crate::app_state::AppState

/// Generate an `Arc<dyn $trait>` newtype wrapper with `new`, `inner`, and a redacting `Debug`.
///
/// The generated type is `Clone` (cloning bumps the `Arc` refcount, not the impl), constructs
/// from any concrete `impl $trait + 'static`, and hands back a cloned trait object via `inner()`
/// so callers can invoke its async methods. `Debug` prints only the type name — the wrapped impl
/// often holds secrets (DB pools, Redis connections) that must not leak into logs.
macro_rules! arc_dyn_newtype {
    ($(#[$meta:meta])* $name:ident, $trait:ident) => {
        $(#[$meta])*
        #[derive(Clone)]
        pub struct $name {
            inner: std::sync::Arc<dyn $trait>,
        }

        impl $name {
            /// Wrap a concrete implementation behind a shared, cloneable handle.
            pub fn new(inner: impl $trait + 'static) -> Self {
                Self {
                    inner: std::sync::Arc::new(inner),
                }
            }

            /// Clone out the shared trait object to call its async methods.
            #[must_use]
            pub fn inner(&self) -> std::sync::Arc<dyn $trait> {
                self.inner.clone()
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }
    };
}

pub(crate) use arc_dyn_newtype;
