use actix_web::{web, HttpResponse, Result};
use crate::Asset;

pub(crate) async fn static_files(path: web::Path<String>) -> Result<HttpResponse> {
    let filename = path.into_inner();

    match Asset::get(&filename) {
        Some(content) => {
            let mime_type = mime_guess::from_path(&filename).first_or_octet_stream();
            Ok(HttpResponse::Ok()
                .content_type(mime_type.as_ref())
                .body(content.data))
        }
        None => Ok(HttpResponse::NotFound().body("File not found")),
    }
}
