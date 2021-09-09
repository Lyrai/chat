mod chat;
mod user;

use crate::chat::Message;
use mongodb::{
    bson,
    bson::doc,
    options::{ClientOptions, FindOptions},
    Client, Collection,
};
use rocket::response::stream::{ByteStream, Event, EventStream};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::tokio::time::{self, Duration};
use rocket::{
    futures::TryStreamExt, http::Status, response::status, tokio, Build, Rocket, Shutdown, State,
};
use serde::{Deserialize, Serialize};
use serde_json as json;
use std::error::Error;
use std::sync::Arc;
use user::{User, Users};

macro_rules! log_id {
    ($db: ident) => {
        $db.database("users").collection::<User>("log_id")
    };
}

#[macro_use]
extern crate rocket;

#[launch]
async fn launch() -> Rocket<Build> {
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
        .manage(channel::<Message>(1024).0)
        .mount("/", routes![register, get_users, connect, send, login])
}

#[get("/connect")]
async fn connect(user: User, queue: &State<Sender<Message>>, mut end: Shutdown) -> ByteStream![Vec<u8>] {
    let mut rx = queue.subscribe();
    let id = user.id;
    ByteStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) if msg.id == id as u8 => continue,
                    Ok(msg) => msg.bytes,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield msg;
        }
    }
}

#[post("/send", data = "<msg>")]
fn send(msg: Message, queue: &State<Sender<Message>>) {
    queue.send(msg);
}

#[post("/register")]
async fn register(
    info: User,
    db_client: &State<Client>,
    users: &State<Users>,
) -> status::Custom<String> {
    let db = log_id!(db_client);
    let found = db.find_one(doc! {"login": info.login.clone()}, None).await;

    if let Ok(Some(_)) = found {
        return status::Custom(
            Status::Conflict,
            format!("User with login {} already exists", info.login),
        );
    } else if let Err(e) = found {
        eprintln!("Registration error: {:?}", e);
        return status::Custom(
            Status::BadGateway,
            "Error occurred during registration".to_owned(),
        );
    }

    let id = db.count_documents(None, None).await.unwrap();

    let new = User {
        login: info.login,
        id: id as i32,
    };

    let insert_result = db.insert_one(new.clone(), None).await;

    if let Err(e) = insert_result {
        eprintln!("Error adding db: {:?}", e);
        return status::Custom(
            Status::BadGateway,
            "Error occurred while adding to db".to_owned(),
        );
    }

    users.add(new.clone());
    status::Custom(Status::Accepted, json::to_string(&new).unwrap())
}

#[post("/login")]
async fn login(login: User, db_client: &State<Client>) -> String {
    let db = log_id!(db_client);
    let found = db
        .find_one(doc! {"login": login.login.clone()}, None)
        .await
        .unwrap()
        .unwrap();

    json::to_string(&found).unwrap()
}

#[get("/get_users")]
async fn get_users(db_client: &State<Client>) -> String {
    let result = get_users_internal(db_client).await;
    if let Err(e) = result {
        eprintln!("Error retrieving users: {:?}", e);
        return "Internal server error".to_owned();
    }

    json::to_string(&result.unwrap().users).unwrap()
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