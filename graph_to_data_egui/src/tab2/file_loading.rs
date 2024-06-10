#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct FileState {
    file_path: Option<std::path::PathBuf>,
    #[serde(skip)]
    file_name: Option<String>,
    #[serde(skip)]
    title: Option<String>,
    #[serde(skip)]
    state: FileStateEnum,
}
impl FileState {
    #[must_use]
    pub fn progress(&mut self) -> bool {
        use FileStateEnum::*;
        self.state = match std::mem::take(&mut self.state) {
            NoFileSelected => {
                if let Some(path) = self.file_path.take() {
                    self.load_from_path(path);
                    return true;
                } else {
                    NoFileSelected
                }
            }
            ReadingFile(handle) => {
                if handle.is_finished() {
                    match handle.join() {
                        Ok(r) => match r {
                            Ok(bytes) => {
                                self.load_from_bytes(bytes);
                                return true;
                            }
                            Err(e) => {
                                self.title =
                                    Some(format!("Error: {}", self.file_name.as_ref().unwrap()));
                                Error(format!("{e:?}"))
                            }
                        },
                        Err(e) => {
                            self.title =
                                Some(format!("Error: {}", self.file_name.as_ref().unwrap()));
                            Error(format!("{e:?}"))
                        }
                    }
                } else {
                    ReadingFile(handle)
                }
            }
            LoadingFromBytes(handle) => {
                if handle.is_finished() {
                    match handle.join() {
                        Ok(r) => match r {
                            Ok(image) => {
                                self.state = Loaded(Some(image));
                                self.title = Some(self.file_name.as_ref().unwrap().clone());
                                return true;
                            }
                            Err(e) => {
                                self.title =
                                    Some(format!("Error: {}", self.file_name.as_ref().unwrap()));
                                Error(format!("{e:?}"))
                            }
                        },
                        Err(e) => {
                            self.title =
                                Some(format!("Error: {}", self.file_name.as_ref().unwrap()));
                            Error(format!("{e:?}"))
                        }
                    }
                } else {
                    LoadingFromBytes(handle)
                }
            }
            Error(s) => Error(s),
            Loaded(image) => Loaded(image),
        };
        match &self.state {
            NoFileSelected | Loaded(_) | Error(_) => false,
            ReadingFile(_) | LoadingFromBytes(_) => true,
        }
    }

    pub fn load(&mut self, file: egui::DroppedFile) {
        let egui::DroppedFile {
            path,
            name,
            mime: _,
            last_modified: _,
            bytes,
        } = file;
        if let Some(path) = path {
            self.load_from_path(path);
        } else if let Some(bytes) = bytes {
            self.file_name = Some(name);
            self.load_from_bytes(bytes.to_vec());
        } else {
            panic!("Unexpected egui file");
        }
    }

    fn load_from_path(&mut self, path: std::path::PathBuf) {
        self.file_name = Some(
            path.file_name()
                .unwrap_or(path.as_os_str())
                .to_string_lossy()
                .to_string(),
        );
        self.state = {
            let path = path.clone();
            let handle = std::thread::spawn(|| std::fs::read(path));
            FileStateEnum::ReadingFile(handle)
        };
        self.file_path = Some(path);
        self.title = Some(format!("Loading: {}", self.file_name.as_ref().unwrap()));
    }

    fn load_from_bytes(&mut self, bytes: Vec<u8>) {
        self.state = {
            let handle =
                std::thread::spawn(move || image::load_from_memory(&bytes).map(|x| x.to_rgba8()));
            FileStateEnum::LoadingFromBytes(handle)
        };
        self.title = Some(format!("Parsing: {}", self.file_name.as_ref().unwrap()));
    }

    pub(crate) fn title(&self) -> egui::WidgetText {
        self.title.as_deref().unwrap_or("Load file").into()
    }

    pub(crate) fn file_dropped(
        &mut self,
        file: egui::DroppedFile,
    ) -> Result<(), egui::DroppedFile> {
        if let FileStateEnum::NoFileSelected = &self.state {
            self.load(file);
            Ok(())
        } else {
            Err(file)
        }
    }

    pub(crate) fn is_error(&self) -> Option<&str> {
        if let FileStateEnum::Error(e) = &self.state {
            Some(e.as_str())
        } else {
            None
        }
    }

    pub(crate) fn show_select_image_button(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Select file").clicked() {
                if let Some(path) = rfd::FileDialog::new().set_title("Select image").pick_file() {
                    self.load_from_path(path);
                }
            }
            if ui.button("From clipboard").clicked() {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Ok(image) = clipboard.get_image() {
                        let buffer = image::ImageBuffer::from_fn(
                            image.width as u32,
                            image.height as u32,
                            |x, y| {
                                let i = y as usize * image.width + x as usize;
                                let [r, g, b, a] =
                                    image.bytes[4 * i..4 * (i + 1)].try_into().unwrap();
                                image::Rgba([r, g, b, a])
                            },
                        );
                        let mut bytes: Vec<u8> = Vec::new();
                        if let Ok(()) = buffer.write_to(
                            &mut std::io::Cursor::new(&mut bytes),
                            image::ImageFormat::Png,
                        ) {
                            self.file_name = Some("From Clipboard".into());
                            self.load_from_bytes(bytes);
                        }
                    }
                }
            }
        });
    }

    pub(crate) fn is_loaded(&mut self) -> Option<Option<super::ImageBuf>> {
        match &mut self.state {
            FileStateEnum::Loaded(image) => Some(image.take()),
            _ => None,
        }
    }

    pub(crate) fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }
}
#[derive(Default, Debug)]
pub enum FileStateEnum {
    #[default]
    NoFileSelected,
    ReadingFile(std::thread::JoinHandle<Result<Vec<u8>, std::io::Error>>),
    Error(String),
    LoadingFromBytes(std::thread::JoinHandle<Result<super::ImageBuf, image::ImageError>>),
    Loaded(Option<super::ImageBuf>),
}
