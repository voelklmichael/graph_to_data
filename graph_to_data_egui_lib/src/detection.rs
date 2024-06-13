fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        use task_simple::gloo_worker::Registrable;
        task_simple::WebWorker::<graph_to_data_egui::tasks::DetectionTask>::registrar().register();
    }
}
