use plotters::prelude::*;
use raytracer::color::xyz::CIE_XYZ;
use raytracer::spectrum::{DenselySampled, Spectrum};
use raytracer::Result;

fn plot_spectrum(spectrum: &Spectrum, file_name: &str) -> Result<()> {
    let sampled = DenselySampled::from_spectrum(spectrum.clone());
    let to_plot: Vec<(f32, f32)> = sampled
        .range()
        .map(|i| i as f32)
        .zip(sampled.values().iter().copied())
        .collect();

    let backend = BitMapBackend::new(file_name, (800, 800)).into_drawing_area();
    backend.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&backend)
        .margin(10)
        .set_left_and_bottom_label_area_size(20)
        .build_cartesian_2d(380.0f32..830.0, 0.0f32..1.0)?;
    chart.draw_series(LineSeries::new(to_plot, &BLUE))?;
    chart.configure_mesh().draw()?;
    chart
        .configure_series_labels()
        .background_style(WHITE)
        .border_style(BLACK)
        .draw()?;

    backend.present()?;

    Ok(())
}

fn main() -> Result<()> {
    let spectrum = &CIE_XYZ.z;
    plot_spectrum(spectrum, "rgb.png")?;

    Ok(())
}
