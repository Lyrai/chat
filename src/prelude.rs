use rocket::Data;
use rocket::data::ToByteUnit;
use mongodb::Collection;

pub async fn read_data(data: Data<'_>) -> String {
    let data = data.open((1 << 15).bytes());
    data.into_string().await.unwrap().value
}

pub async fn get_db_size<T>(db: &Collection<T>) -> i32 {
    db.count_documents(None, None).await.unwrap() as i32
}