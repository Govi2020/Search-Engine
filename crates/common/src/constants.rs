use std::collections::HashSet;

pub const CUSTOM_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

// pub const TITLE_SELECTOR: &str = "title";
// pub const TITLE_ELEMENT: &str = "title";
// pub const TITLE_CASE: &str = "title";
// pub const TITLE_CASE_HEADERS: &str = "title";

// pub const H1_SELECTOR: &str = "h1";
// pub const H2_SELECTOR: &str = "h2";
// pub const H3_SELECTOR: &str = "h3";
// pub const P_SELECTOR: &str = "p";
// pub const A_SELECTOR: &str = "a";
// pub const META_SELECTOR: &str = "meta";

pub const SKIP_EXTENSIONS: &[&str] = &[
    "pdf", "jpg", "jpeg", "png", "gif", "svg", "webp",
    "mp3", "wav", "mp4", "avi", "mkv", "mov",
    "zip", "rar", "7z", "tar", "gz",
    "exe", "msi", "apk", "deb", "rpm",
    "doc", "docx", "xls", "xlsx", "ppt", "pptx",
    "ttf", "woff", "woff2", "ico",
];


pub fn get_stop_words() -> HashSet<&'static str> {
    let mut stop_words = HashSet::new();
    let words = [
        "the", "be", "to", "of", "and", "a", "in", "that", "have", "i", "it", "for", "not", "on",
        "with", "he", "as", "you", "do", "at", "this", "but", "his", "by", "from", "they", "we",
        "say", "her", "she", "or", "an", "will", "my", "one", "all", "would", "there", "their",
        "what", "so", "up", "out", "if", "about", "who", "get", "which", "go", "me", "when",
        "make", "can", "like", "time", "no", "just", "him", "know", "take", "people", "into",
        "year", "your", "good", "some", "could", "them", "see", "other", "than", "then", "now",
        "look", "only", "come", "its", "over", "think", "also", "back", "after", "use", "two",
        "how", "our", "work", "first", "well", "way", "even", "new", "want", "because", "any",
        "these", "give", "day", "most", "us", "is", "am", "are", "was", "were", "been", "being",
        "has", "had", "did", "does", "done", "should", "may", "might", "must", "shall", "need",
        "very", "much", "more", "such", "each", "own", "same", "still", "too",
    ];
    for word in words {
        stop_words.insert(word);
    }
    stop_words
}
