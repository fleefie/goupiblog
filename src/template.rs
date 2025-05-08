use std::collections::HashMap;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};
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
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let date_str = format_date(now);
        result = result.replace("<GoupiDate/>", &date_str);
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

// This fucking sucks and I need to create a proper crate for it.
fn format_date(timestamp: u64) -> String {
    let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let days_since_epoch = timestamp / (24 * 60 * 60);
    let weekday = days[(days_since_epoch % 7) as usize];

    let years = 1970 + (days_since_epoch / 365);
    let days_in_year = days_since_epoch % 365;

    let mut month = 0;
    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut days_left = days_in_year;

    for (i, &days) in days_in_month.iter().enumerate() {
        if days_left < days {
            month = i;
            break;
        }
        days_left -= days;
    }

    let day = days_left + 1;

    format!("{}, {} {} {}", weekday, day, months[month], years)
}
