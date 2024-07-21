use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use raytracer::color::CoefficientsFile;
use regex::Regex;

fn parse_coefficients(regex: &Regex, content: &str) -> Vec<f32> {
    let expected = 3 * 64 * 64 * 64 * 3;
    let mut numbers: Vec<f32> = Vec::with_capacity(expected);
    for captures in regex.captures_iter(&content) {
        let x: f32 = captures.get(1).unwrap().as_str().parse().unwrap();
        let y: f32 = captures.get(2).unwrap().as_str().parse().unwrap();
        let z: f32 = captures.get(3).unwrap().as_str().parse().unwrap();
        numbers.extend(&[x, y, z]);
    }
    assert_eq!(expected, numbers.len());

    numbers
}

fn parse_scale(content: &str) -> Vec<f32> {
    let third_line = content.split("\n").nth(2).expect("line not found");

    third_line
        .split(",")
        .map(|s| s.trim())
        .filter(|n| n.len() > 0 && !n.contains("{") && !n.contains("}"))
        .map(|n| n.parse::<f32>().unwrap())
        .collect()
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let coeff_regex = Regex::new(r#"\{\s+([-e0-9\.]+),\s+([-e0-9\.]+),\s+([-e0-9\.]+),\s+}"#)?;

    let files: Vec<_> = std::fs::read_dir("./data")?.collect();
    for file in files {
        let file = file?;
        let file_name = file.file_name().into_string().expect("must have file name");
        if file_name.ends_with(".c") {
            println!("processing file {}", file.path().display());
            let content = std::fs::read_to_string(file.path())?;
            let coefficients = parse_coefficients(&coeff_regex, &content);
            let scale = parse_scale(&content);

            let data = CoefficientsFile {
                coefficients,
                scale,
            };
            let file_name = format!(
                "{}.bin",
                file_name.replace("rgbspectrum_", "").replace(".c", "")
            );
            let path = Path::new("../raytracer/data/color-spaces").join(file_name);
            let mut writer = BufWriter::new(File::create(path)?);
            bincode::serialize_into(&mut writer, &data)?;
        }
    }

    Ok(())
}
