use graph_to_data::{UnitInterval, UnitPoint, UnitQuadrilateral};

use super::CropByRectangle;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CropDimension {
    fraction: Option<UnitInterval>,
    before: String,
    current: String,
    is_parse_error: Option<String>,
}
impl CropDimension {
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
                    Ok(v) => match UnitInterval::new(v) {
                        Ok(v) => {
                            self.fraction = Some(v);
                            self.is_parse_error = None;
                        }
                        Err(_) => {
                            self.is_parse_error =
                                Some(format!("Value not between 0.0 and 1.0: {v}"))
                        }
                    },
                    Err(e) => self.is_parse_error = Some(e.to_string()),
                }
            }
        }
    }

    fn set(&mut self, fraction: UnitInterval) {
        *self = Self {
            fraction: Some(fraction),
            before: fraction.0.to_string(),
            current: fraction.0.to_string(),
            is_parse_error: None,
        }
    }
}
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CropPoint {
    x: CropDimension,
    y: CropDimension,
}
impl CropPoint {
    pub fn show(&mut self, label: &str, ui: &mut egui::Ui) {
        ui.label(label);
        self.x.show(ui);
        self.y.show(ui);
    }

    pub fn set(&mut self, point: UnitPoint) {
        self.x.set(point.x);
        self.y.set(point.y);
    }

    pub fn is_set(&self) -> Option<UnitPoint> {
        self.x
            .fraction
            .and_then(|x| self.y.fraction.map(|y| UnitPoint { x, y }))
    }
}
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CropSettings {
    pub left_top: CropPoint,
    pub right_top: CropPoint,
    pub left_bottom: CropPoint,
    pub right_bottom: CropPoint,
}
impl CropSettings {
    pub fn set(&mut self, area: UnitQuadrilateral) {
        self.left_top.set(area.lt);
        self.left_bottom.set(area.lb);
        self.right_top.set(area.rt);
        self.right_bottom.set(area.rb);
    }

    pub fn is_set(&self) -> Option<UnitQuadrilateral> {
        let lt = self.left_top.is_set();
        let lb = self.left_bottom.is_set();
        let rt = self.right_top.is_set();
        let rb = self.right_bottom.is_set();
        if let (Some(lt), Some(lb), Some(rt), Some(rb)) = (lt, lb, rt, rb) {
            Some(UnitQuadrilateral { lt, lb, rt, rb })
        } else {
            None
        }
    }

    pub(crate) fn set_point(&mut self, refine: &super::RefineCrop, point: UnitPoint) {
        let corner = match refine {
            super::RefineCrop::LeftTop => &mut self.left_top,
            super::RefineCrop::LeftBottom => &mut self.left_bottom,
            super::RefineCrop::RightTop => &mut self.right_top,
            super::RefineCrop::RightBottom => &mut self.right_bottom,
        };
        corner.set(point);
    }

    pub(crate) fn convert(&self) -> super::CropByRectangle {
        if let Some(area) = self.is_set() {
            CropByRectangle::with_previous(area)
        } else {
            CropByRectangle::new()
        }
    }
}
