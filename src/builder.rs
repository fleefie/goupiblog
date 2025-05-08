use crate::config;
use crate::template::process_template;

use std::path::PathBuf;
use std::process;

use std::fs;
use std::fs::copy;
use std::io;
use std::path::Path;

use std::collections::HashMap;

use markdown::CompileOptions;
use markdown::Constructs;
use markdown::Options;
use markdown::ParseOptions;

#[derive(Debug)]
#[allow(dead_code)]
pub enum PostBuildError {
    GeneralIOError(std::io::Error),
    TemplateBuildError(std::io::Error),
    MissingRequiredKey(String),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum SiteBuildError {
    CannotLoadConfig(std::io::Error),
    CannotLoadPrelude(std::io::Error),
    CannotLoadPosts(String),
    GeneralIOError(std::io::Error),
    MissingRequiredKey(String),
}

pub fn build_site(source_dir: &PathBuf, output_dir: &PathBuf) -> Result<(), SiteBuildError> {
    match fs::create_dir_all(&output_dir) {
        Ok(_) => (),
        Err(e) => return Err(SiteBuildError::GeneralIOError(e)),
    };

    let site_config = match config::load_config(&source_dir.join("site.toml")) {
        Ok(config) => config,
        Err(e) => return Err(SiteBuildError::CannotLoadConfig(e)),
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
        match fs::create_dir_all(&output_res) {
            Ok(_) => (),
            Err(e) => return Err(SiteBuildError::GeneralIOError(e)),
        };

        match copy_directory(&site_resources, &output_res) {
            Ok(_) => (),
            Err(e) => return Err(SiteBuildError::GeneralIOError(e)),
        };
    }

    let site_index = source_dir.join("index.html");
    if site_index.exists() {
        match copy(&site_index, &output_dir.join("index.html")) {
            Ok(_) => (),
            Err(e) => return Err(SiteBuildError::GeneralIOError(e)),
        }
    }

    let prelude_path = source_dir.join("prelude.html");
    let prelude = match fs::read_to_string(&prelude_path) {
        Ok(content) => content,
        Err(err) => {
            return Err(SiteBuildError::CannotLoadPrelude(err));
        }
    };

    let posts_dir = source_dir.join("posts");
    if !posts_dir.exists() {
        return Err(SiteBuildError::CannotLoadPosts(
            "Posts directory does not exist".to_string(),
        ));
    }

    let entries = match fs::read_dir(posts_dir) {
        Ok(v) => v,
        Err(e) => return Err(SiteBuildError::GeneralIOError(e)),
    };

    for entry in entries {
        let post_dir = match entry {
            Ok(p) => p.path(),
            Err(e) => return Err(SiteBuildError::GeneralIOError(e)),
        };

        if post_dir.is_dir() {
            match process_post(&post_dir, &output_dir, &site_config, &prelude) {
                Ok(_) => continue,
                Err(e) => {
                    eprintln!("Failed to build post: {e:?}. Continuing...");
                    continue;
                }
            };
        }
    }

    Ok(())
}

fn process_post(
    post_dir: &Path,
    output_dir: &Path,
    site_config: &HashMap<String, toml::Value>,
    prelude: &str,
) -> Result<(), PostBuildError> {
    let post_toml_path = post_dir.join("post.toml");
    let content_path = post_dir.join("content.md");

    if !post_toml_path.exists() || !content_path.exists() {
        return Ok(());
    }

    println!("Processing post: {}", post_dir.display());

    let post_config = match config::load_config(&post_toml_path) {
        Ok(config) => config,
        Err(e) => {
            return Err(PostBuildError::GeneralIOError(e));
        }
    };

    for req_key in ["Title", "Description"] {
        if !post_config.contains_key(req_key) {
            return Err(PostBuildError::MissingRequiredKey(req_key.to_string()));
        }
    }

    let markdown_content = match fs::read_to_string(&content_path) {
        Ok(content) => content,
        Err(err) => return Err(PostBuildError::GeneralIOError(err)),
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
    match fs::create_dir_all(&post_output_dir) {
        Ok(_) => (),
        Err(e) => return Err(PostBuildError::GeneralIOError(e)),
    };

    match process_template(prelude, &post_config, site_config, &html_content) {
        Ok(processed) => {
            match fs::write(post_output_dir.join("index.html"), processed) {
                Ok(_) => (),
                Err(e) => return Err(PostBuildError::GeneralIOError(e)),
            }
            let post_resources = post_dir.join("res");
            if post_resources.exists() {
                let post_res_dir = post_output_dir.join("res");

                match fs::create_dir_all(&post_res_dir) {
                    Ok(_) => (),
                    Err(e) => return Err(PostBuildError::GeneralIOError(e)),
                };

                match copy_directory(&post_resources, &post_res_dir) {
                    Ok(_) => (),
                    Err(e) => return Err(PostBuildError::GeneralIOError(e)),
                }
            }

            println!("  Built post: {}", post_name);
        }
        Err(e) => return Err(PostBuildError::TemplateBuildError(e)),
    }

    Ok(())
}

pub fn copy_directory(src: &Path, dst: &Path) -> io::Result<()> {
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
