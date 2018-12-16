extern crate reqwest;
extern crate select;
extern crate feed_rs;

use select::document::Document;
use select::predicate::Name;

fn main() {
    hacker_news("http://feeds.nightvalepresents.com/welcometonightvalepodcast");
}

fn hacker_news(url: &str) {

    let mut resp = reqwest::get(url).unwrap();
    assert!(resp.status().is_success());
    
    let parsed = feed_rs::parser::parse(&mut resp).unwrap();
    println!("{:?}", parsed );
    /*
    Document::from_read(resp)
        .unwrap()
        .find(Name("enclosure"))
        .filter_map(|n| n.attr("url"))
        .for_each(|x| println!("{}", x));
        */
}
