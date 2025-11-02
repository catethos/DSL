//! Matcher implementations using external crates

#[cfg(feature = "fuzzy")]
pub mod nucleo;

#[cfg(feature = "fuzzy")]
pub use self::nucleo::NucleoMatcher;
