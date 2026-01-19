use crate::skeleton::{Skeleton, SkeletonPoint};
use crate::turtle::{TurtleOp, TurtleState};
use glam::{Mat3, Quat, Vec3};
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
            let get_val =
                |default: f32| -> f32 { view.params.first().map(|&x| x as f32).unwrap_or(default) };

            match op {
                TurtleOp::Draw => {
                    let len = get_val(self.config.default_step);

                    if skeleton.strands.is_empty() {
                        skeleton.add_node(
                            SkeletonPoint {
                                position: turtle.position,
                                rotation: turtle.rotation,
                                radius: turtle.width / 2.0,
                            },
                            true,
                        );
                    }

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

                    skeleton.add_node(
                        SkeletonPoint {
                            position: turtle.position,
                            rotation: turtle.rotation,
                            radius: turtle.width / 2.0,
                        },
                        false,
                    );
                }
                TurtleOp::Move => {
                    let len = get_val(self.config.default_step);
                    turtle.position += turtle.up() * len;
                    skeleton.add_node(
                        SkeletonPoint {
                            position: turtle.position,
                            rotation: turtle.rotation,
                            radius: turtle.width / 2.0,
                        },
                        true,
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
                TurtleOp::Push => {
                    stack.push(turtle);
                    // Explicitly break the strand on Push to isolate the branch
                    skeleton.add_node(
                        SkeletonPoint {
                            position: turtle.position,
                            rotation: turtle.rotation,
                            radius: turtle.width / 2.0,
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
                            },
                            true,
                        );
                    }
                }
                TurtleOp::Spawn(_) => {}
                TurtleOp::Ignore => {}
            }
        }
        skeleton
    }
}
