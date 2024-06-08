mod image_wrapper;

use graph_to_data::UnitQuadrilateral;
use strum::VariantArray;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
#[serde(default)]

pub struct Settings {
    crop: image_wrapper::CropSettings,
    //   line_detection: graph_to_data::LineDetectionSettings,
}

mod file_picker;
#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
#[serde(default)]
pub struct Tab {
    settings: Settings,

    loaded: Option<(String, Option<std::path::PathBuf>)>,
    #[serde(skip)]
    deserialized: bool,
    #[serde(skip)]
    file_picker: file_picker::FilePicker,
    #[serde(skip)]
    selected_step: Step,
    #[serde(skip)]
    request_repaint: bool,

    // Step 0
    #[serde(skip)]
    step_0_error: Option<String>,

    // Step 1 - crop
    select_area: Option<SelectedArea>,
    #[serde(skip)]
    step_1_input: Option<image_wrapper::ImageWrapper>,

    // Step 2 - select first point
    select_first_point: Option<image_wrapper::ImagePixel>,
    #[serde(skip)]
    step_2_input: Option<Result<image_wrapper::ImageWrapper, String>>,

    // Step 3 - select first point
    //line_points: Option<Vec<(u32, u32)>>,
    #[serde(skip)]
    step_3_input: Option<Vec<(u32, u32)>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
struct SelectedArea(UnitQuadrilateral);

#[derive(
    serde::Serialize,
    serde::Deserialize,
    Debug,
    Default,
    PartialEq,
    strum::VariantArray,
    Clone,
    Copy,
    PartialOrd,
)]
pub enum Step {
    #[default]
    ImageSelection,
    ImageLoadedAreaSelection,
    AreaSelectedFirstPointSelection,
    LineDetection,
}
impl Step {
    fn label(&self) -> &str {
        match self {
            Step::ImageSelection => "Load image image",
            Step::ImageLoadedAreaSelection => "Crop image",
            Step::AreaSelectedFirstPointSelection => "Select first point",
            Step::LineDetection => "Line detection",
        }
    }
}

impl Tab {
    pub(crate) fn title(&self) -> egui::WidgetText {
        self.loaded
            .as_ref()
            .map(|x| x.0.as_str())
            .unwrap_or("No file selected")
            .into()
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui) {
        // show deserialized data if any
        {
            if let Some((_, Some(path))) = &self.loaded {
                if !self.deserialized {
                    self.load_from_path(path.to_owned())
                }
            }
            self.deserialized = true;
        }

        ui.horizontal(|ui| {
            let mut is_active = true;
            let mut fallback = Step::default();

            for step in Step::VARIANTS {
                if !match step {
                    Step::ImageSelection => true,
                    Step::ImageLoadedAreaSelection => self.step_1_input.is_some(),
                    Step::AreaSelectedFirstPointSelection => self.step_2_input.is_some(),
                    Step::LineDetection => self.step_3_input.is_some(),
                } {
                    is_active = false;
                } else if is_active {
                    fallback = *step;
                }
                if self.selected_step > fallback && !is_active {
                    self.selected_step = fallback;
                }
                ui.scope(|ui| {
                    ui.set_enabled(is_active);
                    ui.selectable_value(&mut self.selected_step, *step, step.label());
                });
            }
        });
        ui.separator();
        match self.selected_step {
            Step::ImageSelection => {
                if let Some(file) = self.file_picker.show_open(ui) {
                    self.file_loaded(file);
                }
                if let Some(error) = &self.step_0_error {
                    show_error(ui, error);
                }
            }
            Step::ImageLoadedAreaSelection => {
                if let Some(Err(msg)) = &self.step_2_input {
                    show_error(ui, msg)
                }
                if let Some(image) = &mut self.step_1_input {
                    if let Some(selected) = image.select_area(ui) {
                        let cropped = image.crop(selected, &self.settings.crop);
                        if cropped.is_ok() {
                            self.step_completed();
                        }
                        self.step_2_input = Some(cropped);
                        self.select_area = Some(selected);
                    }
                }
            }
            Step::AreaSelectedFirstPointSelection => {
                if let Some(Ok(image)) = &mut self.step_2_input {
                    if let Some(first_point) = image.show_clickable_image(ui) {
                        let detection =
                            image.detect_line(first_point, &self.settings.line_detection);
                        self.step_3_input = Some(detection);
                        self.select_first_point = Some(first_point);
                        self.step_completed();
                    }
                }
            }
            Step::LineDetection => {
                if let (Some(line), Some(Ok(cropped))) =
                    (&mut self.step_3_input, &mut self.step_2_input)
                {
                    self.file_picker.show_save(ui, || {
                        Ok(line
                            .iter()
                            .map(|(x, y)| format!("{x};{y}"))
                            .collect::<Vec<_>>()
                            .join("\n"))
                    });
                    cropped.show_fitted_line(line, ui);
                }
            }
        }

        if std::mem::take(&mut self.request_repaint) {
            ui.ctx().request_repaint();
        }
    }

    fn file_loaded(&mut self, file: file_picker::FileLoaded) {
        let file_picker::FileLoaded {
            file_name,
            bytes,
            path,
        } = file;
        self.loaded = Some((file_name, path));

        let image_result = image::io::Reader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|e| format!("{e:?}"))
            .and_then(|x| x.decode().map_err(|e| format!("{e:?}")))
            .map(|x| image_wrapper::ImageWrapper::new(x.to_rgba8()));
        match image_result {
            Ok(image) => {
                self.step_0_error = None;
                self.step_1_input = Some(image);
                self.step_completed();
            }
            Err(error) => {
                self.step_0_error = Some(error);
                self.step_1_input = None;
            }
        }
    }

    fn step_completed(&mut self) {
        if let Some(step) = Step::VARIANTS
            .iter()
            .find(|step: &&Step| step > &&self.selected_step)
        {
            self.selected_step = *step;
        }
        self.request_repaint = true;
        for step in Step::VARIANTS
            .iter()
            .filter(|step: &&Step| step > &&self.selected_step)
        {
            match step {
                Step::ImageSelection => {}
                Step::ImageLoadedAreaSelection => self.step_1_input = None,
                Step::AreaSelectedFirstPointSelection => self.step_2_input = None,
                Step::LineDetection => self.step_3_input = None,
            }
        }
    }
}

fn show_error(ui: &mut egui::Ui, error: &String) {
    ui.heading(
        egui::RichText::new(error.to_string())
            .background_color(egui::Color32::RED)
            .color(egui::Color32::WHITE),
    );
}
