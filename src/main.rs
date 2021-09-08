mod chat;
mod user;

use chat::Chat;
use mongodb::{
    bson,
    bson::doc,
    options::{ClientOptions, FindOptions},
    Client,
    Collection
};
use rocket::{futures::TryStreamExt, http::Status, response::status, tokio, Build, Rocket, State};
use std::sync::Arc;
use user::{User, Users};
use serde_json as json;
use serde::Serialize;
use rocket::serde::Serializer;
use std::borrow::Borrow;
use std::error::Error;

macro_rules! log_id {
    ($db: ident) => {
        $db.database("users").collection::<User>("log_id")
    };
}

#[macro_use]
extern crate rocket;

#[launch]
async fn launch() -> Rocket<Build> {
    start_chat();
    let mut db_client_options =
        ClientOptions::parse("mongodb+srv://server:jRk3JcqhsOXsT4RC@cluster0.us09s.mongodb.net/myFirstDatabase?retryWrites=true&w=majority")
            .await
            .unwrap();

    db_client_options.app_name = Some("Chat".to_string());
    let db_client = Client::with_options(db_client_options).unwrap();
    let users = get_users_internal(&db_client).await.unwrap();

    rocket::build()
        .manage(users)
        .manage(db_client)
        .mount("/", routes![register, get_users])
}

#[post("/register")]
async fn register(info: User, db_client: &State<Client>, users: &State<Users>) -> status::Custom<String> {
    let db = log_id!(db_client);
    let found = db.find_one(doc! {"login": info.login.clone()}, None).await;

    if let Ok(Some(_)) = found {
        return status::Custom(Status::Conflict, format!("User with login {} already exists", info.login));
    } else if let Err(e) = found {
        eprintln!("Registration error: {:?}", e);
        return status::Custom(Status::BadGateway, "Error occurred during registration".to_owned());
    }

    let id = db.count_documents(None, None).await.unwrap();

    let new = User {
        login: info.login,
        id: id as i32
    };

    let insert_result = db.insert_one(new.clone(), None).await;

    if let Err(e) = insert_result {
        eprintln!("Error adding db: {:?}", e);
        return status::Custom(Status::BadGateway, "Error occurred while adding to db".to_owned())
    }

    users.add(new);
    status::Custom(Status::Accepted, "Success".to_owned())
}

#[get("/get_users")]
async fn get_users(db_client: &State<Client>) -> String {
    let result = get_users_internal(db_client).await;
    if let Err(e) = result {
        eprintln!("Error retrieving users: {:?}", e);
        return "Internal server error".to_owned();
    }

    json::to_string(&result.unwrap()).unwrap()
}

async fn get_users_internal(db_client: &Client) -> Result<Users, Box<dyn Error>> {
    let db = log_id!(db_client);

    let mut users = db.find(None, None).await?;
    let result = Users::new();
    while let Some(user) = users.try_next().await? {
        result.add(user);
    }

    Ok(result)
}

#[get("/get/<id>")]
fn get_user_by_id(id: i32, users: &State<Users>) -> Option<User> {
    users.get(id)
}

fn start_chat() {
    tokio::spawn(async {
        //let port = std::env::var("PORT").unwrap();
        let listener = tokio::net::TcpListener::bind("0.0.0.0:1488")
            .await
            .unwrap();
        let chat = Arc::new(Chat::new());

        loop {
            let (stream, _) = listener.accept().await.unwrap();
            println!("Connected");
            Chat::add(chat.clone(), stream).await;
        }
    });
}
