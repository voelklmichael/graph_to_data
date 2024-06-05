use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct X(pub u32);

#[derive(Debug, Clone)]
pub struct VerticalComponent {
    pub y_min: u32, // this is include
    pub y_max: u32, // this is also include
}
impl VerticalComponent {
    fn distance_to(&self, other: &Self) -> u32 {
        if self.y_min > other.y_max {
            self.y_min - other.y_max
        } else if other.y_min > self.y_max {
            other.y_min - self.y_max
        } else {
            0
        }
    }

    fn min_distance_to_previous_component(
        &self,
        start_connected_components: &[Comp],
        i: X,
    ) -> usize {
        todo!()
    }
}

pub struct VerticalComponentList {
    pub components: Vec<VerticalComponents>,
}
pub struct GroupSettings {
    /// Minimum fraction of width of connected group with constant plot count
    pub step1_min_width: u32,
}
impl Default for GroupSettings {
    fn default() -> Self {
        Self {
            step1_min_width: 50,
        }
    }
}
struct Comp {
    ys: Vec<Vec<VerticalComponent>>,
}
impl VerticalComponentList {
    pub fn convert(image: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>) -> Self {
        let components = (0..image.width())
            .map(|x| {
                let mut components = Vec::new();
                let mut current_component = Vec::new();
                #[inline(always)]
                fn complete_component(
                    current_component: &mut Vec<u32>,
                    components: &mut Vec<VerticalComponent>,
                ) {
                    let current_component = std::mem::take(current_component);
                    if !current_component.is_empty() {
                        let y_min = *current_component.first().unwrap();
                        let y_max = *current_component.last().unwrap();
                        components.push(VerticalComponent { y_min, y_max });
                    }
                }
                for y in 0..image.height() {
                    if image.get_pixel(x, y) == &crate::helpers::HIT {
                        current_component.push(y);
                    } else {
                        complete_component(&mut current_component, &mut components)
                    }
                }
                complete_component(&mut current_component, &mut components);
                VerticalComponents { components }
            })
            .collect();
        Self { components }
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    pub fn max_component_count(&self) -> Option<usize> {
        self.components.iter().map(|x| x.component_count()).max()
    }

    pub fn group_into_connected_components(
        mut self,
        group_settings: GroupSettings,
        
    ) -> Vec<ConnectedComponentPart> {
        // group large parts
        let total_width = 
        let component_counts = self
            .components
            .iter()
            .map(|x| x.component_count())
            .collect_vec();
        let mut start = (X(0), component_counts[0]);
        #[derive(Debug)]
        struct Range {
            start: X,
            end: X, // one after the last element
            count: usize,
        }
        impl Range {
            fn width(&self) -> u32 {
                self.end.0 - self.start.0
            }
        }
        let mut ranges = Vec::new();
        for i in 0..component_counts.len() {
            if let Some(next_count) = component_counts.get(i + 1) {
                if *next_count != start.1 {
                    if start.1 > 0 {
                        ranges.push(Range {
                            start: start.0,
                            end: X(i as _),
                            count: start.1,
                        });
                    }
                    start = (X(i as _), *next_count)
                }
            } else {
                if start.1 > 0 {
                    ranges.push(Range {
                        start: start.0,
                        end: X(i as _),
                        count: start.1,
                    });
                }
                break;
            }
        }

        let mut later_indices = Vec::new();
        let start_ranges = ranges
            .into_iter()
            .filter_map(|r| {
                if r.width() >= group_settings.step1_min_width {
                    Some(r)
                } else {
                    later_indices.extend(r.start.0..r.end.0);
                    None
                }
            })
            .collect_vec();
        if start_ranges.is_empty() {
            panic!("Failed to find start range");
        }
        dbg!(&start_ranges);

        let mut start_connected_components = start_ranges
            .into_iter()
            .flat_map(|r| {
                assert!(r.count > 0);
                let mut components = vec![Vec::with_capacity(r.width() as _); r.count];
                for verticals in (r.start.0..r.end.0)
                    .map(|i| std::mem::take(&mut self.components[i as usize]).components)
                {
                    // verticals are already sorted top-down
                    verticals
                        .into_iter()
                        .zip(&mut components)
                        .for_each(|(v, c)| c.push(v));
                }
                let x = r.start.0;
                components.into_iter().map(move |comp| Comp {

                    ys: ,
                })
            })
            .collect_vec();
        for i in later_indices {
            let later = std::mem::take(&mut self.components[i as usize]).components;
            later
                .into_iter()
                .map(|v| {
                    (
                        v.min_distance_to_previous_component(&start_connected_components, X(i)),
                        v,
                    )
                })
                .for_each(|(index, vertical)| {
                    let comp = &mut start_connected_components[index];
                    let offset
                });
        }

        /*
                struct Comp {
                    x: X,
                    c: VerticalComponent,
                }
                impl Comp {
                    fn distance(&self, right: &Comp) -> u32 {
                        self.x.distance(right.x.clone()) + self.c.distance_to(&right.c)
                    }
                }
                let components = self
                    .components
                    .into_iter()
                    .enumerate()
                    .flat_map(|(x, c)| {
                        c.components
                            .into_iter()
                            .map(move |c| Comp { x: X(x as _), c })
                    })
                    .collect::<Vec<_>>();
                let mut distances = components
                    .iter()
                    .map(|left| {
                        components
                            .iter()
                            .map(|right| left.distance(right))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
        */

        /*let components = self.components;
        let mut distances = components
            .iter()
            .enumerate()
            .map(|(x_left, comp_left)| comp_left.iter().enumerate())
            .collect::<Vec<_>>();*/

        todo!()
        /*let mut connected_components = Vec::new();
        let mut ongoing_components = Vec::new();

        for (x, components) in self
            .components
            .into_iter()
            .enumerate()
            .map(|(i, c)| (X(i), c.components))
        {
            if ongoing_components.len() != components.len() {
                complete(&mut connected_components, ongoing_components);
                ongoing_components = vec![];
            } else {
                let mut components = components.into_iter().map(Some).collect::<Vec<_>>();
                let mut used_ongoing = vec![false; ongoing_components.len()];
                while components.iter().any(|x| x.is_some()) {
                    let distance = {
                        ongoing_components
                            .iter()
                            .map(|connected| connected.1.last().unwrap())
                            .map(|last| {
                                components
                                    .iter()
                                    .map(|component| {
                                        component
                                            .as_ref()
                                            .map(|component| last.distance_to(component))
                                    })
                                    .enumerate()
                                    .fold(None, |min, (i, d)| {
                                        if let Some(d) = d {
                                            if let Some((i_min, min)) = min {
                                                if min < d {
                                                    Some((i_min, min))
                                                } else {
                                                    Some((i, d))
                                                }
                                            } else {
                                                Some((i, d))
                                            }
                                        } else {
                                            min
                                        }
                                    })
                            })
                            .enumerate()
                            .filter(|(i, _)| used_ongoing[*i])
                            .flat_map(|(ongoing_index, d)| {
                                d.map(|(component_index, d)| (component_index, ongoing_index, d))
                            })
                            .fold(None, |min:Option<(usize, usize, u32)>, (component_index, ongoing_index, d)| {
                                if let Some(min) = min {
                                    if min.2 < d {
                                        Some(min)
                                    } else {
                                        Some((component_index, ongoing_index, d))
                                    }
                                } else {
                                    Some((component_index, ongoing_index, d))
                                }
                            })
                    };
                    if let Some((component_index, ongoing_index, d)) = distance {
                        used_ongoing.push(ongoing_index)
                    } else {
                        break;
                    }
                }
            }
        }
        complete(&mut connected_components, ongoing_components);
        connected_components*/
    }
}
/*
fn complete(
    connected_components: &mut Vec<ConnectedComponentPart>,
    ongoing_components: Vec<(X, Vec<VerticalComponent>)>,
) {
    connected_components.extend(
        ongoing_components
            .into_iter()
            .map(|(x_start, ys)| ConnectedComponentPart { x_start, ys }),
    )
}
*/
#[derive(Debug)]
pub struct ConnectedComponentPart {
    pub x_start: X,
    pub ys: Vec<VerticalComponent>,
}

#[derive(Debug, Default)]
pub struct VerticalComponents {
    pub components: Vec<VerticalComponent>,
}
impl VerticalComponents {
    pub fn convert(image: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>) -> Vec<Self> {
        (0..image.width())
            .map(|x| {
                let mut components = Vec::new();
                let mut current_component = Vec::new();
                #[inline(always)]
                fn complete_component(
                    current_component: &mut Vec<u32>,
                    components: &mut Vec<VerticalComponent>,
                ) {
                    let current_component = std::mem::take(current_component);
                    if !current_component.is_empty() {
                        let y_min = *current_component.first().unwrap();
                        let y_max = *current_component.last().unwrap();
                        components.push(VerticalComponent { y_min, y_max });
                    }
                }
                for y in 0..image.height() {
                    if image.get_pixel(x, y) == &crate::helpers::HIT {
                        current_component.push(y);
                    } else {
                        complete_component(&mut current_component, &mut components)
                    }
                }
                complete_component(&mut current_component, &mut components);
                Self { components }
            })
            .collect()
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }
}
