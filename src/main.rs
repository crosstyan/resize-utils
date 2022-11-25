use clap::Parser;
use indicatif::ParallelProgressIterator;
use magick_rust::{MagickWand, PixelWand};
use rayon::prelude::*;
use walkdir::{DirEntry, WalkDir};
use std::path::{Path, PathBuf};

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn is_picture(entry: &DirEntry) -> bool {
    let possible_suffixes = vec![".jpg", ".jpeg", ".png", ".bmp", ".tiff", ".webp"];
    let res = possible_suffixes
        .iter()
        .map(|s| entry.path().to_str().unwrap().ends_with(s))
        .any(|x| x);
    res
}

// should remove all the ng files
// I will handle them later
fn is_gif(entry: &DirEntry) -> bool {
    entry.path().to_str().unwrap().ends_with(".gif")
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The directory to search for pictures
    #[arg(short, long)]
    input_dir: String,
    #[arg(short, long, default_value_t = 90)]
    quality: usize,
    #[arg(short, long, default_value_t = 768)]
    length: usize,
    #[arg(long, default_value_t = true)]
    preserve_long_side: bool, // #[arg(short, long)]
                              // output_dir: String,
                              // #[arg(long, action=clap::ArgAction::SetTrue)]
                              // inplace: bool,
}

fn new_size(old_size: (usize, usize), new_l: usize, preserve_long_side: bool) -> (usize, usize) {
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
fn main() {
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
    let preserve_long_side = args.preserve_long_side;
    walker
        .par_iter()
        .progress_count(walker.len() as u64)
        .for_each(|entry| {
            // println!("Found picture: {}", entry.path().display());
            let mut wand = MagickWand::new();
            let mut white = PixelWand::new();
            white.set_color("white").unwrap();
            wand.read_image(entry.path().to_str().unwrap()).unwrap();
            wand.set_background_color(&white).unwrap();
            // https://imagemagick.org/api/MagickCore/image_8h.html
            // RemoveAlphaChannel
            // The author of bindings didn't expose this enum
            // I'll fix this later
            pub const AlphaChannelOption_RemoveAlphaChannel: u32 = 12;
            wand.set_image_alpha_channel(AlphaChannelOption_RemoveAlphaChannel)
                .unwrap();
            let w = wand.get_image_width();
            let h = wand.get_image_height();
            let (new_w, new_h) = new_size((w, h), length, preserve_long_side);
            pub const FilterType_LanczosFilter: u32 = 22;
            wand.resize_image(new_w, new_h, FilterType_LanczosFilter);
            wand.set_compression_quality(quality as usize).unwrap();
            wand.set_image_format("jpg").unwrap();
            let new_path = entry.path().with_extension("jpg");
            wand.write_image(new_path.to_str().unwrap()).unwrap();
        });
}
