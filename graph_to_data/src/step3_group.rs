use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct X(pub u32);

#[derive(Debug, Clone)]
pub struct VerticalComponentCombined {
    pub y_min: u32, // this is include
    pub y_max: u32, // this is also include
}
impl VerticalComponentCombined {
    fn new(v: VerticalComponent) -> Self {
        let VerticalComponent { y_min, y_max } = v;
        Self { y_min, y_max }
    }
    fn distance_to(&self, other: &VerticalComponent) -> u32 {
        if self.y_min > other.y_max {
            self.y_min - other.y_max
        } else if other.y_min > self.y_max {
            other.y_min - self.y_max
        } else {
            0
        }
    }
    fn distance_to_other(&self, other: &VerticalComponentCombined) -> u32 {
        if self.y_min > other.y_max {
            self.y_min - other.y_max
        } else if other.y_min > self.y_max {
            other.y_min - self.y_max
        } else {
            0
        }
    }
    pub fn mean(&self) -> u32 {
        (self.y_max + self.y_min) / 2
    }

    fn merge(new: Vec<VerticalComponent>) -> VerticalComponentCombined {
        let y_min = new.iter().map(|x| x.y_min).min().unwrap();
        let y_max = new.into_iter().map(|x| x.y_max).max().unwrap();
        Self { y_min, y_max }
    }

    fn combine(self, other: VerticalComponentCombined) -> VerticalComponentCombined {
        let y_min = self.y_min.min(other.y_min);
        let y_max = self.y_max.min(other.y_max);
        Self { y_min, y_max }
    }

    fn convert(self) -> VerticalComponent {
        let Self { y_min, y_max } = self;
        VerticalComponent { y_min, y_max }
    }
}
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
}

struct VerticalComponentList {
    pub components: Vec<VerticalComponents>,
}
struct VerticalComponents {
    pub components: Vec<VerticalComponent>,
}
impl VerticalComponents {
    pub fn component_count(&self) -> usize {
        self.components.len()
    }
}
impl VerticalComponentList {
    /// Convert black-white image in List of white points
    /// Vertically connected stripes of white points are combined into a single item
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
                    if image.get_pixel(x, y) == &crate::HIT {
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

    fn counts(&self) -> Vec<usize> {
        self.components
            .iter()
            .map(|x| x.component_count())
            .collect_vec()
    }

    fn find_long_components(&mut self, min_width: u32) -> Vec<Range> {
        let counts = self.counts();
        let width = counts.len() as u32;

        let mut ranges = Vec::new();
        let mut start = (X(0), counts[0]);
        for i in 0..width {
            let i = i + 1;
            if let Some(next_count) = counts.get(i as usize) {
                if next_count != &start.1 {
                    let (start_x, count) = start;
                    if i - start_x.0 >= min_width {
                        ranges.push(Range {
                            start: start_x,
                            end: X(i),
                            count,
                        });
                    }
                    start = (X(i), *next_count);
                }
            } else {
                if start.1 > 0 {
                    let (start_x, count) = start;
                    if i - start_x.0 >= min_width {
                        ranges.push(Range {
                            start: start_x,
                            end: X(i),
                            count,
                        });
                    }
                }
                break;
            }
        }
        ranges
    }

    fn extract_components(&mut self, ranges: Vec<Range>, width: u32) -> Vec<GraphMultiNode> {
        let mut components = Vec::new();
        for range in ranges {
            let Range { start, end, count } = range;
            let initial_line = { GraphMultiNode::new(width, start) };
            let mut new_components = vec![initial_line; count];

            self.components
                .iter_mut()
                .skip(start.0 as _)
                .take((end.0 - start.0) as _)
                .for_each(|x| {
                    std::mem::take(&mut x.components)
                        .into_iter()
                        .zip(&mut new_components)
                        .for_each(|(v, n)| n.ys.push(MultiNode::new(v)))
                });
            new_components.iter_mut().for_each(|v| {
                v.ys.extend(vec![MultiNode::default(); (width - end.0) as usize])
            });
            for comp in &new_components {
                debug_assert_eq!(comp.ys.len(), width as _);
            }
            debug_assert_eq!(new_components.len(), count);
            components.extend(new_components);
        }
        components
    }

    fn combining_horizontally(self) -> Vec<CombinedVerticals> {
        let mut verticals = self.components;
        let mut combined: Vec<CombinedVerticals> = Vec::new();
        let mut x_offset = 0;
        while let Some((x_start, v)) = verticals
            .iter_mut()
            .skip(x_offset)
            .enumerate()
            .filter_map(|(x, v)| v.components.pop().map(|v| (x + x_offset, v)))
            .next()
        {
            x_offset = x_start;
            // combined forward
            let mut combined_verticals = vec![VerticalComponentCombined::new(v)];
            for verticals in verticals
                .iter_mut()
                .skip(x_offset + 1)
                .map(|v| &mut v.components)
            {
                let last = combined_verticals.last().unwrap();
                let mut new = vec![];
                while let Some(i) = verticals.iter().position(|v| last.distance_to(v) <= 1) {
                    new.push(verticals.remove(i));
                }
                if new.is_empty() {
                    break;
                } else {
                    combined_verticals.push(VerticalComponentCombined::merge(new));
                }
            }
            let combined_verticals = CombinedVerticals {
                x_start: X(x_start as _),
                combined: combined_verticals,
            };
            if let Some(previous) = combined
                .iter_mut()
                .find(|previous| previous.distance_to(&combined_verticals))
            {
                previous.merge(combined_verticals);
            } else {
                combined.push(combined_verticals);
            }
        }
        combined
    }
}

pub struct CombinedVerticals {
    pub x_start: X,
    pub combined: Vec<VerticalComponentCombined>,
}
impl CombinedVerticals {
    fn distance_to<'a>(&'a self, other: &'a Self) -> bool {
        let (left, right) = {
            if self.x_start < other.x_start {
                (self, other)
            } else {
                (other, self)
            }
        };
        let offset = (right.x_start.0 - left.x_start.0) as usize;
        let distance = left
            .combined
            .iter()
            .enumerate()
            .filter_map(|(x_l, left)| {
                right
                    .combined
                    .iter()
                    .enumerate()
                    .map(move |(x_r, right)| {
                        let dy = right.distance_to_other(left);
                        let x_r = x_r + offset;
                        let dx = x_r.max(x_l) - x_r.min(x_l);
                        dx as u32 + dy
                    })
                    .min()
            })
            .min()
            .unwrap();
        distance <= 1
    }

    fn merge(&mut self, other: CombinedVerticals) {
        let s = std::mem::replace(
            self,
            Self {
                x_start: X(0),
                combined: Default::default(),
            },
        );
        let (mut left, mut right) = {
            if s.x_start <= other.x_start {
                (s, other)
            } else {
                (other, s)
            }
        };
        // note: CombinedVerticals always are connected strips of verticals, there are no holes
        self.x_start = left.x_start;
        for x in left.x_start.0.. {
            if x < right.x_start.0 || right.combined.is_empty() {
                if left.combined.is_empty() {
                    break;
                }
                self.combined.push(left.combined.remove(0));
            } else {
                let right = right.combined.remove(0);
                if let Some(left) = (!left.combined.is_empty()).then(|| left.combined.remove(0)) {
                    self.combined.push(right.combine(left));
                } else {
                    self.combined.push(right);
                }
            }
        }
        debug_assert!(left.combined.is_empty());
        debug_assert!(right.combined.is_empty());
    }
}

#[derive(Debug)]
struct Range {
    start: X,
    end: X, // one after the last element
    count: usize,
}

pub fn group_large_components_and_remaining(
    image: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
    settings: &crate::Settings,
) -> (Vec<GraphMultiNode>, Vec<CombinedVerticals>) {
    let width = image.width();
    let min_width = (width as f32 * settings.step3_min_width_fraction) as u32;

    let mut verticals = VerticalComponentList::convert(image);
    let ranges = verticals.find_long_components(min_width);

    let components = verticals.extract_components(ranges, width);

    let verticals = verticals.combining_horizontally();
    (components, verticals)
}

#[derive(PartialEq, PartialOrd)]
pub(crate) enum Distance {
    CanBeExtendend { distance: u32 },
    CannotBeExtended, // no new x coordinate
}

#[derive(Default, Clone)]
pub struct MultiNode {
    verticals: Vec<VerticalComponent>,
}
impl MultiNode {
    fn new(v: VerticalComponent) -> Self {
        Self { verticals: vec![v] }
    }

    fn distance(&self, r: &MultiNode) -> u32 {
        self.verticals
            .iter()
            .zip(&r.verticals)
            .map(|(l, r)| l.distance_to(r))
            .min()
            .unwrap_or(u32::MAX)
    }

    fn combine(&mut self, o: MultiNode) {
        self.verticals.extend(o.verticals)
    }

    pub fn mean(&self) -> Option<u32> {
        if let Some(min) = self.verticals.iter().map(|x| x.y_min).min() {
            let max = self.verticals.iter().map(|x| x.y_max).max().unwrap();
            Some((min + max) / 2)
        } else {
            None
        }
    }
}
#[derive(Clone)]
pub struct GraphMultiNode {
    pub ys: Vec<MultiNode>,
}
impl GraphMultiNode {
    fn new(width: u32, start: X) -> Self {
        let mut ys = Vec::with_capacity(width as usize);
        ys.extend(vec![MultiNode::default(); start.0 as usize]);
        Self { ys }
    }

    pub(crate) fn distance(&self, other: &GraphMultiNode) -> u32 {
        if self.min_x() >= other.min_x() && self.max_x() <= other.max_x() {
            return u32::MAX;
        }
        // todo: ensure that at least several points are close-by
        self.ys
            .iter()
            .enumerate()
            .flat_map(|(x, ys)| {
                other.ys.iter().enumerate().map(move |(ox, oys)| {
                    let dx = x.max(ox) - x.min(ox);
                    let dy = ys.distance(oys);
                    (dx as u32).saturating_add(dy)
                })
            })
            .min()
            .unwrap()
    }

    pub fn stitch_together(&mut self, other: Self) {
        self.ys
            .iter_mut()
            .zip(other.ys)
            .for_each(|(s, o)| s.combine(o))
    }

    fn min_x(&self) -> usize {
        self.ys
            .iter()
            .enumerate()
            .filter(|(_, ys)| !ys.verticals.is_empty())
            .map(|(x, _)| x)
            .min()
            .unwrap_or(usize::MIN)
    }
    fn max_x(&self) -> usize {
        self.ys
            .iter()
            .enumerate()
            .filter(|(_, ys)| !ys.verticals.is_empty())
            .map(|(x, _)| x)
            .max()
            .unwrap_or(usize::MAX)
    }

    pub(crate) fn distance_to_vertical(&self, v: &CombinedVerticals) -> Distance {
        if self
            .ys
            .iter()
            .skip(v.x_start.0 as _)
            .take(v.combined.len())
            .any(|ys| ys.verticals.is_empty())
        {
            let distance = self
                .ys
                .iter()
                .enumerate()
                .flat_map(|(x, ys)| {
                    v.combined
                        .iter()
                        .enumerate()
                        .flat_map(|(vx, vy)| {
                            let vx = vx + v.x_start.0 as usize;
                            let dx = vx.max(x) - vx.min(x);
                            ys.verticals
                                .iter()
                                .map(|y| vy.distance_to(y))
                                .min()
                                .map(|dy| dx as u32 + dy)
                        })
                        .min()
                })
                .min();
            if let Some(distance) = distance {
                Distance::CanBeExtendend { distance }
            } else {
                Distance::CannotBeExtended
            }
        } else {
            Distance::CannotBeExtended
        }
    }

    pub(crate) fn merge(&mut self, v: CombinedVerticals) {
        let CombinedVerticals { x_start, combined } = v;
        for (x_offset, v) in combined.into_iter().enumerate() {
            let x = x_offset + x_start.0 as usize;
            self.ys[x].verticals.push(v.convert())
        }
    }

    pub fn overlaps(&self, other: &GraphMultiNode) -> bool {
        self.ys
            .iter()
            .zip(&other.ys)
            .any(|(s, o)| !s.verticals.is_empty() && !o.verticals.is_empty())
    }

    pub fn aggregate(&mut self, other: GraphMultiNode) {
        self.ys
            .iter_mut()
            .zip(other.ys)
            .for_each(|(s, o)| s.verticals.extend(o.verticals));
    }

    pub(crate) fn to_plot(
        &self,
        x_limits: (f32, f32),
        y_limits: (f32, f32),
        steps_x: u32,
        steps_y: u32,
    ) -> Vec<(f32, f32)> {
        assert_eq!(self.ys.len(), steps_x as usize);
        self.ys
            .iter()
            .enumerate()
            .flat_map(|(x, ys)| {
                if let Some(y) = ys.mean() {
                    fn convert(x: u32, limits: (f32, f32), n: u32, min_max: bool) -> f32 {
                        let delta = limits.1 - limits.0;
                        assert!(delta.is_finite());
                        assert!(delta > 0.);
                        let delta_divided_n = delta / (n + 1) as f32;
                        let t = (x + 1) as f32 * delta_divided_n;
                        if min_max {
                            limits.1 - t
                        } else {
                            limits.0 + t
                        }
                    }
                    let x = convert(x as u32, x_limits, steps_x, false);
                    let y = convert(y, y_limits, steps_y, true);
                    Some((x, y))
                } else {
                    None
                }
            })
            .collect()
    }
}
