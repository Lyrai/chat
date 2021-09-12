use rocket::data::{FromData, Outcome};
use rocket::{Request, Data};
use serde::{Serialize, Deserialize};
use crate::content_length;
use crate::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginData {
    login: String,
    password: String
}

#[async_trait]
impl<'r> FromData<'r> for LoginData {
    type Error = ();

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let size = content_length!(req, 1 << 15);

        let user = read_data(data, size).await;
        Outcome::Success(serde_json::from_str::<LoginData>(&user).unwrap())
    }
}

impl LoginData {
    pub fn login(&self) -> String {
        self.login.clone()
    }

    pub fn password(&self) -> String {
        self.password.clone()
    }
}