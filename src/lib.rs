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
//! - Branch-hierarchy tracking via [`Skeleton::strand_parents`]
//! - Bounded `[` recursion via [`TurtleConfig::max_stack_depth`]; overflow is
//!   reported on [`Skeleton::warnings`] as [`TurtleWarning::StackOverflow`]
//!   without corrupting downstream geometry
//!
//! ## Example
//!
//! ```
//! use symbios::{SymbiosState, SymbolTable};
//! use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};
//!
//! // Builder-style setup: interns the canonical turtle alphabet and
//! // registers each symbol's default operation in one call.
//! let mut interner = SymbolTable::new();
//! let interpreter = TurtleInterpreter::new(TurtleConfig::default())
//!     .with_standard_symbols(&mut interner);
//!
//! // Build an L-System state (normally produced by symbios expansion).
//! let mut state = SymbiosState::new();
//! let f_id = interner.resolve_id("F").unwrap();
//! state.push(f_id, 0.0, &[10.0]).unwrap(); // F(10)
//!
//! let skeleton = interpreter.build_skeleton(&state);
//! assert_eq!(skeleton.strands.len(), 1);
//! assert!(skeleton.warnings.is_empty());
//! ```

pub mod interpreter;
pub mod skeleton;
pub mod turtle;

pub use interpreter::{STANDARD_TURTLE_SYMBOLS, TurtleConfig, TurtleInterpreter};
pub use skeleton::{Skeleton, SkeletonPoint, SkeletonProp, TurtleWarning};
pub use turtle::{TurtleOp, TurtleState};
