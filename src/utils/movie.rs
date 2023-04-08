use serde::{Serialize, Deserialize};
use std::{fmt, io};
use std::collections::HashMap;
use std::fs::File;
use tokio::time::{sleep, Duration};

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

#[derive(Serialize, Deserialize, Debug)]
struct Watch {
    #[serde(rename = "480")]
    url: String,
}

// The good response we want
#[derive(Serialize, Deserialize, Debug)]
struct Response {
    success: bool,
    watch: Watch,
}


impl fmt::Display for Movie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}


pub async fn download(movie: &Movie) -> Result<(), Box<dyn std::error::Error>> {
    let pre_watch = reqwest::get("https://api.sratim.tv/movie/preWatch")
        .await?
        .text()
        .await?;

    // Need to sleep cuz sratim.tv stuff
    println!("Sleeping for 30 sec");
    sleep(Duration::from_secs(30)).await;
    println!("Finished waiting");

    let req = reqwest::get(format!("https://api.sratim.tv/movie/watch/id/{}/token/{}", movie.id, pre_watch))
        .await?;

    let body: Response = match req.json().await {
        Ok(body) => body,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let movie_link = body.watch.url;
    let seret = reqwest::get(format!("https:{:?}", movie_link)).await?.text().await?;
    let mut out = File::create(format!("{}.mp4", movie.name)).expect("Failed to create file");
    io::copy(&mut seret.as_bytes(), &mut out).expect("Failed to download file");

    Ok(())
}
