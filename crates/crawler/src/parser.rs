use scraper::{ElementRef, Html, Node, Selector};
use url::Url;
use std::collections::HashMap;

pub fn get_html_parser(html: &str) -> Html {
    Html::parse_document(html)
}

pub fn is_valid_url(url_info : &Url) -> bool {
    if (url_info.cannot_be_a_base() || url_info.scheme() != "http" || url_info.scheme() != "https") {
            return false;
    }

    let url_path = url_info.path();

    for ext in common::constants::SKIP_EXTENSIONS {
        if (url_path.ends_with(ext)) {
            return false;
        }
    }

    return true;
}

pub fn get_text_only(document: &Html) -> String {
    let root_document = document.root_element();

    let mut output_text = String::new();

    get_child_text(root_document, &mut output_text);

    return output_text;
}

fn get_child_text(element: ElementRef, output: &mut String) {
    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                output.push_str(text);
                output.push(' ');
            }

            Node::Element(_) => {
                if let Some(child_element) = ElementRef::wrap(child) {
                    let tag = child_element.value().name();

                    if tag == "script"
                        || tag == "style"
                        || tag == "head"
                        || tag == "meta"
                        || tag == "canvas"
                        || tag == "iframe"
                        || tag == "svg"
                        || tag == "noscript"
                        || tag == "template"
                    {
                        continue;
                    }

                    get_child_text(child_element, output);
                }
            }

            _ => {}
        }
    }
}

pub fn get_meta_data(document: &Html) -> HashMap<String, String> {
    let mut meta_data = HashMap::new();

    let title_selector = Selector::parse("title").unwrap();
    if let Some(title_element) = document.select(&title_selector).next() {
        let title_text = title_element.text().collect::<String>();
        meta_data.insert("title".to_string(), title_text);
    }

    let meta_selector = Selector::parse("meta").unwrap();

    for meta_element in document.select(&meta_selector) {
        let name = meta_element.value().attr("name").unwrap_or("");
        let content = meta_element.value().attr("content").unwrap_or("");

        if !name.is_empty() && !content.is_empty() {
            meta_data.insert(name.to_string(), content.to_string());
        }
    }

    return meta_data;
}

pub fn get_link_list(document: &Html, base_url: &str) -> Vec<String> {
    let a_tag_selector = Selector::parse("a").unwrap();
    let mut link_list = Vec::new();

    let base = url::Url::parse(base_url).unwrap();

    for a_tag in document.select(&a_tag_selector) {
        let href = a_tag.value().attr("href").unwrap_or("");

        if href == "" {
            continue;
        }

        let abs_url_handle = base.join(href);

        if let Ok(mut abs_url) = abs_url_handle {
            abs_url.set_query(None);
            abs_url.set_fragment(None);

            link_list.push(abs_url.to_string());
        }
    }

    return link_list;
}

pub fn arrange_count(words: Vec<String>) -> HashMap<String, i32> {
    let mut word_freq_count: HashMap<String, i32> = HashMap::new();
    for word in words {
        *word_freq_count.entry(word).or_insert(0) += 1;
    }

    return word_freq_count;
}

pub fn calculate_importance(document: &Html, word: &str) -> i32 {
    let mut importance = 0;

    let title_selector = Selector::parse("title").unwrap();
    if let Some(title_element) = document.select(&title_selector).next() {
        let text = title_element.text().collect::<String>();
        if text.contains(word) {
            importance += 10;
        }
    }

    let h1_selector = Selector::parse("h1").unwrap();
    let h1_elements = document.select(&h1_selector);
    for h1_element in h1_elements {
        let text = h1_element.text().collect::<String>();
        if text.contains(word) {
            importance += 8;
        }
    }

    let h2_selector = Selector::parse("h2").unwrap();
    let h2_elements = document.select(&h2_selector);
    for h2_element in h2_elements {
        let text = h2_element.text().collect::<String>();
        if text.contains(word) {
            importance += 5;
        }
    }

    let h3_selector = Selector::parse("h3").unwrap();
    let h3_elements = document.select(&h3_selector);
    for h3_element in h3_elements {
        let text = h3_element.text().collect::<String>();
        if text.contains(word) {
            importance += 3;
        }
    }

    let p_selector = Selector::parse("p").unwrap();
    let p_elements = document.select(&p_selector);
    for p_element in p_elements {
        let text = p_element.text().collect::<String>();
        if text.contains(word) {
            importance += 1;
        }
    }

    return importance;
}
