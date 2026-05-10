//! Interpreter that converts L-System symbols into 3D turtle movements.

use crate::skeleton::{Skeleton, SkeletonPoint, TurtleWarning};
use crate::turtle::{TurtleOp, TurtleState};
use glam::{Mat3, Quat, Vec3, Vec4};
use std::collections::HashMap;
use std::f32::consts::PI;
use symbios::{SymbiosState, SymbolTable};

/// Symbol IDs strictly below this bound use the dense [`Vec`] tier of
/// [`OpMap`]; IDs at or above it spill into the sparse [`HashMap`] tier.
const OP_MAP_DENSE_CAPACITY: usize = 1024;

/// Tiered symbol-id → [`TurtleOp`] lookup.
///
/// Hot path: a `Vec` indexed directly by symbol id, capped at the
/// dense-tier capacity (1024) so a stray high id can't force the table to
/// grow into hundreds of KB of `TurtleOp::Ignore` padding. Symbol ids at or
/// above the cap fall through to a `HashMap` — slower but bounded by the
/// number of *registered* high ids, not their numeric magnitude.
#[derive(Clone, Debug, Default)]
struct OpMap {
    dense: Vec<TurtleOp>,
    sparse: HashMap<u16, TurtleOp>,
}

impl OpMap {
    fn new() -> Self {
        Self::default()
    }

    fn get(&self, sym_id: u16) -> TurtleOp {
        let idx = sym_id as usize;
        if idx < OP_MAP_DENSE_CAPACITY {
            self.dense.get(idx).copied().unwrap_or(TurtleOp::Ignore)
        } else {
            self.sparse
                .get(&sym_id)
                .copied()
                .unwrap_or(TurtleOp::Ignore)
        }
    }

    fn set(&mut self, sym_id: u16, op: TurtleOp) {
        let idx = sym_id as usize;
        if idx < OP_MAP_DENSE_CAPACITY {
            if idx >= self.dense.len() {
                self.dense.resize(idx + 1, TurtleOp::Ignore);
            }
            self.dense[idx] = op;
        } else {
            self.sparse.insert(sym_id, op);
        }
    }
}

/// Canonical set of L-System turtle symbols this crate maps to
/// [`TurtleOp`]s by default.
///
/// Exposed so consumers can audit which symbols
/// [`TurtleInterpreter::populate_standard_symbols`] and
/// [`TurtleInterpreter::with_standard_symbols`] register.
pub const STANDARD_TURTLE_SYMBOLS: &[&str] = &[
    "F", "f", "+", "-", "&", "^", "\\", "/", "|", "$", "!", "[", "]", "~", "'", ",", ";",
];

/// Configuration for turtle interpretation.
#[derive(Clone, Debug)]
pub struct TurtleConfig {
    /// Default step length for Draw/Move when no parameter is given.
    pub default_step: f32,
    /// Default rotation angle (in radians) for Yaw/Pitch/Roll.
    pub default_angle: f32,
    /// Initial stroke width.
    pub initial_width: f32,
    /// Optional tropism vector (e.g., gravity direction for plant growth).
    pub tropism: Option<Vec3>,
    /// Tropism elasticity - how strongly the turtle bends toward tropism vector.
    pub elasticity: f32,
    /// Maximum stack depth for push/pop operations.
    ///
    /// Prevents denial-of-service via infinite recursion (e.g., `A -> [ A ]`).
    /// Push operations are ignored when this limit is reached.
    pub max_stack_depth: usize,
}

impl Default for TurtleConfig {
    fn default() -> Self {
        Self {
            default_step: 1.0,
            default_angle: 45.0f32.to_radians(),
            initial_width: 0.1,
            tropism: None,
            elasticity: 0.0,
            max_stack_depth: 1024,
        }
    }
}

/// Interprets L-System output as 3D turtle graphics, producing a [`Skeleton`].
///
/// Maps symbol IDs to [`TurtleOp`]s using a tiered lookup: a dense `Vec`
/// for ids below the dense-tier capacity (1024) — the common case, O(1)
/// indexing — with a `HashMap` fallback for sparse high ids. Memory stays
/// bounded even when consumers register single symbols at id 10_000+.
pub struct TurtleInterpreter {
    op_map: OpMap,
    config: TurtleConfig,
}

impl TurtleInterpreter {
    /// Creates a new interpreter with the given configuration.
    pub fn new(config: TurtleConfig) -> Self {
        Self {
            op_map: OpMap::new(),
            config,
        }
    }

    /// Builder method to seed the dense tier of the operation map from a Vec.
    ///
    /// `map[i]` becomes the operation for symbol id `i`. The Vec is truncated
    /// at the dense-tier capacity (1024); entries beyond that bound are
    /// dropped — register them via [`Self::set_op`] instead so they land in
    /// the sparse tier.
    pub fn with_map(mut self, map: Vec<TurtleOp>) -> Self {
        let len = map.len().min(OP_MAP_DENSE_CAPACITY);
        self.op_map.dense = map.into_iter().take(len).collect();
        self
    }

    /// Builder method that interns every symbol in [`STANDARD_TURTLE_SYMBOLS`]
    /// into `interner` and registers each one's default [`TurtleOp`].
    ///
    /// Eliminates the per-consumer boilerplate of interning the canonical
    /// turtle alphabet one symbol at a time before calling
    /// [`Self::populate_standard_symbols`].
    ///
    /// Already-interned symbols retain their existing IDs — calling this on a
    /// pre-populated `SymbolTable` is safe and idempotent.
    ///
    /// # Example
    /// ```
    /// use symbios::SymbolTable;
    /// use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};
    ///
    /// let mut interner = SymbolTable::new();
    /// let interpreter = TurtleInterpreter::new(TurtleConfig::default())
    ///     .with_standard_symbols(&mut interner);
    /// assert!(interner.resolve_id("F").is_some());
    /// # let _ = interpreter;
    /// ```
    pub fn with_standard_symbols(mut self, interner: &mut SymbolTable) -> Self {
        for sym in STANDARD_TURTLE_SYMBOLS {
            let _ = interner.intern(sym);
        }
        self.populate_standard_symbols(interner);
        self
    }

    /// Maps a symbol ID to a turtle operation.
    ///
    /// Symbol ids below the dense-tier capacity (1024) are stored in the
    /// dense `Vec` tier (with intermediate slots filled by `TurtleOp::Ignore`);
    /// ids at or above the cap go to the sparse `HashMap` tier so they don't
    /// force the `Vec` to grow proportionally to the id's magnitude.
    pub fn set_op(&mut self, sym_id: u16, op: TurtleOp) {
        self.op_map.set(sym_id, op);
    }

    /// Populates the operation map with standard L-System symbols from a symbol table.
    ///
    /// Maps: `F`, `f`, `+`, `-`, `&`, `^`, `\`, `/`, `|`, `$`, `!`, `[`, `]`, `~`,
    /// and material symbols: `'`, `,`, `;`.
    pub fn populate_standard_symbols(&mut self, interner: &SymbolTable) {
        let mappings = [
            ("F", TurtleOp::Draw),
            ("f", TurtleOp::Move),
            ("+", TurtleOp::Yaw(1.0)),
            ("-", TurtleOp::Yaw(-1.0)),
            ("&", TurtleOp::Pitch(1.0)),
            ("^", TurtleOp::Pitch(-1.0)),
            ("\\", TurtleOp::Roll(1.0)),
            ("/", TurtleOp::Roll(-1.0)),
            ("|", TurtleOp::TurnAround),
            ("$", TurtleOp::Vertical),
            ("!", TurtleOp::SetWidth),
            ("[", TurtleOp::Push),
            ("]", TurtleOp::Pop),
            ("~", TurtleOp::Spawn(0)),
            // Material Mappings
            ("'", TurtleOp::SetColor),
            (",", TurtleOp::SetMaterial),
            (";", TurtleOp::SetUVScale),
        ];

        for (sym, op) in mappings {
            if let Some(id) = interner.resolve_id(sym) {
                self.set_op(id, op);
            }
        }
    }

    /// Interprets a [`SymbiosState`] and builds a [`Skeleton`] from turtle movements.
    ///
    /// Iterates through all symbols in the state, executing the corresponding
    /// turtle operations and accumulating geometry into the skeleton.
    pub fn build_skeleton(&self, state: &SymbiosState) -> Skeleton {
        let mut skeleton = Skeleton::new();
        let mut turtle = TurtleState {
            width: self.config.initial_width,
            ..Default::default()
        };
        let mut stack = Vec::new();
        // Counts pushes that were dropped due to max_stack_depth. The next
        // `skipped_pushes` Pops are skipped so the stack stays in sync.
        let mut skipped_pushes: usize = 0;
        // Parent strand index recorded at each successful Push, used to set
        // parent linkage on both the branch strand and the post-Pop trunk
        // resume strand. Parallel to `stack`; len() always matches.
        let mut branch_parent_stack: Vec<Option<usize>> = Vec::new();

        for i in 0..state.len() {
            let view = match state.get_view(i) {
                Some(v) => v,
                None => break,
            };

            let op = self.op_map.get(view.sym);
            // Helper to get param at index with default
            let p = |idx: usize, def: f32| -> f32 {
                view.params.get(idx).map(|&x| x as f32).unwrap_or(def)
            };
            let p0 = p(0, 0.0); // Common case helper
            let get_val =
                |default: f32| -> f32 { view.params.first().map(|&x| x as f32).unwrap_or(default) };

            match op {
                TurtleOp::Draw | TurtleOp::Move => {
                    let len = get_val(self.config.default_step);
                    let is_move = matches!(op, TurtleOp::Move);

                    if skeleton.strands.is_empty() {
                        skeleton.start_strand(
                            SkeletonPoint {
                                position: turtle.position,
                                rotation: turtle.rotation,
                                radius: turtle.width / 2.0,
                                color: turtle.color,
                                material_id: turtle.material_id,
                                uv_scale: turtle.uv_scale,
                            },
                            None,
                        );
                    }

                    if !is_move {
                        turtle.position += turtle.up() * len;

                        if let Some(t_vec) = self.config.tropism
                            && self.config.elasticity > 0.0
                        {
                            let head = turtle.up();
                            let h_cross_t = head.cross(t_vec);
                            let mag = h_cross_t.length();
                            if mag > 0.0001 {
                                let angle = self.config.elasticity * mag;
                                let axis = h_cross_t.normalize();
                                turtle.rotate_axis(axis, angle);
                            }
                        }
                    } else {
                        turtle.position += turtle.up() * len;
                    }

                    let pt = SkeletonPoint {
                        position: turtle.position,
                        rotation: turtle.rotation,
                        radius: turtle.width / 2.0,
                        color: turtle.color,
                        material_id: turtle.material_id,
                        uv_scale: turtle.uv_scale,
                    };
                    if is_move {
                        // Move breaks the strand: the new strand sprouts from
                        // wherever the turtle was just drawing.
                        let parent = skeleton.strands.len().checked_sub(1);
                        skeleton.start_strand(pt, parent);
                    } else {
                        skeleton.push_node(pt);
                    }
                }
                TurtleOp::Yaw(sign) => {
                    let angle = get_val(self.config.default_angle.to_degrees()).to_radians() * sign;
                    turtle.rotate_local_z(angle);
                }
                TurtleOp::Pitch(sign) => {
                    let angle = get_val(self.config.default_angle.to_degrees()).to_radians() * sign;
                    turtle.rotate_local_x(angle);
                }
                TurtleOp::Roll(sign) => {
                    let angle = get_val(self.config.default_angle.to_degrees()).to_radians() * sign;
                    turtle.rotate_local_y(angle);
                }
                TurtleOp::TurnAround => {
                    turtle.rotate_local_z(PI);
                }
                TurtleOp::Vertical => {
                    let h = turtle.up();
                    let v = Vec3::Y;
                    let l = v.cross(h).normalize_or_zero();
                    if l.length_squared() > 0.001 {
                        let u = h.cross(l).normalize();
                        let rot_matrix = Mat3::from_cols(-l, h, u);
                        turtle.rotation = Quat::from_mat3(&rot_matrix);
                    }
                }
                TurtleOp::SetWidth => {
                    turtle.width = get_val(turtle.width);
                }
                TurtleOp::SetColor => {
                    // Logic: Supports 1 arg (Grayscale), 3 args (RGB), 4 args (RGBA)
                    let count = view.params.len();
                    match count {
                        1 => turtle.color = Vec4::new(p0, p0, p0, 1.0),
                        3 => turtle.color = Vec4::new(p(0, 0.), p(1, 0.), p(2, 0.), 1.0),
                        4 => turtle.color = Vec4::new(p(0, 0.), p(1, 0.), p(2, 0.), p(3, 1.)),
                        _ => {} // No change if no params
                    }
                }
                TurtleOp::SetMaterial => {
                    turtle.material_id = p0 as u8;
                }
                TurtleOp::SetUVScale => {
                    turtle.uv_scale = get_val(1.0).max(0.0);
                }
                TurtleOp::Push => {
                    if stack.len() >= self.config.max_stack_depth {
                        skipped_pushes += 1;
                        skeleton
                            .warnings
                            .push(TurtleWarning::StackOverflow { symbol_index: i });
                        continue;
                    }
                    stack.push(turtle);
                    // The branch we're about to draw sprouts from whichever
                    // strand the turtle was last on. Record the parent for
                    // both this branch strand AND the trunk-resume strand
                    // that the matching Pop will create.
                    let parent = skeleton.strands.len().checked_sub(1);
                    branch_parent_stack.push(parent);
                    skeleton.start_strand(
                        SkeletonPoint {
                            position: turtle.position,
                            rotation: turtle.rotation,
                            radius: turtle.width / 2.0,
                            color: turtle.color,
                            material_id: turtle.material_id,
                            uv_scale: turtle.uv_scale,
                        },
                        parent,
                    );
                }
                TurtleOp::Pop => {
                    if skipped_pushes > 0 {
                        skipped_pushes -= 1;
                        continue;
                    }
                    if let Some(saved_state) = stack.pop() {
                        turtle = saved_state;
                        // Trunk resumes from the same parent the branch
                        // sprouted from. If the branch_parent_stack is empty
                        // (defensive — would mean stack/branch_parent drift)
                        // fall back to None.
                        let parent = branch_parent_stack.pop().unwrap_or(None);
                        skeleton.start_strand(
                            SkeletonPoint {
                                position: turtle.position,
                                rotation: turtle.rotation,
                                radius: turtle.width / 2.0,
                                color: turtle.color,
                                material_id: turtle.material_id,
                                uv_scale: turtle.uv_scale,
                            },
                            parent,
                        );
                    }
                }
                TurtleOp::Spawn(default_id) => {
                    // Param layout for `~`:
                    //   0 args:        default prop, scale = 1
                    //   1 arg:         (prop_id), scale = 1
                    //   2 or 3 args:   (prop_id, s [, _]) — uniform scale s
                    //   4+ args:       (prop_id, sx, sy, sz) — per-axis scale
                    // Extra args beyond 4 are ignored. The 2-arg form preserves
                    // pre-#20 behavior so existing L-Systems keep working.
                    let prop_id = view.params.first().map(|&x| x as u16).unwrap_or(default_id);
                    let scale = match view.params.len() {
                        0 | 1 => Vec3::ONE,
                        n if n >= 4 => Vec3::new(
                            view.params[1] as f32,
                            view.params[2] as f32,
                            view.params[3] as f32,
                        ),
                        _ => Vec3::splat(view.params[1] as f32),
                    };

                    skeleton.add_prop(crate::skeleton::SkeletonProp {
                        prop_id,
                        position: turtle.position,
                        rotation: turtle.rotation,
                        scale,
                        color: turtle.color,
                        material_id: turtle.material_id,
                    });
                }
                TurtleOp::Ignore => {}
            }
        }
        skeleton
    }
}
