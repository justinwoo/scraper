use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Url;
use soup::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(PartialEq, Eq, Debug)]
struct Target {
    href: String,
    title: String,
}

type BannedWords = Vec<String>;

// banned titles should not be used
fn is_not_banned(banned_words: &BannedWords, title: &String) -> bool {
    for word in banned_words.iter() {
        if title.contains(word) {
            return false;
        }
    }

    return true;
}

// get targets out of some html string
fn get_targets_from_html(banned_words: BannedWords, html: &str) -> Vec<Target> {
    let soup = Soup::new(html);

    let tds = soup.tag("td").find_all();
    let anchors = tds.flat_map(|td| td.tag("a").find_all());

    anchors
        .filter_map(|node| {
            // make sure the title in the correct format and follows the rules
            lazy_static! {
                static ref TITLE_REGEX: Regex =
                    Regex::new(r#"\[\w+\] .+ - \d+ \[\d+p\]\.mkv"#).unwrap();
            }

            let title = node.get("title")?;
            let href = node.get("href")?;

            let has_valid_title =
                TITLE_REGEX.is_match(&title) && is_not_banned(&banned_words, &title);

            if !has_valid_title {
                return None;
            }

            Some(Target { href, title })
        })
        .collect()
}

fn filter_existing<'r>(target: &'r &Target) -> bool {
    let title = &target.title;
    let existing_test1 = format!("downloads/{}.torrent", title);
    let existing_test2 = format!("downloads/{}.torrent.added", title);

    if Path::new(&existing_test1).exists() {
        return false;
    }

    if Path::new(&existing_test2).exists() {
        return false;
    }

    return true;
}

fn main() {
    // ensure downloads dir
    let _ = fs::create_dir("downloads");

    let banned_words: Vec<String> = {
        match std::env::var("BANNED_WORDS") {
            Ok(words) => words.split(",").map(|s| s.to_string()).collect(),
            Err(_) => vec![],
        }
    };

    let scraper_url = std::env::var("SCRAPER_URL").expect("SCRAPER_URL was not set");
    let url = Url::parse(&scraper_url).expect("SCRAPER_URL did not parse correctly");
    let host = url
        .host_str()
        .expect("SCRAPER_URL did not have proper host url segment (???)")
        .to_string();

    let results_page_html = reqwest::get(url)
        .expect("Could not get page from SCRAPER_URL")
        .text()
        .expect("Could not extract text from request");

    let fetched_targets: Vec<Target> = get_targets_from_html(banned_words, &results_page_html);

    let targets = fetched_targets.iter().filter(filter_existing);

    let mut has_items = false;

    for target in targets {
        has_items = true;

        let path = format!("downloads/{}.torrent", &target.title);
        let link = format!("{}{}/torrent", host, &target.href);

        let status_code = Command::new("curl")
            .arg("-L")
            .arg(&link)
            .arg("-o")
            .arg(&path)
            .status()
            .expect("Curl failed to launch");

        if !status_code.success() {
            println!("Error downloading {} from {}", path, link);
        }

        println!("downloaded {}", path);
    }

    if !has_items {
        println!("nothing new to download");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_test() {
        let test_html = r#"
          <table>
            <td><a href="/view/item1" title="[crap] hello - 01 [720p].mkv">content1</a></td>
            <td><a href="/view/item2" title="[crap] hello - 02 [720p].mkv">content2</a></td>
            <td><a href="/view/item3" title="[crap] hello - 03 [720p].mkv">content3</a></td>
            <td><a href="/view/item3" title="[crap] hello! - 04 [720p].mkv">content3</a></td>
            <td><a href="/view/item4" title="[crap] banned - 01 [720p].mkv">content3</a></td>
            <td><a href="/view/item5" title="unknown pattern">content3</a></td>
            <td><a href="/view/item5" title="[crap] wrong format">content3</a></td>
          </table>
        "#;

        let targets = get_targets_from_html(vec!["banned".to_string()], test_html);

        assert_eq!(
            targets,
            vec![
                Target {
                    href: "/view/item1".to_string(),
                    title: "[crap] hello - 01 [720p].mkv".to_string()
                },
                Target {
                    href: "/view/item2".to_string(),
                    title: "[crap] hello - 02 [720p].mkv".to_string()
                },
                Target {
                    href: "/view/item3".to_string(),
                    title: "[crap] hello - 03 [720p].mkv".to_string()
                },
                Target {
                    href: "/view/item3".to_string(),
                    title: "[crap] hello! - 04 [720p].mkv".to_string()
                }
            ]
        );
    }
}
