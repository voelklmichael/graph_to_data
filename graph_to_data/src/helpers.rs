const HIT: image::Luma<u8> = image::Luma([255]);

#[derive(Debug, Clone, Copy)]
pub struct Vertical {
    pub min: u32,
    pub max: u32,
}
impl Vertical {
    fn distance(&self, other: Vertical) -> u32 {
        if self.min > other.max {
            self.min - other.max
        } else if other.min > self.max {
            other.min - self.max
        } else {
            0
        }
    }

    fn mean(self) -> u32 {
        (self.min + self.max) / 2
    }

    fn merge(&mut self, other: Vertical) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }
}
pub fn vertical_components(
    image: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
) -> Vec<Vec<Vertical>> {
    let vertical_components = (0..image.width())
        .map(|x| {
            let mut components = Vec::new();
            let mut current_component = Vec::new();
            #[inline(always)]
            fn complete_component(
                current_component: &mut Vec<u32>,
                components: &mut Vec<Vertical>,
            ) {
                let current_component = std::mem::take(current_component);
                if !current_component.is_empty() {
                    let min = *current_component.first().unwrap();
                    let max = *current_component.last().unwrap();
                    components.push(Vertical { min, max });
                }
            }
            for y in 0..image.height() {
                if image.get_pixel(x, y) == &HIT {
                    current_component.push(y);
                } else {
                    complete_component(&mut current_component, &mut components)
                }
            }
            complete_component(&mut current_component, &mut components);
            components
        })
        .collect::<Vec<_>>();
    vertical_components
}

pub struct StitchedComponentInternal {
    parts: Vec<ComponentPart>,
}
impl StitchedComponentInternal {
    fn convert(self) -> StitchedComponent {
        // fetch x positions
        let xx = self
            .parts
            .iter()
            .flat_map(|x| x.points.iter().map(|p| p.0))
            .collect::<Vec<_>>();
        let mut xx_unique = xx.clone();
        xx_unique.sort();
        xx_unique.dedup();
        let xx_unique = xx_unique;
        // count: how often do the x positions occur?
        let counts = xx_unique
            .iter()
            .map(|&x| xx.iter().filter(|&&xx| xx == x).count())
            .collect::<Vec<_>>();
        std::mem::drop(xx);
        // find longest strip of unique positions
        let start = {
            let mut start = (0, 0);
            let mut ongoing = None;
            fn complete(ongoing: &mut Option<(usize, u32)>, start: &mut (usize, u32)) {
                if let Some((x, c)) = ongoing.take() {
                    if c > start.1 {
                        *start = (x, c);
                    }
                }
            }
            for (c, x) in counts.into_iter().zip(&xx_unique) {
                if c == 1 {
                    if let Some((_, count)) = &mut ongoing {
                        *count += 1
                    } else {
                        ongoing = Some((*x, 1));
                    }
                } else {
                    complete(&mut ongoing, &mut start);
                }
            }
            complete(&mut ongoing, &mut start);
            start.0
        };

        let mut parts = self.parts;
        let mut components = Vec::new();
        // first component
        {
            let index = parts
                .iter()
                .position(|part| {
                    if let Some((px, _)) = part.points.first() {
                        *px == start
                    } else {
                        false
                    }
                })
                .unwrap();
            components.push(parts.remove(index));
        }
        // complete to the right and left
        loop {
            let current_x_max = components
                .iter()
                .flat_map(|x| x.points.iter().map(|x| x.0))
                .max()
                .unwrap();
            let current_x_min = components
                .iter()
                .flat_map(|x| x.points.iter().map(|x| x.0))
                .min()
                .unwrap();
            if let Some((index, _)) = parts
                .iter()
                // ensure that part enlarges to the right or left
                .filter(|part| {
                    part.points.last().unwrap().0 > current_x_max
                        || part.points.first().unwrap().0 < current_x_min
                })
                // compute minimum distance to existing components
                .map(|part| {
                    components
                        .iter()
                        .map(|comp| comp.compute_distance(part, usize::MAX).unwrap())
                        .min()
                        .unwrap()
                })
                .enumerate()
                .min_by_key(|x| x.1)
            {
                components.push(parts.remove(index));
            } else {
                break;
            }
        }

        let mut points: Vec<(usize, u32)> =
            Vec::with_capacity(xx_unique.last().unwrap() - xx_unique.first().unwrap());
        while let Some(component) = components.pop() {
            for (x, mut p) in component.points {
                components.iter_mut().for_each(|comp| {
                    let points = std::mem::take(&mut comp.points);
                    comp.points = points
                        .into_iter()
                        .filter_map(|(xx, pp)| {
                            if xx == x {
                                p.merge(pp);
                                None
                            } else {
                                Some((xx, pp))
                            }
                        })
                        .collect();
                });
                points.push((x, p.mean()));
            }
        }

        points.sort_by_key(|x| x.0);
        StitchedComponent { points }
    }
}

pub struct ComponentPart {
    pub points: Vec<(usize, Vertical)>,
}
impl ComponentPart {
    fn compute_distance(&self, part: &ComponentPart, x_diff_max: usize) -> Option<u32> {
        let cx_f = self.points.first().unwrap().0;
        let cx_l = self.points.last().unwrap().0;
        let px_f = part.points.first().unwrap().0;
        let px_l = part.points.last().unwrap().0;
        if cx_f.saturating_sub(px_l) > x_diff_max || px_f.saturating_sub(cx_l) > x_diff_max {
            return None;
        }
        self.points
            .iter()
            .flat_map(|&(cx, cy)| {
                part.points.iter().filter_map(move |&(px, py)| {
                    let dx = px.max(cx) - px.min(cx);
                    if dx > x_diff_max {
                        None
                    } else {
                        let dy = cy.distance(py);
                        Some(dx as u32 + dy)
                    }
                })
            })
            .min()
    }
}
pub fn connect_vertical_components(vertical_components: Vec<Vec<Vertical>>) -> Vec<ComponentPart> {
    let mut connected_components_parts = Vec::new();
    let mut current_start = 0;
    let mut current_parts = Vec::new();
    for (x, vertical_component) in vertical_components.into_iter().enumerate() {
        if current_parts.len() != vertical_component.len() {
            let completed = std::mem::replace(
                &mut current_parts,
                vertical_component.into_iter().map(|x| vec![x]).collect(),
            );
            if !completed.is_empty() {
                connected_components_parts.push((current_start, completed));
            }
            current_start = x;
        } else {
            for vertical in vertical_component {
                let mut used_indices = vec![];
                fn distance(current: Vertical, next: Vertical) -> u32 {
                    if next.min > current.max {
                        next.min - current.max
                    } else if current.min > next.max {
                        current.min - next.max
                    } else {
                        0
                    }
                }
                let (index, _) = {
                    current_parts
                        .iter()
                        .map(|x| x.last().unwrap())
                        .map(|current| distance(*current, vertical))
                        .enumerate()
                        .filter(|(index, _)| !used_indices.contains(index))
                        .fold((0, u32::MAX), |p, n| {
                            if p.1 < n.1 {
                                p
                            } else if p.1 > n.1 {
                                n
                            } else {
                                dbg!("Same distance for two components");
                                p
                            }
                        })
                };
                used_indices.push(index);
                current_parts[index].push(vertical);
            }
        }
    }

    connected_components_parts
        .into_iter()
        .flat_map(|(x_start, parts)| parts.into_iter().map(move |x| (x_start, x)))
        .map(|(x_start, part)| {
            let points = part
                .into_iter()
                .enumerate()
                .map(|(x_offset, vertical)| (x_offset + x_start, vertical))
                .collect::<Vec<_>>();
            ComponentPart { points }
        })
        .collect()
}

pub struct StitchedComponent {
    pub points: Vec<(usize, u32)>,
}
pub fn stitch_components(
    connected_components_parts: Vec<ComponentPart>,
    x_diff_max: usize,
) -> Vec<StitchedComponent> {
    let mut connected_components_parts = connected_components_parts;
    let mut connected_components = Vec::new();
    while !connected_components_parts.is_empty() {
        let mut min_distance = None;
        fn compute_distance(
            cc: &StitchedComponentInternal,
            part: &ComponentPart,
            x_diff_max: usize,
        ) -> Option<u32> {
            cc.parts
                .iter()
                .filter_map(|c| c.compute_distance(part, x_diff_max))
                .min()
        }

        for (cc_index, cc) in connected_components.iter().enumerate() {
            for (p_index, part) in connected_components_parts.iter().enumerate() {
                let distance = compute_distance(cc, part, x_diff_max);
                if let Some(distance) = distance {
                    if min_distance.is_none()
                        || min_distance.map(|(_, _, dist)| dist).unwrap() > distance
                    {
                        min_distance = Some((cc_index, p_index, distance))
                    }
                }
            }
        }

        if let Some((cc_index, p_index, _)) = min_distance {
            let connected_component = &mut connected_components[cc_index];
            let component = connected_components_parts.remove(p_index);
            connected_component.parts.push(component);
        } else {
            if let Some((p_index, _)) = connected_components_parts
                .iter()
                .enumerate()
                .max_by_key(|(_, part)| part.points.len())
            {
                let component = connected_components_parts.remove(p_index);
                connected_components.push(StitchedComponentInternal {
                    parts: vec![component],
                });
            } else {
                dbg!("This should never happen");
                break;
            }
        }
    }
    connected_components
        .into_iter()
        .map(|x| x.convert())
        .collect()
}
