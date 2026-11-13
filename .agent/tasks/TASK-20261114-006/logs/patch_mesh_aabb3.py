import io

# 1. monster-core mesh.rs: normals + vn parsing
p = 'crates/monster-core/src/mesh.rs'
with io.open(p, 'r', encoding='utf-8') as f:
    c = f.read()

c = c.replace('''pub struct Mesh3D {
    pub vertices: Vec<Vec3>,
    /// Triangle indices into `vertices` (triples).
    pub indices: Vec<u32>,
}''', '''pub struct Mesh3D {
    pub vertices: Vec<Vec3>,
    /// Optional per-normal list (OBJ `vn` lines, declaration order).
    pub normals: Vec<Vec3>,
    /// Triangle indices into `vertices` (triples).
    pub indices: Vec<u32>,
}''')

c = c.replace('''                Some("f") => {
                    let idx: Result<Vec<i64>, _> = tokens.map(|t| t.parse::<i64>()).collect();
                    let idx = idx.map_err(|_| format!("line {}: invalid face index", line_no + 1))?;
                    if idx.len() < 3 {
                        return Err(format!("line {}: face needs >= 3 indices", line_no + 1));
                    }''', '''                Some("vn") => {
                    let coords: Result<Vec<f32>, _> =
                        tokens.take(3).map(|t| t.parse::<f32>()).collect();
                    let coords = coords
                        .map_err(|_| format!("line {}: invalid normal coordinate", line_no + 1))?;
                    if coords.len() != 3 {
                        return Err(format!("line {}: normal needs 3 coordinates", line_no + 1));
                    }
                    mesh.normals.push(Vec3::new(coords[0], coords[1], coords[2]));
                }
                Some("f") => {
                    // face token forms: "i", "i/j", "i//k", "i/j/k" — the
                    // vertex index is everything before the first '/'
                    let mut vertex_idx: Vec<i64> = Vec::new();
                    for token in tokens {
                        let head = token.split('/').next().unwrap_or(token);
                        let i: i64 = head
                            .parse()
                            .map_err(|_| format!("line {}: invalid face index", line_no + 1))?;
                        vertex_idx.push(i);
                    }
                    if vertex_idx.len() < 3 {
                        return Err(format!("line {}: face needs >= 3 indices", line_no + 1));
                    }''')
c = c.replace('''                    for k in 1..idx.len() - 1 {
                        mesh.indices.push(resolve(idx[0])?);
                        mesh.indices.push(resolve(idx[k])?);
                        mesh.indices.push(resolve(idx[k + 1])?);
                    }''', '''                    for k in 1..vertex_idx.len() - 1 {
                        mesh.indices.push(resolve(vertex_idx[0])?);
                        mesh.indices.push(resolve(vertex_idx[k])?);
                        mesh.indices.push(resolve(vertex_idx[k + 1])?);
                    }''')
c = c.replace('''        let idx: Result<Vec<i64>, _> = tokens.map(|t| t.parse::<i64>()).collect();
        let idx = idx.map_err(|_| format!("line {}: invalid face index", line_no + 1))?;''', '''        let idx: Result<Vec<i64>, _> = tokens.map(|t| t.parse::<i64>()).collect();''')

# OBJ vn test
c = c.replace('''    #[test]
    fn cube_first_triangle_front_face() {''', '''    #[test]
    fn obj_parses_vn_and_v_vn_faces() {
        let obj = "vn 0 0 1\\nvn 0 1 0\\nv 0 0 0\\nv 1 0 0\\nv 1 1 0\\nf 1//2 2//1 3//2\\n";
        let m = Mesh3D::from_obj(obj).expect("parse");
        assert_eq!(m.vertices.len(), 3);
        assert_eq!(m.normals, vec![Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0)]);
        assert_eq!(m.indices, vec![0, 1, 2]);
    }

    #[test]
    fn cube_first_triangle_front_face() {''')

with io.open(p, 'w', encoding='utf-8') as f:
    f.write(c)
print('mesh ok')

# 2. monster-physics: Aabb3
p = 'crates/monster-physics/src/lib.rs'
with io.open(p, 'r', encoding='utf-8') as f:
    c = f.read()

aabb3 = '''/// Axis-aligned bounding box in 3D (M3: physics 3D foundation).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb3 {
    pub min: monster_math::Vec3,
    pub max: monster_math::Vec3,
}

impl Aabb3 {
    pub fn new(min: monster_math::Vec3, max: monster_math::Vec3) -> Aabb3 {
        debug_assert!(
            max.x >= min.x && max.y >= min.y && max.z >= min.z,
            "Aabb3 min must be <= max"
        );
        Aabb3 { min, max }
    }

    pub fn from_center_half(
        center: monster_math::Vec3,
        half: monster_math::Vec3,
    ) -> Aabb3 {
        Aabb3 { min: center - half, max: center + half }
    }

    pub fn center(&self) -> monster_math::Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn half_extents(&self) -> monster_math::Vec3 {
        (self.max - self.min) * 0.5
    }

    pub fn contains(&self, p: monster_math::Vec3) -> bool {
        p.x >= self.min.x
            && p.x <= self.max.x
            && p.y >= self.min.y
            && p.y <= self.max.y
            && p.z >= self.min.z
            && p.z <= self.max.z
    }

    pub fn intersects(&self, other: &Aabb3) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Minimal translation vector (MTV) on the smallest-penetration axis.
    pub fn separation(&self, other: &Aabb3) -> Option<monster_math::Vec3> {
        let overlaps = [
            (other.max.x - self.min.x, monster_math::Vec3::new(1.0, 0.0, 0.0)),
            (self.max.x - other.min.x, monster_math::Vec3::new(-1.0, 0.0, 0.0)),
            (other.max.y - self.min.y, monster_math::Vec3::new(0.0, 1.0, 0.0)),
            (self.max.y - other.min.y, monster_math::Vec3::new(0.0, -1.0, 0.0)),
            (other.max.z - self.min.z, monster_math::Vec3::new(0.0, 0.0, 1.0)),
            (self.max.z - other.min.z, monster_math::Vec3::new(0.0, 0.0, -1.0)),
        ];
        for &(amount, _) in &overlaps {
            if amount < 0.0 {
                return None; // separated on this axis
            }
        }
        let best = overlaps
            .iter()
            .cloned()
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())?;
        Some(best.1 * best.0)
    }
}

/// Euler integration step'''
assert '/// Euler integration step' in c
c = c.replace('/// Euler integration step', aabb3, 1)

# Aabb3 tests module
c = c.replace('''#[cfg(test)]
mod tests {
    use super::*;
    use monster_math::approx_eq;

    #[test]
    fn intersects_variants() {''', '''#[cfg(test)]
mod aabb3_tests {
    use super::*;
    use monster_math::{approx_eq, Vec3};

    #[test]
    fn aabb3_intersects_and_contains() {
        let a = Aabb3::from_center_half(Vec3::new(0.0, 0.0, 0.0), Vec3::new(5.0, 5.0, 5.0));
        let b = Aabb3::from_center_half(Vec3::new(6.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 2.0));
        assert!(a.intersects(&b)); // x overlap [4, 5]
        assert!(a.contains(Vec3::new(1.0, 1.0, 1.0)));
        assert!(!a.contains(Vec3::new(6.0, 0.0, 0.0)));
        let far = Aabb3::from_center_half(Vec3::new(50.0, 50.0, 50.0), Vec3::ONE);
        assert!(!a.intersects(&far));
    }

    #[test]
    fn aabb3_separation_smallest_axis() {
        // deep x overlap (20), shallow y overlap (2) -> move on y
        let a = Aabb3::from_center_half(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 2.0, 10.0));
        let b = Aabb3::from_center_half(Vec3::new(0.0, 3.0, 0.0), Vec3::new(10.0, 2.0, 10.0));
        let mtv = a.separation(&b).unwrap();
        assert!(approx_eq(mtv.y, -1.0, 1e-5));
        assert!(approx_eq(mtv.x, 0.0, 1e-5));
        assert!(approx_eq(mtv.z, 0.0, 1e-5));
        let none = a.separation(&Aabb3::from_center_half(
            Vec3::new(50.0, 50.0, 50.0),
            Vec3::ONE,
        ));
        assert!(none.is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monster_math::approx_eq;

    #[test]
    fn intersects_variants() {''')

with io.open(p, 'w', encoding='utf-8') as f:
    f.write(c)
print('physics aabb3 ok')
