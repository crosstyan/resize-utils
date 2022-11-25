use clap::Parser;
use indicatif::ParallelProgressIterator;
use magick_rust::MagickWand;
use rayon::prelude::*;
use walkdir::{DirEntry, WalkDir};

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
    // #[arg(short, long)]
    // output_dir: String,
    // #[arg(long, action=clap::ArgAction::SetTrue)]
    // inplace: bool,
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
    walker
        .par_iter()
        .progress_count(walker.len() as u64)
        .for_each(|entry| {
            println!("Found picture: {}", entry.path().display());
            let mut wand = MagickWand::new();
            wand.read_image(entry.path().to_str().unwrap()).unwrap();
            wand.set_image_format("png").unwrap();
            wand.write_image(entry.path().to_str().unwrap()).unwrap();
        });
}
