use serde::{Serialize, Deserialize};

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