//! Turtle state and operations for 3D L-System interpretation.

use glam::{Quat, Vec3, Vec4};
use serde::{Deserialize, Serialize};

/// The current state of the turtle in 3D space.
///
/// Includes position, orientation, stroke width, and palette-based material properties.
/// Roughness, metallic, and texture are controlled by the material palette via `material_id`.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TurtleState {
    /// Current world-space position.
    pub position: Vec3,
    /// Current orientation as a quaternion.
    pub rotation: Quat,
    /// Stroke width for drawing operations.
    pub width: f32,
    /// RGBA color (PBR albedo).
    pub color: Vec4,
    /// Material ID for multi-material mesh generation.
    ///
    /// References a material palette entry that defines roughness, metallic,
    /// and other PBR properties externally.
    pub material_id: u8,
    /// UV texture coordinate scale factor.
    pub uv_scale: f32,
}

impl Default for TurtleState {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            width: 0.1,
            color: Vec4::ONE, // White, opaque
            material_id: 0,
            uv_scale: 1.0,
        }
    }
}

impl TurtleState {
    /// Returns the turtle's local up direction (Y-axis) in world space.
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// Returns the turtle's local forward direction (Z-axis) in world space.
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    /// Returns the turtle's local right direction (X-axis) in world space.
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Rotates around the local X-axis (pitch).
    pub fn rotate_local_x(&mut self, angle: f32) {
        let rot = Quat::from_axis_angle(Vec3::X, angle);
        self.rotation *= rot;
    }

    /// Rotates around the local Y-axis (roll).
    pub fn rotate_local_y(&mut self, angle: f32) {
        let rot = Quat::from_axis_angle(Vec3::Y, angle);
        self.rotation *= rot;
    }

    /// Rotates around the local Z-axis (yaw).
    pub fn rotate_local_z(&mut self, angle: f32) {
        let rot = Quat::from_axis_angle(Vec3::Z, angle);
        self.rotation *= rot;
    }

    /// Rotates around an arbitrary world-space axis.
    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
        let rot = Quat::from_axis_angle(axis, angle);
        self.rotation = rot * self.rotation;
    }

    /// Aligns the turtle's up vector to the target direction, minimizing twist.
    pub fn align_up_to(&mut self, target_up: Vec3) {
        let current_up = self.up();
        let rotation = Quat::from_rotation_arc(current_up, target_up);
        self.rotation = rotation * self.rotation;
    }
}

/// Operations that can be performed by the turtle during L-System interpretation.
///
/// Each variant corresponds to a standard L-System symbol or PBR extension.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TurtleOp {
    /// Draw forward, adding a segment to the skeleton (`F`).
    Draw,
    /// Move forward without drawing (`f`).
    Move,
    /// Rotate around Z-axis. Sign indicates direction (`+` / `-`).
    Yaw(f32),
    /// Rotate around X-axis. Sign indicates direction (`&` / `^`).
    Pitch(f32),
    /// Rotate around Y-axis. Sign indicates direction (`\` / `/`).
    Roll(f32),
    /// Turn around 180 degrees (`|`).
    TurnAround,
    /// Align to vertical/gravity direction (`$`).
    Vertical,
    /// Set stroke width (`!`).
    SetWidth,
    /// Push current state onto stack (`[`).
    Push,
    /// Pop state from stack (`]`).
    Pop,
    /// Spawn a prop/surface at current position (`~`). Contains default surface ID.
    Spawn(u16),
    /// Set color - accepts 1 (grayscale), 3 (RGB), or 4 (RGBA) params (`'`).
    SetColor,
    /// Set material ID (`,`).
    SetMaterial,
    /// Set UV texture coordinate scale (`;`).
    SetUVScale,
    /// Ignored symbol (no operation).
    Ignore,
}
