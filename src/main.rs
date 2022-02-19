#[allow(unused, unused_imports, dead_code)]
use cli_table::format::Justify;
use cli_table::{print_stdout, Cell, Style, Table, TableStruct};
use reqwest::Response;
use select::document::Document;
use select::predicate::Name;
use std::env;
use std::fmt::{Display, Formatter};
use std::io::{stdout, Write};

#[derive(Debug)]
struct UrlsToParse {
    pub urls: Vec<String>,
    pub domain: String,
}

#[derive(Debug)]
struct ParsedUrls {
    urls: Vec<ParsedUrl>,
}

#[derive(Debug)]
struct Url {
    url: String,
    status_code: u16,
    body: String,
}

impl Url {
    pub fn create(url: String, status_code: u16, body: String) -> Self {
        Url {
            url,
            status_code,
            body,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct ParsedUrl {
    url: String,
    status_code: u16,
}

impl UrlsToParse {
    pub fn create(domain: String) -> Self {
        UrlsToParse {
            urls: vec![],
            domain,
        }
    }

    pub fn is_valid_url(&self, url: &str) -> bool {
        url.starts_with(&self.domain.to_string())
            && !url.starts_with('#')
            && !url.starts_with("tel")
    }

    pub fn normalize_url(&self, url: &str) -> String {
        match url.chars().next() {
            Some(string) => {
                if string.to_string() == "/" {
                    self.domain.to_string() + &url[1..url.len()]
                } else {
                    url.to_string()
                }
            }
            None => url.to_string(),
        }
    }

    pub fn add(&mut self, url: String) {
        self.urls.push(url);
    }

    pub fn has(&self, url: &str) -> bool {
        self.urls.iter().any(|u| u == url)
    }
}

impl ParsedUrls {
    pub fn create() -> Self {
        ParsedUrls { urls: vec![] }
    }

    pub fn add(&mut self, url: ParsedUrl) -> &mut Self {
        self.urls.push(url);
        self
    }

    pub fn has(&self, url: &str) -> bool {
        self.urls.iter().any(|u| u.url == *url)
    }
}

impl Display for ParsedUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

impl From<&Url> for ParsedUrl {
    fn from(url: &Url) -> Self {
        ParsedUrl {
            url: url.url.to_string(),
            status_code: url.status_code,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    println!("Starting to parse: {}", args[1]);

    let mut domain = args[1].to_string();

    // Add missing trailing slash at the end if necessary
    if !domain.ends_with('/') {
        domain += "/";
    }

    let mut urls_to_parse = UrlsToParse::create(domain);
    let mut parsed_urls = ParsedUrls::create();

    urls_to_parse.add(urls_to_parse.normalize_url("/"));

    while !urls_to_parse.urls.is_empty() {
        let current_url = urls_to_parse.urls.pop();
        match current_url {
            Some(actual_url) => {
                let response = reqwest::get(&actual_url).await?;
                let status_code = response.status().as_u16();

                // Should be configurable - skipping files
                if !is_site(&response) {
                    continue;
                }

                let body_text = response.text().await?;
                let url = &Url::create(actual_url, status_code, body_text);
                parsed_urls.add(ParsedUrl::from(url));

                if !parsed_urls.urls.is_empty() && !urls_to_parse.urls.is_empty() {
                    print!(
                        "\rProcessed {} sites, in queue {} ",
                        parsed_urls.urls.len(),
                        urls_to_parse.urls.len()
                    );
                    stdout().flush().unwrap();
                }

                let doc = Document::try_from(url.body.as_str());

                if let Ok(document) = doc {
                    let nodes = document.find(Name("a"));
                    for node in nodes {
                        if node.attr("href").is_some() {
                            let url = node.attr("href").unwrap().to_string();
                            let normalized_url = urls_to_parse.normalize_url(url.as_str());
                            if urls_to_parse.is_valid_url(&normalized_url)
                                && not_already_processed(
                                    &normalized_url,
                                    &parsed_urls,
                                    &urls_to_parse,
                                )
                            {
                                urls_to_parse.add(normalized_url.clone().to_string());
                            }
                        }
                    }
                }
            }
            None => println!("Something went wrong, execution is continuing..."),
        }
    }

    println!(
        "\n\nFinished check! Scanned {} sites.\n",
        parsed_urls.urls.len()
    );
    if let Some(t) = calculate_summary(&parsed_urls) {
        print_stdout(t)?;
    };

    for x in parsed_urls.urls {
        println!("{}", x);
    }

    Ok(())
}

fn not_already_processed(url: &str, parsed_urls: &ParsedUrls, urls_to_parse: &UrlsToParse) -> bool {
    !parsed_urls.has(url) && !urls_to_parse.has(url)
}

fn is_site(response: &Response) -> bool {
    match response.headers().get("content-type") {
        Some(header) => match header.to_str() {
            Ok(h) => h.contains("text/html"),
            _ => false,
        },
        _ => false,
    }
}

fn calculate_summary(parsed_urls: &ParsedUrls) -> Option<TableStruct> {
    let mut summary = [0, 0, 0, 0];

    for u in &parsed_urls.urls {
        if u.status_code >= 200 && u.status_code < 300 {
            summary[0] += 1;
        } else if u.status_code >= 300 && u.status_code < 400 {
            summary[1] += 1;
        } else if u.status_code >= 400 && u.status_code < 500 {
            summary[2] += 1;
        } else {
            summary[3] += 1;
        }
    }

    let data = vec![vec![
        "Total".cell(),
        summary[0].cell(),
        summary[1].cell(),
        summary[2].cell(),
        summary[3].cell().justify(Justify::Right),
    ]];

    Some(data.table().title(vec![
        "HTTP status code".cell().bold(true),
        "2xx".cell().bold(true),
        "3xx".cell().bold(true),
        "4xx".cell().bold(true),
        "5xx".cell().bold(true),
    ]))
}
