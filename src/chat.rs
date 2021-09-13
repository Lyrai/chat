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
        let mut stream = data.open((1 << 15).bytes());
        let size = stream.hint();
        println!("Size {}", size);

        let mut msg = vec![0; size];

        stream.read(&mut msg).await;
        let bytes = msg[1..msg.len()].to_vec();
        if msg[1] != 0 {
            println!("Id: {}\nMessage {}", msg[0], String::from_utf8(bytes.clone()).unwrap());
        }

        Outcome::Success(Message { id: msg[0], bytes })
    }
}
