use plotters::prelude::*;
use raytracer::color::colorspace::S_RGB;
use raytracer::color::rgb::Rgb;
use raytracer::spectrum::{DenselySampled, HasWavelength, RgbAlbedo};
use raytracer::Result;

fn plot_spectrum(spectrum: impl HasWavelength, file_name: &str) -> Result<()> {
    let sampled = DenselySampled::from_spectrum(spectrum);
    let to_plot: Vec<(f32, f32)> = sampled
        .range()
        .map(|i| i as f32)
        .zip(sampled.values().iter().copied())
        .collect();
    let min = sampled
        .values()
        .iter()
        .copied()
        .fold(f32::INFINITY, f32::min);
    let max = sampled
        .values()
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);

    let backend = BitMapBackend::new(file_name, (800, 800)).into_drawing_area();
    backend.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&backend)
        .margin(10)
        .set_left_and_bottom_label_area_size(20)
        .build_cartesian_2d(380.0f32..830.0, min..max)?;
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
    let sigmoid = S_RGB.to_rgb_coefficients(Rgb::new(1.0, 0.0, 0.0));
    let rgb = RgbAlbedo::new(sigmoid);
    plot_spectrum(rgb, "rgb.png")?;

    Ok(())
}
