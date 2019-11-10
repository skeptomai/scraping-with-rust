extern crate feed_rs;
extern crate num_cpus;
extern crate reqwest;
extern crate scoped_threadpool;
extern crate select;
extern crate tempfile;

use scoped_threadpool::Pool;
use std::fs::File;
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, Clone)]
struct MP3ToFetch {
    mp3_url: String,
}

impl MP3ToFetch {
    pub fn fetch_mp3(&self) {
        match self.internal_fetch() {
            Ok(bytes_written) => println!("Wrote: {}", bytes_written),
            Err(e) => {
                println!("Cypress Hill, ya'll fucked up! : {:#?}", e);
                std::process::exit(-1)
            }
        }
    }

    fn internal_fetch(&self) -> Result<u64> {
        let mp3 = self.mp3_url.clone();
        println!("internal_fetch would fetch {}", mp3);

        match reqwest::get(mp3.as_str()) {
            Ok(mut response) => {
                let (_tempdir, fname) = self.get_file_name(&response)?;
                println!("internal_fetch would write {}", fname);
                let mut f = File::create(fname.as_str())?;
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

    fn get_file_name(&self, response: &reqwest::Response) -> Result<(tempfile::TempDir, String)> {
        let dir = tempfile::tempdir()?;
        let local_name = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) });

        if let Some(name) = local_name {
            if let Some(f) = dir.path().join(name).to_str() {
                return Ok((dir, f.to_string()));
            }
        }

        Err(Error::new(ErrorKind::Other, "Failed to get filename"))
    }
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

    let cpus = num_cpus::get();

    let mut pool = Pool::new(cpus as u32);

    pool.scoped(|scoped| {
        for mp3 in mp3s {
            scoped.execute(move || {
                mp3.fetch_mp3();
            })
        }
    })
}
