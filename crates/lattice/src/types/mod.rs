//! Core type definitions for Lattice
//!
//! This module contains the IR types (Class, Enum, Field, FieldType, Value)
//! that form the foundation of Lattice's type system.

pub mod ir;
pub mod checker;

pub use ir::{Class, Enum, Field, FieldType, Function, Value, IR};
pub use checker::{Type, TypeChecker, TypeContext, TypeError};
