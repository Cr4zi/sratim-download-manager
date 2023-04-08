use std::{io, collections::HashMap, fmt};
use serde::{Serialize, Deserialize};
use mongodb::Client;
use mongodb::bson::{doc, Document};
use futures::stream::TryStreamExt;
use question::{Question, Answer};

#[derive(Serialize, Deserialize, Debug)]
struct Movie {
    name: String,
    id: String,

}

// Used to deserialize response after searching
#[derive(Debug, Deserialize, Serialize)]
struct ApiResponse {
    success: bool,
    results: Vec<Movie>,
}

impl fmt::Display for Movie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}


/// Asynchronously searches for movies in the "Sratim" API and prompts the user to select one to add to the "Sratim" database.
///
/// # Arguments
///
/// *`client` - A reference to a reqwest::Client object, which represents an HTTP client used to make requests to the "Sratim" API.
///
/// * mongo_client - A reference to a Client object from the mongodb library, which represents a connection to a MongoDB server.
///
/// # Returns
///
/// * Ok(()) if the operation succeeds, or an error of type Box<dyn std::error::Error> if it fails.
///
/// # Examples
/// ```
/// use mongodb::{Client, error::Error};
/// use reqwest::Client as ReqwestClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
///     let mongo_client = Client::with_options(client_options)?;
///
///     search_for_existing_movie(&mongo_client).await?;
///
///     Ok(())
/// }
/// ```
pub async fn search_for_movies(client: &reqwest::Client, mongo_client: &Client) -> Result<(), Box<dyn std::error::Error>>{
    let mut name = String::new();

    println!("Enter a search name for a movie: ");

    io::stdin()
        .read_line(&mut name)
        .expect("Failed to read line");

    let data = HashMap::from([("term", &name)]);
    let search = client.post("https://api.sratim.tv/movie/search")
        .form(&data)
        .send()
        .await?;

    let res: ApiResponse = search.json().await?;
    let movies: Vec<Movie> = res.results;

    println!("Choose which movie you want to add to queue:");
    for i in 0..movies.len() {
        println!("[{}] {}", i+1, movies[i]);
    }
    println!("[{}] Abort", movies.len()+1);

    let mut choice = String::new();

    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read line");

    let choice: usize = choice.
        trim()
        .parse::<usize>()
        .expect("Failed to parse choice");

    if choice-1 == movies.len() {
        println!("Aborting...");
        return Ok(())
    }

    println!("Adding {} to queue", movies[choice-1]);

    let db = mongo_client.database("Sratim");
    let queue = db.collection::<Document>("Queue");

    queue.insert_one(doc! { "id": &movies[choice-1].id, "name": &movies[choice-1].name }, None).await?;

    Ok(())
}

/// Asynchronously searches for existing movies in the "Sratim" database and prints out their names.
///
/// # Arguments
///
/// * `mongo_client` - A reference to a Client object from the mongodb library,
/// which represents a connection to a MongoDB server.
///
/// # Returns
///
/// * Ok(()) if the operation succeeds, or an error of type Box<dyn std::error::Error> if it fails.
///
/// # Examples
///
/// ```
/// use mongodb::{Client, error::Error};
/// use reqwest::Client as ReqwestClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
///     let mongo_client = Client::with_options(client_options)?;
///
///     let client = ReqwestClient::new();
///
///     search_for_movies(&client, &mongo_client).await?;
///
///     Ok(())
/// }
/// ```
pub async fn search_for_existing_movie(mongo_client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let db = mongo_client.database("Sratim");
    let queue = db.collection::<Document>("Queue");

    println!("What movie do you want to search for: ");
    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .expect("Failed to read line");


    let filter = doc! { "name": { "$regex": format!(".*{}.*", name.trim()), "$options": "i"}};


    let mut cur = queue.find(filter, None).await?;
    while let Some(mv) = cur.try_next().await? {
        println!("{}", mv.get("name").unwrap());
    }


    Ok(())
}

pub async fn delete_movie() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}


/// Asynchronously prompts the user to confirm whether they want to delete all movies from the "Sratim" queue, and deletes all movies if the user confirms.
///
/// # Arguments
///
/// *`mongo_client` - A reference to a Client object from the mongodb library, which represents a connection to a MongoDB server.
///
/// # Returns
///
/// * `Ok(())` if the operation succeeds, or an error of type `Box<dyn std::error::Error>` if it fails.
///
/// # Examples
///
/// ```
/// use mongodb::Client;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
///     let mongo_client = Client::with_options(client_options)?;
///
///     empty_queue(&mongo_client).await?;
///
///     Ok(())
/// }
/// ```
pub async fn empty_queue(mongo_client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let ans = Question::new("Are you sure you want to empty queue? [y/n]").confirm();

    if ans == Answer::YES {
        println!("Deleteing queue");
        let db = mongo_client.database("Sratim");
        let queue = db.collection::<Document>("Queue");
        queue.delete_many(doc! {}, None).await?;
    } else {
        println!("Aborting...")
    }

        Ok(())
}
