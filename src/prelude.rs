use rocket::Data;
use rocket::data::ToByteUnit;
use rocket::tokio::io::AsyncReadExt;
use mongodb::Collection;
use rocket::futures::executor;

pub async fn read_data(data: Data<'_>, size: usize) -> String {
    let mut data = data.open(size.bytes());
    let size = data.hint();
    let mut buf = vec![0u8; size];
    data.read(&mut buf).await;
    String::from_utf8(buf).unwrap()
}

pub async fn get_db_size<T>(db: &Collection<T>) -> i32 {
    db.count_documents(None, None).await.unwrap() as i32
}