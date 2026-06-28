//! UseCase。otegami の業務手順を集約する。
//!
//! 各 UseCase は `async fn execute(&self, Input) -> Result<Output, Error>` 形で、
//! 依存(Port)はコンストラクタで `Arc<dyn Port>` として受け取る。時刻は Clock 経由。

mod create_note;
mod purge_notes;
mod view_note;

pub use create_note::{CreateNote, CreateNoteInput, CreateNoteOutput};
pub use purge_notes::{PurgeNotes, PurgeNotesOutput};
pub use view_note::{ViewNote, ViewNoteInput, ViewNoteOutput};
