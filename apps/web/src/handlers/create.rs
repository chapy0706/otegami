use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Response;
use axum::Form;
use serde::Deserialize;

use application::use_cases::CreateNoteInput;
use domain::errors::CreateNoteError;

use crate::handlers::{render, render_error};
use crate::state::AppState;

#[derive(Template)]
#[template(path = "create.html")]
struct CreateTemplate {
    ttl_days: i64,
    slug_length: usize,
    password_max_len: usize,
    /// 作成成功時の短い URL(`/n/<slug>`)。
    created_url: Option<String>,
    /// 入力不正時のメッセージ。
    error: Option<String>,
}

impl CreateTemplate {
    /// 設定値だけ埋めた空のフォーム。
    fn blank(state: &AppState) -> Self {
        Self {
            ttl_days: state.config.ttl.whole_days(),
            slug_length: state.config.slug_length,
            password_max_len: state.config.password_max_len,
            created_url: None,
            error: None,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateForm {
    body: String,
    title: Option<String>,
    password: String,
    // チェックボックスは未チェックだと送信されない。存在すれば true とみなす。
    burn_after_view: Option<String>,
}

/// 作成フォーム。認証は Cloudflare Access(エッジ)に委ね、アプリでは扱わない(ADR-0004)。
pub async fn show_form(State(state): State<AppState>) -> Response {
    render(CreateTemplate::blank(&state), StatusCode::OK)
}

/// 作成。検証・ハッシュ化・slug 採番・保存は UseCase に委ねる。
/// 本文・パスワードはログに出さない。
pub async fn submit(State(state): State<AppState>, Form(form): Form<CreateForm>) -> Response {
    // 空タイトルは「タイトルなし」として扱う。
    let title = form.title.filter(|t| !t.is_empty());

    let input = CreateNoteInput {
        body: form.body,
        title,
        raw_password: form.password,
        burn_after_view: form.burn_after_view.is_some(),
    };

    match state.create_note.execute(input).await {
        Ok(out) => render(
            CreateTemplate {
                created_url: Some(format!("/n/{}", out.slug)),
                ..CreateTemplate::blank(&state)
            },
            StatusCode::OK,
        ),
        // 入力不正はフォームに戻して理由を示す(運用者向けなので原因は出してよい)。
        Err(CreateNoteError::InvalidInput(msg)) => render(
            CreateTemplate {
                error: Some(msg),
                ..CreateTemplate::blank(&state)
            },
            StatusCode::BAD_REQUEST,
        ),
        Err(CreateNoteError::SlugUnavailable | CreateNoteError::Unexpected) => render_error(
            "時間をおいて、もう一度お試しください",
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}
