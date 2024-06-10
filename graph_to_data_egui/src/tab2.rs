use graph_to_data::{UnitInterval, UnitPoint, UnitQuadrilateral};

mod axis_settings;
mod crop_settings;
mod file_loading;

use axis_settings::AxisSettings;
use crop_settings::CropSettings;

type ImageBuf = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
#[derive(Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Tab {
    file_state: file_loading::FileState,
    hide_settings: bool,
    #[serde(skip)]
    original_image: Option<(std::sync::Arc<ImageBuf>, egui::TextureHandle)>,
    #[serde(skip)]
    state: State,

    settings: graph_to_data::Settings,
    settings_as_string: Option<SettingsAsString>,
    crop_settings: CropSettings,
    axis_settings: AxisSettings,
}
#[derive(Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]

struct ParseableTextBox {
    before: String,
    current: String,
    is_parse_error: Option<String>,
}
trait Parseable: Sized {
    fn parse(s: &str) -> Result<Self, String>;
}
impl Parseable for u8 {
    fn parse(s: &str) -> Result<Self, String> {
        s.parse().map_err(|e| format!("{e:?}"))
    }
}
impl Parseable for f32 {
    fn parse(s: &str) -> Result<Self, String> {
        s.parse().map_err(|e| format!("{e:?}"))
    }
}
impl ParseableTextBox {
    fn show_and_parse<T: Parseable>(
        &mut self,
        label: &str,
        tooltip: &str,
        value: &mut T,
        ui: &mut egui::Ui,
    ) {
        let tooltip = if let Some(error) = &self.is_parse_error {
            error
        } else {
            tooltip
        };
        ui.label(label).on_hover_text(tooltip);

        ui.scope(|ui| {
            if self.is_parse_error.is_some() {
                ui.style_mut().visuals.extreme_bg_color = egui::Color32::RED;
                ui.style_mut().visuals.override_text_color = Some(egui::Color32::WHITE);
            }
            ui.text_edit_singleline(&mut self.current)
                .on_hover_text(tooltip);
        });

        if self.current != self.before {
            self.before = self.current.clone();
            match T::parse(&self.current) {
                Ok(v) => {
                    *value = v;
                    self.is_parse_error = None;
                }
                Err(e) => self.is_parse_error = Some(e),
            }
        }
    }

    fn new<T: std::fmt::Display>(default: T) -> Self {
        let s = default.to_string();
        Self {
            before: s.clone(),
            current: s,
            is_parse_error: None,
        }
    }
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct SettingsAsString {
    step1_width_minimial_fraction: ParseableTextBox,
    step1_height_maximal_fraction: ParseableTextBox,
    step1_close_count: ParseableTextBox,
    step1_step2_color_radius: ParseableTextBox,
    step3_min_width_fraction: ParseableTextBox,
    step4_component_jump_height_fraction: ParseableTextBox,
}
impl SettingsAsString {
    fn new(settings: &graph_to_data::Settings) -> Self {
        Self {
            step1_width_minimial_fraction: ParseableTextBox::new(
                &settings.step1_width_minimial_fraction,
            ),
            step1_height_maximal_fraction: ParseableTextBox::new(
                &settings.step1_height_maximal_fraction,
            ),
            step1_step2_color_radius: ParseableTextBox::new(&settings.step1_step2_color_radius),
            step3_min_width_fraction: ParseableTextBox::new(&settings.step3_min_width_fraction),
            step4_component_jump_height_fraction: ParseableTextBox::new(
                &settings.step4_component_jump_height_fraction,
            ),
            step1_close_count: ParseableTextBox::new(&settings.step1_close_count),
        }
    }
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
                self.original_image = Some((std::sync::Arc::new(image), texture_id));
                self.state = State::CropByRectangle(self.crop_settings.convert());
                ui.ctx().request_repaint();
            }
            assert!(self.original_image.is_some());
            egui::SidePanel::left("settings_panel")
                .resizable(true)
                .default_width(200.)
                .show_inside(ui, |ui| self.show_settings(ui));
            egui::CentralPanel::default().show_inside(ui, |ui| self.show_state(ui));
        } else {
            ui.vertical(|ui| {
                ui.heading("Select image");
                self.file_state.show_select_image_button(ui);
            });
        }
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        if self.hide_settings {
            if ui.button(">").on_hover_text("Show settings").clicked() {
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

                ui.heading("Crop settings");

                egui::Grid::new("crop_settings_grid")
                    .num_columns(4)
                    .show(ui, |ui| {
                        {
                            ui.label(" ");
                            ui.label("X");
                            ui.label("Y");
                            ui.label("Refine");
                            ui.end_row();
                        }

                        self.crop_settings.left_top.show("Left Top", ui);
                        if ui.button("Refine").clicked() {
                            self.state = State::RefineCrop(RefineCrop::LeftTop)
                        }
                        ui.end_row();
                        self.crop_settings.left_bottom.show("Left Bottom", ui);
                        if ui.button("Refine").clicked() {
                            self.state = State::RefineCrop(RefineCrop::LeftBottom)
                        }
                        ui.end_row();
                        self.crop_settings.right_top.show("Right Top", ui);
                        if ui.button("Refine").clicked() {
                            self.state = State::RefineCrop(RefineCrop::RightTop)
                        }
                        ui.end_row();
                        self.crop_settings.right_bottom.show("Right Bottom", ui);
                        if ui.button("Refine").clicked() {
                            self.state = State::RefineCrop(RefineCrop::RightBottom)
                        }
                        ui.end_row();
                    });

                if ui.button("Crop by rectangle").clicked() {
                    self.state = State::CropByRectangle(self.crop_settings.convert())
                }
                ui.separator();

                ui.heading("Detection settings");
                {
                    if self.settings_as_string.is_none() {
                        self.settings_as_string = Some(SettingsAsString::new(&self.settings));
                    }
                    let SettingsAsString {
                        step1_width_minimial_fraction,
                        step1_height_maximal_fraction,
                        step1_close_count,
                        step1_step2_color_radius,
                        step3_min_width_fraction,
                        step4_component_jump_height_fraction,
                    } = &mut self.settings_as_string.as_mut().unwrap();
                    egui::Grid::new("detection_settings_grid")
                        .num_columns(2)
                        .show(ui, |ui| {
                            {
                                ui.label("Step 1: ignore gray");
                                ui.checkbox(&mut self.settings.step1_ignore_gray, "");
                            }
                            ui.end_row();
                            step1_width_minimial_fraction.show_and_parse(
                                "Step 1: Minimal Width",
                                "Minimal allowed fraction of image width \
                            for color detection.\n \
                            Colors that appear in less columns are \
                            not considered graphs\
                            Value between 0.0 and 1.0",
                                &mut self.settings.step1_width_minimial_fraction,
                                ui,
                            );
                            ui.end_row();
                            step1_height_maximal_fraction.show_and_parse(
                                "Step 1: Maximum Height",
                                "Maximal allowed fraction of image height \
                            for color detection.\n \
                            Colors that appear more often in a column \
                            color are not considered graphs\
                            Value between 0.0 and 1.0",
                                &mut self.settings.step1_height_maximal_fraction,
                                ui,
                            );
                            ui.end_row();
                            step1_close_count.show_and_parse(
                                "Step 1: Close count",
                                "This removes points which are not connected to\
                        larger clusters of the same color.\
                        This settings controls the distances \
                        (larger value=>more is removed).\
                        Value between 0 and 255",
                                &mut self.settings.step1_close_count,
                                ui,
                            );
                            ui.end_row();
                            step1_step2_color_radius.show_and_parse(
                                "Step 2: Color radius",
                                "Radius in color space of colors\
                        which are considered equal\
                        Value between 0 and 255",
                                &mut self.settings.step1_step2_color_radius,
                                ui,
                            );
                            ui.end_row();
                            step3_min_width_fraction.show_and_parse(
                                "Step 3: Minimal width",
                                "Minimal width of connected pixels \
                        to be used as a starting line\
                        Value between 0.0 and 1.0",
                                &mut self.settings.step3_min_width_fraction,
                                ui,
                            );
                            ui.end_row();
                            step4_component_jump_height_fraction.show_and_parse(
                                "Step 4: Jump height",
                                "Maximal vertical jump hight allowed to join \
                        connected components.\
                        Value between 0.0 and 1.0",
                                &mut self.settings.step4_component_jump_height_fraction,
                                ui,
                            );
                            ui.end_row();
                            {
                                ui.label("Override fit color");
                                let mut override_fit_color =
                                    self.settings.step6_fit_graph_color.is_some();
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut override_fit_color, "");
                                    if override_fit_color {
                                        if self.settings.step6_fit_graph_color.is_none() {
                                            self.settings.step6_fit_graph_color =
                                                Some(graph_to_data::GOLD_AS_RGB);
                                        }
                                        let color =
                                            self.settings.step6_fit_graph_color.as_mut().unwrap();
                                        egui::color_picker::color_edit_button_srgb(ui, color);
                                    } else {
                                        self.settings.step6_fit_graph_color = None;
                                    }
                                });
                            }
                            ui.end_row();
                        });
                    if ui.button("Detect").clicked() {
                        self.state = self.detect(ui)
                    }
                }

                ui.heading("Axis settings");

                egui::Grid::new("axis_settings_grid")
                    .num_columns(3)
                    .max_col_width(ui.available_width() / 2.)
                    .show(ui, |ui| {
                        {
                            ui.label(" ");
                            ui.label("Min");
                            ui.label("Max");
                            ui.end_row();
                        }

                        self.axis_settings.x_axis.show("X", ui);
                        ui.end_row();
                        self.axis_settings.y_axis.show("Y", ui);
                        ui.end_row();
                    });

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
                ui.heading("No image loaded yet");
                None
            }
            State::CropByRectangle(crop_by_rectangle) => {
                ui.heading("Crop image: Select rectangle via drag and drop");
                enum Request {
                    None,
                    Refine,
                    Detect,
                    CropByRectangle(UnitQuadrilateral),
                }
                let mut requested = Request::None;
                if self.crop_settings.is_set().is_some() {
                    ui.horizontal(|ui| {
                        if ui.button("Refine").clicked() {
                            requested = Request::Refine;
                        }
                        if ui.button("Detect").clicked() {
                            requested = Request::Detect;
                        }
                    });
                }
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
                            requested = Request::CropByRectangle(UnitQuadrilateral::rectangular(
                                start, end,
                            ));
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
                if let Some(previous) = &crop_by_rectangle.previous_rectangle {
                    let UnitQuadrilateral { lt, lb, rt, rb } = previous;
                    let lerp = |p: &UnitPoint| response.rect.lerp_inside(egui::vec2(p.x.0, p.y.0));
                    let lt = lerp(lt);
                    let lb = lerp(lb);
                    let rt = lerp(rt);
                    let rb = lerp(rb);
                    for points in [[lt, lb], [lb, rb], [rb, rt], [rt, lt]] {
                        ui.painter()
                            .with_clip_rect(response.rect)
                            .line_segment(points, egui::Stroke::new(3.0, egui::Color32::GOLD));
                    }
                }

                match requested {
                    Request::None => None,
                    Request::Refine => Some(Work::RefineCrop(Default::default())),
                    Request::Detect => Some(Work::Detect),
                    Request::CropByRectangle(selected_area) => {
                        Some(Work::CroppedByRectangle(selected_area))
                    }
                }
            }
            State::LineDetecting(handle, x) => {
                if handle.is_finished() {
                    Some(Work::LineDetectionDone)
                } else {
                    ui.label("Computing ...");
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(16));
                    let delta_time = std::time::Instant::now() - *x;
                    let x = delta_time.as_millis() as f32 / 5000.0;
                    egui::Widget::ui(egui::ProgressBar::new(x.fract()), ui);
                    None
                }
            }
            State::LineDetected(result) => {
                let mut error = None;
                match result {
                    Ok((image, lines)) => {
                        if let Some(image) = image {
                            ui.horizontal(|ui| {
                                if ui.button("Save csv to file").clicked() {
                                    let dialog = rfd::FileDialog::new().set_title("Save csv to");
                                    let dialog =
                                        if let Some(file_name) = self.file_state.file_name() {
                                            dialog.set_file_name(format!("{file_name}.csv"))
                                        } else {
                                            dialog
                                        };
                                    if let Some(path) = dialog.save_file() {
                                        let csv: String = lines.as_csv();
                                        match std::fs::write(path, csv) {
                                            Ok(()) => {}
                                            Err(e) => error = Some(e.to_string()),
                                        }
                                    }
                                }
                                if ui.button("Copy csv to clipboard").clicked() {
                                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                        let csv: String = lines.as_csv();
                                        let _ = clipboard.set_text(csv);
                                    }
                                }
                            });
                            let image = egui::Image::from_texture(egui::load::SizedTexture {
                                id: image.id(),
                                size: ui.available_size_before_wrap(),
                            })
                            .sense(egui::Sense::click());
                            egui::Widget::ui(image, ui)
                                .on_hover_text("Cropped image with detected lines");
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
            State::RefineCrop(refine) => {
                ui.horizontal(|ui| {
                    ui.heading("Click to refine crop point: ");
                    ui.heading(refine.label())
                });
                if let Some((id, point)) = {
                    self.original_image
                        .as_ref()
                        .map(|x| x.1.id())
                        .and_then(|id| {
                            let point = match refine {
                                RefineCrop::LeftTop => self.crop_settings.left_top.is_set(),
                                RefineCrop::LeftBottom => self.crop_settings.left_bottom.is_set(),
                                RefineCrop::RightTop => self.crop_settings.right_top.is_set(),
                                RefineCrop::RightBottom => self.crop_settings.right_bottom.is_set(),
                            };
                            point.map(|p| (id, p))
                        })
                } {
                    let mut new_position = None;
                    {
                        let x = point.x.0;
                        let y = point.y.0;
                        const OFFSET: f32 = 0.05;
                        fn reduce(x: f32) -> f32 {
                            let x = x - OFFSET;
                            if x > 0. {
                                x
                            } else {
                                0.
                            }
                        }
                        fn increase(x: f32) -> f32 {
                            let x = x + OFFSET;
                            if x < 1. {
                                x
                            } else {
                                1.
                            }
                        }
                        let x_min = reduce(x);
                        let x_max = increase(x);
                        let y_min = reduce(y);
                        let y_max = increase(y);
                        let rect_around_point = egui::Rect::from_min_max(
                            egui::pos2(x_min, y_min),
                            egui::pos2(x_max, y_max),
                        );
                        ui.group(|ui| {
                            let response = ui.allocate_response(
                                ui.available_size_before_wrap(),
                                egui::Sense::click(),
                            );
                            ui.painter().image(
                                id,
                                response.interact_rect,
                                rect_around_point,
                                egui::Color32::WHITE,
                            );
                            let response = response.on_hover_cursor(egui::CursorIcon::Crosshair);
                            if let Some(pos) = response.interact_pointer_pos() {
                                if response.clicked() && response.interact_rect.contains(pos) {
                                    let transformer = egui::emath::RectTransform::from_to(
                                        response.rect,
                                        rect_around_point,
                                    );
                                    new_position = Some(transformer.transform_pos(pos));
                                }
                            }
                        });
                    }
                    if let Some(new_position) = new_position {
                        if let Some(point) = UnitPoint::new([new_position.x, new_position.y]) {
                            self.crop_settings.set_point(refine, point);
                            Some(refine.next_work())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    self.state = State::NothingLoaded;
                    None
                }
            }
        };
        if let Some(work) = work {
            ui.ctx().request_repaint();
            self.state = match work {
                Work::CroppedByRectangle(area) => {
                    self.crop_settings.set(area);
                    State::RefineCrop(RefineCrop::LeftTop)
                }
                Work::CropByRectangle => State::CropByRectangle(self.crop_settings.convert()),
                Work::RefineCrop(refine) => State::RefineCrop(refine),
                Work::Detect => self.detect(ui),
                Work::LineDetectionDone => match std::mem::take(&mut self.state) {
                    State::LineDetecting(handle, x) => {
                        if handle.is_finished() {
                            let r = handle
                                .join()
                                .map_err(|e| format!("{e:?}"))
                                .map(|r| r.map_err(|e| format!("{e:?}")));
                            let r = match r {
                                Ok(r) => match r {
                                    Ok(x) => Ok(x),
                                    Err(e) => Err(e),
                                },
                                Err(e) => Err(e),
                            };
                            State::LineDetected(r)
                        } else {
                            State::LineDetecting(handle, x)
                        }
                    }
                    _ => panic!("Unexpected state"),
                },
            }
        }
    }

    #[must_use]
    fn detect(&mut self, ui: &mut egui::Ui) -> State {
        if let Some(crop_area) = self.crop_settings.is_set() {
            if let Some(axes) = self.axis_settings.is_set() {
                let ui_context = ui.ctx().clone();
                let id = format!("ID: {:?}", ui.auto_id_with("Image"));

                let image = self.original_image.as_ref().unwrap().0.clone();
                let settings = self.settings.clone();
                let thread = std::thread::spawn(move || {
                    let cropped = crop_area.transform([image.width(), image.height()]);
                    let line_detection = graph_to_data::line_detection(
                        &image,
                        &settings,
                        crop_area,
                        cropped.width(),
                        cropped.height(),
                        axes.x_limits(),
                        axes.y_limits(),
                    );
                    line_detection
                        .map(|l| {
                            (
                                l.final_image_with_plots().map(|image| {
                                    let size = [image.width() as _, image.height() as _];
                                    let pixels = image.as_flat_samples();
                                    let egui_image = egui::ColorImage::from_rgba_unmultiplied(
                                        size,
                                        pixels.as_slice(),
                                    );
                                    let texture_id = ui_context.load_texture(
                                        id,
                                        egui_image,
                                        egui::TextureOptions::NEAREST,
                                    );
                                    texture_id
                                }),
                                l,
                            )
                        })
                        .map_err(|e| format!("{e:?}"))
                });
                State::LineDetecting(thread, std::time::Instant::now())
            } else {
                State::LineDetected(Err("Axes not set".into()))
            }
        } else {
            State::CropByRectangle(self.crop_settings.convert())
        }
    }
}
enum Work {
    CroppedByRectangle(UnitQuadrilateral),
    CropByRectangle,
    RefineCrop(RefineCrop),
    Detect,
    LineDetectionDone,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct CropByRectangle {
    previous_rectangle: Option<UnitQuadrilateral>,
    drag_start: Option<egui::Pos2>,
}
impl CropByRectangle {
    fn with_previous(area: UnitQuadrilateral) -> Self {
        Self {
            previous_rectangle: Some(area),
            drag_start: None,
        }
    }

    fn new() -> Self {
        Self {
            previous_rectangle: None,
            drag_start: None,
        }
    }
}
#[derive(Default)]
enum State {
    #[default]
    NothingLoaded,
    CropByRectangle(CropByRectangle),
    LineDetecting(
        std::thread::JoinHandle<
            Result<(Option<egui::TextureHandle>, graph_to_data::LineDetected), String>,
        >,
        std::time::Instant,
    ),
    LineDetected(Result<(Option<egui::TextureHandle>, graph_to_data::LineDetected), String>),
    RefineCrop(RefineCrop),
}
#[derive(Default)]
enum RefineCrop {
    #[default]
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
}
impl RefineCrop {
    fn next_work(&self) -> Work {
        use RefineCrop::*;
        match self {
            LeftTop => Work::RefineCrop(LeftBottom),
            LeftBottom => Work::RefineCrop(RightTop),
            RightTop => Work::RefineCrop(RightBottom),
            RightBottom => Work::CropByRectangle,
        }
    }

    fn label(&self) -> &str {
        match self {
            RefineCrop::LeftTop => "Left Top",
            RefineCrop::LeftBottom => "Left Bottom",
            RefineCrop::RightTop => "Right Top",
            RefineCrop::RightBottom => "Right Bottom",
        }
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
