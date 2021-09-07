mod chat;
mod user;

use chat::Chat;
use mongodb::{
    bson::doc,
    options::{ClientOptions, FindOptions},
    Client,
};
use rocket::{futures::TryStreamExt, http::Status, response::status, tokio, Build, Rocket, State};
use std::sync::Arc;
use user::{User, Users};
use rocket::response::content::Json;

#[macro_use]
extern crate rocket;

#[launch]
async fn launch() -> Rocket<Build> {
    start_chat();
    let mut db_client_options =
        ClientOptions::parse("mongodb+srv://admin:BsJf03UnsmgQnXeW@cluster0.us09s.mongodb.net/myFirstDatabase?retryWrites=true&w=majority").await.unwrap();

    db_client_options.app_name = Some("Chat".to_string());
    let db_client = Client::with_options(db_client_options).unwrap();

    rocket::build()
        .manage(Users::new())
        .manage(db_client)
        .mount("/", routes![register, get_users])
}

#[post("/register")]
fn register(info: User, users: &State<Client>) -> status::Custom<&'static str> {
    //if let Err(e) = users.add(info) {
    //    eprintln!("User {} already exists", e);
    //    return status::Custom(Status::BadRequest, "User with given login is already registered");
    //}
    //
    //println!("Registered {}", info.login);
    status::Custom(Status::Accepted, "")
}

#[get("/get_users")]
async fn get_users(db_client: &State<Client>) -> String {
    let db = db_client.database("users").collection::<User>("log_id");
    let mut users = db.find(doc! {}, FindOptions::default()).await.unwrap();
    let result = Users::new();
    while let Some(user) = users.try_next().await.unwrap() {
        result.add(user);
    }

    serde_json::to_string(&result).unwrap()
}

fn start_chat() {
    tokio::spawn(async {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:1488")
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
