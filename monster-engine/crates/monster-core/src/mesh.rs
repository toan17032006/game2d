//! Mesh data + a minimal Wavefront OBJ loader (M3 slice 2).
//!
//! `Mesh3D` is indexed triangle soup in local space. The OBJ loader parses
//! `v` (vertex) and `f` (face) lines — faces are fan-triangulated, negative
//! indices resolve relative to the current vertex count.

use monster_math::Vec3;

/// An indexed triangle mesh in local space.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mesh3D {
    pub vertices: Vec<Vec3>,
    /// Triangle indices into `vertices` (triples).
    pub indices: Vec<u32>,
}

impl Mesh3D {
    /// Unit cube centred at the origin: 8 vertices / 12 triangles.
    pub fn cube(half: f32) -> Mesh3D {
        Mesh3D {
            vertices: vec![
                Vec3::new(-half, -half, half),  // 0 front-bottom-left
                Vec3::new(half, -half, half),   // 1 front-bottom-right
                Vec3::new(half, half, half),    // 2 front-top-right
                Vec3::new(-half, half, half),   // 3 front-top-left
                Vec3::new(-half, -half, -half), // 4 back-bottom-left
                Vec3::new(half, -half, -half),  // 5 back-bottom-right
                Vec3::new(half, half, -half),   // 6 back-top-right
                Vec3::new(-half, half, -half),  // 7 back-top-left
            ],
            indices: vec![
                0, 1, 2, 0, 2, 3, // front
                5, 4, 7, 5, 7, 6, // back
                1, 5, 6, 1, 6, 2, // right
                4, 0, 3, 4, 3, 7, // left
                3, 2, 6, 3, 6, 7, // top
                4, 5, 1, 4, 1, 0, // bottom
            ],
        }
    }

    /// Triangle count.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Parse a Wavefront OBJ subset: `v x y z` + `f i j k ...` (fan
    /// triangulation, negative indices relative). Returns a descriptive
    /// error on malformed lines.
    pub fn from_obj(text: &str) -> Result<Mesh3D, String> {
        let mut mesh = Mesh3D::default();
        for (line_no, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut tokens = line.split_whitespace();
            match tokens.next() {
                Some("v") => {
                    let coords: Result<Vec<f32>, _> =
                        tokens.take(3).map(|t| t.parse::<f32>()).collect();
                    let coords = coords
                        .map_err(|_| format!("line {}: invalid vertex coordinate", line_no + 1))?;
                    if coords.len() != 3 {
                        return Err(format!("line {}: vertex needs 3 coordinates", line_no + 1));
                    }
                    mesh.vertices
                        .push(Vec3::new(coords[0], coords[1], coords[2]));
                }
                Some("f") => {
                    let idx: Result<Vec<i64>, _> = tokens.map(|t| t.parse::<i64>()).collect();
                    let idx =
                        idx.map_err(|_| format!("line {}: invalid face index", line_no + 1))?;
                    if idx.len() < 3 {
                        return Err(format!("line {}: face needs >= 3 indices", line_no + 1));
                    }
                    // resolve + fan triangulate
                    let n = mesh.vertices.len() as i64;
                    let resolve = |i: i64| -> Result<u32, String> {
                        let resolved = if i > 0 { i - 1 } else { n + i };
                        if resolved < 0 || resolved >= n {
                            return Err(format!("line {}: face index out of range", line_no + 1));
                        }
                        Ok(resolved as u32)
                    };
                    for k in 1..idx.len() - 1 {
                        mesh.indices.push(resolve(idx[0])?);
                        mesh.indices.push(resolve(idx[k])?);
                        mesh.indices.push(resolve(idx[k + 1])?);
                    }
                }
                _ => {} // vn/vt/o/g/usemtl... ignored by this loader
            }
        }
        if mesh.vertices.is_empty() {
            return Err("OBJ has no vertices".into());
        }
        if mesh.indices.is_empty() {
            return Err("OBJ has no faces".into());
        }
        Ok(mesh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_math::approx_eq;

    #[test]
    fn cube_mesh_shape() {
        let m = Mesh3D::cube(0.5);
        assert_eq!(m.vertices.len(), 8);
        assert_eq!(m.triangle_count(), 12);
        // all indices in range
        assert!(m.indices.iter().all(|&i| (i as usize) < m.vertices.len()));
    }

    #[test]
    fn obj_parses_vertices_and_faces() {
        let obj = "# comment\nv 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n";
        let m = Mesh3D::from_obj(obj).expect("parse");
        assert_eq!(m.vertices.len(), 4);
        assert_eq!(m.triangle_count(), 2); // quad fanned into 2 triangles
        assert_eq!(m.indices, vec![0, 1, 2, 0, 2, 3]);
    }

    #[test]
    fn obj_negative_and_relative_indices() {
        // faces written relative to the end of the vertex list
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf -3 -2 -1\nf 1 2 3\n";
        let m = Mesh3D::from_obj(obj).expect("parse");
        assert_eq!(m.indices, vec![0, 1, 2, 0, 1, 2]);
    }

    #[test]
    fn obj_errors_are_descriptive() {
        assert!(Mesh3D::from_obj("v 1 2").is_err()); // 2 coords
        assert!(Mesh3D::from_obj("v 0 0 0\nf 1 2").is_err()); // 2-index face
        assert!(Mesh3D::from_obj("v 0 0 0\nf 1 2 9").is_err()); // out of range
        assert!(Mesh3D::from_obj("# nothing").is_err()); // no vertices
        assert!(Mesh3D::from_obj("v 0 0 0").is_err()); // no faces
    }

    #[test]
    fn cube_first_triangle_front_face() {
        let m = Mesh3D::cube(1.0);
        let tri = [
            m.vertices[m.indices[0] as usize],
            m.vertices[m.indices[1] as usize],
            m.vertices[m.indices[2] as usize],
        ];
        // front face sits at z = +1
        for v in tri {
            assert!(approx_eq(v.z, 1.0, 1e-6));
        }
    }
}
