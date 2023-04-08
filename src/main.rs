use mongodb::{Client, options::ClientOptions};
use std::{process, io, env};

mod cli;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("please use `cargo run <cli/download>`");
        process::exit(1);

    }

    if args[1] == "cli"{
        loop {
            println!("Please choose which option you want to use");
            println!("[1] Search for movie and add to queue.");
            println!("[2] Search for existing movie");
            println!("[3] Delete an existing movie");
            println!("[4] Empty queue");
            println!("[5] exit");

            let client_options = ClientOptions::parse("mongodb://192.168.1.55").await?;
            let mongo_client = Client::with_options(client_options)?;

            let mut choice = String::new();

            io::stdin()
                .read_line(&mut choice)
                .expect("Failed to read line");

            let choice = choice.trim().parse::<u8>().expect("Failed to parse into u8");
            match choice {
                1 => cli::search_for_movies(&client, &mongo_client).await?,
                2 => cli::search_for_existing_movie(&mongo_client).await?,
                3 => cli::delete_movie().await?,
                4 => cli::empty_queue(&mongo_client).await?,
                _ => break,
        }

        println!();
        println!("---------------------------------");
        println!();

        }
    } else if args[1] == "downloader" {
        // Downloader stuff
        println!("...");
    } else {
        println!("please use `cargo run <cli/downloader>`");
    }


    Ok(())
}
