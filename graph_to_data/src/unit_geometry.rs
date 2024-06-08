/// This represents a number between 0. and 1.
/// Note: 0. corresponds to left/top and 1. to right/bottom
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, PartialOrd)]
pub struct UnitInterval(pub f32);

impl UnitInterval {
    pub fn new(x: f32) -> Result<Self, f32> {
        if (0. ..=1.).contains(&x) {
            Ok(Self(x))
        } else {
            Err(x)
        }
    }
}
impl UnitInterval {
    #[inline(always)]
    fn interpolate(min: Self, max: Self, delta: f32) -> Self {
        let target = max.0 * delta + (1. - delta) * min.0;
        Self(target.clamp(0., 1.))
    }
}
impl Eq for UnitInterval {}
#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for UnitInterval {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.partial_cmp(other) {
            Some(ordering) => ordering,
            None => unreachable!("Constructor checked that 'NaN' does not occur"),
        }
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct UnitPoint {
    pub x: UnitInterval,
    pub y: UnitInterval,
}
impl UnitPoint {
    #[inline(always)]
    pub(super) fn interpolate(min: Self, max: Self, steps: u32, target: u32) -> Self {
        let delta = target as f32 / (steps - 1) as f32;
        let x = UnitInterval::interpolate(min.x, max.x, delta);
        let y = UnitInterval::interpolate(min.y, max.y, delta);
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct UnitQuadrilateral {
    pub lt: UnitPoint,
    pub lb: UnitPoint,
    pub rt: UnitPoint,
    pub rb: UnitPoint,
}
impl UnitQuadrilateral {
    pub fn rectangular(p1: UnitPoint, p2: UnitPoint) -> Self {
        let left = p1.x.min(p2.x);
        let right = p1.x.max(p2.x);
        let top = p1.y.min(p2.y);
        let bottom = p1.y.max(p2.y);
        Self {
            lt: UnitPoint { x: left, y: top },
            lb: UnitPoint { x: left, y: bottom },
            rt: UnitPoint { x: right, y: top },
            rb: UnitPoint {
                x: right,
                y: bottom,
            },
        }
    }

    pub fn width(&self) -> f32 {
        self.rt.x.max(self.rb.x).0 - self.lt.x.min(self.lb.x).0
    }
    pub fn height(&self) -> f32 {
        self.rb.y.max(self.lb.y).0 - self.lt.y.min(self.rt.y).0
    }
    pub fn unit_square() -> Self {
        Self::rectangular(
            UnitPoint {
                x: UnitInterval(0.),
                y: UnitInterval(0.),
            },
            UnitPoint {
                x: UnitInterval(1.),
                y: UnitInterval(1.),
            },
        )
    }
}
