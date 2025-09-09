use crate::AppState;
use actix_web::web::Path;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use slog::info;

pub(crate) static ORG_NAME: &'static str = "hzMomoStarsky";
pub(crate) static ROOT_NAME: &'static str = "dai.hanzhang";
#[get("/client/registe")]
async fn client_registe(
    client_id: String, // 客户端ID
    client_name:String,// 客户名称
    end_date: String,  // 结束时间YYYYMMDD 的格式
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let log = &app_state.log;
    info!(log, "retrieve_study_metadata  client_id:{} , client:{}, endDate:{}", client_id,client_name,end_date);
    return HttpResponse::NotAcceptable().body(format!(
        "retrieve_study_metadata Accept header must be {}",
        "application/json"
    ));
}

#[get("/client/validate")]
async fn client_validate(
    client_id: String,     // 客户端ID
    client_seckey: String, // 结束时间
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let log = &app_state.log;
    info!(log, "retrieve_study_metadata  client_id:{} ", client_id);
    return HttpResponse::NotAcceptable().body(format!(
        "retrieve_study_metadata Accept header must be {}",
        "application/json"
    ));
}
pub(crate) async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
