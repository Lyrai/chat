use rocket::futures::lock::Mutex;
use rocket::tokio::io::{AsyncReadExt, AsyncWriteExt};
use rocket::tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use rocket::tokio::net::TcpStream;
use std::sync::Arc;

pub struct Chat {
    pub client_write_streams: Mutex<Vec<OwnedWriteHalf>>,
}

impl Chat {
    pub fn new() -> Self {
        Chat {
            client_write_streams: Mutex::new(vec![]),
        }
    }

    pub async fn add(self: Arc<Chat>, client: TcpStream) {
        let (read, write) = client.into_split();
        let mut vec = self.client_write_streams.lock().await;

        vec.push(write);
        Chat::start_read_stream(self.clone(), read, vec.len() - 1);
    }

    async fn write_all(self: Arc<Chat>, message: &[u8], size: &[u8], sender: usize) {
        let mut clients = self.client_write_streams.lock().await;

        for i in 0..clients.len() {
            if sender == i {
                continue;
            }

            clients[i].write(size).await;
            clients[i].write(message).await;

            println!("Written");
        }
    }

    fn start_read_stream(chat: Arc<Chat>, mut read_stream: OwnedReadHalf, id: usize) {
        rocket::tokio::spawn(async move {
            loop {
                match read_stream.read_u32_le().await {
                    Ok(n) if n == 0 => continue,

                    Ok(n) => {
                        let size = n as usize;
                        let mut msg = vec![0u8; size];
                        let len = read_stream.read(&mut msg).await.unwrap();

                        if len != size {
                            eprintln!("Mismatched expected {} and actual length {}", size, len);
                            read_stream.read(&mut [0; 1024]).await;
                            return;
                        }

                        Chat::write_all(chat.clone(), &msg, &n.to_le_bytes(), id).await;
                    }

                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,

                    Err(e) => {
                        eprintln!("{}", e);
                        return;
                    }
                };
            }
        });
    }
}
