use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use std::cell::RefCell;
use crate::user::User;
use crate::unwrap_mutex;

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