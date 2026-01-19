use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TurtleState {
    pub position: Vec3,
    pub rotation: Quat,
    pub width: f32,
}

impl Default for TurtleState {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            width: 0.1,
        }
    }
}

impl TurtleState {
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn rotate_local_x(&mut self, angle: f32) {
        let rot = Quat::from_axis_angle(Vec3::X, angle);
        self.rotation *= rot;
    }

    pub fn rotate_local_y(&mut self, angle: f32) {
        let rot = Quat::from_axis_angle(Vec3::Y, angle);
        self.rotation *= rot;
    }

    pub fn rotate_local_z(&mut self, angle: f32) {
        let rot = Quat::from_axis_angle(Vec3::Z, angle);
        self.rotation *= rot;
    }

    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
        let rot = Quat::from_axis_angle(axis, angle);
        self.rotation = rot * self.rotation;
    }

    // Aligns the turtle so its Up vector matches the target, minimizing twist
    pub fn align_up_to(&mut self, target_up: Vec3) {
        let current_up = self.up();
        let rotation = Quat::from_rotation_arc(current_up, target_up);
        self.rotation = rotation * self.rotation;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TurtleOp {
    Draw,
    Move,
    Yaw(f32),   // + / -
    Pitch(f32), // & / ^
    Roll(f32),  // \ / /
    TurnAround, // |
    Vertical,   // $
    SetWidth,   // !
    Push,       // [
    Pop,        // ]
    Spawn(u16), // ~ (New: Predefined Surface ID)
    Ignore,
}
