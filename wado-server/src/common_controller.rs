use actix_web::{get, HttpResponse, Responder};

/// nothing to do ,just for test to verify serices is running
#[utoipa::path(
    get,
    responses(
        (status = 200, description = "Echo Success"),
    ),
    tag = "WEBAPI",
    description = "Echo endpoint"
)]
#[get("/echo")]
pub async fn echo() -> impl Responder {
    HttpResponse::Ok().body("Success")
}