#[derive(serde::Serialize, serde::Deserialize)]
pub struct DetectionTaskInput {
    pub image: super::ImageSerde,
    pub settings: graph_to_data::Settings,
    pub crop_area: graph_to_data::UnitQuadrilateral,
    pub axes: crate::tab::Axes,
}
#[derive(Default)]
pub struct DetectionTask;
impl task_simple::Function for DetectionTask {
    type Input = DetectionTaskInput;

    type Output = Result<(Option<super::ImageSerde>, String), String>;

    fn call(&mut self, input: Self::Input) -> Self::Output {
        let DetectionTaskInput {
            image,
            settings,
            crop_area,
            axes,
        } = input;
        let image: crate::ImageBuf = image.into();
        let cropped = crop_area.transform([image.width(), image.height()]);
        graph_to_data::line_detection(
            &image,
            &settings,
            crop_area,
            cropped.width(),
            cropped.height(),
            axes.x_limits(),
            axes.y_limits(),
        )
        .map_err(|e| format!("{e:?}"))
        .map(|l| {
            (
                l.final_image_with_plots().map(|x| x.clone().into()),
                l.as_csv(),
            )
        })        
    }
}
