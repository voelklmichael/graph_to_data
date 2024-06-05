const H: u8 = 255;
const M: u8 = 128;
const N: u8 = 0;

fn main() {
    let image_bytes = include_bytes!(
        //"../example_data/Mplwp_dispersion_curves.svg.png"
        //"../example_data/Polynomial_of_degree_three.svg.png"
        //"../example_data/X^4_4^x.PNG"
        //"../example_data/FFT_of_Cosine_Summation_Function.svg.png"
        "../example_data/Tuberculosis_incidence_US_1953-2009.png"
    );
    let image = image::io::Reader::new(std::io::Cursor::new(image_bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .to_rgb8();

    if image.width() < 100 && image.height() < 100 {
        panic!("Image is too small")
    }
    let settings = graph_to_data::Settings::default();
    // step 1 - extract colors
    let colors = graph_to_data::extract_colors(&image, &settings);
    for (index, color) in colors.iter().enumerate() {
        // step 2 - filter colors
        let color_filtered = graph_to_data::color_filtering(&image, color, &settings);
        color_filtered
            .save(format!("step2_{index}_color_filtered.png"))
            .unwrap();
        // step 3 - group into large components and remaining
        let (large_components, mut remaining_verticals) = {
            let (large_components, remaining_verticals) =
                graph_to_data::group_large_components_and_remaining(&color_filtered, &settings);

            let mut grouped_image = imageproc::map::map_colors(&color_filtered, |c| {
                if c == image::Luma([H; 1]) {
                    image::Rgba([N, N, M, H])
                } else {
                    image::Rgba([N, N, N, H])
                }
            });
            for (color_index, component) in large_components.iter().enumerate() {
                dbg!(component.x_used_count());
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
            dbg!("Remaining verticals: ", remaining_verticals.len());
            for vertical in &remaining_verticals {
                let graph_to_data::CombinedVerticals { x_start, combined } = vertical;
                for (x_offset, ys) in combined.iter().enumerate() {
                    let x = x_start.0 + x_offset as u32;
                    let y = ys.mean();
                    let color = image::Rgba([M, M, M, H]);
                    *grouped_image.get_pixel_mut(x as _, y) = color;
                }
            }

            grouped_image
                .save(format!("step3_{index}_large_components.png"))
                .unwrap();
            (large_components, remaining_verticals)
        };
        // step 4 - combine components/remaining
        {
            let graphs = graph_to_data::stitch(
                large_components,
                &mut remaining_verticals,
                &settings,
                &color_filtered,
            );

            let mut stitched_image = imageproc::map::map_colors(&color_filtered, |c| {
                if c == image::Luma([H; 1]) {
                    image::Rgba([N, N, M, H])
                } else {
                    image::Rgba([N, N, N, H])
                }
            });
            for (color_index, graph) in graphs.iter().enumerate() {
                dbg!(graph.x_used_count());
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
            dbg!("Remaining verticals: ", remaining_verticals.len());

            stitched_image
                .save(format!("step4_{index}_stitched.png"))
                .unwrap();
        }
    }
    return;

    /*

    */

    /*let curve_count = match vertical_components.max_component_count() {
        Some(curve_count) => curve_count,
        None => panic!("No curves found - your color does not appear in plot"),
    };
    dbg!(curve_count);

    let connected_components_parts =
        vertical_components.group_into_connected_components(Default::default());
    dbg!(connected_components_parts);*/
    /*
        let mut connected_components = graph_to_data::helpers::stitch_components(
            connected_components_parts,
            settings.stitch_x_diff_max,
        );
        connected_components.sort_by_key(|x| x.points.len());
        connected_components.reverse();

        let mut connected_components_image = imageproc::map::map_colors(
            &color_filtered,
            //&dilated,
            |c| {
                if c == HIT {
                    image::Rgba([0u8, 0, 128, H])
                } else {
                    image::Rgba([0u8, 0, 0, H])
                }
            },
        );
        for (color_index, component) in connected_components.iter().enumerate() {
            let component = &component.points;
            dbg!(component.len());
            let color = match color_index {
                0 => image::Rgba([H, H, H, H]),
                1 => image::Rgba([H, H, 0, H]),
                2 => image::Rgba([0, H, H, H]),
                3 => image::Rgba([H, 0, H, H]),
                4 => image::Rgba([H, 0, 0, H]),
                5 => image::Rgba([0, H, 0, H]),
                6 => image::Rgba([0, 0, H, H]),
                _ => image::Rgba([H / 2, H / 2, 0, H]),
            };

            for (x, y) in component {
                *connected_components_image.get_pixel_mut(*x as _, *y) = color;
            }
        }

        connected_components_image
            .save("step4_stitched_components.png")
            .unwrap();
    */

    // part 4 - stitch parts together

    /*return;

    let connected_components = {
        let mut connected_components = Vec::new();
        while let Some((x, yy)) = vertical_components
            .iter_mut()
            .enumerate()
            .filter_map(|(x, yy)| yy.pop().map(|yy| (x, yy)))
            .next()
        {
            let (mut min, mut max) = yy;
            let mut connected_component = vec![(x, (min + max) / 2)];
            for (x, verticals) in vertical_components.iter_mut().enumerate().skip(x + 1) {
                if let Some(index) = verticals.iter().position(|&(mmin, mmax)| {
                    let jumping = settings.step3_jump_step_size;
                    min.saturating_sub(jumping) <= mmin && max.saturating_add(jumping) >= mmax
                }) {
                    (min, max) = verticals.remove(index);
                    connected_component.push((x, (min + max) / 2));
                } else {
                    break;
                }
            }
            //if connected_component.len() >= settings.step3_connected_component_min_length {
            connected_components.push(connected_component);
            //}
        }
        connected_components
    };
    let mut connected_components_image = imageproc::map::map_colors(&color_filtered, |c| {
        if c == HIT {
            image::Rgba([0u8, 0, 128, H])
        } else {
            image::Rgba([0u8, 0, 0, H])
        }
    });
    for connected_component in &connected_components {
        let fraction = connected_component.len() as f32 / image.width() as f32;
        let color = if fraction > 0.1 {
            image::Rgba([h, H, H, H])
        } else if fraction > 0.01 {
            image::Rgba([0u8, H, 0, H])
        } else {
            image::Rgba([h, 0, 0, H])
        };
        for (x, y) in connected_component {
            *connected_components_image.get_pixel_mut(*x as _, *y) = color;
        }
    }
    connected_components_image
        .save("step3_connected_components.png")
        .unwrap();
    let mut cc_lengths = connected_components
        .iter()
        .map(|x| x.len())
        .collect::<Vec<_>>();
    cc_lengths.sort();
    cc_lengths.reverse();
    dbg!(&cc_lengths[0..25.min(cc_lengths.len())]);
    dbg!(image.width(), image.height());
    /*let mut used =
        image::ImageBuffer::<image::Luma<u8>, Vec<u8>>::new(dilated.width(), dilated.height());
    for x in 0..dilated.width() {
        for y in 0..dilated.height() {
            if dilated.get_pixel(x, y) == &HIT && used.get_pixel(x, y) != &HIT {

            }
        }
    }*/

    /*
    let hit_components = (0..color_filtered.width())
        .map(|x| {
            let mut components = Vec::new();
            let mut component = Vec::new();
            for y in 0..color_filtered.height() {
                if color_filtered.get_pixel(x, y) == &HIT {
                    component.push(y);
                } else {
                    finish_component(&mut component, &mut components);
                }
            }
            finish_component(&mut component, &mut components);
            components
        })
        .collect::<Vec<_>>();
    */

    /*
    /*let x = (image.width() / 3) as usize;
    dbg!(x, &hit_components[x]);*/
    let offset = color_filtered.height() * 2 / 100;
    let target_component = {
        let [tx, ty] = target_position;
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
            for x in (start.0 + 1)..color_filtered.width() {
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
    //dbg!(&target_component);
    */
    /*let mut single_component = imageproc::map::map_colors(&color_filtered, |p| {
        if p == HIT {
            image::Rgba([h, H, H, H])
        } else {
            image::Rgba([0u8, 0, 0, H])
        }
    });
    for (x, y) in &target_component {
        *single_component.get_pixel_mut(*x, *y) = image::Rgba([0u8, H, 0, H]);
    }
    single_component.save("single_component.png").unwrap();
    */

    /*
    let single_component = imageproc::map::map_pixels(&filtered, |x, y, _| {
        let components = &hit_components[x as usize];
        if components.iter().any(|c| *c == y) {
            if components.len() == 1 {
                image::Rgba([0u8, H, 0, H])
            } else if components.len() == 1 {
                image::Rgba([0u8, 0, H, H])
            } else {
                image::Rgba([h, 0, 0, H])
            }
        } else {
            image::Rgba([0u8, 0, 0, H])
        }
    });
    single_component.save("single_components.png").unwrap();
    let mut long_components = Vec::new();
    let mut current: Option<(usize, usize, Vec<u32>)> = None;
    for (x, components) in hit_components.iter().enumerate() {
        if components.len() == 1 {
            let y = components[0];
            if let Some((_start, end, yy)) = &mut current {
                let yyy = *yy.last().unwrap();
                let diff = yyy.max(y) - yyy.min(y);
                if x == *end + 1 && diff <= 3 {
                    *end = x;
                    yy.push(y);
                } else {
                    complete(&mut current, &mut long_components, filtered.width());
                    current = Some((x, x, vec![y]));
                }
            } else {
                current = Some((x, x, vec![y]));
            }
        } else {
            complete(&mut current, &mut long_components, filtered.width());
        }
    }
    complete(&mut current, &mut long_components, filtered.width());
    for long in &long_components {
        println!(
            "{s}->{e}",
            s = long.first().unwrap().0,
            e = long.last().unwrap().0
        );
    }*/

    /*let diameter =
        (color_filtered.width().pow(2) as f32 + color_filtered.height().pow(2) as f32).sqrt();
    let dilated = imageproc::morphology::dilate(
        &color_filtered,
        imageproc::distance_transform::Norm::LInf,
        (diameter * 0.01) as _,
    );
    dilated.save("dilated.png").unwrap();*/
    /*
        fn complete(
            current: &mut Option<(usize, usize, Vec<u32>)>,
            long_components: &mut Vec<Vec<(usize, u32)>>,
            width: u32,
        ) {
            if let Some((current_start, current_end, ys)) = current.take() {
                if current_end - current_start > ((width as f32) * 0.01) as usize {
                    long_components.push(
                        ys.into_iter()
                            .enumerate()
                            .map(|(i, y)| (i + current_start, y))
                            .collect(),
                    );
                }
            }
        }
    */

    */
}

/*
fn finish_component(component: &mut Vec<u32>, components: &mut Vec<(u32, u32)>) {
    if !component.is_empty() {
        let component = std::mem::take(component);
        components.push((
            component.iter().cloned().min().unwrap(),
            component.into_iter().max().unwrap(),
        ));
    }
}
*/
