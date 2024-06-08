fn main() {
    let image_bytes = include_bytes!(
        //"../example_data/Mplwp_dispersion_curves.svg.png"
        //"../example_data/Polynomial_of_degree_three.svg.png"
        //"../example_data/X^4_4^x.PNG"
        //"../example_data/FFT_of_Cosine_Summation_Function.svg.png"
        "../example_data/Tuberculosis_incidence_US_1953-2009.png"
    );
    let image = image::io::Reader::new(std::io::Cursor::new(image_bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .to_rgb8();

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
