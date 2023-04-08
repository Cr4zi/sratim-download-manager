use mongodb::Client;
use mongodb::bson::{doc, Document};
use futures::stream::TryStreamExt;

use crate::utils::movie;

pub async fn download_movies(mongo_client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let db = mongo_client.database("Sratim");
    let downloaded = db.collection::<Document>("Downloaded");
    let queue = db.collection::<Document>("Queue");

    let mut cur = queue.find(None, None).await?;
    while let Some(mv) = cur.try_next().await? {
        let mv = movie::Movie { name: mv.get("name").unwrap().to_string(), id: mv.get("id").unwrap().to_string() };

        println!("Downloading {}", &mv.name);
        movie::download(&mv).await?;
        println!("Finished to download movie");

        let add = doc! {
            "name": &mv.name,
            "id": &mv.id,
            "img": &mv.get_image()
        };
        downloaded.insert_one(add, None).await?;

        let del = doc! {
            "name": &mv.name,
            "id": &mv.id
        };
        queue.delete_one(del, None).await?;
    }

    Ok(())
}
