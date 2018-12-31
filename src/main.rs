extern crate feed_rs;
extern crate num_cpus;
extern crate reqwest;
extern crate select;
extern crate tempfile;

use std::fs::File;
use std::io::{Error, ErrorKind, Result};
use std::thread;

#[derive(Debug)]
struct MP3ToFetch {
    mp3_url: String,
}

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

    let mp3s: Vec<MP3ToFetch> = parsed
        .entries
        .into_iter()
        .map(|e| MP3ToFetch {
            mp3_url: e.enclosure[0].href.clone(),
        })
        .collect();

    println!("{}", num_cpus::get());

    let handler = thread::spawn(move || {
        fetch_mp3(mp3s);
    });

    handler.join().unwrap();
}

fn get_file_name(response: &reqwest::Response) -> Result<(tempfile::TempDir, String)> {
    let dir = tempfile::tempdir()?;
    let local_name = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.mp3");

    let fname = dir.path().join(local_name).to_str().unwrap().to_string();
    println!("fname is {:#}", fname);
    Ok((dir, fname))
}

fn internal_fetch(mp3: &str) -> Result<u64> {
    println!("internal_fetch would fetch {}", mp3);

    match reqwest::get(mp3) {
        Ok(mut response) => {
            let (_tempdir, fname) = get_file_name(&response)?;
            println!("internal_fetch would write {}", fname);
            let mut f = match File::create(fname) {
                Ok(f) => f,
                Err(e) => {
                    println!("File::create failed {:#?}", e);
                    return Err(Error::new(ErrorKind::Other, e));
                }
            };
            println!("f is {:#?}", f);
            let bytes_written: u64 = match response.copy_to(&mut f) {
                Ok(f) => f,
                Err(e) => {
                    println!("copy_to failed {:#?}", e);
                    return Err(Error::new(ErrorKind::Other, e));
                }
            };
            println!("Wrote {:#} bytes", bytes_written);
            Ok(bytes_written)
        }
        Err(e) => {
            println!("internal_fetch failed: {:#?}", e);
            return Err(Error::new(ErrorKind::Other, e));
        }
    }
}

fn fetch_mp3(mp3s: Vec<MP3ToFetch>) {
    for mp3 in mp3s {
        match internal_fetch(&mp3.mp3_url) {
            Ok(bytes_written) => println!("Wrote: {}", bytes_written),
            Err(e) => {
                println!("Cypress Hill, ya'll fucked up! : {:#?}", e);
                std::process::exit(-1)
            }
        }
    }
}
