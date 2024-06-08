pub use graph_to_data::ImagePixel;
use graph_to_data::{LineDetectionSettings, UnitInterval, UnitPoint, UnitQuadrilateral};

pub struct ImageWrapper {
    image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    egui: Option<egui::epaint::TextureHandle>,
    drag_start: Option<egui::Pos2>,
}
impl std::fmt::Debug for ImageWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageOriginalData").finish()
    }
}

impl ImageWrapper {
    fn load_egui_image(&mut self, ui: &mut egui::Ui) {
        if self.egui.is_none() {
            let image = &self.image;
            let size = [image.width() as _, image.height() as _];
            let pixels = image.as_flat_samples();
            let egui_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            let id = format!("ID: {:?}", ui.auto_id_with("Image"));
            let texture_id = ui
                .ctx()
                .load_texture(id, egui_image, egui::TextureOptions::NEAREST);
            self.egui = Some(texture_id);
        }
    }

    pub fn new(image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) -> Self {
        Self {
            image,
            egui: None,
            drag_start: None,
        }
    }

    #[must_use]
    pub fn show_clickable_image(&mut self, ui: &mut egui::Ui) -> Option<ImagePixel> {
        self.load_egui_image(ui);

        let mut clicked = None;
        let image = egui::Image::from_texture(egui::load::SizedTexture {
            id: self.egui.as_ref().unwrap().id(),
            size: ui.available_size_before_wrap(),
        })
        .sense(egui::Sense::click_and_drag());
        let response = egui::Widget::ui(image, ui);
        if let Some(pos) = response.hover_pos() {
            if let Some(pixel) = position_converter(pos, response.rect, &self.image) {
                let ImagePixel { x, y, color } = pixel;
                let [r, g, b, _a] = color;
                let color_egui = egui::Color32::from_rgba_unmultiplied(r, g, b, 128);
                ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                ui.painter()
                    .with_clip_rect(response.rect)
                    .add(egui::Shape::circle_filled(pos, 15., color_egui));
                if response.clicked() {
                    clicked = Some(ImagePixel { x, y, color })
                }
            }
        }
        clicked
    }

    #[must_use]
    pub(crate) fn select_area(&mut self, ui: &mut egui::Ui) -> Option<super::SelectedArea> {
        self.load_egui_image(ui);

        let mut selected_area = None;
        let image = egui::Image::from_texture(egui::load::SizedTexture {
            id: self.egui.as_ref().unwrap().id(),
            size: ui.available_size_before_wrap(),
        })
        .sense(egui::Sense::click_and_drag());
        let response = egui::Widget::ui(image, ui);
        if response.drag_started() {
            self.drag_start = response.hover_pos()
        }
        if response.drag_stopped() {
            if let (Some(start), Some(end)) = (self.drag_start.take(), response.hover_pos()) {
                let start = position_converter_relative(start, response.rect);
                let end = position_converter_relative(end, response.rect);
                if let (Some(start), Some(end)) = (start, end) {
                    selected_area = Some(super::SelectedArea(UnitQuadrilateral::rectangular(
                        start, end,
                    )));
                }
            }
        }
        if let Some(dragging) = response.dragged().then_some(response.hover_pos()).flatten() {
            if let Some(start) = self.drag_start {
                let rect = egui::epaint::Rect::from_two_pos(start, dragging);
                ui.painter().with_clip_rect(response.rect).rect_stroke(
                    rect,
                    egui::Rounding::ZERO,
                    egui::Stroke::new(3.0, egui::Color32::GOLD),
                );
            }
        }

        selected_area
    }

    pub(crate) fn crop(
        &self,
        selected: super::SelectedArea,
        settings: &CropSettings,
    ) -> Result<ImageWrapper, String> {
        let image = &self.image;
        let quadrilateral = selected.0;
        use graph_to_data::ImageInterpolate;
        let CropSettings { steps_x, steps_y } = settings;
        let steps_x = if let Some(steps_x) = *steps_x {
            steps_x.min(image.width())
        } else {
            (image.width() as f32 * quadrilateral.width()) as u32
        };
        let steps_y = if let Some(steps_y) = *steps_y {
            steps_y.min(image.height())
        } else {
            (quadrilateral.height() / quadrilateral.width() * steps_x as f32) as u32
        };
        if steps_x > 100 && steps_y > 100 {
            let image = image.interpolate_image(quadrilateral, steps_x, steps_y);
            Ok(Self::new(image))
        } else {
            Err("Select larger area, possible change input image resolution".to_string())
        }
    }

    pub(crate) fn detect_line(
        &self,
        first_point: ImagePixel,
        settings: &LineDetectionSettings,
    ) -> Vec<(u32, u32)> {
        use graph_to_data::LineDetected;
        self.image.detect_line(first_point, settings)
    }

    pub(crate) fn show_fitted_line(&mut self, line: &[(u32, u32)], ui: &mut egui::Ui) {
        self.load_egui_image(ui);

        let image = egui::Image::from_texture(egui::load::SizedTexture {
            id: self.egui.as_ref().unwrap().id(),
            size: ui.available_size_before_wrap(),
        })
        .sense(egui::Sense::click_and_drag());
        let response = egui::Widget::ui(image, ui);
        line.iter()
            .map(|(x, y)| {
                let x = *x as f32 / self.image.width() as f32;
                let y = *y as f32 / self.image.height() as f32;
                response.rect.lerp_inside(egui::vec2(x, y))
            })
            .for_each(|pos| {
                ui.painter().circle_filled(pos, 12., egui::Color32::GOLD);
            });
    }
}

#[must_use]
fn position_converter_relative(pos: egui::Pos2, rect: egui::Rect) -> Option<UnitPoint> {
    let diff = pos - rect.min;
    let relative_pos = diff / rect.size();
    let x = UnitInterval::new(relative_pos.x);
    let y = UnitInterval::new(relative_pos.y);
    if let (Ok(x), Ok(y)) = (x, y) {
        Some(UnitPoint { x, y })
    } else {
        None
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Default)]
#[serde(default)]
pub struct CropSettings {
    steps_x: Option<u32>,
    steps_y: Option<u32>,
}

#[must_use]
fn position_converter(
    pos: egui::Pos2,
    rect: egui::Rect,
    image: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
) -> Option<ImagePixel> {
    if let Some(UnitPoint {
        x: UnitInterval(x),
        y: UnitInterval(y),
    }) = position_converter_relative(pos, rect)
    {
        let x = ((x * image.width() as f32) as u32).min(image.width() - 1);
        let y = ((y * image.height() as f32) as u32).min(image.height() - 1);
        let color = image.get_pixel(x, y);
        Some(ImagePixel {
            x,
            y,
            color: color.0,
        })
    } else {
        None
    }
}
