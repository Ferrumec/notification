use serde::{Deserialize, Serialize};
use sqlx::Type;

/// All supported notification delivery channels.
///
/// Add variants here as new channels are introduced; the compiler will
/// surface every match that needs updating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum Channel {
    Email,
    Sms,
    Push,
    InApp,
}
