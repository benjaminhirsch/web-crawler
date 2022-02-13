#[allow(unused,unused_imports,dead_code)]
use std::env;
use select::document::Document;
use select::predicate::Name;


#[derive(Debug)]
struct UrlsToParse {
    pub urls: Vec<String>
}

#[derive(Debug)]
struct ParsedUrls {
    urls: Vec<ParsedUrl>
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
            body
        }
    }
}

#[derive(Debug)]
struct ParsedUrl {
    url: String,
    status_code: u16,
}


impl UrlsToParse {
    pub fn create() -> Self {
        UrlsToParse {
            urls: vec![]
        }
    }

    pub fn add(&mut self, url: String) -> &Self {
        self.urls.push(url);
        self
    }

    pub fn has(&mut self, url: &String) -> bool {
        self.urls.iter().any(|u| u == url)
    }
}

impl ParsedUrls {
    pub fn create() -> Self {
        ParsedUrls {
            urls: vec![]
        }
    }

    pub fn add(&mut self, url: ParsedUrl) -> &Self {
        self.urls.push(url);
        self
    }

    pub fn has(&mut self, url: &String) -> bool {
        self.urls.iter().any(|u| u.url == url.to_string())
    }
}

impl From<&Url> for ParsedUrl {
    fn from(url: &Url) -> Self {
        ParsedUrl {
            url: url.url.to_string(),
            status_code: url.status_code
        }
    }
}

#[tokio::main]
async fn main()  -> Result<(), Box<dyn std::error::Error>> {

    let args: Vec<String> = env::args().collect();
    println!("Starting to parse: {}", args[1]);

    let mut sites = UrlsToParse::create();
    let mut sites_processed = ParsedUrls::create();

    sites.add(args[1].to_string());
    let domain = args[1].clone();

    while sites.urls.len() > 0 {
        let current_url = sites.urls.pop();
        match current_url {
            Some(actual_url) => {
                //println!("Parsing: {}", actual_url);
                let response = reqwest::get(&actual_url).await?;
                let is_success = response.status().is_success();
                let status_code = response.status().as_u16();
                let body_text = response.text().await?;

                let url = &Url::create(
                    actual_url,
                    status_code,
                    body_text
                );
                sites_processed.add(ParsedUrl::from(url));
                println!("Processed  {} of {} sites", sites_processed.urls.len(), sites.urls.len());

                if is_success {
                    match Document::try_from(url.body.as_str()) {
                        Ok(document) => {
                            document.find(Name("a"))
                                //.filter(|n| !vec!["#", ""].iter().any(|&nv| nv == n.attr("href")))
                                .filter_map(|n| n.attr("href"))
                                .for_each(|x| {
                                    // We need to add checks here, to prevent parsing errors because of invalid urls
                                    let normalized_url = normalize_url(x, &domain);
                                    if is_valid_url(&normalized_url, &domain) && (!sites_processed.has(&normalized_url) && !sites.has(&normalized_url)) {
                                        sites.add(normalized_url);
                                    }
                                })
                        }
                        Err(_) => {
                            println!("Unable to parse node...")
                        }
                    }

                        /*.unwrap()
                        .find(Name("a"))
                        .filter_map(|n| n.attr("href"))
                        .for_each(|x| println!("{}", x));*/


                    //println!("{}", body_text);
                    //let links_found = Hyperlink::new(body_text.as_str(), false);
                    //let links_found = html_text2dest_link(&body_text.as_str());
                    //for link in links_found {
                    //    println!("{:?}",link);
                    //}

                    //println!("{:?}", links_found);


                    //let all_links = links_found.into_iter().map(|&link| =>  ).collect();
                    //for link in html_text2dest_link(links_found) {

                    //}
                }
            },
            None => println!("{}", "Something went wrong, execution is continuing...")
        }
    }

    println!("Finished, checked {} sites", sites_processed.urls.len());
    Ok(())
}

fn is_valid_url(url: &str, domain: &String) -> bool {
    url.starts_with(domain) && !url.starts_with("#") && !url.starts_with("tel")
}

fn normalize_url(url: &str, domain: &String) -> String {
    match url.chars().nth(0) {
        Some(string) => {
            return if string.to_string() == "/" {
                domain.to_string() + &url[1..url.len()]
            } else {
                url.to_string()
            }
        },
        None => url.to_string()
    }
    //if url.chars().nth(0).unwrap().to_string() == "/" {
        //return domain.to_string() + &url[1..url.len()]
    //}


}