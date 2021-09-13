mod dbuser;
pub use dbuser::DbUser;

mod users;
pub use users::Users;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::response::{Responder, Result};
use rocket::{Request, Response, Data};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use rocket::data::FromData;
use crate::content_length;
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
            login: user.login(),
            id: user.id()
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
        let user = read_data(data).await;
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
