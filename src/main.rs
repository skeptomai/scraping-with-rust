extern crate reqwest;
extern crate select;
extern crate feed_rs;
extern crate tempfile;

use std::thread;
use std::io::copy;
use std::fs::File;

fn main() {
    hacker_news("http://feeds.nightvalepresents.com/welcometonightvalepodcast");
}

fn hacker_news(url: &str) {

    let mut resp = reqwest::get(url).unwrap();
    assert!(resp.status().is_success());
    let parsed = feed_rs::parser::parse(&mut resp).unwrap();
    
    println!("title: {:#?}, number of entries: {}, description: {:#?}", parsed.title.unwrap(), parsed.entries.len(), parsed.description.unwrap() );
    /* for e in parsed.entries {
        for enc in e.enclosure {
            println!("{:?}", enc.href);
        }

    } */

    let mp3s : Vec<_> = parsed.entries
        .into_iter()
        .map(|e| e.enclosure[0].href.clone())
        .collect();

    let handler = thread::spawn(move || {
        fetch_mp3();
          
    });

    handler.join().unwrap();
}

fn fetch_mp3() -> Result<_> {
    let dir = tempfile::tempdir()?;
        let mut response = reqwest::get(&mp3s[0]).unwrap();
        assert!(response.status().is_success());

        let fname = response
                .url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .unwrap_or("tmp.bin");
        let fname = dir.path().join(fname);
      /*   let mut f = match File::create(fname){
            
        }

        std::io::copy(&mut response, &mut dest); */
}
