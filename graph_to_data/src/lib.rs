pub const GOLD_AS_RGB: [u8; 3] = [218, 165, 32];
const HIT: image::Luma<u8> = image::Luma([255]);
const MISSED: image::Luma<u8> = image::Luma([0]);

mod step0_crop;
mod step1_color_extraction;
mod step2_color_filtering;
mod step3_group;
mod step4_stitch;
mod unit_geometry;

use std::path::Path;

use itertools::Itertools;
pub use unit_geometry::{UnitInterval, UnitPoint, UnitQuadrilateral};

pub type ImageRgba = (image::Rgba<u8>, Vec<(f32, f32)>);

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(default)]
pub struct Settings {
    pub step1_width_minimial_fraction: f32,
    pub step1_height_maximal_fraction: f32,
    pub step1_ignore_gray: bool,
    pub step1_close_count: u8,
    pub step1_step2_color_radius: u8,
    pub step3_min_width_fraction: f32,
    pub step4_component_jump_height_fraction: f32,
    pub step6_fit_graph_color: Option<[u8; 3]>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            step1_step2_color_radius: 5,
            step1_width_minimial_fraction: 0.3,
            step1_height_maximal_fraction: 0.1,
            step1_close_count: 0,
            step1_ignore_gray: true,
            step3_min_width_fraction: 0.05,
            step4_component_jump_height_fraction: 0.02,
            step6_fit_graph_color: Some(GOLD_AS_RGB),
        }
    }
}

fn color_distance(cc: &image::Rgba<u8>, c: &image::Rgba<u8>) -> u8 {
    cc.0.iter()
        .zip(c.0)
        .map(|(&c, cc)| cc.max(c) - cc.min(c))
        .fold(0, |a, b| a.saturating_add(b))
}
fn color_distance_three(cc: &image::Rgb<u8>, c: &image::Rgba<u8>) -> u8 {
    cc.0.iter()
        .zip(c.0)
        .map(|(&c, cc)| cc.max(c) - cc.min(c))
        .fold(0, |a, b| a.saturating_add(b))
}

#[derive(Debug)]
pub enum Error {
    StepSettingsInvalid { steps_x: u32, steps_y: u32 },
    CroppedImageToSmall { width: u32, height: u32 },
}
#[derive(Default)]
pub struct LineDetected {
    cropped: Option<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>,
    colors: Option<Vec<image::Rgba<u8>>>,
    color_filtered: Vec<image::ImageBuffer<image::Luma<u8>, Vec<u8>>>,
    grouped_image: Vec<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>,
    stitched_image: Vec<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>,
    aggregated_image: Vec<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>,
    remaining_vertices: Vec<Vec<step3_group::CombinedVerticals>>,
    graphs: Vec<(image::Rgba<u8>, Vec<step3_group::GraphMultiNode>)>,
    cropped_with_plots: Option<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>,
    plots: Vec<ImageRgba>,
}
impl LineDetected {
    pub fn save<P: AsRef<std::path::Path>>(&self, output_folder: P) -> image::ImageResult<()> {
        self.save_internal(output_folder.as_ref())
    }
    fn save_internal(&self, output_folder: &Path) -> image::ImageResult<()> {
        let Self {
            cropped,
            colors: _,
            color_filtered,
            grouped_image,
            stitched_image,
            aggregated_image,
            remaining_vertices: _,
            graphs: _,
            cropped_with_plots: image_with_plots,
            plots: _,
        } = self;
        if let Some(cropped) = cropped {
            cropped.save(output_folder.join("step0_cropped.png"))?;
        }
        for (index, color_filtered) in color_filtered.iter().enumerate() {
            color_filtered.save(output_folder.join(format!("step2_{index}_color_filtered.png")))?;
        }
        for (index, grouped_image) in grouped_image.iter().enumerate() {
            grouped_image
                .save(output_folder.join(format!("step3_{index}_large_components.png")))?;
        }
        for (index, stitched_image) in stitched_image.iter().enumerate() {
            stitched_image.save(output_folder.join(format!("step4_{index}_stitched.png")))?;
        }
        for (index, aggregated_image) in aggregated_image.iter().enumerate() {
            aggregated_image.save(output_folder.join(format!("step5_{index}_aggregate.png")))?;
        }
        if let Some(image_with_plots) = image_with_plots {
            image_with_plots.save(output_folder.join("final_image_with_plots.png"))?;
        }

        Ok(())
    }

    pub fn final_image_with_plots(&self) -> Option<&image::ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
        self.cropped_with_plots.as_ref()
    }

    pub fn as_csv(&self) -> String {
        let mut x_grid = Vec::new();
        for (_, graph) in &self.plots {
            x_grid.extend(graph.iter().map(|(x, _)| *x));
        }
        x_grid.sort_by(|left, right| {
            if left < right {
                std::cmp::Ordering::Less
            } else if left == right {
                std::cmp::Ordering::Equal
            } else {
                std::cmp::Ordering::Greater
            }
        });
        let mut ys = Vec::with_capacity(self.plots.len());
        let mut header = vec!["x".to_string()];
        for (_, graph) in &self.plots {
            let mut yy = vec![f32::NAN; x_grid.len()];
            let mut previous_index = 0;
            for (x, y) in graph {
                let index = x_grid
                    .iter()
                    .skip(previous_index)
                    .position(|xx| xx == x)
                    .unwrap();
                yy[index + previous_index] = *y;
                previous_index += index;
            }
            ys.push(yy);
            header.push(format!("Graph #{}", header.len()));
        }
        let mut lines = vec![header.join(";")];
        for (i, x) in x_grid.into_iter().enumerate() {
            let mut line = Vec::with_capacity(self.plots.len() + 1);
            line.push(x.to_string());
            for yy in &ys {
                line.push(yy[i].to_string())
            }
            lines.push(line.join(";"));
        }
        lines.join("\n")
    }
}
pub fn line_detection(
    image: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    settings: &Settings,
    quadrilateral: UnitQuadrilateral,
    steps_x: u32,
    steps_y: u32,
    x_limits: (f32, f32),
    y_limits: (f32, f32),
) -> Result<LineDetected, Error> {
    if steps_x < 100 || steps_y < 100 {
        return Err(Error::StepSettingsInvalid { steps_x, steps_y });
    }
    let cropped = step0_crop::ImageInterpolate::crop(image, quadrilateral, steps_x, steps_y);

    if cropped.width() < 100 && cropped.height() < 100 {
        return Err(Error::CroppedImageToSmall {
            width: cropped.width(),
            height: cropped.height(),
        });
    }
    let mut line_detected = LineDetected {
        cropped: Some(cropped),
        ..Default::default()
    };
    let cropped = line_detected.cropped.as_ref().unwrap();
    // step 1 - extract colors
    let colors = step1_color_extraction::extract_colors(cropped, settings);
    //line_detected.colors = Some(colors);
    //let colors = line_detected.colors.as_ref().unwrap();
    let mut colors_to_use = Vec::new();
    for color in colors {
        const H: u8 = 255;
        const M: u8 = 128;
        const N: u8 = 0;

        // step 2 - filter colors
        let color_filtered = step2_color_filtering::color_filtering(cropped, &color, settings);
        let counts = (0..color_filtered.width())
            .map(|x| {
                (0..color_filtered.height())
                    .filter(|y| color_filtered.get_pixel(x, *y) == &HIT)
                    .count()
            })
            .collect_vec();
        if counts.iter().any(|hits| {
            (*hits as f32 / color_filtered.height() as f32) > settings.step1_height_maximal_fraction
        }) || (counts.iter().filter(|hits| hits > &&0).count() as f32
            / color_filtered.width() as f32)
            < settings.step1_width_minimial_fraction
        {
            continue;
        }
        {
            let mut opened = color_filtered.clone();
            for _ in 0..settings.step1_close_count {
                opened = imageproc::morphology::open(
                    &opened,
                    imageproc::distance_transform::Norm::LInf,
                    1,
                );
                opened
                    .iter_mut()
                    .zip(color_filtered.iter())
                    .for_each(|(a, b)| *a = (*a).min(*b));
            }
            if opened.iter().all(|&p| p == 0) {
                continue;
            }
        }
        line_detected.color_filtered.push(color_filtered);
        let color_filtered = line_detected.color_filtered.last().unwrap();

        colors_to_use.push(color);
        // step 3 - group into large components and remaining
        let (large_components, mut remaining_verticals) = {
            let (large_components, remaining_verticals) =
                step3_group::group_large_components_and_remaining(color_filtered, settings);

            let mut grouped_image = imageproc::map::map_colors(color_filtered, |c| {
                if c == image::Luma([H; 1]) {
                    image::Rgba([N, N, M, H])
                } else {
                    image::Rgba([N, N, N, H])
                }
            });
            for (color_index, component) in large_components.iter().enumerate() {
                let color = match color_index % 7 {
                    0 => image::Rgba([H, H, H, H]),
                    1 => image::Rgba([H, H, N, H]),
                    2 => image::Rgba([N, H, H, H]),
                    3 => image::Rgba([H, N, H, H]),
                    4 => image::Rgba([H, N, N, H]),
                    5 => image::Rgba([N, H, N, H]),
                    6 => image::Rgba([N, N, H, H]),
                    _ => unreachable!(),
                };
                for (x, y) in component.ys.iter().enumerate() {
                    if let Some(y) = y.mean() {
                        *grouped_image.get_pixel_mut(x as _, y) = color;
                    }
                }
            }
            for vertical in &remaining_verticals {
                let step3_group::CombinedVerticals { x_start, combined } = vertical;
                for (x_offset, ys) in combined.iter().enumerate() {
                    let x = x_start.0 + x_offset as u32;
                    let y = ys.mean();
                    let color = image::Rgba([M, M, M, H]);
                    *grouped_image.get_pixel_mut(x as _, y) = color;
                }
            }
            line_detected.grouped_image.push(grouped_image);

            (large_components, remaining_verticals)
        };
        // step 4 - combine components/remaining
        let graphs = {
            let graphs = step4_stitch::stitch(
                large_components,
                &mut remaining_verticals,
                settings,
                color_filtered,
            );

            let mut stitched_image = imageproc::map::map_colors(color_filtered, |c| {
                if c == image::Luma([H; 1]) {
                    image::Rgba([N, N, M, H])
                } else {
                    image::Rgba([N, N, N, H])
                }
            });
            for (color_index, graph) in graphs.iter().enumerate() {
                let color = match color_index % 7 {
                    0 => image::Rgba([H, H, H, H]),
                    1 => image::Rgba([H, H, N, H]),
                    2 => image::Rgba([N, H, H, H]),
                    3 => image::Rgba([H, N, H, H]),
                    4 => image::Rgba([H, N, N, H]),
                    5 => image::Rgba([N, H, N, H]),
                    6 => image::Rgba([N, N, H, H]),
                    _ => unreachable!(),
                };
                for (x, y) in graph.ys.iter().enumerate() {
                    if let Some(y) = y.mean() {
                        *stitched_image.get_pixel_mut(x as _, y) = color;
                    }
                }
            }
            line_detected.stitched_image.push(stitched_image);
            line_detected.remaining_vertices.push(remaining_verticals);
            graphs
        };
        // step 5 - combine components
        let graphs = {
            let aggregate = {
                if graphs.iter().enumerate().any(|(i, g1)| {
                    graphs
                        .iter()
                        .enumerate()
                        .any(|(j, g2)| if i == j { false } else { g1.overlaps(g2) })
                }) {
                    graphs
                } else {
                    let mut graphs = graphs;
                    if let Some(mut g) = graphs.pop() {
                        for h in graphs {
                            g.aggregate(h);
                        }
                        vec![g]
                    } else {
                        continue;
                    }
                }
            };
            let mut aggregate_image = imageproc::map::map_colors(color_filtered, |c| {
                if c == image::Luma([H; 1]) {
                    image::Rgba([N, N, M, H])
                } else {
                    image::Rgba([N, N, N, H])
                }
            });
            for (color_index, graph) in aggregate.iter().enumerate() {
                let color = match color_index % 7 {
                    0 => image::Rgba([H, H, H, H]),
                    1 => image::Rgba([H, H, N, H]),
                    2 => image::Rgba([N, H, H, H]),
                    3 => image::Rgba([H, N, H, H]),
                    4 => image::Rgba([H, N, N, H]),
                    5 => image::Rgba([N, H, N, H]),
                    6 => image::Rgba([N, N, H, H]),
                    _ => unreachable!(),
                };
                for (x, y) in graph.ys.iter().enumerate() {
                    if let Some(y) = y.mean() {
                        *aggregate_image.get_pixel_mut(x as _, y) = color;
                    }
                }
            }

            line_detected.aggregated_image.push(aggregate_image);
            aggregate
        };
        if !graphs.is_empty() {
            line_detected.graphs.push((color, graphs));
        }
    }
    if let Some(cropped) = &line_detected.cropped {
        if !line_detected.graphs.is_empty() {
            let mut image_with_plots = cropped.clone();
            for (color, graphs) in &line_detected.graphs {
                let color = if let Some([r, g, b]) = settings.step6_fit_graph_color {
                    image::Rgba([r, g, b, 255])
                } else {
                    *color
                };
                for graph in graphs {
                    for (x, y) in graph.ys.iter().enumerate() {
                        if let Some(y) = y.mean() {
                            *image_with_plots.get_pixel_mut(x as _, y) = color;
                        }
                    }
                }
            }
            line_detected.cropped_with_plots = Some(image_with_plots);
        }
    }
    line_detected.colors = Some(colors_to_use);
    line_detected.plots = line_detected
        .graphs
        .iter()
        .flat_map(|(color, graphs)| {
            graphs.iter().map(|graph| {
                let plot = graph.to_plot(x_limits, y_limits, steps_x, steps_y);
                (*color, plot)
            })
        })
        .collect();

    Ok(line_detected)
}
