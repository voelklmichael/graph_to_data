#[derive(Default)]
pub struct LoadFromBytesTask {}
impl task_simple::Function for LoadFromBytesTask {
    type Input = Vec<u8>;

    type Output = Result<ImageSerde, String>;

    fn call(&mut self, bytes: Self::Input) -> Self::Output {
        image::load_from_memory(&bytes)
            .map(|x| x.to_rgba8())
            .map(Into::into)
            .map_err(|e| format!("{e:?}"))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ImageSerde {
    width: u32,
    height: u32,
    bytes: Vec<u8>,
}
impl From<crate::ImageBuf> for ImageSerde {
    fn from(value: crate::ImageBuf) -> Self {
        Self {
            width: value.width(),
            height: value.height(),
            bytes: value.into_vec(),
        }
    }
}
impl From<ImageSerde> for crate::ImageBuf {
    fn from(value: ImageSerde) -> Self {
        let ImageSerde {
            width,
            height,
            bytes,
        } = value;
        Self::from_vec(width, height, bytes).expect("Failed to convert ImageSerde to ImageBuffer")
    }
}
