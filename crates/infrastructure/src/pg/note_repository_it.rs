//! PgNoteRepository の統合テスト(実 DB)。
//!
//! `#[sqlx::test]` は隔離されたテスト用 DB を用意し、`migrations/` を自動適用する。
//! DATABASE_URL を設定し `--features integration` で実行する。

use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

use domain::entities::{Note, NoteSnapshot};
use domain::ports::NoteRepository;
use domain::value_objects::{PasswordHash, Slug};

use crate::pg::PgNoteRepository;

fn note(slug: &str, burn: bool, expires_at: OffsetDateTime) -> Note {
    Note::restore(NoteSnapshot {
        slug: Slug::parse(slug).unwrap(),
        title: Some("memo".to_owned()),
        body: "hello".to_owned(),
        password_hash: PasswordHash::from_stored(
            "$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHQ$aGFzaGhhc2g".to_owned(),
        ),
        burn_after_view: burn,
        viewed_at: None,
        expires_at,
        created_at: OffsetDateTime::now_utc(),
    })
}

#[sqlx::test(migrations = "../../migrations")]
async fn insert_したノートを_find_できる(pool: PgPool) -> sqlx::Result<()> {
    let repo = PgNoteRepository::new(pool);
    repo.insert(&note(
        "ABC234",
        false,
        OffsetDateTime::now_utc() + Duration::days(1),
    ))
    .await
    .unwrap();

    let found = repo
        .find_by_slug(&Slug::parse("ABC234").unwrap())
        .await
        .unwrap()
        .expect("存在するはず");
    assert_eq!(found.body(), "hello");
    assert_eq!(found.title(), Some("memo"));
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn 期限切れのノートは見つからない(pool: PgPool) -> sqlx::Result<()> {
    let repo = PgNoteRepository::new(pool);
    repo.insert(&note(
        "ABC235",
        false,
        OffsetDateTime::now_utc() - Duration::hours(1),
    ))
    .await
    .unwrap();

    let found = repo
        .find_by_slug(&Slug::parse("ABC235").unwrap())
        .await
        .unwrap();
    assert!(found.is_none());
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn slug_衝突は_conflict_を返す(pool: PgPool) -> sqlx::Result<()> {
    use domain::errors::InsertError;

    let repo = PgNoteRepository::new(pool);
    let n = note(
        "ABC236",
        false,
        OffsetDateTime::now_utc() + Duration::days(1),
    );
    repo.insert(&n).await.unwrap();

    let err = repo.insert(&n).await.unwrap_err();
    assert!(matches!(err, InsertError::Conflict));
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn mark_viewed_は初回のみ確定する(pool: PgPool) -> sqlx::Result<()> {
    let repo = PgNoteRepository::new(pool);
    let slug = Slug::parse("ABC237").unwrap();
    repo.insert(&note(
        "ABC237",
        true,
        OffsetDateTime::now_utc() + Duration::days(1),
    ))
    .await
    .unwrap();

    let first = OffsetDateTime::now_utc();
    repo.mark_viewed(&slug, first).await.unwrap();
    // 二度目は上書きされない。
    repo.mark_viewed(&slug, first + Duration::minutes(5))
        .await
        .unwrap();

    let viewed = repo
        .find_by_slug(&slug)
        .await
        .unwrap()
        .unwrap()
        .viewed_at()
        .expect("初回で確定しているはず");
    assert!((viewed - first).abs() < Duration::seconds(1));
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_purgeable_は失効と閲覧済み_burn_を消す(
    pool: PgPool,
) -> sqlx::Result<()> {
    let repo = PgNoteRepository::new(pool);
    let now = OffsetDateTime::now_utc();

    // 失効分
    repo.insert(&note("ABC238", false, now - Duration::hours(1)))
        .await
        .unwrap();
    // 閲覧済み burn 分(insert 後に閲覧済みにする)
    repo.insert(&note("ABC239", true, now + Duration::days(1)))
        .await
        .unwrap();
    repo.mark_viewed(&Slug::parse("ABC239").unwrap(), now)
        .await
        .unwrap();
    // 生存(未失効・未閲覧)
    repo.insert(&note("ABC23A", false, now + Duration::days(1)))
        .await
        .unwrap();

    let deleted = repo.delete_purgeable(now).await.unwrap();
    assert_eq!(deleted, 2);

    assert!(repo
        .find_by_slug(&Slug::parse("ABC23A").unwrap())
        .await
        .unwrap()
        .is_some());
    Ok(())
}
