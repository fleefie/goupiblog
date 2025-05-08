use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::io;
use toml::Value;

pub fn process_template(
    template: &str,
    post_config: &HashMap<String, Value>,
    site_config: &HashMap<String, Value>,
    content: &str,
) -> Result<String, io::Error> {
    let mut result = template.to_string();

    result = result.replace("<GoupiContent/>", content);

    // Date has some special handling for formatting
    if result.contains("<GoupiDate/>") {
        let current_local: DateTime<Local> = Local::now();
        let current_time = current_local.format("%Y-%m-%d %H:%M:%S").to_string();
        result = result.replace("<GoupiDate/>", &current_time);
    }

    let mut tags = Vec::new();
    let mut start_idx = 0;

    while let Some(tag_start) = result[start_idx..].find("<Goupi") {
        let abs_start = start_idx + tag_start;
        if let Some(tag_end) = result[abs_start..].find("/>") {
            let abs_end = abs_start + tag_end + 2; // +2 for "/>"
            let tag = &result[abs_start..abs_end];
            tags.push(tag.to_string());
            start_idx = abs_end;
        } else {
            start_idx += tag_start + 6; // Skip "<Goupi". Shouldn't happen but y'know
        }
    }

    for tag in tags {
        // Extract tag name (remove <Goupi and />)
        let tag_name = tag[6..tag.len() - 2].to_string();

        let value = if let Some(value) = post_config.get(&tag_name) {
            toml_value_to_string(value)
        } else if let Some(value) = site_config.get(&tag_name) {
            toml_value_to_string(value)
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Tag '{}' not found in post.toml or site.toml", tag_name),
            ));
        };

        result = result.replace(&tag, &value);
    }

    Ok(result)
}

fn toml_value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Datetime(dt) => dt.to_string(),
        Value::Array(_) => "[array]".to_string(), // Meh. That's for later.
        Value::Table(_) => "[table]".to_string(),
    }
}
