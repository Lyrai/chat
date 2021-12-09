mod chat;
mod user;
mod auth;
mod macros;
mod prelude;

use crate::chat::Message;
use crate::user::{User, Users};
use mongodb::{
    bson::doc,
    options::ClientOptions,
    Client
};
use rocket::response::{status, stream::ByteStream, content::Json};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::{futures::TryStreamExt, http::Status, Build, Rocket, Shutdown, State};
use serde_json as json;
use std::error::Error;
use crate::auth::LoginData;
use crate::user::DbUser;
use crate::prelude::get_db_size;

#[macro_use]
extern crate rocket;

#[launch]
async fn launch() -> Rocket<Build> {
    let mut db_client_options =
        ClientOptions::parse("mongodb+srv://server:jRk3JcqhsOXsT4RC@cluster0.us09s.mongodb.net/myFirstDatabase?retryWrites=true&w=majority")
            .await
            .unwrap();
    println!("Start 3");

    db_client_options.app_name = Some("Chat".to_string());
    let db_client = Client::with_options(db_client_options).unwrap();
    let users = get_users_internal(&db_client).await.unwrap();

    rocket::build()
        .manage(users)
        .manage(db_client)
        .manage(channel::<Message>(1024).0)
        .mount("/", routes![register, get_users, connect, send, login, get_user_by_id, test])
}

#[post("/connect", data="<user>")]
async fn connect(user: User, queue: &State<Sender<Message>>, mut end: Shutdown) -> ByteStream![Vec<u8>] {
    let mut rx = queue.subscribe();
    let id = user.id();

    if id != 0 {
        let mut start_message: Vec<u8> = vec![0];
        start_message.append(&mut format!("{} connected", user.login()).as_bytes().to_vec());
        let _ = queue.send(Message::Message(0, start_message));
    }

    ByteStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(Message::KeepAlive(msg_id)) => {
                        if msg_id == id as u8 {
                            vec![0u8]
                        } else {
                            continue
                        }
                    }
                    Ok(Message::Message(msg_id, _)) if msg_id == id as u8 => continue,
                    Ok(Message::Message(_, msg)) => msg,
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
fn send(msg: Message, queue: &State<Sender<Message>>) -> Status {
    let _ = queue.send(msg);
    Status::Accepted
}

#[post("/register", data="<info>")]
async fn register(info: LoginData, db_client: &State<Client>, users: &State<Users>) -> status::Custom<String> {
    let db = log_id!(db_client, DbUser);
    println!("Connected to db");
    let found = find_in! {
        database db
        param "login": info.login()
    };
    println!("Got db result");

    if let Ok(Some(_)) = found {
        return status::Custom(
            Status::Conflict,
            format!("User with login {} already exists", info.login())
        );
    } else if let Err(e) = found {
        eprintln!("Registration error: {:?}", e);
        return status::Custom(
            Status::BadGateway,
            "Error occurred during registration".to_owned()
        );
    }
    println!("Errors handled");

    let id = get_db_size(&db).await;
    let password = bcrypt::hash(info.password(), 5).unwrap();
    println!("Got size and hash");

    let new = DbUser::new(info.login(), password, id);
    let insert_result = db.insert_one(new.clone(), None).await;
    println!("Inserted");

    if let Err(e) = insert_result {
        eprintln!("Error adding db: {:?}", e);
        return status::Custom(
            Status::BadGateway,
            "Error occurred while adding to db".to_owned(),
        );
    }

    let user = User::from(new.clone());
    users.add(user.clone());
    status::Custom(Status::Found, json::to_string(&user).unwrap())
}

#[post("/login", data="<login>")]
async fn login(login: LoginData, db_client: &State<Client>) -> Result<Json<User>, status::Custom<String>> {
    let db = log_id!(db_client, DbUser);

    let found = find_in! {
        database db
        param "login": login.login()
    };

    let user = found.unwrap_or(None);
    match user {
        None => Err(status::Custom(Status::NotFound, String::new())),
        Some(user) => {
            let matches = bcrypt::verify(login.password(), &user.password()).unwrap();
            if matches {
                Ok(Json(User::from(user)))
            } else {
                Err(status::Custom(Status::BadRequest, "Passwords do not match".to_owned()))
            }
        }
    }
}

#[get("/get_users")]
async fn get_users(db_client: &State<Client>) -> Json<String> {
    let result = get_users_internal(db_client).await;

    if let Err(e) = result {
        eprintln!("Error retrieving users: {:?}", e);
        return Json("Internal server error".to_owned());
    }

    Json(json::to_string(&result.unwrap().users).unwrap())
}

async fn get_users_internal(db_client: &Client) -> Result<Users, Box<dyn Error>> {
    let db = log_id!(db_client, User);

    let mut users = db.find(None, None).await?;
    let result = Users::new();
    while let Some(user) = users.try_next().await? {
        result.add(user);
    }

    Ok(result)
}

#[get("/test")]
fn test() -> String {
    String::from("It works again and again")
}

#[allow(dead_code)]
#[get("/get/<id>")]
fn get_user_by_id(id: i32, users: &State<Users>) -> Option<User> {
    users.get(id)
}
