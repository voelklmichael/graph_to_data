#[derive(Debug)]
pub enum Error {
    Image(image::ImageError),
    StdIO(std::io::Error),
}
impl From<image::ImageError> for Error {
    fn from(value: image::ImageError) -> Self {
        Self::Image(value)
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::StdIO(value)
    }
}
fn main() -> Result<(), Error> {
    let image_bytes = include_bytes!("../example_data/X^4_4^x.PNG");
    let image = image::io::Reader::new(std::io::Cursor::new(image_bytes))
        .with_guessed_format()?
        .decode()?;

    let l = 0.05;
    let r = 0.993;
    let t = 0.026;
    let b = 0.915;
    let cropped = image.interpolate_image(
        UnitPoint::unchecked(l, t),
        UnitPoint::unchecked(l, b),
        UnitPoint::unchecked(r, t),
        UnitPoint::unchecked(r, b),
        500,
        500,
    );
    cropped.save("cropped.png").unwrap();
    let target_color = image::Rgba([0u8, 0, 153, 255]);
    let diff_max = 5;
    const HIT: image::Luma<u8> = image::Luma([255u8]);
    const MISSED: image::Luma<u8> = image::Luma([0]);
    let filtered = imageproc::map::map_pixels(&cropped, |_, _, p| {
        let p = p.0;
        let diff = p
            .into_iter()
            .zip(target_color.0)
            .map(|(p, t)| p.max(t) - p.min(t))
            .fold(0u8, |p, c| p.saturating_add(c));
        if diff < diff_max {
            HIT
        } else {
            MISSED
        }
    });
    filtered.save("filtered.png").unwrap();
    let hit_components = (0..filtered.width())
        .map(|x| {
            let mut component_centers = Vec::new();
            let mut component = Vec::new();
            for y in 0..filtered.height() {
                if filtered.get_pixel(x, y) == &HIT {
                    component.push(y);
                } else {
                    finish_component(&mut component, &mut component_centers);
                }
            }
            finish_component(&mut component, &mut component_centers);
            component_centers
        })
        .collect::<Vec<_>>();
    let single_component = imageproc::map::map_pixels(&filtered, |x, y, _| {
        let components = &hit_components[x as usize];
        if components.iter().any(|c| *c == y) {
            if components.len() == 1 {
                image::Rgba([0u8, 255, 0, 255])
            } else if components.len() == 1 {
                image::Rgba([0u8, 0, 255, 255])
            } else {
                image::Rgba([255u8, 0, 0, 255])
            }
        } else {
            image::Rgba([0u8, 0, 0, 255])
        }
    });
    single_component.save("dummy.png").unwrap();
    let mut longest = None;
    let mut current: Option<(usize, usize, u32)> = None;
    for (x, components) in hit_components.iter().enumerate() {
        if components.len() == 1 {
            let y = components[0];
            if let Some((_start, end, yy)) = &mut current {
                let yyy = *yy;
                let diff = yyy.max(y) - yyy.min(y);
                if x == *end + 1 && diff <= 3 {
                    *end = x;
                    *yy = y;
                } else {
                    complete(&mut current, &mut longest);
                }
            } else {
                current = Some((x, x, y));
            }
        }
    }
    complete(&mut current, &mut longest);
    dbg!(longest);
    Ok(())
}

fn complete(current: &mut Option<(usize, usize, u32)>, longest: &mut Option<(usize, usize)>) {
    if let Some((current_start, current_end, _)) = current.take() {
        if let Some((longest_start, longest_end)) = *longest {
            if longest_end - longest_start < current_end - current_start {
                *longest = Some((current_start, current_end));
            }
        } else {
            *longest = Some((current_start, current_end));
        }
    }
}

fn finish_component(component: &mut Vec<u32>, component_centers: &mut Vec<u32>) {
    if !component.is_empty() {
        let n = component.len();
        let sum: u32 = std::mem::take(component).into_iter().sum();
        component_centers.push(sum / n as u32);
    }
}

/// This represents a number between 0. and 1.
/// Note: 0. corresponds to left/top and 1. to right/bottom
#[derive(Clone, Copy, Debug)]
pub struct UnitInterval(f32);
#[derive(Clone, Copy, Debug)]
pub struct UnitPoint {
    x: UnitInterval,
    y: UnitInterval,
}
impl UnitInterval {
    fn interpolate(min: Self, max: Self, delta: f32) -> Self {
        let target = max.0 * delta + (1. - delta) * min.0;
        debug_assert!(target >= 0.);
        debug_assert!(target <= 1.);
        Self(target.clamp(0., 1.))
    }
}
impl UnitPoint {
    fn interpolate(min: Self, max: Self, steps: u32, target: u32) -> Self {
        let delta = target as f32 / (steps - 1) as f32;
        let x = UnitInterval::interpolate(min.x, max.x, delta);
        let y = UnitInterval::interpolate(min.y, max.y, delta);
        Self { x, y }
    }

    fn unchecked(x: f32, y: f32) -> Self {
        Self {
            x: UnitInterval(x),
            y: UnitInterval(y),
        }
    }
}
pub trait ImageToGraph {
    type Pixel: image::Pixel;
    fn interpolate_pixel(&self, point: UnitPoint) -> Self::Pixel;
    fn interpolate_image(
        &self,
        lt: UnitPoint,
        lb: UnitPoint,
        rt: UnitPoint,
        rb: UnitPoint,
        steps_x: u32,
        steps_y: u32,
    ) -> image::ImageBuffer<Self::Pixel, Vec<<Self::Pixel as image::Pixel>::Subpixel>> {
        image::ImageBuffer::from_fn(steps_x, steps_y, |x, y| {
            let l = UnitPoint::interpolate(lt, lb, steps_y, y);
            let r = UnitPoint::interpolate(rt, rb, steps_y, y);
            let target = UnitPoint::interpolate(l, r, steps_x, x);
            self.interpolate_pixel(target)
        })
    }
}
impl ImageToGraph for image::DynamicImage {
    type Pixel = image::Rgba<u8>;

    fn interpolate_pixel(&self, point: UnitPoint) -> Self::Pixel {
        let UnitPoint { x, y } = point;
        let left = x.0 * self.width() as f32;
        let x_fraction = left.fract();
        let left = left as u32;
        let top = y.0 * self.height() as f32;
        let y_fraction = top.fract();
        let top = top as u32;
        let fetch_pixel = |x: u32, y: u32| {
            let x = x.min(self.width() - 1);
            let y = y.min(self.height() - 1);
            use image::GenericImageView;
            self.get_pixel(x, y)
        };
        let lt = fetch_pixel(left, top);
        let lb = fetch_pixel(left, top + 1);
        let rt = fetch_pixel(left + 1, top);
        let rb = fetch_pixel(left + 1, top + 1);
        let l = imageproc::pixelops::interpolate(lt, lb, 1. - y_fraction);
        let r = imageproc::pixelops::interpolate(rt, rb, 1. - y_fraction);
        imageproc::pixelops::interpolate(l, r, 1. - x_fraction)
    }
}
