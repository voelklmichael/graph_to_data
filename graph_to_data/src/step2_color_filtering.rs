pub fn color_filtering(
    image: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    target_color: &image::Rgba<u8>,
    settings: &crate::Settings,
) -> image::ImageBuffer<image::Luma<u8>, Vec<u8>> {
    imageproc::map::map_pixels(image, |_, _, p| {
        let diff = crate::color_distance(&p, target_color);
        if diff < settings.step1_step2_color_radius {
            crate::HIT
        } else {
            crate::MISSED
        }
    })
}
