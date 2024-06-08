use crate::color_distance;

#[derive(Debug, Default)]
struct ColorExtractor {
    colors: Vec<image::Rgb<u8>>,
    color_occurences: Vec<Vec<u32>>,
}

pub fn extract_colors(
    image: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>,
    settings: &crate::Settings,
) -> Vec<image::Rgb<u8>> {
    let color_extractor = ColorExtractor::classify_image(
        image,
        settings.step1_step2_color_radius,
        settings.step1_ignore_gray,
    );
    color_extractor.extract(
        image,
        settings.step1_width_minimial_fraction,
        settings.step1_height_maximal_fraction,
    )
}
impl ColorExtractor {
    fn classify_image(
        image: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>,
        color_radius: u8,
        ignore_gray: bool,
    ) -> Self {
        let mut colors: Vec<image::Rgb<u8>> = Vec::new();
        let mut color_occurences = Vec::new();
        for x in 0..image.width() {
            for y in 0..image.height() {
                let c = image.get_pixel(x, y);
                let mut mean = c.0;
                mean.sort();
                let mean = mean[1];
                if color_distance(&image::Rgb([255, 255, 255]), c) < color_radius
                    || color_distance(&image::Rgb([0, 0, 0]), c) < color_radius
                    || (ignore_gray
                        && color_distance(&image::Rgb([mean, mean, mean]), c) < color_radius)
                {
                    continue;
                }
                let color_occurences = {
                    if let Some(color_index) = colors
                        .iter()
                        .position(|cc| color_distance(cc, c) <= color_radius)
                    {
                        color_occurences.get_mut(color_index).unwrap()
                    } else {
                        colors.push(*c);
                        color_occurences.push(vec![0u32; image.width() as _]);
                        color_occurences.last_mut().unwrap()
                    }
                };
                color_occurences[x as usize] += 1;
            }
        }
        Self {
            colors,
            color_occurences,
        }
    }

    fn extract(
        self,
        image: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>,
        width_minimial_fraction: f32,
        height_maximal_fraction: f32,
    ) -> Vec<image::Rgb<u8>> {
        let Self {
            mut colors,
            color_occurences,
        } = self;

        for (color_index, color_occurence) in color_occurences.into_iter().enumerate().rev() {
            let max = color_occurence.iter().cloned().max().unwrap();
            let count = color_occurence.into_iter().filter(|x| x > &0).count();
            if (count as f32 / image.width() as f32) < width_minimial_fraction
                || (max as f32 / image.height() as f32) > height_maximal_fraction
            {
                colors.remove(color_index);
            }
        }
        colors
    }
}
