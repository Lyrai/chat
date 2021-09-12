use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::response::{Responder, Result};
use rocket::{Request, Response, Data};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::io::Cursor;
use std::sync::Mutex;
use rocket::data::FromData;
use crate::{content_length, unwrap_mutex};
use crate::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    login: String,
    id: i32,
}

impl User {
    pub fn login(&self) -> String {
        self.login.clone()
    }

    pub fn id(&self) -> i32 {
        self.id
    }
}

impl From<DbUser> for User {
    fn from(user: DbUser) -> Self {
        User {
            login: user.login,
            id: user.id
        }
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let login = request.headers().get_one("login");
        let id = request.headers().get_one("id");

        match login {
            Some(login) => Outcome::Success(User {
                login: String::from(login),
                id: id.unwrap_or("0").parse::<i32>().unwrap_or(0),
            }),
            None => Outcome::Failure((Status::BadRequest, ()))
        }
    }
}

#[async_trait]
impl <'r> FromData<'r> for User {
    type Error = ();

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> rocket::data::Outcome<'r, Self> {
        let size = content_length!(req, 1 << 15);

        let user = read_data(data, size).await;
        let user = serde_json::from_str::<User>(&user).unwrap();

        rocket::data::Outcome::Success(user)
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for User {
    fn respond_to(self, _: &'r Request<'_>) -> Result<'o> {
        let json = serde_json::to_string(&self).unwrap();
        let len = json.len();

        let response = Response::build()
            .status(Status::Found)
            .raw_header("Content-Type", "application/json")
            .raw_header("Content-Length", len.to_string())
            .sized_body(len, Cursor::new(json))
            .finalize();

        Ok(response)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DbUser {
    login: String,
    password: String,
    id: i32
}

impl DbUser {
    pub fn new(login: String, password: String, id: i32) -> Self {
        DbUser {
            login,
            password,
            id
        }
    }

    pub fn login(&self) -> String {
        self.login.clone()
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn password(&self) -> String {
        self.password.clone()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Users {
    pub users: Mutex<RefCell<Vec<User>>>,
}

impl Users {
    pub fn new() -> Self {
        Users {
            users: Mutex::new(RefCell::new(vec![])),
        }
    }

    pub fn get(&self, id: i32) -> Option<User> {
        unwrap_mutex!(self.users, borrow)
            .iter()
            .find(|x| x.id == id)
            .cloned()
    }

    pub fn add(&self, user: User) {
        unwrap_mutex!(self.users, borrow_mut).push(user);
    }
}
