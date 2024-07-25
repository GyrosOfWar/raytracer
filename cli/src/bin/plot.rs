use plotters::prelude::*;
use raytracer::color::colorspace::S_RGB;
use raytracer::color::rgb::Rgb;
use raytracer::spectrum::{HasWavelength, RgbAlbedo, Spectrum};
use raytracer::Result;

fn main() -> Result<()> {
    let spectrum: Spectrum =
        RgbAlbedo::with_color_space(S_RGB.as_ref(), Rgb::new(0.1, 0.1, 0.5)).into();

    let samples: Vec<_> = (360..830)
        .map(|l| (l as f32, spectrum.evaluate(l as f32)))
        .collect();

    let backend = BitMapBackend::new("chart.png", (800, 800)).into_drawing_area();
    backend.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&backend)
        .margin(10)
        .set_left_and_bottom_label_area_size(20)
        .build_cartesian_2d(380.0f32..830.0, 0.0f32..1.0)?;
    chart.draw_series(LineSeries::new(samples, &BLUE))?;
    chart.configure_mesh().draw()?;
    chart
        .configure_series_labels()
        .background_style(WHITE)
        .border_style(BLACK)
        .draw()?;

    backend.present()?;

    Ok(())
}
