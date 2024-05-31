const H: u8 = 255;

struct Settings {
    step1_color_radius: u8,
    stitch_x_diff_max: usize,
    //step2_dilation_steps: u8,
    //step3_jump_step_size: u32,
    //step3_connected_component_min_length_fraction: UnitInterval,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            step1_color_radius: 180,
            stitch_x_diff_max: 8, //step2_dilation_steps: 0,
                                  //step3_jump_step_size: 3,
                                  //step3_connected_component_min_length_fraction: UnitInterval::new(0.05).unwrap(),
        }
    }
}

fn main() {
    let settings = Settings::default();
    let image_bytes = include_bytes!(
        "../example_data/Mplwp_dispersion_curves.svg.png" //"../example_data/Polynomial_of_degree_three.svg.png"
                                                          //"../example_data/X^4_4^x.PNG"
                                                          //"../example_data/FFT_of_Cosine_Summation_Function.svg.png"
    );
    let image = image::io::Reader::new(std::io::Cursor::new(image_bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    /*let l = 0.1;
    let r = 0.956;
    let t = 0.06;
    let b = 0.875;
    let cropped = image.interpolate_image(
        UnitPoint::unchecked(l, t),
        UnitPoint::unchecked(l, b),
        UnitPoint::unchecked(r, t),
        UnitPoint::unchecked(r, b),
        150,
        200,
    );
    cropped.save("cropped.png").unwrap();*/

    //let target_position = [426, 300];
    let target_color = image::Rgba([0u8, 0, 200, H]);
    //let target_color = image::Rgba([200u8, 0, 0, H]);
    //let target_color = dbg!(image.get_pixel(target_position[0], target_position[1]));

    const HIT: image::Luma<u8> = image::Luma([H]);
    const MISSED: image::Luma<u8> = image::Luma([0]);
    let color_filtered = imageproc::map::map_pixels(&image, |_, _, p| {
        let p = p.0;
        let diff = p
            .into_iter()
            .zip(target_color.0)
            .map(|(p, t)| p.max(t) - p.min(t))
            .take(3)
            .fold(0u8, |p, c| p.saturating_add(c));
        if diff < settings.step1_color_radius {
            HIT
        } else {
            MISSED
        }
    });
    color_filtered.save("step1_color_filtered.png").unwrap();

    /*
    let dilated = imageproc::morphology::dilate(
        &color_filtered,
        imageproc::distance_transform::Norm::LInf,
        settings.step2_dilation_steps,
    );
    dilated.save("step2_dilated.png").unwrap();
    */

    // find vertical components
    let vertical_components = graph_to_data::helpers::vertical_components(&color_filtered);
    let curve_count = match vertical_components.iter().map(|x| x.len()).max() {
        Some(curve_count) => curve_count,
        None => panic!("No curves found - your color does not appear in plot"),
    };
    dbg!(curve_count);

    let connected_components_parts =
        graph_to_data::helpers::connect_vertical_components(vertical_components);

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
