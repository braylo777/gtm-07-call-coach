//! Stage 7 — AI Sales Enablement & Coaching.
//!
//! * `text`       — tokenizer and stop words shared by the modules.
//! * `search`     — transcript vector search: TF-IDF vectors over utterance windows, cosine
//!   ranking, with call / speaker / timestamp on every hit.
//! * `coach`      — per-call metrics (talk ratio, questions, longest monologue, topics, next
//!   steps) with coaching flags against targets, and a rep trend across calls.
//! * `battlecard` — competitor mentions across calls, the objections that come with them, and
//!   playbook responses, assembled into a battlecard.

mod api;
pub mod battlecard;
pub mod coach;
pub mod search;
pub mod text;
pub mod util;

pub use api::dispatch;
use serde_json::Value;

pub const OPS: &[&str] = &["search_calls", "coach_calls", "battlecards"];

pub fn handle(op: &str, i: &Value) -> Result<Value, String> {
    match op {
        "search_calls" => search::search_op(i),
        "coach_calls" => coach::coach_op(i),
        "battlecards" => battlecard::cards_op(i),
        _ => Err(format!("unknown op `{op}`; known: {}", OPS.join(", "))),
    }
}
