fn main() {
    let image_bytes = include_bytes!("..../Readme_Image_Graph.png");
    let image = image::io::Reader::new(std::io::Cursor::new(image_bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .to_rgba8();

    let lines = graph_to_data::line_detection(
        &image,
        &Default::default(),
        graph_to_data::UnitQuadrilateral::unit_square(),
        image.width(),
        image.height(),
        (1950., 2010.),
        (0., 60.),
    )
    .unwrap();
    lines.save("").unwrap();
}
