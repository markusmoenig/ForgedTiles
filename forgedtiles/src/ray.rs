use crate::prelude::*;

/// Ray
#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub struct Ray {
    pub o: Vec3f,
    pub d: Vec3f,
}

impl Ray {
    pub fn new(o: Vec3f, d: Vec3f) -> Self {
        Self { o, d }
    }

    /// Returns the position on the ray at the given distance
    pub fn at(&self, d: f32) -> Vec3f {
        self.o + self.d * d
    }

    pub fn sphere(&self, center: Vec3f, radius: f32) -> Option<f32> {
        let l = center - self.o;
        let tca = dot(l, self.d);
        let d2 = dot(l, l) - tca * tca;
        let radius2 = radius * radius;
        if d2 > radius2 {
            return None;
        }
        let thc = (radius2 - d2).sqrt();
        let mut t0 = tca - thc;
        let mut t1 = tca + thc;

        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }

        if t0 < 0.0 {
            t0 = t1;
            if t0 < 0.0 {
                return None;
            }
        }

        Some(t0)
    }
}
