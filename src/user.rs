use rocket::http::{Status, Header};
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, Response};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::sync::Mutex;
use rocket::response::{Responder, Result};
use std::io::Cursor;

macro_rules! unwrap_mutex {
    ($id: expr, $m: ident) => {
        $id.lock().unwrap().$m()
    };
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub login: String,
    pub id: i32,
}

#[async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let login = request.headers().get_one("login");

        match login {
            Some(login) => Outcome::Success(User {
                login: String::from(login),
                id: 0,
            }),
            None => Outcome::Failure((Status::BadRequest, ())),
        }
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for User {
    fn respond_to(self, request: &'r Request<'_>) -> Result<'o> {
        let json = serde_json::to_string(&self).unwrap();

        let response = Response::build()
            .status(Status::Found)
            .sized_body(json.len(), Cursor::new(json))
            .raw_header("Content-Type", "application/json")
            .finalize();

        Ok(response)
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