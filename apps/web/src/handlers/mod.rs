//! Axum ハンドラ。境界での入出力変換に徹し、業務判断は UseCase に委ねる。

pub mod create;
pub mod view;

use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

/// 失敗表示の共通テンプレート(error.html)。
#[derive(Template)]
#[template(path = "error.html")]
pub(crate) struct ErrorTemplate {
    pub message: String,
}

/// テンプレートを描画して Response に包む。askama の統合クレートには依存しない。
pub(crate) fn render<T: Template>(tpl: T, status: StatusCode) -> Response {
    match tpl.render() {
        Ok(html) => (status, Html(html)).into_response(),
        // 描画失敗は本文・パスワードを含めず、汎用メッセージだけ返す。
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "render error").into_response(),
    }
}

/// 失敗応答のヘルパー。
pub(crate) fn render_error(message: &str, status: StatusCode) -> Response {
    render(
        ErrorTemplate {
            message: message.to_owned(),
        },
        status,
    )
}
