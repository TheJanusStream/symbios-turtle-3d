use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SkeletonPoint {
    pub position: Vec3,
    pub rotation: Quat,
    pub radius: f32,
}

/// A discrete object (leaf, flower) spawned by the turtle.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SkeletonProp {
    /// The ID of the surface asset to spawn.
    pub surface_id: u16,
    /// World position.
    pub position: Vec3,
    /// World rotation.
    pub rotation: Quat,
    /// Uniform scale factor.
    pub scale: Vec3,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Skeleton {
    pub strands: Vec<Vec<SkeletonPoint>>,
    pub props: Vec<SkeletonProp>,
}

impl Skeleton {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, point: SkeletonPoint, force_new_strand: bool) {
        if force_new_strand || self.strands.is_empty() {
            self.strands.push(vec![point]);
        } else if let Some(last_strand) = self.strands.last_mut() {
            if let Some(last_point) = last_strand.last()
                && last_point.position.distance_squared(point.position) < 0.00001
            {
                return;
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
