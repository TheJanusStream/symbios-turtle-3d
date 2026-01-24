//! Skeleton data structures representing the geometric output of turtle interpretation.

use glam::{Quat, Vec3, Vec4};
use serde::{Deserialize, Serialize};

/// A point along a skeleton strand with position, orientation, and material properties.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SkeletonPoint {
    /// World-space position.
    pub position: Vec3,
    /// Orientation quaternion.
    pub rotation: Quat,
    /// Radius at this point (half of width).
    pub radius: f32,
    // --- PBR Extensions ---
    /// RGBA color.
    pub color: Vec4,
    /// Material ID for multi-material meshes.
    pub material_id: u8,
    /// PBR roughness (0.0 = smooth, 1.0 = rough).
    pub roughness: f32,
    /// PBR metallic (0.0 = dielectric, 1.0 = metal).
    pub metallic: f32,
    /// Texture ID for texture mapping.
    pub texture_id: u16,
}

/// A discrete object (leaf, flower, etc.) spawned by the turtle at a specific location.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SkeletonProp {
    /// The ID of the surface asset to spawn.
    pub surface_id: u16,
    /// World-space position.
    pub position: Vec3,
    /// World-space rotation.
    pub rotation: Quat,
    /// Scale factor (can be non-uniform).
    pub scale: Vec3,
}

/// The geometric output of turtle interpretation: a collection of strands and props.
///
/// Strands are sequences of connected [`SkeletonPoint`]s representing branches/stems.
/// Props are discrete objects spawned at specific locations.
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Skeleton {
    /// Connected sequences of skeleton points forming branches.
    pub strands: Vec<Vec<SkeletonPoint>>,
    /// Discrete props (leaves, flowers, etc.) spawned during interpretation.
    pub props: Vec<SkeletonProp>,
}

impl Skeleton {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a point to the skeleton.
    ///
    /// If `force_new_strand` is true, starts a new strand. Otherwise appends to the current strand,
    /// unless the point is too close to the previous one (deduplication).
    pub fn add_node(&mut self, point: SkeletonPoint, force_new_strand: bool) {
        if force_new_strand || self.strands.is_empty() {
            self.strands.push(vec![point]);
        } else if let Some(last_strand) = self.strands.last_mut() {
            if let Some(last_point) = last_strand.last() {
                if last_point.position.distance_squared(point.position) < 0.00001 {
                    return;
                }
            }
            last_strand.push(point);
        }
    }

    pub fn add_prop(&mut self, prop: SkeletonProp) {
        self.props.push(prop);
    }

    pub fn clear(&mut self) {
        self.strands.clear();
        self.props.clear();
    }
}
