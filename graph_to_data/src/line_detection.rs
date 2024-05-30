use crate::UnitInterval;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct ImagePixel {
    pub x: u32,
    pub y: u32,
    pub color: [u8; 4],
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(default)]
pub struct LineDetectionSettings {
    /// This controls when two colors are considered equal (L1-metric is used)
    max_color_diff: u8,
    /// How much can a line jump?
    line_jump_fraction: UnitInterval,
}
impl Default for LineDetectionSettings {
    fn default() -> Self {
        Self {
            max_color_diff: 15,
            line_jump_fraction: UnitInterval::new(0.035).unwrap(),
        }
    }
}
pub trait LineDetection {
    fn detect_line(
        &self,
        first_point: ImagePixel,
        settings: &LineDetectionSettings,
    ) -> Vec<(u32, u32)>;
}
impl LineDetection for image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    fn detect_line(
        &self,
        first_point: ImagePixel,
        settings: &LineDetectionSettings,
    ) -> Vec<(u32, u32)> {
        const HIT: image::Luma<u8> = image::Luma([255u8]);
        const MISSED: image::Luma<u8> = image::Luma([0]);
        let filtered = imageproc::map::map_pixels(self, |_, _, p| {
            let p = p.0;
            let diff = p
                .into_iter()
                .zip(first_point.color)
                .map(|(p, t)| p.max(t) - p.min(t))
                .fold(0u8, |p, c| p.saturating_add(c));
            if diff < settings.max_color_diff {
                HIT
            } else {
                MISSED
            }
        });
        //filtered.save("filtered.png").unwrap();
        #[inline(always)]
        fn finish_component(component: &mut Vec<u32>, components: &mut Vec<(u32, u32)>) {
            if !component.is_empty() {
                let component = std::mem::take(component);
                components.push((
                    component.iter().cloned().min().unwrap(),
                    component.into_iter().max().unwrap(),
                ));
            }
        }
        let hit_components = (0..filtered.width())
            .map(|x| {
                let mut components = Vec::new();
                let mut component = Vec::new();
                for y in 0..filtered.height() {
                    if filtered.get_pixel(x, y) == &HIT {
                        component.push(y);
                    } else {
                        finish_component(&mut component, &mut components);
                    }
                }
                finish_component(&mut component, &mut components);
                components
            })
            .collect::<Vec<_>>();

        let offset = (filtered.height() as f32 * settings.line_jump_fraction.0) as u32;
        let target_component = {
            let tx = first_point.x;
            let ty = first_point.y;
            let target_component = hit_components[tx as usize]
                .iter()
                .find(|(min, max)| min <= &ty && max >= &ty)
                .unwrap()
                .clone();
            let start = (tx, (target_component.0 + target_component.1) / 2);
            // enlarge to the right
            let right = {
                let mut right = Vec::new();
                let mut previous_component = target_component;
                for x in (start.0 + 1)..filtered.width() {
                    let current_components = &hit_components[x as usize];
                    let (pmin, pmax) = previous_component;
                    if let Some(new_component) = current_components.iter().find(|(min, max)| {
                        min.saturating_add(offset) >= pmin && max.saturating_sub(offset) <= pmax
                    }) {
                        right.push((x, (new_component.0 + new_component.1) / 2));
                        previous_component = *new_component;
                    } else {
                        break;
                    }
                }
                right
            };
            // enlarge to the left
            let mut left = {
                let mut left = Vec::new();
                let mut previous_component = target_component;
                for x in 0..start.0 {
                    let x = start.0 - x - 1;
                    let current_components = &hit_components[x as usize];
                    let (pmin, pmax) = previous_component;
                    if let Some(new_component) = current_components.iter().find(|(min, max)| {
                        min.saturating_add(offset) >= pmin && max.saturating_sub(offset) <= pmax
                    }) {
                        left.push((x, (new_component.0 + new_component.1) / 2));
                        previous_component = *new_component;
                    } else {
                        break;
                    }
                }
                left
            };
            left.reverse();
            left.push(start);
            left.extend(right);
            left
        };

        target_component
    }
}
