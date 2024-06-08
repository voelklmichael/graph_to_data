use graph_to_data::{UnitInterval, UnitPoint, UnitQuadrilateral};

mod file_loading;

type ImageBuf = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
#[derive(Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Tab {
    file_state: file_loading::FileState,
    hide_settings: bool,
    #[serde(skip)]
    original_image: Option<(ImageBuf, egui::TextureHandle)>,
    #[serde(skip)]
    state: State,

    settings: graph_to_data::Settings,
    crop_area: Option<UnitQuadrilateral>,
}
impl Tab {
    pub(crate) fn file_dropped(
        &mut self,
        file: egui::DroppedFile,
    ) -> Result<(), egui::DroppedFile> {
        self.file_state.file_dropped(file)
    }

    pub(crate) fn from_dropped_file(file: egui::DroppedFile) -> Tab {
        let mut tab = Self::default();
        tab.file_dropped(file)
            .expect("There is no file loaded for this default tab, so this never fails");
        tab
    }

    pub(crate) fn title(&self) -> egui::WidgetText {
        self.file_state.title()
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui) {
        if self.file_state.progress() {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs(1));
            ui.set_enabled(false);
        }
        if let Some(error) = self.file_state.is_error() {
            ui.heading(
                egui::RichText::new(error)
                    .background_color(egui::Color32::RED)
                    .color(egui::Color32::WHITE),
            );
            self.file_state.show_select_image_button(ui);
            return;
        } else if let Some(image) = self.file_state.is_loaded() {
            if let Some(image) = image {
                let size = [image.width() as _, image.height() as _];
                let pixels = image.as_flat_samples();
                let egui_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                let id = format!("ID: {:?}", ui.auto_id_with("Image"));
                let texture_id =
                    ui.ctx()
                        .load_texture(id, egui_image, egui::TextureOptions::NEAREST);
                self.original_image = Some((image, texture_id));
                self.state = State::CropByRectangle(Default::default());
                ui.ctx().request_repaint();
            }
            assert!(self.original_image.is_some());
            egui::SidePanel::left("settings_panel")
                .resizable(true)
                .default_width(200.)
                .show_inside(ui, |ui| self.show_settings(ui));
            egui::CentralPanel::default().show_inside(ui, |ui| self.show_state(ui));
        } else {
            self.file_state.show_select_image_button(ui);
        }
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        if self.hide_settings {
            let button = ui.button(">");
            if button.on_hover_text("Show settings").clicked() {
                self.hide_settings = false;
            }
            // TODO: enlarge this button to take all available space
        } else {
            ui.vertical(|ui| {
                let button = ui.button("< Hide Settings");
                if button.on_hover_text("Hide settings").clicked() {
                    self.hide_settings = true;
                }
                self.file_state.show_select_image_button(ui);
                ui.separator();
                ui.label("Crop settings");
                if ui.button("Crop by rectangle").clicked() {
                    self.state = State::CropByRectangle(Default::default())
                }
                ui.separator();
                let image = egui::Image::from_texture(egui::load::SizedTexture {
                    id: self.original_image.as_ref().unwrap().1.id(),
                    size: ui.available_size_before_wrap(),
                })
                .sense(egui::Sense::click());
                egui::Widget::ui(image, ui).on_hover_text("Original image");
            });
        }
    }

    fn show_state(&mut self, ui: &mut egui::Ui) {
        let work = match &mut self.state {
            State::NothingLoaded => {
                ui.label("No image loaded yet");
                None
            }
            State::CropByRectangle(crop_by_rectangle) => {
                ui.label("Crop image by via drag and drop");
                let mut selected_area = None;
                let image = egui::Image::from_texture(egui::load::SizedTexture {
                    id: self.original_image.as_ref().unwrap().1.id(),
                    size: ui.available_size_before_wrap(),
                })
                .sense(egui::Sense::click_and_drag());
                let response = egui::Widget::ui(image, ui);
                if response.drag_started() {
                    crop_by_rectangle.drag_start = response.hover_pos()
                }
                if response.drag_stopped() {
                    if let (Some(start), Some(end)) =
                        (crop_by_rectangle.drag_start.take(), response.hover_pos())
                    {
                        let start = position_converter_relative(start, response.rect);
                        let end = position_converter_relative(end, response.rect);
                        if let (Some(start), Some(end)) = (start, end) {
                            selected_area = Some(UnitQuadrilateral::rectangular(start, end));
                        }
                    }
                }
                if let Some(dragging) = response.dragged().then_some(response.hover_pos()).flatten()
                {
                    if let Some(start) = crop_by_rectangle.drag_start {
                        let rect = egui::epaint::Rect::from_two_pos(start, dragging);
                        ui.painter().with_clip_rect(response.rect).rect_stroke(
                            rect,
                            egui::Rounding::ZERO,
                            egui::Stroke::new(3.0, egui::Color32::GOLD),
                        );
                    }
                }
                selected_area.map(Work::CroppedByRectangle)
            }
            State::LineDetection(result) => {
                match result {
                    Ok((image, _)) => {
                        if let Some(image) = image {
                            let image = egui::Image::from_texture(egui::load::SizedTexture {
                                id: image.id(),
                                size: ui.available_size_before_wrap(),
                            })
                            .sense(egui::Sense::click());
                            egui::Widget::ui(image, ui).on_hover_text("Fitted image");
                        } else {
                            ui.label("Failed to fit curves");
                        }
                    }
                    Err(error) => {
                        ui.heading(
                            egui::RichText::new(error.clone())
                                .background_color(egui::Color32::RED)
                                .color(egui::Color32::WHITE),
                        );
                    }
                }
                None
            }
        };
        if let Some(work) = work {
            ui.ctx().request_repaint();
            match work {
                Work::CroppedByRectangle(area) => {
                    self.crop_area = Some(area);
                    let image = &self.original_image.as_ref().unwrap().0;
                    let line_detection = graph_to_data::line_detection(
                        image,
                        &self.settings,
                        self.crop_area.clone().unwrap(),
                        image.width(),
                        image.height(),
                        (0., 1.),
                        (0., 1.),
                    );
                    let line_detection = line_detection
                        .map(|l| {
                            (
                                l.final_image_with_plots().map(|image| {
                                    let size = [image.width() as _, image.height() as _];
                                    let pixels = image.as_flat_samples();
                                    let egui_image = egui::ColorImage::from_rgba_unmultiplied(
                                        size,
                                        pixels.as_slice(),
                                    );
                                    let id = format!("ID: {:?}", ui.auto_id_with("Image"));
                                    let texture_id = ui.ctx().load_texture(
                                        id,
                                        egui_image,
                                        egui::TextureOptions::NEAREST,
                                    );
                                    texture_id
                                }),
                                l,
                            )
                        })
                        .map_err(|e| format!("{e:?}"));

                    self.state = State::LineDetection(line_detection);
                }
            }
        }
    }
}
enum Work {
    CroppedByRectangle(UnitQuadrilateral),
}
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct CropByRectangle {
    drag_start: Option<egui::Pos2>,
}
#[derive(Default)]
enum State {
    #[default]
    NothingLoaded,
    CropByRectangle(CropByRectangle),
    LineDetection(Result<(Option<egui::TextureHandle>, graph_to_data::LineDetected), String>),
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
