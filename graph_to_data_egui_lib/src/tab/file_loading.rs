use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct FileState {
    file_path: Option<std::path::PathBuf>,
    #[serde(skip)]
    file_name: Option<String>,
    #[serde(skip)]
    title: Option<String>,
    #[serde(skip)]
    state: FileStateEnum,
    #[serde(skip)]
    sender_receiver: Option<(Sender<BackgroundTask>, Receiver<BackgroundTask>)>,
    #[serde(skip)]
    load_from_bytes: LoadFromBytesTaskWrapper,
}

pub struct LoadFromBytesTaskWrapper {
    task: task_simple::Task<crate::tasks::LoadFromBytesTask>,
}
impl std::fmt::Debug for LoadFromBytesTaskWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadFromBytesTaskWrapper").finish()
    }
}
impl Default for LoadFromBytesTaskWrapper {
    fn default() -> Self {
        let task = task_simple::Task::<crate::tasks::LoadFromBytesTask>::new("load_from_bytes");
        Self { task }
    }
}

pub enum BackgroundTask {
    #[cfg(target_arch = "wasm32")]
    BytesWithFilename {
        file_name: String,
        bytes: Vec<u8>,
    },
    BytesFromClipboard(Vec<u8>),
}
impl FileState {
    #[must_use]
    pub fn progress(&mut self) -> bool {
        if let Some((_, receiver)) = &self.sender_receiver {
            if let Ok(task) = receiver.try_recv() {
                match task {
                    #[cfg(target_arch = "wasm32")]
                    BackgroundTask::BytesWithFilename { file_name, bytes } => {
                        self.file_name = Some(file_name);
                        self.load_from_bytes(bytes);
                    }
                    BackgroundTask::BytesFromClipboard(bytes) => {
                        self.file_name = Some("From Clipboard".into());
                        self.load_from_bytes(bytes);
                    }
                }
            }
        }

        use FileStateEnum::*;
        self.state = match std::mem::take(&mut self.state) {
            NoFileSelected => {
                if let Some(_path) = self.file_path.take() {
                    #[cfg(not(target_arch = "wasm32"))]
                    self.load_from_path(_path);
                    return true;
                } else {
                    NoFileSelected
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
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
            LoadingFromBytes => {
                if let Some(result) = self.load_from_bytes.task.check() {
                    match result {
                        Ok(image) => {
                            self.title = Some(self.file_name.as_ref().unwrap().clone());
                            Loaded(Some(image.into()))
                        }
                        Err(e) => {
                            self.title =
                                Some(format!("Error: {}", self.file_name.as_ref().unwrap()));
                            Error(format!("{e:?}"))
                        }
                    }
                } else {
                    LoadingFromBytes
                }
            }
            Error(s) => Error(s),
            Loaded(image) => Loaded(image),
        };
        match &self.state {
            NoFileSelected | Loaded(_) | Error(_) => false,
            #[cfg(not(target_arch = "wasm32"))]
            ReadingFile(_) => true,
            LoadingFromBytes => true,
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
        if let Some(_path) = path {
            #[cfg(not(target_arch = "wasm32"))]
            self.load_from_path(_path);
        } else if let Some(bytes) = bytes {
            self.file_name = Some(name);
            self.load_from_bytes(bytes.to_vec());
        } else {
            panic!("Unexpected egui file");
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
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
        self.load_from_bytes.task.enqueue(bytes);
        self.state = FileStateEnum::LoadingFromBytes;
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
    fn get_sender(&mut self) -> Sender<BackgroundTask> {
        if self.sender_receiver.is_none() {
            self.sender_receiver = Some(channel());
        }
        self.sender_receiver.as_ref().unwrap().0.clone()
    }
    pub(crate) fn show_select_image_button(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("📂 Select file").clicked() {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(path) = rfd::FileDialog::new().set_title("Select image").pick_file() {
                    self.load_from_path(path);
                }

                #[cfg(target_arch = "wasm32")]
                {
                    let task = rfd::AsyncFileDialog::new()
                        .set_title("Select image")
                        .pick_file();

                    let ctx = ui.ctx().clone();
                    let sender = self.get_sender();
                    super::execute(async move {
                        if let Some(file) = task.await {
                            let file_name = file.file_name();
                            let bytes = file.read().await;
                            let _ =
                                sender.send(BackgroundTask::BytesWithFilename { file_name, bytes });
                        }
                        ctx.request_repaint();
                    });
                }
            }
            if ui.button("📋 From clipboard").clicked() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let sender = self.get_sender();
                    std::thread::spawn(move || {
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
                                    let _ = sender.send(BackgroundTask::BytesFromClipboard(bytes));
                                }
                            }
                        }
                    });
                }
                #[cfg(target_arch = "wasm32")]
                {
                    let sender = self.get_sender();
                    super::execute(async move {
                        if let Some(clipboard) = {
                            web_sys::window()
                                .map(|x| x.navigator())
                                .and_then(|x| x.clipboard())
                        } {
                            if let Ok(js) =
                                wasm_bindgen_futures::JsFuture::from(clipboard.read()).await
                            {
                                let array = web_sys::js_sys::Uint8Array::new(&js);
                                let bytes: Vec<u8> = array.to_vec();
                                let _ = sender.send(BackgroundTask::BytesFromClipboard(bytes));
                            }
                        }
                    });
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

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }
}
#[derive(Default, Debug)]
pub enum FileStateEnum {
    #[default]
    NoFileSelected,
    #[cfg(not(target_arch = "wasm32"))]
    ReadingFile(std::thread::JoinHandle<Result<Vec<u8>, std::io::Error>>),
    Error(String),
    LoadingFromBytes,
    Loaded(Option<super::ImageBuf>),
}
