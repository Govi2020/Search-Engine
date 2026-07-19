use crate::constants;


pub fn remove_unneeded_words(html_text: &str) -> String {
    let stop_words = constants::get_stop_words();
    let mut result   = html_text.to_string();

    let re_patterns = vec![
        ".", ",", "!", "?", ";", ":", "(", ")", "[", "]", "{", "}", "\"", "'", "-", "_", "/",
        "\\", "@", "#", "$", "%", "^", "&", "*", "+", "=", "<", ">", "|", "~", "`",
    ];

    for pattern in re_patterns {
        result = result.replace(pattern, "");
    }

    let words: Vec<&str> = result.split_whitespace().collect();
    let filtered: Vec<&str> = words
        .into_iter()
        .filter(|w| !stop_words.contains(*w))
        .collect();

    return filtered.join(" ");
}

pub fn tokonize(html_text: &str) -> Vec<String> {
    return html_text
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect();
}
