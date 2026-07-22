use crate::constants;
use waken_snowball::{Algorithm, stem};

pub fn get_stemmer_word(word: &str, language: Algorithm) -> String {
    let stemmed = stem(language, word);

    return stemmed.to_string();
}

pub fn find_language(text: &str) -> Algorithm {
    let language = whichlang::detect_language(text);

    return constants::language_to_algorithm(language).unwrap_or(Algorithm::English);
}

pub fn remove_unneeded_words(html_text: &str) -> String {
    let stop_words = constants::get_stop_words();
    let mut result = html_text.to_string();

    let re_patterns = vec![
        ".", ",", "!", "?", ";", ":", "(", ")", "[", "]", "{", "}", "\"", "'", "-", "_", "/", "\\",
        "@", "#", "$", "%", "^", "&", "*", "+", "=", "<", ">", "|", "~", "`",
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

pub fn tokonize(html_text: &str, language: Algorithm) -> Vec<String> {
    let language = Algorithm::English;
    println!("---------------------------------");
    println!("{:?}", get_stemmer_word("run", language));
    println!("{:?}", get_stemmer_word("running", language));
    println!("---------------------------------");

    return html_text
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| get_stemmer_word(s.to_lowercase().as_str(), language))
        .collect();
}
