use crate::{
    step3_group::{CombinedVerticals, Distance, GraphMultiNode},
    Settings,
};

pub fn stitch(
    large_components: Vec<GraphMultiNode>,
    remaining_verticals: &mut Vec<CombinedVerticals>,
    settings: &Settings,
    image: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
) -> Vec<GraphMultiNode> {
    let mut components = large_components;
    let max_distance =
        ((settings.step4_component_jump_height_fraction * image.height() as f32) as u32).max(2);
    'outer: loop {
        // combining components
        if let Some((i, j, d)) = {
            components
                .iter()
                .enumerate()
                .flat_map(|(i, c)| {
                    components
                        .iter()
                        .enumerate()
                        .skip(i + 1)
                        .map(move |(j, cc)| (i, j, cc.distance(c)))
                })
                .min_by_key(|(_, _, d)| *d)
        } {
            if d <= max_distance {
                let c = components.remove(j);
                components[i].stitch_together(c);
                continue 'outer;
            }
        }
        // add verticals to components
        if let Some((vertical_index, comp_index, d)) = {
            remaining_verticals
                .iter()
                .enumerate()
                .flat_map(|(vertical_index, v)| {
                    components
                        .iter()
                        .map(|c| c.distance_to_vertical(v))
                        .enumerate()
                        .filter_map(move |(comp_index, d)| match d {
                            Distance::CanBeExtendend { distance } => {
                                Some((vertical_index, comp_index, distance))
                            }
                            Distance::CannotBeExtended => None,
                        })
                })
                .min_by_key(|(_, _, d)| *d)
        } {
            if d <= max_distance {
                let v = remaining_verticals.remove(vertical_index);
                components[comp_index].merge(v);
                continue 'outer;
            } else {
                break;
            }
        }
        break;
    }
    components
}
