use std::{fs::File, io::BufWriter, path::Path};

use ndarray::Array5;
use raytracer::{color::colorspace::CoefficientsFile, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct JsonFile {
    data: Vec<Vec<Vec<Vec<Vec<f32>>>>>,
    scale: Vec<f32>,
    resolution: usize,
}

fn read_coefficients(path: impl AsRef<Path>) -> Result<JsonFile> {
    use bzip2::bufread::BzDecoder;
    use std::io::BufReader;

    let file = BufReader::new(File::open(path)?);
    let reader = BzDecoder::new(file);

    Ok(serde_json::from_reader(reader)?)
}

fn convert_coefficients(file: JsonFile) -> CoefficientsFile {
    let flattened = file
        .data
        .into_iter()
        .flatten()
        .flatten()
        .flatten()
        .flatten()
        .collect();
    let array = Array5::from_shape_vec((3, 64, 64, 64, 3), flattened).expect("invalid shape");

    CoefficientsFile {
        data: array,
        scale: file.scale,
        resolution: file.resolution,
    }
}

fn main() -> Result<()> {
    let color_spaces = vec!["srgb", "rec2020", "dci_p3", "aces"];
    for color_space in color_spaces {
        let path = format!("../data/{}.json.bz2", color_space);
        let coefficients = read_coefficients(path)?;
        let converted = convert_coefficients(coefficients);

        let output_path = format!("../data/{}.json", color_space);
        let writer = BufWriter::new(File::create(output_path)?);

        serde_json::to_writer(writer, &converted)?;
    }
    Ok(())
}
