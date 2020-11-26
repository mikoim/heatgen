use std::collections::HashMap;
use std::error::Error;
use std::process;
use std::sync::RwLock;

use image::{GrayImage, Luma};
use ndarray::Array2;
use rayon::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, Hash, PartialEq, Eq)]
struct Point {
    x: u32,
    y: u32,
}

fn example(width: u32, height: u32) -> Result<(), Box<dyn Error>> {
    let mut img = GrayImage::new(width, height);
    let freq = RwLock::new(Array2::<u32>::zeros((height as usize, width as usize)));

    let mut points = HashMap::<Point, u32>::new();

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path("input.csv")?;
    let mut raw_record = csv::StringRecord::new();

    while reader.read_record(&mut raw_record)? {
        let record: Point = raw_record.deserialize(None)?;
        let count = points.entry(record).or_insert(0);
        *count += 1;
    }

    let max = RwLock::new(0);

    points.par_iter().for_each(|a| {
        let r = 50;
        let cx = a.0.x as i32;
        let cy = a.0.y as i32;
        for y in -r..r {
            for x in -r..r {
                if x * x + y * y <= r * r {
                    let xx = cx + x;
                    let yy = cy + y;
                    if 0 < xx && xx < width as i32 && 0 < yy && yy < height as i32 {
                        let mut aa = freq.write().unwrap();
                        let v = aa.get_mut((yy as usize, xx as usize)).unwrap();
                        *v += a.1;
                        if *max.read().unwrap() < *v {
                            *max.write().unwrap() = *v;
                        }
                    }
                }
            }
        }
    });

    img.enumerate_rows_mut().par_bridge().for_each(|(y, line)| {
        let foo = freq.read().unwrap();
        let aa = foo.row(y as usize);
        let bb = line;

        aa.iter().zip(bb.into_iter()).for_each(|(a, b)| {
            let (_, _, p) = b;
            *p = Luma([(255.0 * *a as f32 / *max.read().unwrap() as f32) as u8]);
        });
    });

    img.save("output.png");

    Ok(())
}

fn main() {
    if let Err(err) = example(1920, 1080) {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
