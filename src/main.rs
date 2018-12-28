extern crate feed_rs;
extern crate reqwest;
extern crate select;
extern crate tempfile;

use std::fs::File;
use std::io::{Error, ErrorKind, Result};
use std::thread;

fn main() {
    hacker_news("http://feeds.nightvalepresents.com/welcometonightvalepodcast");
}

fn hacker_news(url: &str) {
    let mut resp = reqwest::get(url).unwrap();
    assert!(resp.status().is_success());
    let parsed = feed_rs::parser::parse(&mut resp).unwrap();
    /*
    println!(
        "title: {:#?}, number of entries: {}, description: {:#?}",
        parsed.title.unwrap(),
        parsed.entries.len(),
        parsed.description.unwrap()
    );
    for e in parsed.entries {
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

fn get_file_name(response: &reqwest::Response) -> Result<String> {
    let dir = tempfile::tempdir()?;
    let local_name = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.mp3");

    Ok(dir.path().join(local_name).to_str().unwrap().to_string())
}

fn internal_fetch(mp3: &str) -> Result<u64> {
    println!("internal_fetch would fetch {}", mp3);

    match reqwest::get(mp3) {
        Ok(mut response) => {
            let fname = get_file_name(&response)?;
            println!("internal_fetch would write {}", fname);
            let mut f = File::create(fname)?;
            let bytes_written: u64 = response.copy_to(&mut f).unwrap();
            println!("Wrote {:#} bytes", bytes_written);
            Ok(bytes_written)
        }
        Err(e) => return Err(Error::new(ErrorKind::Other, e)),
    }
}

fn fetch_mp3(mp3s: Vec<String>) {
    for mp3 in mp3s {
        let _u = internal_fetch(&mp3).unwrap();
    }
}
