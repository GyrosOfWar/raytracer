use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use raytracer::color::colorspace::{CoefficientsFile, CoefficientsFile2};
use raytracer::Result;

fn read_coefficients(path: impl AsRef<Path>) -> Result<CoefficientsFile> {
    use std::io::BufReader;

    use bzip2::bufread::BzDecoder;

    let file = BufReader::new(File::open(path)?);
    let reader = BzDecoder::new(file);

    Ok(serde_json::from_reader(reader)?)
}

fn convert_coefficients(file: CoefficientsFile) -> CoefficientsFile2 {
    let (flattened, _) = file.data.into_raw_vec_and_offset();
    CoefficientsFile2 {
        data: flattened,
        scale: file.scale,
        resolution: file.resolution,
    }
}

fn main() -> Result<()> {
    let color_spaces = vec!["srgb", "rec2020", "dci_p3", "aces"];
    for color_space in color_spaces {
        let path = format!("../data/color-spaces/{}.json.bz2", color_space);
        let coefficients = read_coefficients(path)?;
        let converted = convert_coefficients(coefficients);

        let output_path = format!("../data/color-spaces/{}_2.json", color_space);
        let writer = BufWriter::new(File::create(output_path)?);

        serde_json::to_writer(writer, &converted)?;
    }
    Ok(())
}
