use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rayon::prelude::*;
use std::fs::File;
use std::io;
use std::str::FromStr;

pub type Complex = num::Complex<f64>;

fn escape_time(c: Complex) -> Option<u8> {
    const LIMIT: u8 = 255;
    let mut z = Complex { re: 0., im: 0. };
    for i in 0..LIMIT {
        if z.norm_sqr() > 4. {
            return Some(i);
        }
        z = z * z + c;
    }

    None
}

pub(crate) fn parse_pair<T: FromStr>(s: &str, sep: char) -> Option<(T, T)> {
    match s.find(sep) {
        Some(i) => match (T::from_str(&s[..i]), T::from_str(&s[i + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
        None => None,
    }
}

pub(crate) fn parse_complex(s: &str) -> Option<Complex> {
    parse_pair(s, ',').map(|(re, im)| Complex { re, im })
}

fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    ul: Complex,
    lr: Complex,
) -> Complex {
    let (width, height) = (lr.re - ul.re, ul.im - lr.im);

    Complex {
        re: ul.re + (pixel.0 as f64 / bounds.0 as f64) * width,
        im: ul.im - (pixel.1 as f64 / bounds.1 as f64) * height,
    }
}

pub fn render(pixels: &mut [u8], bounds: (usize, usize), ul: Complex, lr: Complex) {
    let triplets = if pixels.len() == bounds.0 * bounds.1 {
        false
    } else if pixels.len() == bounds.0 * bounds.1 * 3 {
        true
    } else {
        panic!("pixels.len() != image size")
    };

    if triplets {
        (0..bounds.1)
            .cartesian_product(0..bounds.0) // number of rows (id of col)
            .map(|(row, column)| pixel_to_point(bounds, (column, row), ul, lr))
            .zip(pixels.chunks_exact_mut(3))
            .par_bridge()
            // .progress_count((bounds.0 * bounds.1 * 3) as u64)
            .for_each(|(point, pixel)| {
                let color = match escape_time(point) {
                    Some(count) => 255 - count,
                    None => 0,
                };
                (*pixel)[0] = color;
                (*pixel)[1] = color;
                (*pixel)[2] = color;
            })
    } else {
        (0..bounds.1)
            .cartesian_product(0..bounds.0) // number of rows (id of col)
            .map(|(row, column)| pixel_to_point(bounds, (column, row), ul, lr))
            .zip(pixels.iter_mut())
            .par_bridge()
            .progress_count((bounds.0 * bounds.1) as u64)
            .for_each(|(point, pixel)| {
                *pixel = match escape_time(point) {
                    Some(count) => 255 - count,
                    None => 0,
                }
            })
    }
}

pub(crate) fn write_image(
    filename: &str,
    pixels: &[u8],
    bounds: (usize, usize),
) -> Result<(), io::Error> {
    let output = File::create(filename)?;

    let encoder = PngEncoder::new(output);
    encoder
        .write_image(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::L8)
        .expect("couldn't write png image");

    Ok(())
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("", ','), None);
    assert_eq!(parse_pair::<i32>("10,", ','), None);
    assert_eq!(parse_pair::<i32>(",10", ','), None);
    assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x", 'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

#[test]
fn test_parse_complex() {
    assert_eq!(
        parse_complex("1.25,-0.0625"),
        Some(Complex {
            re: 1.25,
            im: -0.0625
        })
    );
    assert_eq!(parse_complex(",-0.0625"), None);
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(
        pixel_to_point(
            (100, 200),
            (25, 175),
            Complex { re: -1.0, im: 1.0 },
            Complex { re: 1.0, im: -1.0 }
        ),
        Complex {
            re: -0.5,
            im: -0.75
        }
    );
}
