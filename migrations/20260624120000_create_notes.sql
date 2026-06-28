-- notes テーブル。otegami は単一テーブルで完結する(design-spec 6章)。
--
-- 外部に出るのは slug だけで、連番 id は秘匿する(列挙耐性のため)。
-- RLS は使わず、アクセス制御は slug + パスワード + レート制限 + Cloudflare Access
-- (作成導線)で担う。
CREATE TABLE notes (
    id              BIGSERIAL    PRIMARY KEY,            -- 内部ID。外部には露出しない
    slug            TEXT         NOT NULL UNIQUE,        -- 公開ハンドル
    title           TEXT,                                -- 任意
    body            TEXT         NOT NULL,               -- 素のテキストとして保存
    password_hash   TEXT         NOT NULL,               -- Argon2
    burn_after_view BOOLEAN      NOT NULL DEFAULT FALSE,
    viewed_at       TIMESTAMPTZ,                         -- 初回閲覧成功時刻
    expires_at      TIMESTAMPTZ  NOT NULL,               -- 既定TTLから算出
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- slug 引き(閲覧経路)の索引。UNIQUE 制約とは別に明示する。
CREATE INDEX idx_notes_slug ON notes (slug);

-- 掃除バッチの失効判定(expires_at)を支える索引。
CREATE INDEX idx_notes_purge ON notes (expires_at);
