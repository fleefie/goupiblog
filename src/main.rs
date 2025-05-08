use std::env;
use std::io;
use std::path::PathBuf;

mod builder;
mod config;
mod template;

use crate::builder::build_site;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let source_dir = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./source"));

    let output_dir = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./output"));

    let _ = build_site(&source_dir, &output_dir);

    println!("Site built successfully at {}", output_dir.display());
    Ok(())
}
