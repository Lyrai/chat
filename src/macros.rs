#[macro_export]
macro_rules! log_id {
    ($db: ident, $type: ty) => {
        $db.database("users").collection::<$type>("log_id")
    };
}

#[macro_export]
macro_rules! find_in {
    (database $db: ident $(param $name: literal: $value: expr),+) => {{
        $db.find_one(doc! {$($name: $value),+}, None).await
    }};
}

#[macro_export]
macro_rules! unwrap_mutex {
    ($id: expr, $m: ident) => {
        $id.lock().unwrap().$m()
    };
}

#[macro_export]
macro_rules! content_length {
    ($request: ident, $default_len: expr) => {
        $request
            .headers()
            .get_one("Content-Length")
            .map(|len| len.parse::<usize>().unwrap())
            .unwrap_or($default_len)
    };
}