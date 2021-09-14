use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::tokio::io::{AsyncReadExt};
use rocket::{Data, Request};

#[derive(Clone)]
pub enum Message {
    KeepAlive(u8),
    Message(u8, Vec<u8>)
}

#[async_trait]
impl<'r> FromData<'r> for Message {
    type Error = ();

    async fn from_data(_: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let mut stream = data.open((1 << 15).bytes());
        let size = stream.hint();
        println!("Size {}", size);

        let mut msg = vec![0; size];

        stream.read(&mut msg).await;

        if msg[1] != 0 {
            let bytes = msg[1..msg.len()].to_vec();
            println!("Id: {}\nMessage: {}", msg[0], String::from_utf8(bytes).unwrap());
            Outcome::Success(Message::Message(msg[0], msg))
        } else {
            println!("Id: {} Keep-Alive", msg[0]);
            Outcome::Success(Message::KeepAlive(msg[0]))
        }
    }
}
