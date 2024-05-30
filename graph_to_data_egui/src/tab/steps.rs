mod image_original;

#[derive(Default, Debug)]
pub enum Steps {
    #[default]
    NoFileLoaded,
    FileLoaded(image_original::ImageOriginal),
}
impl Steps {
    pub(crate) fn file_loaded(image: Result<image::DynamicImage, String>) -> Steps {
        image_original::ImageOriginal::new(image).into()
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui) {
        *self = match std::mem::take(self) {
            Steps::NoFileLoaded => Steps::NoFileLoaded,
            Steps::FileLoaded(image_original) => image_original.show(ui).into(),
        };
    }
}
