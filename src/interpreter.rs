use crate::skeleton::{Skeleton, SkeletonPoint};
use crate::turtle::{TurtleOp, TurtleState};
use glam::{Mat3, Quat, Vec3, Vec4};
use std::collections::HashMap;
use std::f32::consts::PI;
use symbios::{SymbiosState, SymbolTable};

#[derive(Clone, Debug)]
pub struct TurtleConfig {
    pub default_step: f32,
    pub default_angle: f32,
    pub initial_width: f32,
    pub tropism: Option<Vec3>,
    pub elasticity: f32,
}

impl Default for TurtleConfig {
    fn default() -> Self {
        Self {
            default_step: 1.0,
            default_angle: 45.0f32.to_radians(),
            initial_width: 0.1,
            tropism: None,
            elasticity: 0.0,
        }
    }
}

pub struct TurtleInterpreter {
    op_map: HashMap<u16, TurtleOp>,
    config: TurtleConfig,
}

impl TurtleInterpreter {
    pub fn new(config: TurtleConfig) -> Self {
        Self {
            op_map: HashMap::new(),
            config,
        }
    }

    pub fn with_map(mut self, map: HashMap<u16, TurtleOp>) -> Self {
        self.op_map = map;
        self
    }

    pub fn set_op(&mut self, sym_id: u16, op: TurtleOp) {
        self.op_map.insert(sym_id, op);
    }

    pub fn populate_standard_symbols(&mut self, interner: &SymbolTable) {
        let mappings = [
            ("F", TurtleOp::Draw),
            ("f", TurtleOp::Move),
            ("+", TurtleOp::Yaw(1.0)),
            ("-", TurtleOp::Yaw(-1.0)),
            ("&", TurtleOp::Pitch(1.0)),
            ("^", TurtleOp::Pitch(-1.0)),
            ("\\\\", TurtleOp::Roll(1.0)),
            ("/", TurtleOp::Roll(-1.0)),
            ("|", TurtleOp::TurnAround),
            ("$", TurtleOp::Vertical),
            ("!", TurtleOp::SetWidth),
            ("[", TurtleOp::Push),
            ("]", TurtleOp::Pop),
            ("~", TurtleOp::Spawn(0)),
            // PBR Mappings
            ("'", TurtleOp::SetColor),
            (",", TurtleOp::SetMaterial),
            ("#", TurtleOp::SetRoughness),
            ("@", TurtleOp::SetMetallic),
            (";", TurtleOp::SetTexture),
        ];

        for (sym, op) in mappings {
            if let Some(id) = interner.resolve_id(sym) {
                self.op_map.insert(id, op);
            }
        }
    }

    pub fn build_skeleton(&self, state: &SymbiosState) -> Skeleton {
        let mut skeleton = Skeleton::new();
        let mut turtle = TurtleState {
            width: self.config.initial_width,
            ..Default::default()
        };
        let mut stack = Vec::new();

        for i in 0..state.len() {
            let view = match state.get_view(i) {
                Some(v) => v,
                None => break,
            };

            let op = self.op_map.get(&view.sym).unwrap_or(&TurtleOp::Ignore);
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

                    // Logic for Tropism and Movement (same as before)...
                    // ... [Truncated for brevity, assuming standard move logic] ...

                    if skeleton.strands.is_empty() {
                        skeleton.add_node(
                            SkeletonPoint {
                                position: turtle.position,
                                rotation: turtle.rotation,
                                radius: turtle.width / 2.0,
                                color: turtle.color,
                                material_id: turtle.material_id,
                                roughness: turtle.roughness,
                                metallic: turtle.metallic,
                            },
                            true,
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

                    // Push Node with FULL STATE
                    skeleton.add_node(
                        SkeletonPoint {
                            position: turtle.position,
                            rotation: turtle.rotation,
                            radius: turtle.width / 2.0,
                            color: turtle.color,
                            material_id: turtle.material_id,
                            roughness: turtle.roughness,
                            metallic: turtle.metallic,
                        },
                        is_move, // Force new strand if this was a Move
                    );
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
                TurtleOp::SetRoughness => {
                    turtle.roughness = p0.clamp(0.0, 1.0);
                }
                TurtleOp::SetMetallic => {
                    turtle.metallic = p0.clamp(0.0, 1.0);
                }
                TurtleOp::SetTexture => {
                    turtle.texture_id = p0 as u16;
                }
                TurtleOp::Push => {
                    stack.push(turtle);
                    // Explicitly break the strand on Push to isolate the branch
                    skeleton.add_node(
                        SkeletonPoint {
                            position: turtle.position,
                            rotation: turtle.rotation,
                            radius: turtle.width / 2.0,
                            color: turtle.color,
                            material_id: turtle.material_id,
                            roughness: turtle.roughness,
                            metallic: turtle.metallic,
                        },
                        true,
                    );
                }
                TurtleOp::Pop => {
                    if let Some(saved_state) = stack.pop() {
                        turtle = saved_state;
                        skeleton.add_node(
                            SkeletonPoint {
                                position: turtle.position,
                                rotation: turtle.rotation,
                                radius: turtle.width / 2.0,
                                color: turtle.color,
                                material_id: turtle.material_id,
                                roughness: turtle.roughness,
                                metallic: turtle.metallic,
                            },
                            true,
                        );
                    }
                }
                TurtleOp::Spawn(default_id) => {
                    let surface_id = view
                        .params
                        .first()
                        .map(|&x| x as u16)
                        .unwrap_or(*default_id);
                    let scale_scalar = view.params.get(1).map(|&x| x as f32).unwrap_or(1.0);

                    skeleton.add_prop(crate::skeleton::SkeletonProp {
                        surface_id,
                        position: turtle.position,
                        rotation: turtle.rotation,
                        scale: Vec3::splat(scale_scalar),
                    });
                }
                TurtleOp::Ignore => {}
            }
        }
        skeleton
    }
}
