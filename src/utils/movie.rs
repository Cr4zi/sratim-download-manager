use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::{fmt, io, thread, time};
use reqwest::header::{HeaderMap, HeaderValue};
use std::fs::File;
use chrono;

#[derive(Serialize, Deserialize, Debug)]
pub struct Movie {
    pub name: String,
    pub id: String,

}

impl Movie {
    pub fn get_image(&self) -> String {
        format!("https://static.sratim.tv/movies/{}.jpg", self.id)
    }
}


impl fmt::Display for Movie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}


pub async fn download(movie: &Movie) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let pre_watch = client.get("https://api.sratim.tv/movie/preWatch")
        .send()
        .await?;

    let mut cookie_headers = HeaderMap::new();

    if let Some(set_cookie_header) = pre_watch.headers().get("Set-Cookie") {
        let set_cookie_value = set_cookie_header.to_str().unwrap();
        cookie_headers.insert("Cookie", HeaderValue::from_str(set_cookie_value).unwrap());
    }

    if let Some(sratim_cookie_header) = pre_watch.headers().get("Sratim") {
        let sratim_cookie_value = sratim_cookie_header.to_str().unwrap();
        cookie_headers.insert("Cookie", HeaderValue::from_str(sratim_cookie_value).unwrap());
    }

    let pre_watch = pre_watch.text().await?;

    // Need to sleep cuz sratim.tv stuff
    println!("[ {:#?} ] Sleeping for 30 sec", chrono::offset::Local::now());

    let movie_id = movie.id.replace("\"", "");
    let url = format!("https://api.sratim.tv/movie/watch/id/{}/token/{}", movie_id, pre_watch);
    println!("{}", url);
    thread::sleep(time::Duration::from_secs(30));
    println!("[ {:#?} ]Finished waiting", chrono::offset::Local::now());


    let req = client.get(&url)
        .headers(cookie_headers)
        .send()
        .await?;

    let body: Value = req.json().await?;
    println!("body: {}", body);

    if body["success"].as_bool() == Some(true) {
        let movie_link = &body["watch"]["url"];
        let seret = reqwest::get(format!("https:{:?}", movie_link)).await?.text().await?;
        let mut out = File::create(format!("{}.mp4", movie.name)).expect("Failed to create file");
        io::copy(&mut seret.as_bytes(), &mut out).expect("Failed to download file");
        return Ok(())
    } else {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, body["errors"].as_array().unwrap()[0].to_string())))

    }

}
