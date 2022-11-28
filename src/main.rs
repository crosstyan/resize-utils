use anyhow::{bail, Context, Result};
use clap::{ArgAction, Parser};
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageFormat, RgbImage};
use indicatif::ParallelProgressIterator;
use num::Num;
use rayon::prelude::*;
use walkdir::{DirEntry, WalkDir};

fn check_is_need_modify(img: &DynamicImage, format: ImageFormat, expected_len: u32) -> bool {
    let (width, height) = img.dimensions();
    let format_criteria = format == ImageFormat::Jpeg;
    let size_criteria = width > expected_len && height > expected_len;
    let is_need_modify = !format_criteria || !size_criteria;
    is_need_modify
}

fn handle_img(entry: &DirEntry) -> Result<()> {
    let img = image::io::Reader::open(entry.path())?;
    let img_format = img.format().context("get image format error")?;
    let mut dyn_img = img.decode()?;
    let expcted_len = 768;
    if check_is_need_modify(&dyn_img, img_format, expcted_len) {
        let (w, h) = dyn_img.dimensions();
        let (new_width, new_height) = new_size((w, h), expcted_len, true);
        let resized =
            dyn_img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);
        let mut out_path = entry.path().to_path_buf();
        out_path.set_extension("jpg");
        // img_buf.save_with_format(out_path, ImageFormat::Jpeg)?;
    } else {
        return Ok(());
    }
    return Ok(());
}

/// should not handle the image if it is already small enough and it is a jpeg
// fn check_is_need_modify(img: &MagickWand, expected_l: usize) -> bool {
//     let pic_format = img.get_image_format().unwrap().to_lowercase();
//     let format_criteria = (pic_format == "jpeg") || (pic_format == "jpg");
//     let w = img.get_image_width();
//     let h = img.get_image_height();
//     let size_criteria = w > expected_l && h > expected_l;
//     !format_criteria || size_criteria
// }

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn is_picture(entry: &DirEntry) -> bool {
    let possible_suffixes = vec!["jpg", "jpeg", "png", "bmp", "tiff", "webp"];
    let res = possible_suffixes
        .iter()
        .map(|s| {
            let empty = std::ffi::OsStr::new("");
            entry.path().extension().unwrap_or(&empty) == *s
        })
        .any(|x| x);
    res
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The directory to search for pictures
    #[arg(short, long)]
    input_dir: String,
    /// JPEG compression quality
    #[arg(short, long, default_value_t = 90)]
    quality: usize,
    /// The length you desire
    #[arg(short, long, default_value_t = 768)]
    length: usize,
    /// flag to indicate whether preserve the long side.
    /// for example, for images with width > height, if this flag is set
    /// i.e. not preserve the long side, the width will be the exact length you set.
    #[arg(long, action=ArgAction::SetTrue, default_value_t = false)]
    no_preserve_long_side: bool,
    // #[arg(short, long)]
    // output_dir: String,
    // #[arg(long, action=clap::ArgAction::SetTrue)]
    // inplace: bool,
}

fn new_size<N: Num + PartialOrd + Copy>(
    old_size: (N, N),
    new_l: N,
    preserve_long_side: bool,
) -> (N, N) {
    let (old_w, old_h) = old_size;
    if old_w < new_l || old_h < new_l {
        return old_size;
    }
    if preserve_long_side {
        if old_w > old_h {
            let new_h = new_l;
            let new_w = new_l * old_w / old_h;
            return (new_w, new_h);
        } else {
            let new_w = new_l;
            let new_h = new_l * old_h / old_w;
            return (new_w, new_h);
        }
    } else {
        if old_w > old_h {
            let new_w = new_l;
            let new_h = new_l * old_h / old_w;
            return (new_w, new_h);
        } else {
            let new_h = new_l;
            let new_w = new_l * old_w / old_h;
            return (new_w, new_h);
        }
    }
}

// How the rayon works for reference
// https://github.com/rayon-rs/rayon/blob/master/FAQ.md
fn main() -> Result<()> {
    let args = Args::parse();
    let input_dir = args.input_dir;
    // skip hidden folder
    let walker: Vec<_> = WalkDir::new(input_dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .filter(|e| is_picture(e))
        .collect();
    let quality = args.quality;
    let length = args.length;
    let is_not_preserve_long_side = args.no_preserve_long_side;
    println!("length {}", walker.len());
    walker
        .par_iter()
        .progress_count(walker.len() as u64)
        .for_each(|entry| {
            match handle_img(entry) {
                Ok(_) => (),
                Err(e) => println!("error: {}", e),
            }
            // println!("Found picture: {}", entry.path().display());
            // let mut wand = MagickWand::new();
            // match wand.read_image(entry.path().to_str().unwrap()){
            //     Ok(_) => {
            //         if check_is_need_modify(&wand, length){
            //             let mut white = PixelWand::new();
            //             white.set_color("white").unwrap();
            //             wand.set_background_color(&white).unwrap();
            //             // https://imagemagick.org/api/MagickCore/image_8h.html
            //             // RemoveAlphaChannel
            //             // The author of bindings didn't expose this enum
            //             // I'll fix this later
            //             pub const AlphaChannelOption_RemoveAlphaChannel: u32 = 12;
            //             wand.set_image_alpha_channel(AlphaChannelOption_RemoveAlphaChannel)
            //                 .unwrap();
            //             let w = wand.get_image_width();
            //             let h = wand.get_image_height();
            //             let (new_w, new_h) = new_size((w, h), length, !is_not_preserve_long_side);
            //             pub const FilterType_LanczosFilter: u32 = 22;
            //             wand.resize_image(new_w, new_h, FilterType_LanczosFilter);
            //             wand.set_compression_quality(quality as usize).unwrap();
            //             wand.set_image_format("jpeg").unwrap();
            //             let new_path = entry.path().with_extension("jpg");
            //             wand.write_image(new_path.to_str().unwrap()).unwrap();
            //             // remove the original file
            //             if entry.path().extension().unwrap() != "jpg" {
            //                 std::fs::remove_file(entry.path()).unwrap();
            //             }
            //         } else {
            //             // println!("skip {}", entry.path().display());
            //         }
            //     },
            //     Err(_) => {
            //         println!("Error reading image: {}", entry.path().display());
            //     }
            // }
        });
    return Ok(());
}
