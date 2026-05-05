//! Skeleton data structures representing the geometric output of turtle interpretation.

use glam::{Quat, Vec3, Vec4};
use serde::{Deserialize, Serialize};

/// Non-fatal issues encountered while interpreting an L-System.
///
/// The interpreter continues producing geometry after recording a warning;
/// consumers (renderers, validators) can inspect [`Skeleton::warnings`] to
/// surface these to users.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TurtleWarning {
    /// A `Push` operation was skipped because the stack reached
    /// [`TurtleConfig::max_stack_depth`](crate::TurtleConfig::max_stack_depth).
    ///
    /// `symbol_index` is the position in the symbol stream where the overflow
    /// occurred. The matching `Pop` is also skipped to keep the stack in sync,
    /// so geometry remains structurally consistent — just truncated.
    StackOverflow { symbol_index: usize },
}

/// A point along a skeleton strand with position, orientation, and material properties.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SkeletonPoint {
    /// World-space position.
    pub position: Vec3,
    /// Orientation quaternion.
    pub rotation: Quat,
    /// Radius at this point (half of width).
    pub radius: f32,
    // --- Material Extensions ---
    /// RGBA color.
    pub color: Vec4,
    /// Material palette ID for multi-material meshes.
    ///
    /// References an external palette entry that defines roughness, metallic,
    /// and other PBR properties.
    pub material_id: u8,
    /// UV texture coordinate scale factor.
    pub uv_scale: f32,
}

/// A discrete object (leaf, flower, etc.) spawned by the turtle at a specific location.
///
/// Inherits material state (color, material ID) from the turtle at spawn time,
/// allowing downstream renderers to style props with the same palette system as strands.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SkeletonProp {
    /// The ID of the prop asset to spawn.
    pub prop_id: u16,
    /// World-space position.
    pub position: Vec3,
    /// World-space rotation.
    pub rotation: Quat,
    /// Scale factor (can be non-uniform).
    pub scale: Vec3,
    /// RGBA color inherited from turtle state at spawn time.
    pub color: Vec4,
    /// Material palette ID inherited from turtle state at spawn time.
    pub material_id: u8,
}

/// The geometric output of turtle interpretation: a collection of strands and props.
///
/// Strands are sequences of connected [`SkeletonPoint`]s representing branches/stems.
/// Props are discrete objects spawned at specific locations.
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Skeleton {
    /// Connected sequences of skeleton points forming branches.
    pub strands: Vec<Vec<SkeletonPoint>>,
    /// For each strand, the index of the strand it sprouts from
    /// (`None` for root strands). `strand_parents.len() == strands.len()`
    /// is an invariant maintained by [`Skeleton::push_node`] and
    /// [`Skeleton::start_strand`].
    ///
    /// Lets renderers distinguish a child branch from a continuation of the
    /// main stem without re-running the turtle.
    pub strand_parents: Vec<Option<usize>>,
    /// Discrete props (leaves, flowers, etc.) spawned during interpretation.
    pub props: Vec<SkeletonProp>,
    /// Non-fatal issues recorded during interpretation. Empty on a clean build.
    pub warnings: Vec<TurtleWarning>,
}

impl Skeleton {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends `point` to the current strand. If no strand exists yet,
    /// implicitly starts a root strand (parent `None`).
    ///
    /// Skips the append if `point` is within ~3.16e-3 units of the strand's
    /// last point (deduplication of zero-length segments).
    pub fn push_node(&mut self, point: SkeletonPoint) {
        if self.strands.is_empty() {
            self.start_strand(point, None);
            return;
        }
        let last_strand = self.strands.last_mut().expect("non-empty checked above");
        if let Some(last_point) = last_strand.last()
            && last_point.position.distance_squared(point.position) < 0.00001
        {
            return;
        }
        last_strand.push(point);
    }

    /// Starts a new strand seeded with `point`, recording its parent strand
    /// index (`None` for a root strand).
    pub fn start_strand(&mut self, point: SkeletonPoint, parent_strand_idx: Option<usize>) {
        self.strands.push(vec![point]);
        self.strand_parents.push(parent_strand_idx);
    }

    pub fn add_prop(&mut self, prop: SkeletonProp) {
        self.props.push(prop);
    }

    pub fn clear(&mut self) {
        self.strands.clear();
        self.strand_parents.clear();
        self.props.clear();
        self.warnings.clear();
    }
}
