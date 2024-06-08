use super::unit_geometry::{UnitPoint, UnitQuadrilateral};

pub trait ImageInterpolate<Pixel: image::Pixel> {
    fn interpolate_pixel(&self, point: UnitPoint) -> Pixel;
    fn crop(
        &self,
        quadrilateral: UnitQuadrilateral,
        steps_x: u32,
        steps_y: u32,
    ) -> image::ImageBuffer<Pixel, Vec<<Pixel as image::Pixel>::Subpixel>> {
        let UnitQuadrilateral { lt, lb, rt, rb } = quadrilateral;
        image::ImageBuffer::from_fn(steps_x, steps_y, |x, y| {
            let l = UnitPoint::interpolate(lt, lb, steps_y, y);
            let r = UnitPoint::interpolate(rt, rb, steps_y, y);
            let target = UnitPoint::interpolate(l, r, steps_x, x);
            self.interpolate_pixel(target)
        })
    }
}
impl<Pixel: image::Pixel, T: image::GenericImageView<Pixel = Pixel>> ImageInterpolate<Pixel> for T
where
    Pixel::Subpixel: imageproc::definitions::Clamp<f32>,
    Pixel::Subpixel: Into<f32>,
{
    fn interpolate_pixel(&self, point: UnitPoint) -> Pixel {
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
