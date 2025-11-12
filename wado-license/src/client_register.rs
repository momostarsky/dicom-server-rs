use actix_session::Session;
use actix_web::{HttpResponse, Result};
use std::fs;

pub(crate) async fn show_register_form(
    session: Session, // 添加 Session 参数
) -> Result<HttpResponse> {
    // 检查用户是否已登录
    if let Ok(Some(logged_in)) = session.get::<bool>("logged_in") {
        if !logged_in {
            // 用户未登录，重定向到登录页面
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/"))
                .finish());
        }
    } else {
        // Session 中没有登录信息，重定向到登录页面
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish());
    }
    // 不要清除所有session数据，只读取错误消息
    let error_message = session.get::<String>("register_error").unwrap_or(None);
    // 清除错误消息，避免重复显示
    session.remove("register_error");

    let error_html = if let Some(error) = error_message {
        format!(r#"<div class="alert alert-danger">{}</div>"#, error)
    } else {
        String::new()
    };
    let mut register_info = session.get::<ClientRegisterParams>("register_info")?;
    // 获取当前年份并加1年
    let current_year = Local::now().year();
    let next_year = current_year + 1;
    let end_date_str = format!("{}1231", next_year); // 设置为下一年的12月31日
    // 在生成默认 client_id 的地方修改为：
    let mut rng = rand::rng();
    let random_suffix = rng.random_range(1000..=9999);
    let client_id = format!("HZXS20252701{}", random_suffix);
    // 生成64-128位随机字符串
    let hash_code_length = rand::rng().random_range(64..=128);
    let random_hash_code = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(hash_code_length)
        .map(char::from)
        .collect::<String>();

    if register_info.is_none() {
        register_info = Option::from(ClientRegisterParams {
            client_id: client_id.to_string(),
            client_name: "西部省东海市人民医院".to_string(),
            client_hash_code: random_hash_code.to_string(),
            end_date: end_date_str,
        });
    }
    let reg = register_info.unwrap();
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                <title>注册表单</title>
                <link rel="stylesheet" href="/static/lib/bootstrap/dist/css/bootstrap.min.css"/>
                <link rel="stylesheet" href="/static/css/site.css" asp-append-version="true"/>
            </head>
            <body>
                <div class="container mt-5">
                    <div class="row justify-content-center">
                        <div class="col-md-6">
                            <div class="card">
                                <div class="card-header">
                                    <h3 class="text-center">客户端注册</h3>
                                </div>
                                <div class="card-body">
                                    {}
                                    <form action="/register" method="post">
                                        <div class="mb-3">
                                            <label for="client_id" class="form-label">ClientID:</label>
                                            <input type="text" class="form-control" id="client_id" name="client_id" max-length=32  value="{}" required>
                                        </div>

                                        <div class="mb-3">
                                            <label for="client_name" class="form-label">ClientName:</label>
                                            <input type="text" class="form-control" id="client_name" name="client_name" value="{}"  max-length=64 required>
                                        </div>
                                        <div class="mb-3">
                                            <label for="hash_code" class="form-label">HashCode:</label>
                                            <input type="text" class="form-control" id="hash_code" name="hash_code" value="{}" max-length=128 required>
                                        </div>
                                         <div class="mb-3">
                                            <label for="expired_date" class="form-label">ExpiredDate:</label>
                                            <input type="text" class="form-control" id="expired_date" name="expired_date" value="{}" max-length=8 required>
                                        </div>
                                        <div class="d-grid">
                                            <input type="submit" class="btn btn-primary" value="提交">
                                        </div>
                                    </form>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <script src="/static/lib/jquery/dist/jquery.min.js"></script>
                <script src="/static/lib/bootstrap/dist/js/bootstrap.bundle.min.js"></script>
                <script src="/static/js/site.js" asp-append-version="true"></script>
            </body>
        </html>
    "#,
        error_html, reg.client_id, reg.client_name, reg.client_hash_code, reg.end_date
    );

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

pub(crate) async fn show_login_form(
    session: Session, // 添加 Session 参数
) -> Result<HttpResponse> {
    // 不要清除所有session数据，只读取错误消息
    let error_message = session.get::<String>("login_error").unwrap_or(None);
    // 清除错误消息，避免重复显示
    session.remove("login_error");
    session.remove("register_info");

    let error_html = if let Some(error) = error_message {
        format!(r#"<div class="alert alert-danger">{}</div>"#, error)
    } else {
        String::new()
    };

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                <title>登录</title>
                <link rel="stylesheet" href="/static/lib/bootstrap/dist/css/bootstrap.min.css"/>
                <link rel="stylesheet" href="/static/css/site.css" asp-append-version="true"/>
            </head>
            <body>
                <div class="container mt-5">
                    <div class="row justify-content-center">
                        <div class="col-md-6">
                            <div class="card">
                                <div class="card-header">
                                    <h3 class="text-center">登录</h3>
                                </div>
                                <div class="card-body">
                                     {}
                                    <form action="/login" method="post">
                                        <div class="mb-3">
                                            <label for="name" class="form-label">Username:</label>
                                            <input type="text" class="form-control" id="name" name="name" required>
                                        </div>

                                        <div class="mb-3">
                                            <label for="password" class="form-label">Password:</label>
                                            <input type="password" class="form-control" id="password" name="password" required>
                                        </div>

                                        <div class="d-grid">
                                            <input type="submit" class="btn btn-primary" value="Login">
                                        </div>
                                    </form>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <script src="/static/lib/jquery/dist/jquery.min.js"></script>
                <script src="/static/lib/bootstrap/dist/js/bootstrap.bundle.min.js"></script>
                <script src="/static/js/site.js" asp-append-version="true"></script>
            </body>
        </html>
    "#,
        error_html
    );

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

use crate::register_controller::ClientRegisterParams;
use actix_web::web;
use chrono::{Datelike, Local};
use common::cert_helper;
use rand::distr::Alphanumeric;
use rand::Rng;
use serde::Deserialize;
use tokio_util::io::ReaderStream;

#[derive(Deserialize)]
pub(crate) struct LoginForm {
    name: String,
    password: String,
}

#[derive(Deserialize)]
pub(crate) struct RegistrationForm {
    expired_date: String,
    client_id: String,
    client_name: String,
    hash_code: String,
}

pub(crate) async fn handle_login_post(
    form: web::Form<LoginForm>,
    session: Session, // 添加 Session 参数
) -> Result<HttpResponse> {
    // 验证用户名和密码
    if form.name == "admin" && form.password == "dJp#123x" {
        // 验证成功，设置 Session
        session.insert("logged_in", true)?;
        session.insert("user_name", &form.name)?;

        // 重定向到客户端注册页面
        Ok(HttpResponse::Found()
            .append_header(("Location", "/create"))
            .finish())
    } else {
        // 验证失败，存储错误消息到Session
        session.insert("login_error", "用户名或密码错误")?;

        // 重定向回登录页面
        Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish())
    }
}

pub(crate) async fn handle_form_post(
    form: web::Form<RegistrationForm>,
    session: Session, // 添加 Session 参数
) -> Result<HttpResponse> {
    // 检查用户是否已登录
    if let Ok(Some(logged_in)) = session.get::<bool>("logged_in") {
        if !logged_in {
            // 用户未登录，重定向到登录页面
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/"))
                .finish());
        }
    } else {
        // Session 中没有登录信息，重定向到登录页面
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish());
    }
    // 返回响应
    let crp = ClientRegisterParams {
        client_id: form.client_id.clone(),
        client_name: form.client_name.clone(),
        client_hash_code: form.hash_code.clone(),
        end_date: form.expired_date.clone(),
    };

    match crp.validate() {
        Ok(_) => {}
        Err(e) => {
            session.insert("register_error", e)?;
            session.insert("register_info", crp)?;
            // 重定向回登录页面
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/create"))
                .finish());
        }
    };

    let (client_cert, client_seckey) = match cert_helper::generate_client_and_sign(
        &crp.client_name,
        &crp.client_id,
        &crp.client_hash_code,
        &crp.end_date,
        &crate::register_controller::CA_FILE,
        &crate::register_controller::CA_KEY_FILE,
    ) {
        Ok(result) => result,
        Err(e) => {
            session.insert("register_error", e.to_string())?;
            session.insert("register_info", crp)?;
            // 重定向回登录页面
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/create"))
                .finish());
        }
    };

    let client_cert_file_path = format!("/opt/client-cert/client_{}.crt", &crp.client_id);
    let client_key_file_path = format!("/opt/client-cert/client_{}.key", &crp.client_id);
    match fs::write(&client_cert_file_path, &client_cert) {
        Ok(_) => {
            session.insert("register_error", "write client cert file success")?;
            session.remove("register_info");
        }
        Err(e) => {
            session.insert("register_error", e.to_string())?;
            session.insert("register_info", crp)?;
            // 重定向回登录页面
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/create"))
                .finish());
        }
    };

    match fs::write(&client_key_file_path, &client_seckey) {
        Ok(_) => {
            session.insert("register_error", "write client key file success")?;
            session.remove("register_info");
        }
        Err(e) => {
            session.insert("register_error", e.to_string())?;
            session.insert("register_info", crp)?;
            // 重定向回登录页面
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/create"))
                .finish());
        }
    };
    session.remove("register_info");
    use tokio::fs::File as TokioFile;
    // 尝试以 Tokio 文件方式打开
    let file = match TokioFile::open(&client_cert_file_path).await {
        Ok(f) => f,
        Err(e) => {
            session.insert(
                "register_error",
                format!("Failed to open certificate file: {}", e),
            )?;

            // 重定向回注册页面
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/create"))
                .finish());
        }
    };

    // 将文件转换为 Stream
    let stream = ReaderStream::new(file);

    // 设置 Content-Disposition，指定下载的默认文件名为 client_*.crt
    let content_disposition = format!("attachment; filename=\"client_{}.crt\"", &crp.client_id);

    Ok(HttpResponse::Ok()
        .append_header(("Content-Type", "application/octet-stream"))
        .append_header(("Content-Disposition", content_disposition))
        .streaming(stream))
}
