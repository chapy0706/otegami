use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Form;
use serde::Deserialize;

use application::use_cases::ViewNoteInput;
use domain::errors::ViewNoteError;

use crate::handlers::{render, render_error};
use crate::state::AppState;

#[derive(Template)]
#[template(path = "note.html")]
struct NoteTemplate {
    title: Option<String>,
    // .html テンプレートなので {{ body }} は自動エスケープされる(|safe は使わない)。
    body: String,
}

#[derive(Template)]
#[template(path = "password_form.html")]
struct PasswordFormTemplate {
    slug: String,
}

#[derive(Deserialize)]
pub struct ViewForm {
    password: String,
}

/// パスワード入力フォーム。slug の存在有無を明かさないため、常に同じフォームを出す。
pub async fn show_form(Path(slug): Path<String>) -> Response {
    render(PasswordFormTemplate { slug }, StatusCode::OK)
}

/// 閲覧。slug 照合・パスワード検証は UseCase に委ね、失敗は単一応答に畳む。
pub async fn submit(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Form(form): Form<ViewForm>,
) -> Response {
    // パスワードは UseCase に渡すだけ。ログには出さない。
    // (argon2 verify は CPU 律速。spawn_blocking への退避は issue-08 で検討する。)
    let result = state
        .view_note
        .execute(ViewNoteInput {
            raw_slug: slug,
            password: form.password,
        })
        .await;

    match result {
        Ok(out) => render(
            NoteTemplate {
                title: out.title,
                body: out.body,
            },
            StatusCode::OK,
        ),
        // 見つからない / パスワード違いは区別せず同一応答に畳む(ADR-0001)。
        Err(ViewNoteError::NotFoundOrWrongPassword) => render_error(
            "見つからないか、パスワードが違います",
            StatusCode::NOT_FOUND,
        ),
        Err(ViewNoteError::Unexpected) => render_error(
            "時間をおいて、もう一度お試しください",
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}
