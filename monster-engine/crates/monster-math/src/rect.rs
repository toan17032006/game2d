//! Axis-aligned rectangle in 2D world/screen space.

use crate::vec::Vec2;

/// Axis-aligned rectangle: `position` is the top-left corner in engine
/// space (+Y down), `size` the full width/height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
}

impl Rect {
    #[inline]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect {
            position: Vec2::new(x, y),
            size: Vec2::new(width, height),
        }
    }

    #[inline]
    pub fn from_center(center: Vec2, half_size: Vec2) -> Rect {
        Rect {
            position: center - half_size,
            size: half_size * 2.0,
        }
    }

    #[inline]
    pub fn min(&self) -> Vec2 {
        self.position
    }

    #[inline]
    pub fn max(&self) -> Vec2 {
        self.position + self.size
    }

    #[inline]
    pub fn center(&self) -> Vec2 {
        self.position + self.size * 0.5
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.size.x
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.size.y
    }

    #[inline]
    pub fn contains(&self, p: Vec2) -> bool {
        let max = self.max();
        p.x >= self.position.x && p.x <= max.x && p.y >= self.position.y && p.y <= max.y
    }

    #[inline]
    pub fn intersects(&self, o: &Rect) -> bool {
        let a_min = self.min();
        let a_max = self.max();
        let b_min = o.min();
        let b_max = o.max();
        a_min.x <= b_max.x && a_max.x >= b_min.x && a_min.y <= b_max.y && a_max.y >= b_min.y
    }

    /// Grow the rect by `amount` on every side.
    #[inline]
    pub fn inflate(&self, amount: f32) -> Rect {
        Rect {
            position: self.position - Vec2::splat(amount),
            size: self.size + Vec2::splat(amount * 2.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx_eq;

    #[test]
    fn corners_and_center() {
        let r = Rect::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(r.min(), Vec2::new(10.0, 20.0));
        assert_eq!(r.max(), Vec2::new(40.0, 60.0));
        assert_eq!(r.center(), Vec2::new(25.0, 40.0));
        assert!(approx_eq(r.width(), 30.0, 1e-6));
        assert!(approx_eq(r.height(), 40.0, 1e-6));
    }

    #[test]
    fn contains_and_intersects() {
        let r = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(r.contains(Vec2::new(5.0, 5.0)));
        assert!(r.contains(Vec2::new(0.0, 0.0)));
        assert!(!r.contains(Vec2::new(10.1, 5.0)));
        let touch = Rect::new(10.0, 0.0, 5.0, 5.0);
        assert!(r.intersects(&touch)); // edge-touching counts as intersecting
        let apart = Rect::new(20.0, 20.0, 5.0, 5.0);
        assert!(!r.intersects(&apart));
    }

    #[test]
    fn from_center_and_inflate() {
        let r = Rect::from_center(Vec2::new(0.0, 0.0), Vec2::new(5.0, 5.0));
        assert_eq!(r, Rect::new(-5.0, -5.0, 10.0, 10.0));
        let inflated = r.inflate(2.0);
        assert_eq!(inflated, Rect::new(-7.0, -7.0, 14.0, 14.0));
    }
}
