// based on https://github.com/woelper/egui_pick_file/
use std::future::Future;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub(super) struct FilePicker {
    channel: (Sender<FileLoaded>, Receiver<FileLoaded>),
    save_error: Option<String>,
}
impl Default for FilePicker {
    fn default() -> Self {
        Self {
            channel: channel(),
            save_error: None,
        }
    }
}

pub(super) struct FileLoaded {
    pub file_name: String,
    pub bytes: Vec<u8>,
    pub path: Option<std::path::PathBuf>,
}
impl FilePicker {
    pub(crate) fn show_open(&self, ui: &mut egui::Ui) -> Option<FileLoaded> {
        if ui.button("📂 Select image file").clicked() {
            let sender = self.channel.0.clone();
            let task = rfd::AsyncFileDialog::new().pick_file();

            let ctx = ui.ctx().clone();
            execute(async move {
                let file = task.await;
                if let Some(file) = file {
                    let file_name = file.file_name();
                    #[cfg(not(target_arch = "wasm32"))]
                    let path = Some(file.path().to_path_buf());
                    #[cfg(target_arch = "wasm32")]
                    let path = None;
                    let bytes = file.read().await;
                    let _ = sender.send(FileLoaded {
                        file_name,
                        bytes,
                        path,
                    });
                    ctx.request_repaint();
                }
            });
            #[cfg(not(target_arch = "wasm32"))]
            fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
                std::thread::spawn(move || futures::executor::block_on(f));
            }

            #[cfg(target_arch = "wasm32")]
            fn execute<F: Future<Output = ()> + 'static>(f: F) {
                wasm_bindgen_futures::spawn_local(f);
            }
        }
        if let Ok(mut msg) = self.channel.1.try_recv() {
            while let Ok(m) = self.channel.1.try_recv() {
                msg = m;
            }
            Some(msg)
        } else {
            None
        }
    }
    pub(crate) fn show_save<F: Fn() -> Result<String, String>>(
        &mut self,
        ui: &mut egui::Ui,
        f: F,
    ) -> Option<FileLoaded> {
        if let Some(error) = &self.save_error {
            super::show_error(ui, error);
        }
        if ui.button("📂 Select output file").clicked() {
            let task = rfd::AsyncFileDialog::new().save_file();
            match f() {
                Ok(contents) => {
                    execute(async move {
                        let file = task.await;
                        if let Some(file) = file {
                            _ = file.write(contents.as_bytes()).await;
                        }
                    });
                }
                Err(e) => self.save_error = Some(e),
            }

            #[cfg(not(target_arch = "wasm32"))]
            fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
                std::thread::spawn(move || futures::executor::block_on(f));
            }

            #[cfg(target_arch = "wasm32")]
            fn execute<F: Future<Output = ()> + 'static>(f: F) {
                wasm_bindgen_futures::spawn_local(f);
            }
        }
        if let Ok(mut msg) = self.channel.1.try_recv() {
            while let Ok(m) = self.channel.1.try_recv() {
                msg = m;
            }
            Some(msg)
        } else {
            None
        }
    }
}

impl super::Tab {
    pub(crate) fn file_dropped(
        &mut self,
        file: egui::DroppedFile,
    ) -> Result<(), egui::DroppedFile> {
        if self.loaded.is_some() {
            return Err(file);
        }
        let egui::DroppedFile {
            path,
            name,
            mime: _,
            last_modified: _,
            bytes,
        } = file;
        self.loaded = Some((name.clone(), path.clone()));
        if let Some(bytes) = bytes {
            self.file_loaded(FileLoaded {
                file_name: name,
                bytes: bytes.to_vec(),
                path,
            });
        } else if let Some(path) = path {
            self.load_from_path(path)
        } else {
            log::error!("File dropped: {name}, but unexpected data");
        }
        Ok(())
    }

    pub(crate) fn from_dropped_file(file: egui::DroppedFile) -> Self {
        let mut item = Self::default();
        let r = item.file_dropped(file);
        if r.is_err() {
            log::error!(
                "File dropped failed, but item is in default state - \
                this should never happen"
            )
        }
        item
    }
    pub(super) fn load_from_path(&mut self, path: std::path::PathBuf) {
        let bytes = std::fs::read(&path).unwrap_or_default();
        let file_name = path
            .file_name()
            .map(|x| x.to_string_lossy())
            .unwrap_or(path.as_os_str().to_string_lossy())
            .to_string();
        self.file_loaded(FileLoaded {
            file_name,
            bytes,
            path: Some(path),
        })
    }
}
