use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::tokio::io::{AsyncReadExt};
use rocket::{Data, Request};
use serde::{Deserialize, Serialize};
use crate::content_length;

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub bytes: Vec<u8>,
    pub id: u8
}

#[async_trait]
impl<'r> FromData<'r> for Message {
    type Error = ();

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let size = content_length!(req, 1 << 15);
        println!("Size {}", size);

        let mut stream = data.open(size.bytes());
        let mut msg = vec![0; size];
        println!("Id: {}\nMessage {}", msg[0], String::from_utf8(msg[1..msg.len()].to_vec()).unwrap());

        stream.read(&mut msg).await;

        Outcome::Success(Message { id: msg[0], bytes: msg })
    }
}
