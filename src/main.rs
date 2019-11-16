#[macro_use]
extern crate log;
extern crate env_logger;
extern crate scoped_threadpool;
extern crate tempfile;

use log::{debug, error, info};
use scoped_threadpool::Pool;
use std::env;
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use tempfile::Builder;

const WELCOME_TO_NIGHTVALE: &str = "http://feeds.nightvalepresents.com/welcometonightvalepodcast";
const DIRECTORY_PREFIX: &str = "nightscrape-";

#[derive(Debug, Clone)]
struct MP3ToFetch<'s> {
    mp3_path: &'s str,
    mp3_url: String,
}

impl<'s> MP3ToFetch<'s> {
    pub fn fetch_mp3(&self) {
        match self.internal_fetch() {
            Ok(bytes_written) => info!("Wrote: {}", bytes_written),
            Err(e) => {
                error!("Cypress Hill, ya'll fucked up! : {:#?}", e);
                std::process::exit(-1)
            }
        }
    }

    fn internal_fetch(&self) -> Result<u64> {
        let mp3 = self.mp3_url.clone();
        debug!("internal_fetch would fetch {}", mp3);

        match reqwest::get(mp3.as_str()) {
            Ok(mut response) => {
                let fname = self.get_file_name(&response)?;
                info!("internal_fetch would write {}", fname);
                let mut f = fs::File::create(fname.as_str())?;
                info!("f is {:#?}", f);

                let bytes_written: u64 = match response.copy_to(&mut f) {
                    Ok(f) => f,
                    Err(e) => {
                        error!("copy_to failed {:#?}", e);
                        return Err(Error::new(ErrorKind::Other, e));
                    }
                };
                info!("Wrote {:#} bytes", bytes_written);
                Ok(bytes_written)
            }
            Err(e) => {
                error!("internal_fetch failed: {:#?}", e);
                return Err(Error::new(ErrorKind::Other, e));
            }
        }
    }

    fn get_file_name(&self, response: &reqwest::Response) -> Result<String> {
        let local_name = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) });

        if let Some(name) = local_name {
            if let Some(f) = Path::new(self.mp3_path).join(name).to_str() {
                return Ok(f.to_string());
            }
        }

        Err(Error::new(ErrorKind::Other, "Failed to get filename"))
    }
}

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let dest_dir = &args[1];
    nightscrape(WELCOME_TO_NIGHTVALE, dest_dir);
}

fn fetch(mp3s: &[MP3ToFetch]) {
    let mut pool = Pool::new(num_cpus::get() as u32);
    pool.scoped(|scoped| {
        for mp3 in mp3s {
            scoped.execute(move || {
                mp3.fetch_mp3();
            })
        }
    })
}

fn nightscrape(url: &str, dest_dir: &str) {
    let mut resp = reqwest::get(url).unwrap();
    assert!(resp.status().is_success());
    let parsed = feed_rs::parser::parse(&mut resp).unwrap();

    let dir = Builder::new().prefix(DIRECTORY_PREFIX).tempdir().unwrap();
    // into_path takes ownership and keeps temp directory from being reaped at exit
    let d = dir.into_path();
    let pb = d.to_str().unwrap();
    info!("Path is {:#?}", pb);
    info!("Destination is {:#?}", dest_dir);

    match fs::metadata(dest_dir) {
        Ok(md) => {
            if md.is_dir() {
                panic!("Dest dir {:#?} exists!", dest_dir);
            }
        }
        Err(_) => {}
    }

    let mp3s: Vec<MP3ToFetch> = parsed
        .entries
        .into_iter()
        .map(|e| MP3ToFetch {
            mp3_path: pb,
            mp3_url: e.enclosure[0].href.clone(),
        })
        .collect();

    fetch(&mp3s[0..3]);

    match fs::rename(pb, dest_dir) {
        Ok(_) => println!("renaming {:#?} to {:#?}", pb, dest_dir),
        Err(e) => error!("failed rename #{:#?}", e),
    };
}
