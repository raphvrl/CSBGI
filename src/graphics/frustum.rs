use glam::{Mat4, Vec3, Vec4};

#[derive(Clone, Copy)]
pub struct Frustum {
    planes: [Vec4; 6],
}

impl Frustum {
    pub fn from_view_projection(view_projection: Mat4) -> Self {
        let columns = view_projection.to_cols_array_2d();
        let row = |index: usize| {
            Vec4::new(
                columns[0][index],
                columns[1][index],
                columns[2][index],
                columns[3][index],
            )
        };

        let row0 = row(0);
        let row1 = row(1);
        let row2 = row(2);
        let row3 = row(3);

        Self {
            planes: [
                row3 + row0,
                row3 - row0,
                row3 + row1,
                row3 - row1,
                row2,
                row3 - row2,
            ]
            .map(normalize_plane),
        }
    }

    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        self.planes
            .iter()
            .all(|plane| plane.truncate().dot(center) + plane.w >= -radius)
    }
}

fn normalize_plane(plane: Vec4) -> Vec4 {
    plane / plane.truncate().length()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frustum_rejects_spheres_outside_clip_volume() {
        let frustum = Frustum::from_view_projection(Mat4::IDENTITY);

        assert!(frustum.intersects_sphere(Vec3::new(0.0, 0.0, 0.5), 0.1));
        assert!(!frustum.intersects_sphere(Vec3::new(2.0, 0.0, 0.5), 0.5));
        assert!(!frustum.intersects_sphere(Vec3::new(0.0, 0.0, -0.5), 0.1));
    }
}
