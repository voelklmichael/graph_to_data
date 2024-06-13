/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct Graph2DataEguiApp {
    tabs: crate::dock::DockState,
    is_dark: bool,
}

impl Graph2DataEguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    fn reset(&mut self) {
        *self = Default::default();
    }
}

impl eframe::App for Graph2DataEguiApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for file in ctx.input_mut(|x| std::mem::take(&mut x.raw.dropped_files)) {
            self.tabs.file_dropped(file);
            ctx.request_repaint_after(std::time::Duration::from_secs(3));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_visuals(match self.is_dark {
                true => egui::Visuals::dark(),
                false => egui::Visuals::light(),
            });
            egui::menu::bar(ui, |ui| {
                egui::menu::menu_button(ui, "File", |ui| {
                    // Light/dark mode
                    {
                        let is_dark = &mut self.is_dark;
                        let label = match *is_dark {
                            true => "🌙->☀",
                            false => "☀->🌙",
                        };
                        if ui.button(label).clicked() {
                            ui.close_menu();
                            *is_dark = !*is_dark;
                        }
                    }
                    if ui.button("New Tab").clicked() {
                        ui.close_menu();
                        self.tabs.add_new_item(Default::default());
                    }
                    // Reset
                    if ui.button("Reset").clicked() {
                        ui.close_menu();
                        self.reset()
                    }
                });
            });
            crate::dock::DockWidget::default().show(ui, &mut self.tabs);
        });
    }
}
