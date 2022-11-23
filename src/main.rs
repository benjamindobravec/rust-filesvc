#[macro_use]
extern crate rocket;

use rocket::fs::NamedFile;
use rocket::serde::json::Json;

mod files;

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(std::path::Path::new("index.html"))
        .await
        .ok()
}

#[get("/files")]
fn get_files() -> Result<Json<files::FileMetadatas>, rocket::response::status::NotFound<String>> {
    let result = files::get(".", "");
    match result {
        Ok(files) => Ok(Json(files)),
        Err(e) => Err(rocket::response::status::NotFound(e.to_string())),
    }
}

#[get("/files/<name>/files")]
async fn get_sub_files(
    name: &str,
) -> Result<Json<files::FileMetadatas>, rocket::response::status::NotFound<String>> {
    let result = files::get(".", name);
    match result {
        Ok(files) => Ok(Json(files)),
        Err(e) => Err(rocket::response::status::NotFound(e)),
    }
}

#[get("/search?<query>")]
async fn get_search(
    query: &str,
) -> Result<Json<files::FileMetadatas>, rocket::response::status::NotFound<String>> {
    let result = files::search(".", "", query);
    match result {
        Ok(files) => Ok(Json(files)),
        Err(e) => Err(rocket::response::status::NotFound(e.to_string())),
    }
}

#[get("/search/<name>?<query>")]
async fn get_sub_search(
    name: &str,
    query: &str,
) -> Result<Json<files::FileMetadatas>, rocket::response::status::NotFound<String>> {
    let result = files::search(".", name, query);
    match result {
        Ok(files) => Ok(Json(files)),
        Err(e) => Err(rocket::response::status::NotFound(e.to_string())),
    }
}

#[get("/files/<name>/metadata")]
async fn get_file_metadata(
    name: &str,
) -> Result<Json<files::FileMetadata>, rocket::response::status::NotFound<String>> {
    let result = files::get_file(".", name);
    match result {
        Ok(files) => Ok(Json(files)),
        Err(e) => Err(rocket::response::status::NotFound(e.to_string())),
    }
}

#[get("/files/<name>")]
async fn get_file_data(name: &str) -> Option<NamedFile> {
    NamedFile::open(std::path::Path::new(name)).await.ok()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes![
            index,
            get_files,
            get_sub_files,
            get_file_data,
            get_search,
            get_sub_search,
            get_file_metadata
        ],
    )
}
