use image::GenericImageView;
mod blockdb;
use blockdb::BlockDb;
use std::fs::{self, DirEntry};
use indicatif::{ProgressBar};
use std::convert::TryInto;
use rayon::prelude::*;
use argh::FromArgs;

#[derive(FromArgs)]
/// Builds a collage with images from "./input/*"
struct Args {
    #[argh(positional)]
    target: String,

    /// size of collage snippets
    #[argh(option, default = "32")]
    size: u32,
}

fn main() {
    let args: Args = argh::from_env();
    let size = args.size;
    let input = find_input_images();

    if input.len() < 1 {
        eprintln!("No input images");
        return;
    }

    let bar = ProgressBar::new(input.len() as u64);
    let imgs: Vec<image::RgbImage> = input.iter().filter_map(|p| {
        let i = image::open(p).map(|i| i.into_rgb8()).ok();
        bar.inc(1);
        i
    }).collect();
    bar.finish_and_clear();
    let sub_imgs = imgs.iter().flat_map(
        |img| {
            let (width, height) = img.dimensions();
            let mut imgs = Vec::new();
            for x in (0..width - size).step_by(size.try_into().unwrap()) {
                for y in (0..height - size).step_by(size.try_into().unwrap()) {
                    imgs.push(img.view(x, y, size, size));
                }
            }
            return imgs;
        }).collect();

    let bldb = BlockDb::new(sub_imgs, |img| avg_color(img).into());

    let img2 = image::open(args.target.clone())
        .unwrap()
        .into_rgb8();
    let (width, height) = img2.dimensions();
    let mut out_img: image::RgbImage = image::ImageBuffer::new(width, height);

    let coords: Vec<(u32, u32)> = (0..width - size).step_by(size.try_into().unwrap()).flat_map( |x| {
        (0..height - size).step_by(size.try_into().unwrap()).map(move |y| (x,y))
    }).collect();

    let bar = ProgressBar::new(coords.len().try_into().unwrap());

    let replacements: Vec<(u32, u32, &image::SubImage<&image::RgbImage>)> = coords.into_par_iter().map(|(x,y)| {
        let avg = avg_color(&img2.view(x, y, size, size));
        let new_block = bldb.find_closest_pos(avg.into()).unwrap();
        bar.inc(1);
        (x,y, new_block)
    }).collect();
    bar.finish_and_clear();
    for (x,y, blk) in replacements {
        image::imageops::replace(&mut out_img, blk, x, y);
    }

    out_img.save("out.png").unwrap();
}

fn find_input_images() -> Vec<std::path::PathBuf>
{
 fs::read_dir("input")
        .unwrap()
        .filter_map(|p| p.ok())
        .map(|p| p.path())
        .filter(|p| p.extension().map_or(false, |e| e ==  "jpg"))
        .collect()
}

#[derive(Debug)]
struct Pos {
    r: u64,
    g: u64,
    b: u64,
}

impl From<Pos> for [i16; 3] {
    fn from(p: Pos) -> Self {
        [p.r as i16, p.g as i16, p.b as i16]
    }
}

fn avg_color(img: &image::SubImage<&image::RgbImage>) -> Pos {
    let mut out = Pos { r: 0, g: 0, b: 0 };

    let mut count = 0;
    for p in img.pixels().map(|(_, _, p)| p) {
        count += 1;
        let (r, g, b) = (p[0], p[1], p[2]);
        out.r += r as u64;
        out.g += g as u64;
        out.b += b as u64;
    }

    out.r /= count;
    out.g /= count;
    out.b /= count;

    return out;
}
