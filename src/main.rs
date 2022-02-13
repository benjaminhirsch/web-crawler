use std::borrow::{Borrow, Cow};
use std::convert::Infallible;
#[allow(unused,unused_imports,dead_code)]
use std::env;
use std::os::linux::raw::stat;
//use parse_hyperlinks::iterator::Hyperlink;
//use parse_hyperlinks::parser::html::{html_text2dest, html_text2dest_link};
use reqwest::StatusCode;
use select::document::Document;
use select::predicate::Name;


#[derive(Debug)]
struct UrlsToParse {
    pub urls: Vec<Url>
}

#[derive(Debug)]
struct ParsedUrls {
    urls: Vec<Url>
}

#[derive(Debug)]
struct Url {
    url: String,
    status_code: u8,
}

impl From<String> for Url {
    fn from(url: String) -> Self {
        Url {
            url,
            status_code: 0
        }
    }
}

impl UrlsToParse {
    pub fn create() -> Self {
        UrlsToParse {
            urls: vec![]
        }
    }

    pub fn add(&mut self, url: Url) -> &Self {
        self.urls.push(url);
        self
    }

    pub fn remove(&mut self, index: usize) -> &Self {
        self.urls.remove(index);
        self
    }
}

impl ParsedUrls {
    pub fn create() -> Self {
        ParsedUrls {
            urls: vec![]
        }
    }

    pub fn add(&mut self, url: Url) -> &Self {
        self.urls.push(url);
        self
    }
}

#[tokio::main]
async fn main()  -> Result<(), Box<dyn std::error::Error>> {

    let args: Vec<String> = env::args().collect();
    println!("Starting to parse: {}", args[1]);

    let mut sites = UrlsToParse::create();
    sites.add(Url::from(args[1].clone()));

    while sites.urls.len() > 0 {
        let current_url = sites.urls.pop();
        match current_url {
            Some(ref actual_url) => {
                println!("Parsing: {}", actual_url.url);
                let response = reqwest::get(&actual_url.url).await?;

                let status = response.status().as_u16();

                if status >= 200 && status < 300 {
                    //println!("{:?}", response.text().await?);
                    let body_text = response.text().await?;

                    match Document::try_from(body_text.as_str()) {
                        Ok(document) => {
                            document.find(Name("a"))
                                .filter_map(|n| n.attr("href"))
                                .for_each(|x| println!("{}", x))
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

    //println!("{:#?}", sites);
    Ok(())
}
