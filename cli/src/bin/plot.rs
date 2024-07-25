use ordered_float::OrderedFloat;
use plotters::prelude::*;
use raytracer::color::colorspace::S_RGB;
use raytracer::color::rgb::Rgb;
use raytracer::random::random;
use raytracer::spectrum::{
    HasWavelength, RgbAlbedo, SampledWavelengths, Spectrum, N_SPECTRUM_SAMPLES,
};
use raytracer::Result;

fn plot_rgb_albedo(r: f32, g: f32, b: f32) -> Result<()> {
    let spectrum: Spectrum = RgbAlbedo::with_color_space(S_RGB.as_ref(), Rgb::new(r, g, b)).into();

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

fn plot_wavelength_samples(r: f32, g: f32, b: f32) -> Result<()> {
    let spectrum: Spectrum = RgbAlbedo::with_color_space(S_RGB.as_ref(), Rgb::new(r, g, b)).into();

    let n_samples = 100;
    let samples = (0..n_samples).map(|_| {
        let lambda = SampledWavelengths::sample_visible(random());
        let sample = spectrum.sample(&lambda);
        (lambda, sample)
    });

    let mut to_plot = vec![];
    for (lambda, sample) in samples {
        for i in 0..N_SPECTRUM_SAMPLES {
            to_plot.push((lambda.lambda[i], sample.samples[i]));
        }
    }

    to_plot.sort_by_key(|(l, _)| OrderedFloat(*l));

    let backend = BitMapBackend::new("chart.png", (800, 800)).into_drawing_area();
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
    plot_wavelength_samples(1.0, 0.0, 0.0)?;
    Ok(())
}
