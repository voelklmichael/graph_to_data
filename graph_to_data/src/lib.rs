const HIT: image::Luma<u8> = image::Luma([255]);
const MISSED: image::Luma<u8> = image::Luma([0]);

mod color_extraction;
mod color_filtering;
mod step3_group;
mod step4_stitch;
//mod helpers;
mod image_interpolate;
mod line_detection;
mod unit_geometry;

pub use color_extraction::extract_colors;
pub use color_filtering::color_filtering;
pub use image_interpolate::ImageInterpolate;
pub use line_detection::{ImagePixel, LineDetection, LineDetectionSettings};
pub use step3_group::{
    group_large_components_and_remaining, CombinedVerticals, VerticalComponentCombined,
};
pub use step4_stitch::stitch;
pub use unit_geometry::{UnitInterval, UnitPoint, UnitQuadrilateral};

pub struct Settings {
    pub step1_width_minimial_fraction: f32,
    pub step1_height_maximal_fraction: f32,
    pub step1_ignore_gray: bool,
    pub step1_step2_color_radius: u8,
    pub step3_min_width_fraction: f32,
    pub step4_component_jump_height_fraction: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            step1_step2_color_radius: 5,
            step1_width_minimial_fraction: 0.3,
            step1_height_maximal_fraction: 0.1,
            step1_ignore_gray: true,
            step3_min_width_fraction: 0.05,
            step4_component_jump_height_fraction: 0.02,
        }
    }
}

pub fn color_distance(cc: &image::Rgb<u8>, c: &image::Rgb<u8>) -> u8 {
    cc.0.iter()
        .zip(c.0)
        .map(|(&c, cc)| cc.max(c) - cc.min(c))
        .fold(0, |a, b| a.saturating_add(b))
}
