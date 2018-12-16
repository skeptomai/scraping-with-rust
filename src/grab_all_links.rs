extern crate reqwest;
extern crate select;
extern crate feed_rs;
extern crate atom_syndication;

use select::document::Document;
use select::predicate::Name;
use atom_syndication::Feed;
use std::io::BufReader;

fn main() {
    hacker_news("http://feeds.nightvalepresents.com/welcometonightvalepodcast");
    // hacker_news("https://news.ycombinator.com");
}

fn hacker_news(url: &str) {

    let mut resp = reqwest::get(url).unwrap();
    assert!(resp.status().is_success());
    //let parsed = Feed::read_from(BufReader::new(resp));
    //println!("{:?}", parsed );
    
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
