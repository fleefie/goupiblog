use crate::config;
use crate::template::process_template;

use std::path::PathBuf;
use std::process;

use std::fs;
use std::io;
use std::path::Path;

use std::collections::HashMap;

use markdown::CompileOptions;
use markdown::Constructs;
use markdown::Options;
use markdown::ParseOptions;

use chrono::{DateTime, Local};

#[derive(Debug)]
#[allow(dead_code)]
pub enum PostBuildError {
    GeneralIOError(std::io::Error),
    TemplateBuildError(std::io::Error),
    MissingRequiredKey(String),
    PostFilesMissing,
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

struct PostInfo {
    name: String,
    title: String,
    description: String,
    timestamp_display: String,
    timestamp: i64,
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

    // Impure, but I'm lazy.
    let mut post_infos: Vec<PostInfo> = Vec::new();

    for entry in entries {
        let post_dir = match entry {
            Ok(p) => p.path(),
            Err(e) => return Err(SiteBuildError::GeneralIOError(e)),
        };

        if post_dir.is_dir() {
            match process_post(&post_dir, &output_dir, &site_config, &prelude) {
                Ok(post_info) => {
                    post_infos.push(post_info);
                }
                Err(e) => {
                    eprintln!("Failed to build post: {e:?}. Continuing...");
                    continue;
                }
            };
        }
    }

    // Sort posts by timestamps
    post_infos.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Make a simple posts index.
    // THIS IS DOGSHIT LMAO

    let mut posts_index = String::new();
    posts_index.push_str("<html><head><title>Posts</title></head><body>");
    posts_index.push_str("<h1>Posts</h1><ul>");
    for post in &post_infos {
        posts_index.push_str(&format!(
            "<li><a href=\"{}/index.html\">{}</a> - {} - {}</li>",
            post.name, post.title, post.description, post.timestamp_display
        ));
    }
    posts_index.push_str("</ul></body></html>");
    match fs::write(output_dir.join("index.html"), posts_index) {
        Ok(_) => (),
        Err(e) => return Err(SiteBuildError::GeneralIOError(e)),
    }

    Ok(())
}

fn process_post(
    post_dir: &Path,
    output_dir: &Path,
    site_config: &HashMap<String, toml::Value>,
    prelude: &str,
) -> Result<PostInfo, PostBuildError> {
    let post_toml_path = post_dir.join("post.toml");
    let content_path = post_dir.join("content.md");

    if !post_toml_path.exists() || !content_path.exists() {
        return Err(PostBuildError::PostFilesMissing);
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

    // Check if post is built. If it's good enough for GNU Make, it's good enough for me
    // This just uses timestamps IF YOU COULDN'T TELL
    let newest_post_timestamp = match find_newest_file(post_dir) {
        Ok(t) => t,
        Err(e) => return Err(PostBuildError::GeneralIOError(e)),
    };

    let newest_post_build_timestamp = match find_newest_file(post_output_dir.as_path()) {
        Ok(t) => t,
        Err(_) => std::time::SystemTime::UNIX_EPOCH,
    };

    // If already built, just get the existing post's TS
    if newest_post_build_timestamp > newest_post_timestamp {
        println!("  Skipping already built post: {}", post_name);

        // I feel dirty for using this. This feels like lib abuse.
        let ts: DateTime<Local> = DateTime::from(newest_post_build_timestamp);
        let ts_formatted = ts.format("%Y-%m-%d %H:%M:%S").to_string();

        // Guh
        return Ok(PostInfo {
            name: post_name.to_string(),
            title: post_config.get("Title").unwrap().to_string(),
            description: post_config.get("Description").unwrap().to_string(),
            timestamp_display: ts_formatted,
            timestamp: ts.timestamp(),
        });
    };

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

    let current_local: DateTime<Local> = Local::now();
    let current_time = current_local.format("%Y-%m-%d %H:%M:%S").to_string();

    Ok(PostInfo {
        // Unwrapping is fine here bc we error checked earlier
        name: post_name.to_string(),
        title: post_config.get("Title").unwrap().to_string(),
        description: post_config.get("Description").unwrap().to_string(),
        timestamp_display: current_time,
        timestamp: current_local.timestamp(),
    })
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

fn find_newest_file(dir: &Path) -> std::io::Result<std::time::SystemTime> {
    let mut newest_time = std::time::SystemTime::UNIX_EPOCH;

    if !dir.exists() {
        return Ok(newest_time);
    }

    fn visit_dir(dir: &Path, newest_time: &mut std::time::SystemTime) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                visit_dir(&path, newest_time)?;
            } else if let Ok(metadata) = entry.metadata() {
                if let Ok(modified_time) = metadata.modified() {
                    if modified_time > *newest_time {
                        *newest_time = modified_time;
                    }
                }
            }
        }
        Ok(())
    }

    visit_dir(dir, &mut newest_time)?;
    Ok(newest_time)
}
