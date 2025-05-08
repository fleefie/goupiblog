use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use markdown::CompileOptions;
use markdown::Constructs;
use markdown::Options;
use markdown::ParseOptions;

mod config;
mod template;

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

    fs::create_dir_all(&output_dir)?;

    let site_config = match config::load_config(&source_dir.join("site.toml")) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to load site configuration: {}", err);
            process::exit(1);
        }
    };

    for req_key in ["Site"] {
        if !site_config.contains_key(req_key) {
            eprintln!("Missing required '{req_key}' field in site.toml");
            process::exit(1);
        }
    }

    let site_resources = source_dir.join("res");
    if site_resources.exists() {
        let output_res = output_dir.join("res");
        fs::create_dir_all(&output_res)?;
        copy_directory(&site_resources, &output_res)?;
    }

    let prelude_path = source_dir.join("prelude.html");
    let prelude = match fs::read_to_string(&prelude_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Failed to read prelude.html: {}", err);
            process::exit(1);
        }
    };

    let posts_dir = source_dir.join("posts");
    if !posts_dir.exists() {
        eprintln!("Posts directory does not exist: {}", posts_dir.display());
        process::exit(1);
    }

    for entry in fs::read_dir(posts_dir)? {
        let entry = entry?;
        let post_dir = entry.path();

        if post_dir.is_dir() {
            process_post(&post_dir, &output_dir, &site_config, &prelude)?;
        }
    }

    println!("Site built successfully at {}", output_dir.display());
    Ok(())
}

fn process_post(
    post_dir: &Path,
    output_dir: &Path,
    site_config: &HashMap<String, toml::Value>,
    prelude: &str,
) -> io::Result<()> {
    let post_toml_path = post_dir.join("post.toml");
    let content_path = post_dir.join("content.md");

    if !post_toml_path.exists() || !content_path.exists() {
        return Ok(());
    }

    println!("Processing post: {}", post_dir.display());

    let post_config = match config::load_config(&post_toml_path) {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "Failed to load post configuration from {}: {}",
                post_toml_path.display(),
                err
            );
            return Ok(());
        }
    };

    for req_key in ["Title", "Description"] {
        if !post_config.contains_key(req_key) {
            eprintln!(
                "Missing required '{req_key}' field in {}",
                post_toml_path.display()
            );
            return Ok(()); // HACK: RETURNS OK INSTEAD OF AN ERROR!!!!
        }
    }

    let markdown_content = match fs::read_to_string(&content_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!(
                "Failed to read content from {}: {}",
                content_path.display(),
                err
            );
            return Ok(());
        }
    };

    // Transpile the markdown.
    // We need to talk about people calling transpilation compilation...
    let md_compile = CompileOptions {
        allow_dangerous_html: true,
        allow_dangerous_protocol: true,
        ..CompileOptions::default()
    };

    let md_constructs = Constructs {
        attention: true,
        autolink: true,
        definition: true,
        html_flow: true,
        html_text: true,
        math_flow: true,
        math_text: true,
        ..Constructs::gfm()
    };

    let md_parse = ParseOptions {
        constructs: md_constructs,
        gfm_strikethrough_single_tilde: true,
        math_text_single_dollar: true,
        mdx_esm_parse: None,
        mdx_expression_parse: None,
    };

    let md_options = Options {
        parse: md_parse,
        compile: md_compile,
    };

    let html_content = markdown::to_html_with_options(&markdown_content, &md_options).unwrap();

    let post_name = post_dir.file_name().unwrap().to_string_lossy();
    let post_output_dir = output_dir.join(&*post_name);
    fs::create_dir_all(&post_output_dir)?;

    match template::process_template(prelude, &post_config, site_config, &html_content) {
        Ok(processed) => {
            fs::write(post_output_dir.join("index.html"), processed)?;

            let post_resources = post_dir.join("res");
            if post_resources.exists() {
                let post_res_dir = post_output_dir.join("res");
                fs::create_dir_all(&post_res_dir)?;
                copy_directory(&post_resources, &post_res_dir)?;
            }

            println!("  Built post: {}", post_name);
        }
        Err(err) => {
            eprintln!("  Failed to process template for {}: {}", post_name, err);
        }
    }

    Ok(())
}

fn copy_directory(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_directory(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
