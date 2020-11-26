use std::collections::HashMap;
use std::error::Error;

use clap::{App, Arg};
use image::{GrayImage, Luma};
use ndarray::Array2;
use rayon::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, Hash, PartialEq, Eq)]
struct Point {
    x: u32,
    y: u32,
}

fn process(
    width: u32,
    height: u32,
    radius: u32,
    input: &str,
    output: &str,
) -> Result<(), Box<dyn Error>> {
    let mut img = GrayImage::new(width, height);
    let mut freq = Array2::<u32>::zeros((height as usize, width as usize));

    let mut points = HashMap::<Point, u32>::new();

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(input)?;
    let mut raw_record = csv::StringRecord::new();

    while reader.read_record(&mut raw_record)? {
        raw_record.deserialize(None).map(|record|{
            let count = points.entry(record).or_insert(0);
            *count += 1;
        }).unwrap_or_else(|err|{eprintln!("Skipping unrecognized line: {}", err)});
    }

    let mut max: u32 = 0;

    points.iter().for_each(|a| {
        let r = radius as i32;
        let cx = a.0.x as i32;
        let cy = a.0.y as i32;
        for y in -r..r {
            for x in -r..r {
                if x * x + y * y <= r * r {
                    let xx = cx + x;
                    let yy = cy + y;
                    if 0 < xx && xx < width as i32 && 0 < yy && yy < height as i32 {
                        let v = freq.get_mut((yy as usize, xx as usize)).unwrap();
                        *v += a.1;
                        if max < *v {
                            max = *v;
                        }
                    }
                }
            }
        }
    });

    img.enumerate_rows_mut().par_bridge().for_each(|(y, line)| {
        freq.row(y as usize)
            .iter()
            .zip(line.into_iter())
            .for_each(|(a, b)| {
                let (_, _, p) = b;
                *p = Luma([(255.0 * *a as f32 / max as f32) as u8]);
            });
    });

    img.save(output)?;

    Ok(())
}

fn main() {
    let matches = App::new("Heatmap generator")
        .version("0.0.0")
        .author("Eshin Kunishima <ek@esh.ink>")
        .arg(
            Arg::new("WIDTH")
                .about("Image width")
                .default_value("1920")
                .short('w'),
        )
        .arg(
            Arg::new("HEIGHT")
                .about("Image height")
                .default_value("1080")
                .short('h'),
        )
        .arg(
            Arg::new("RADIUS")
                .about("Radius")
                .default_value("100")
                .short('r'),
        )
        .arg(
            Arg::new("INPUT")
                .about("Input csv file path (e.g., input.csv)")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("OUTPUT")
                .about("Output heatmap image path (e.g., output.png)")
                .required(true)
                .index(2),
        )
        .get_matches();

    let width: u32 = matches.value_of_t("WIDTH").unwrap();
    let height: u32 = matches.value_of_t("HEIGHT").unwrap();
    let radius: u32 = matches.value_of_t("RADIUS").unwrap();
    let input = matches.value_of("INPUT").unwrap();
    let output = matches.value_of("OUTPUT").unwrap();

    match process(width, height, radius, input, output) {
        Ok(_) => println!(""),
        Err(err) => eprintln!("Error! {}", err),
    }
}
