use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::sync::Mutex;

macro_rules! unwrap_mutex {
    ($id: expr, $m: ident) => {
        $id.lock().unwrap().$m()
    };
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub login: String,
    pub id: u8,
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

#[derive(Serialize)]
pub struct Users {
    pub users: Mutex<RefCell<Vec<User>>>,
}

impl Users {
    pub fn new() -> Self {
        Users {
            users: Mutex::new(RefCell::new(vec![])),
        }
    }

    pub fn get(&self, login: String) -> Option<User> {
        unwrap_mutex!(self.users, borrow)
            .iter()
            .find(|x| x.login == login)
            .cloned()
    }

    pub fn add(&self, user: User) {
        unwrap_mutex!(self.users, borrow_mut).push(user);
    }
}
