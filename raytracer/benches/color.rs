use criterion::{black_box, criterion_group, criterion_main, Criterion};
use raytracer::color::colorspace::S_RGB;
use raytracer::color::rgb::Rgb;
use raytracer::color::xyz::Xyz;
use raytracer::spectrum::{DenselySampled, HasWavelength, RgbAlbedo};

pub fn benchmark(c: &mut Criterion) {
    let color_space = &S_RGB;
    let rgb = Rgb::new(0.5, 0.2, 0.4);
    let spectrum = RgbAlbedo::with_color_space(&color_space, rgb);

    let spectrum = DenselySampled::from_fn(|lambda| {
        let rgb = spectrum.evaluate(lambda);
        let w = color_space.illuminant.evaluate(lambda);

        rgb * w
    });

    c.bench_function("Xyz::from ", |b| {
        b.iter(|| black_box(Xyz::from(&spectrum)));
    });

    let xyz = Xyz::from(&spectrum);
    c.bench_function("RgbColorSpace::to_rgb", |b| {
        b.iter(|| black_box(color_space.to_rgb(xyz)));
    });

    c.bench_function("RgbColorSpace::to_rgb_coefficients", |b| {
        b.iter(|| black_box(color_space.to_rgb_coefficients(rgb)))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
