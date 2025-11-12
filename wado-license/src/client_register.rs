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

    let client_code = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect::<String>()
        .to_uppercase();
    let client_id = format!("HZXS2701{}", client_code);
    // 生成32-64位随机字符串
    let hash_code_length = rand::rng().random_range(32..=64);
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
                                    <form id="register-form" action="/register" method="post">
                                        <div class="mb-3">
                                            <label for="client_id" class="form-label">ClientID:</label>
                                            <input type="text" class="form-control" id="client_id" name="client_id" max-length=32  value="{}" required />
                                        </div>

                                        <div class="mb-3">
                                            <label for="client_name" class="form-label">ClientName:</label>
                                            <input type="text" class="form-control" id="client_name" name="client_name" value="{}"  max-length=64 required />
                                        </div>
                                        <div class="mb-3">
                                            <label for="hash_code" class="form-label">HashCode:</label>
                                            <input type="text" class="form-control" id="hash_code" name="hash_code" value="{}" max-length=128 required />
                                        </div>
                                         <div class="mb-3">
                                            <label for="expired_date" class="form-label">ExpiredDate:</label>
                                            <input type="text" class="form-control" id="expired_date" name="expired_date" value="{}" max-length=8 required />
                                        </div>
                                        <div class="d-grid">
                                            <input type="submit" class="btn btn-primary" value="提交" />
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
                <script>
                    document.getElementById('register-form').addEventListener('submit', function(e) {{
                        e.preventDefault(); // 阻止默认提交

                        const form = e.target;
                        const formData = new FormData(form);
                            // 转换为 URL 编码格式
                        const urlEncodedData = new URLSearchParams(formData).toString();

                        fetch(form.action, {{
                            method: 'POST',
                            body: urlEncodedData,
                            headers: {{
                                'Content-Type': 'application/x-www-form-urlencoded'
                            }}
                        }}).then(response => {{
                            // 触发文件下载
                            if (response.ok) {{
                                const disposition = response.headers.get('Content-Disposition');
                                const filename = disposition ? disposition.split('filename=')[1].replace(/"/g, '') : 'client.crt';

                                response.blob().then(blob => {{
                                    const url = window.URL.createObjectURL(blob);
                                    const a = document.createElement('a');
                                    a.href = url;
                                    a.download = filename;
                                    document.body.appendChild(a);
                                    a.click();
                                    window.URL.revokeObjectURL(url);
                                    document.body.removeChild(a);

                                    // 跳转到列表页面
                                    window.location.href = '/list';
                                }});
                            }}
                        }}).catch(error => {{
                            console.error('Error:', error);
                        }});
                    }});
            </script>
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
    // 生成CSRF令牌并存储到session
    let csrf_token = generate_csrf_token();
    session.insert("csrf_token", &csrf_token)?;

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

    let mut captcha = Captcha::new();
    captcha
        .add_chars(4)
        .apply_filter(Noise::new(0.4))
        .apply_filter(Wave::new(2.0, 20.0).horizontal())
        .apply_filter(Wave::new(2.0, 20.0).vertical())
        .view(220, 120)
        .apply_filter(Dots::new(5))
        .as_png();

    // 获取验证码文本
    let captcha_text = captcha.chars_as_string();

    // 生成PNG图片数据
    let png_bytes = captcha.as_png().unwrap();
    // let (captcha_text, png_bytes) = generate_captcha_png();
    // // 将验证码文本存储到session中用于后续验证
    session.insert("captcha", captcha_text)?;
    // // 将PNG字节数据编码为Base64字符串
    let captcha_base64 = general_purpose::STANDARD.encode(&png_bytes);

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
                                        <input type="hidden" name="csrf_token" value="{}">
                                        <div class="mb-3">
                                            <label for="name" class="form-label">Username:</label>
                                            <input type="text" class="form-control" id="name" name="name" required>
                                        </div>

                                        <div class="mb-3">
                                            <label for="password" class="form-label">Password:</label>
                                            <input type="password" class="form-control" id="password" name="password" required>
                                        </div>
                                        <div class="mb-3">
                                            <label for="captcha_input" class="form-label">验证码:</label>
                                            <img id="captcha-img" src="data:image/png;base64,{}" alt="验证码" class="img-fluid mb-2" style="cursor: pointer;" title="点击刷新验证码"/>
                                            <input type="text" class="form-control" id="captcha_input" name="captcha_input" required>
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
                <script>
                    document.getElementById('captcha-img').addEventListener('click', function() {{
                        fetch('/captcha')
                            .then(response => response.blob())
                            .then(blob => {{
                                const url = URL.createObjectURL(blob);
                                this.src = url;
                            }})
                            .catch(error => {{
                                console.error('Error refreshing captcha:', error);
                            }});
                    }});
                </script>
            </body>
        </html>
    "#,
        error_html, csrf_token, captcha_base64
    );

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

use crate::register_controller::ClientRegisterParams;
use actix_web::web;
use base64::Engine;
use base64::engine::general_purpose;
use captcha::Captcha;
use captcha::filters::{Dots, Noise, Wave};
use chrono::{Datelike, Local};
use common::cert_helper;
use rand::Rng;
use rand::distr::Alphanumeric;
use serde::Deserialize;
use tokio_util::io::ReaderStream;

#[derive(Deserialize)]
pub(crate) struct LoginForm {
    name: String,
    password: String,
    csrf_token: String,
    captcha_input: String, // 添加验证码输入字段
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
    // 验证CSRF令牌
    if let Ok(Some(stored_csrf_token)) = session.get::<String>("csrf_token") {
        if form.csrf_token != stored_csrf_token {
            session.insert("login_error", "无效的请求令牌")?;
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/"))
                .finish());
        }
    } else {
        session.insert("login_error", "请求令牌已过期，请重新登录")?;
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish());
    }

    // 清除已使用的CSRF令牌
    session.remove("csrf_token");

    println!("captcha:{}", form.captcha_input);
    // 验证验证码
    if let Ok(Some(stored_captcha)) = session.get::<String>("captcha") {
        if form.captcha_input != stored_captcha {
            session.insert("login_error", "验证码错误")?;
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/"))
                .finish());
        }
    } else {
        session.insert("login_error", "验证码已过期，请重新登录")?;
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish());
    }
    // 验证成功后清除验证码
    session.remove("captcha");
    session.remove("register_error");

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
    let client_cert_file_path = format!("/opt/client-cert/client_{}.crt", &crp.client_id);
    let client_key_file_path = format!("/opt/client-cert/client_{}.key", &crp.client_id);
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
        // .insert_header(("Refresh", "3; url=/create"))
        .append_header(("Content-Type", "application/octet-stream"))
        .append_header(("Content-Disposition", content_disposition))
        .append_header(("Refresh", "0; url=/list")) // 添加这行实现跳转
        .streaming(stream))
}

// 生成CSRF令牌
fn generate_csrf_token() -> String {
    let mut rng = rand::rng();
    (0..32)
        .map(|_| {
            let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            let idx = rng.random_range(0..chars.len());
            chars.chars().nth(idx).unwrap()
        })
        .collect()
}

// 在文件中添加一个新的函数用于生成验证码API
pub(crate) async fn refresh_captcha(session: Session) -> Result<HttpResponse> {
    let mut captcha = Captcha::new();
    captcha
        .add_chars(4)
        .apply_filter(Noise::new(0.4))
        .apply_filter(Wave::new(2.0, 20.0).horizontal())
        .apply_filter(Wave::new(2.0, 20.0).vertical())
        .view(220, 120)
        .apply_filter(Dots::new(5));

    // 获取验证码文本
    let captcha_text = captcha.chars_as_string();

    // 生成PNG图片数据
    let png_bytes = match captcha.as_png() {
        Some(data) => data,
        None => {
            return Ok(HttpResponse::InternalServerError().json("Failed to generate captcha"));
        }
    };

    // 将验证码文本存储到session中用于后续验证
    session.insert("captcha", captcha_text)?;

    // 返回图片数据
    Ok(HttpResponse::Ok().content_type("image/png").body(png_bytes))
}
