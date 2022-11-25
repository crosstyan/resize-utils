use clap::Parser;
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
    let possible_suffixes = vec![".jpg", ".jpeg", ".png", ".gif", ".bmp", ".tiff", ".webp"];
    let res = possible_suffixes
        .iter()
        .map(|s| entry.path().to_str().unwrap().ends_with(s))
        .any(|x| x);
    res
}

// should remove all the ng files
fn is_gif(entry: &DirEntry) -> bool {
    entry.path().to_str().unwrap().ends_with(".gif")
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The directory to search for pictures
    #[arg(short, long)]
    input_dir: String,
}

fn main() {
    let args = Args::parse();
    let input_dir = args.input_dir;
    // skip hidden
    let walker = WalkDir::new(input_dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .par_bridge();
    // you can't do filter things in filter entry
    walker.for_each(|e| {
        let entry = e.unwrap();
        if is_picture(&entry) {
            println!("{}", entry.path().display());
        }
    });
}
