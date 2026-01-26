//! # symbios-turtle-3d
//!
//! A spatial interpretation layer for [Symbios](https://crates.io/crates/symbios) L-Systems
//! using [glam](https://crates.io/crates/glam) for 3D math.
//!
//! This crate provides a 3D turtle graphics interpreter that converts L-System
//! symbol sequences into geometric skeletons suitable for mesh generation.
//!
//! ## Features
//!
//! - Standard L-System turtle operations (draw, move, rotate, branch)
//! - Palette-based material system with per-segment color, material ID, and UV scale
//! - Tropism support for natural plant-like growth
//! - Prop spawning for discrete objects (leaves, flowers)
//!
//! ## Example
//!
//! ```ignore
//! use symbios::{SymbiosState, SymbolTable};
//! use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};
//!
//! let mut interner = SymbolTable::new();
//! let mut interpreter = TurtleInterpreter::new(TurtleConfig::default());
//!
//! // Intern symbols and populate standard mappings
//! interner.intern("F").unwrap();
//! interpreter.populate_standard_symbols(&interner);
//!
//! // Build skeleton from L-System state
//! let state = SymbiosState::new();
//! let skeleton = interpreter.build_skeleton(&state);
//! ```

pub mod interpreter;
pub mod skeleton;
pub mod turtle;

pub use interpreter::{TurtleConfig, TurtleInterpreter};
pub use skeleton::{Skeleton, SkeletonPoint};
pub use turtle::{TurtleOp, TurtleState};
