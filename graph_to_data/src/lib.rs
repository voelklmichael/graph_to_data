mod unit_geometry;
pub use unit_geometry::{UnitInterval, UnitPoint, UnitQuadrilateral};

mod image_interpolate;
pub use image_interpolate::ImageInterpolate;

mod line_detection;
pub use line_detection::{ImagePixel, LineDetection, LineDetectionSettings};

#[test]
fn test_line_dection() {
    let image_bytes = include_bytes!("../example_data/Mplwp_dispersion_curves.svg.png");
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

    let target_position = [426, 300];
    //let target_color = image::Rgba([0u8, 0, 204, 255]);
    use image::GenericImageView;
    let target_color = dbg!(image.get_pixel(target_position[0], target_position[1]));

    let diff_max = 50;
    const HIT: image::Luma<u8> = image::Luma([255u8]);
    const MISSED: image::Luma<u8> = image::Luma([0]);
    let filtered = imageproc::map::map_pixels(&image, |_, _, p| {
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

    /*let x = (image.width() / 3) as usize;
    dbg!(x, &hit_components[x]);*/
    let offset = filtered.height() * 2 / 100;
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
    //dbg!(&target_component);

    let mut single_component = imageproc::map::map_colors(&filtered, |p| {
        if p == HIT {
            image::Rgba([255u8, 255, 255, 255])
        } else {
            image::Rgba([0u8, 0, 0, 255])
        }
    });
    for (x, y) in &target_component {
        *single_component.get_pixel_mut(*x, *y) = image::Rgba([0u8, 255, 0, 255]);
    }
    single_component.save("single_component.png").unwrap();

    /*
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

    let diameter = (filtered.width().pow(2) as f32 + filtered.height().pow(2) as f32).sqrt();
    let dilated = imageproc::morphology::dilate(
        &filtered,
        imageproc::distance_transform::Norm::LInf,
        (diameter * 0.01) as _,
    );
    dilated.save("dilated.png").unwrap();
}

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

fn finish_component(component: &mut Vec<u32>, components: &mut Vec<(u32, u32)>) {
    if !component.is_empty() {
        let component = std::mem::take(component);
        components.push((
            component.iter().cloned().min().unwrap(),
            component.into_iter().max().unwrap(),
        ));
    }
}
