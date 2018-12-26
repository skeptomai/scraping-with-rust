extern crate feed_rs;
extern crate reqwest;
extern crate select;
extern crate tempfile;

use std::fs::File;
use std::io::Result;
use std::thread;

fn main() {
    hacker_news("http://feeds.nightvalepresents.com/welcometonightvalepodcast");
}

fn hacker_news(url: &str) {
    let mut resp = reqwest::get(url).unwrap();
    assert!(resp.status().is_success());
    let parsed = feed_rs::parser::parse(&mut resp).unwrap();

    println!(
        "title: {:#?}, number of entries: {}, description: {:#?}",
        parsed.title.unwrap(),
        parsed.entries.len(),
        parsed.description.unwrap()
    );
    /* for e in parsed.entries {
        for enc in e.enclosure {
            println!("{:?}", enc.href);
        }

    } */

    let mp3s: Vec<String> = parsed
        .entries
        .into_iter()
        .map(|e| e.enclosure[0].href.clone())
        .collect();

    let handler = thread::spawn(move || {
        fetch_mp3(mp3s);
    });

    handler.join().unwrap();
}

fn internal_fetch(mp3s: Vec<String>) -> Result<u64> {
    let dir = tempfile::tempdir()?;
    let mut response = reqwest::get(&mp3s[0]).unwrap();
    assert!(response.status().is_success());

    let fname = {
        dir.path().join(
            response
                .url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .unwrap_or("tmp.mp3"),
        )
    };

    println!("File name: {}", fname.to_str().unwrap());
    let mut f = File::create(fname)?;
    let bytes_written: u64 = response.copy_to(&mut f).unwrap();
    Ok(bytes_written)
}

fn fetch_mp3(mp3s: Vec<String>) {
    let u = internal_fetch(mp3s).unwrap();
    println!("{:#?}", &u);
}
