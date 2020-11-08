use anyhow::{anyhow, bail, ensure};
use clap::{App, Arg};
use itertools::Itertools;
use std::{convert::TryFrom, fs};

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2019-8")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let image_layers_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");

    let image_layers = parse_input(&image_layers_str, 25, 6)?;

    ensure!(!image_layers.is_empty(), "Input image is empty");

    let min_black_layer = image_layers
        .iter()
        .min_by_key(|&l| count_pixel_type(l, Pixel::Black))
        .unwrap();

    println!(
        "Image checksum: {}",
        count_pixel_type(min_black_layer, Pixel::White)
            * count_pixel_type(min_black_layer, Pixel::Transparent)
    );

    let image = decode_image_layers(&image_layers);

    render_image(&image)?;

    Ok(())
}

fn render_image(image: &ImageLayer) -> Result<(), anyhow::Error> {
    for row in image {
        for pixel in row {
            use Pixel::*;

            print!(
                "{}",
                match pixel {
                    Black => '█',
                    White => ' ',
                    Transparent => bail!("Found transparent pixel in image"),
                }
            );
        }

        println!();
    }

    Ok(())
}

fn decode_image_layers(image_layers: &[ImageLayer]) -> ImageLayer {
    let (width, height) = (image_layers[0][0].len(), image_layers[0].len());

    let mut image = vec![vec![Pixel::Transparent; width]; height];

    for layer in image_layers {
        for (row_idx, row) in layer.into_iter().enumerate() {
            for (pixel_idx, &pixel) in row.into_iter().enumerate() {
                use Pixel::*;

                image[row_idx][pixel_idx] = match (image[row_idx][pixel_idx], pixel) {
                    (Black, _) => Black,
                    (White, _) => White,
                    (Transparent, new_pixel) => new_pixel,
                };
            }
        }
    }

    image
}

fn count_pixel_type(layer: &ImageLayer, pixel_type: Pixel) -> usize {
    layer
        .iter()
        .flat_map(|row| row.iter())
        .filter(|&p| p == &pixel_type)
        .count()
}

fn parse_input(
    image_layers_str: &str,
    width: usize,
    height: usize,
) -> Result<Vec<ImageLayer>, anyhow::Error> {
    image_layers_str
        .trim()
        .chars()
        .map(|c| {
            let digit = c
                .to_digit(10)
                .map(|d| d as u8)
                .ok_or_else(|| anyhow!("Could not parse {} into digit", c))?;

            Pixel::try_from(digit)
        })
        .chunks(width)
        .into_iter()
        .map(|c| c.try_collect())
        .chunks(height)
        .into_iter()
        .map(|c| c.try_collect())
        .try_collect()
}

type ImageLayer = Vec<Vec<Pixel>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Pixel {
    Black,
    White,
    Transparent,
}

impl TryFrom<u8> for Pixel {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Black,
            1 => Self::White,
            2 => Self::Transparent,
            _ => bail!("Unknown pixel value: {}", value),
        })
    }
}
