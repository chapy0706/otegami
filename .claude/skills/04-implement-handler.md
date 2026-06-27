# Skill: Axum ハンドラ・Askama テンプレートの実装パターン

このスキルは、otegami における Presentation 層(Axum のハンドラと Askama テンプレート)の実装手順とパターンを定義します。

---

## ハンドラの責務

- リクエストの境界で入力を型へデシリアライズし、必要なら検証する
- UseCase の Input に変換する
- UseCase を呼び出す
- UseCase の Output / エラーをレスポンス(Askama 描画)に変換する
- 業務判断を持たない
- 認証は委ねる。作成導線は Cloudflare Access(エッジ)で保護され、アプリは認証コードを持たない(ADR-0004)

---

## ファイル配置

```
apps/web/src/
├── main.rs           // 合成位置(AppState の組み立て + ルーティング)
├── state.rs          // AppState 定義
└── handlers/
    ├── view.rs       // 閲覧ハンドラ
    └── create.rs     // 作成ハンドラ

templates/
├── note.html         // 本文表示({{ body }} は自動エスケープ)
├── error.html
└── create.html
```

---

## 実装テンプレート

### 合成位置と AppState(state.rs / main.rs)

UseCase と Adapter の結線は Presentation 層の合成位置で一度だけ行います。具象を `Arc<dyn Port>` に包み、UseCase へ注入してから `AppState` に載せます。domain / application の内部では具象を組み立てません。

```rust
// state.rs
use std::sync::Arc;
use application::use_cases::ViewNote;

#[derive(Clone)]
pub struct AppState {
    pub view_note: Arc<ViewNote>,
}
```

```rust
// main.rs
use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use sqlx::postgres::PgPoolOptions;

use application::use_cases::ViewNote;
use domain::ports::{Clock, NoteRepository, PasswordHasher};
use infrastructure::argon2_hasher::Argon2Hasher;
use infrastructure::pg::note_repository::PgNoteRepository;
use infrastructure::system_clock::SystemClock;

mod handlers;
mod state;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = PgPoolOptions::new()
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;

    let notes: Arc<dyn NoteRepository> = Arc::new(PgNoteRepository::new(pool.clone()));
    let hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2Hasher);
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);

    let state = AppState {
        view_note: Arc::new(ViewNote::new(notes, hasher, clock)),
    };

    let app = Router::new()
        .route("/n/:slug", get(handlers::view::show_form).post(handlers::view::submit))
        // /create は Cloudflare Access の背後に置く(アプリ側では認証しない)
        .route("/create", get(handlers::create::show_form).post(handlers::create::submit))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

### Askama テンプレートと描画ヘルパー(handlers/view.rs)

テンプレートは `.html` 拡張子にすることで `{{ body }}` が自動エスケープされます。`|safe` は使いません。描画は `render()` で `String` を得て `Html` に包みます(統合クレートには依存しない)。

```rust
use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::Form;
use serde::Deserialize;

use application::use_cases::ViewNoteInput;
use domain::errors::ViewNoteError;

use crate::state::AppState;

#[derive(Template)]
#[template(path = "note.html")]
struct NoteTemplate {
    title: Option<String>,
    body: String, // .html テンプレートなので {{ body }} は自動エスケープされる
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    message: String,
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

// slug の存在有無を明かさないため、フォームは常に出す
pub async fn show_form(Path(slug): Path<String>) -> Response {
    render(PasswordFormTemplate { slug }, StatusCode::OK)
}

pub async fn submit(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Form(form): Form<ViewForm>,
) -> Response {
    let result = state
        .view_note
        .execute(ViewNoteInput { raw_slug: slug, password: form.password })
        .await;

    match result {
        Ok(out) => render(
            NoteTemplate { title: out.title, body: out.body },
            StatusCode::OK,
        ),
        // 見つからない/パスワード違いは同一の応答に畳む(ADR-0001)
        Err(ViewNoteError::NotFoundOrWrongPassword) => render(
            ErrorTemplate { message: "見つからないか、パスワードが違います".to_owned() },
            StatusCode::NOT_FOUND,
        ),
        Err(ViewNoteError::Unexpected) => render(
            ErrorTemplate { message: "時間をおいて、もう一度お試しください".to_owned() },
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

fn render<T: Template>(tpl: T, status: StatusCode) -> Response {
    match tpl.render() {
        Ok(html) => (status, Html(html)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "render error").into_response(),
    }
}
```

### テンプレート(templates/note.html)

```html
<!doctype html>
<title>otegami</title>
{% match title %}
  {% when Some with (t) %}<h1>{{ t }}</h1>
  {% when None %}
{% endmatch %}
<pre>{{ body }}</pre>
```

`{{ body }}` と `{{ t }}` は自動エスケープされ、`<script>` を含む本文もただの文字として描かれます。エスケープを外す `|safe` は otegami では使いません。

---

## レート制限

閲覧の試行回数制限(ADR-0001 の前提)は、ハンドラ内ではなくミドルウェア層(`tower-governor` など)で `/n/:slug` の POST に課します。短いパスワードの安全性はこのミドルウェアが成立条件です。

---

## 確認チェックリスト

- [ ] リクエスト境界で型デシリアライズ(必要なら追加検証)をしている
- [ ] ハンドラに業務判断を書いていない
- [ ] UseCase と Adapter の結線を合成位置(main / AppState)で行っている
- [ ] エラーを適切なステータス/応答に変換している
- [ ] 閲覧失敗を単一の応答に畳んでいる(存在有無・失敗理由を漏らしていない)
- [ ] テンプレートが `.html` で、`{{ }}` の自動エスケープに委ねている(`|safe` を使っていない)
- [ ] 本文・パスワードをログに出していない
- [ ] 作成導線の認証をアプリに持ち込んでいない(Cloudflare Access に委ねている)
- [ ] レート制限ミドルウェアが閲覧 POST に掛かっている
- [ ] `make verify` が通る
