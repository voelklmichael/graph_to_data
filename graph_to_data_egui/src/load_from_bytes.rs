fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        use graph_to_data_egui_lib::task_simple::gloo_worker::Registrable;
        graph_to_data_egui_lib::task_simple::WebWorker::<
            graph_to_data_egui_lib::tasks::LoadFromBytesTask,
        >::registrar()
        .register();
    }
}
