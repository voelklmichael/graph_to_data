#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AxisDimension {
    fraction: Option<f32>,
    before: String,
    current: String,
    is_parse_error: Option<String>,
}
impl AxisDimension {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            if self.is_parse_error.is_some() {
                ui.style_mut().visuals.extreme_bg_color = egui::Color32::RED;
                ui.style_mut().visuals.override_text_color = Some(egui::Color32::WHITE);
            } else if self.current.is_empty() {
                ui.style_mut().visuals.extreme_bg_color = egui::Color32::YELLOW;
                ui.style_mut().visuals.override_text_color = Some(egui::Color32::WHITE);
            }
            ui.text_edit_singleline(&mut self.current);
        });

        if self.current != self.before {
            self.before.clone_from(&self.current);
            if self.current.trim().is_empty() {
                self.fraction = None;
                self.is_parse_error = None;
            } else {
                match self.current.parse::<f32>() {
                    Ok(v) => {
                        self.fraction = Some(v);
                        self.is_parse_error = None;
                    }
                    Err(e) => self.is_parse_error = Some(e.to_string()),
                }
            }
        }
    }

    fn new(x: f32) -> Self {
        Self {
            fraction: Some(x),
            before: x.to_string(),
            current: x.to_string(),
            is_parse_error: None,
        }
    }
}
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AxisPoint {
    x: AxisDimension,
    y: AxisDimension,
}
impl AxisPoint {
    pub fn show(&mut self, label: &str, ui: &mut egui::Ui) {
        ui.label(label);
        self.x.show(ui);
        self.y.show(ui);
    }

    pub fn is_set(&self) -> Option<Axis> {
        self.x
            .fraction
            .and_then(|min| self.y.fraction.map(|max| Axis { min, max }))
    }

    fn new(x: f32, y: f32) -> Self {
        Self {
            x: AxisDimension::new(x),
            y: AxisDimension::new(y),
        }
    }
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AxisSettings {
    pub x_axis: AxisPoint,
    pub y_axis: AxisPoint,
}
impl Default for AxisSettings {
    fn default() -> Self {
        Self {
            x_axis: AxisPoint::new(0., 1.),
            y_axis: AxisPoint::new(0., 1.),
        }
    }
}
impl AxisSettings {
    pub fn is_set(&self) -> Option<Axes> {
        let x_axis = self.x_axis.is_set();
        let y_axis = self.y_axis.is_set();
        if let (Some(x_axis), Some(y_axis)) = (x_axis, y_axis) {
            Some(Axes { x_axis, y_axis })
        } else {
            None
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct Axis {
    min: f32,
    max: f32,
}
impl Axis {
    fn limits(&self) -> (f32, f32) {
        (self.min, self.max)
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct Axes {
    x_axis: Axis,
    y_axis: Axis,
}
impl Axes {
    pub(crate) fn x_limits(&self) -> (f32, f32) {
        self.x_axis.limits()
    }

    pub(crate) fn y_limits(&self) -> (f32, f32) {
        self.y_axis.limits()
    }
}
