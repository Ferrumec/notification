use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Email,
    Push,
    Sms,
}

impl Channel {
    pub fn all() -> &'static [Channel] {
        &[Channel::Email, Channel::Push, Channel::Sms]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Channel::Email => "email",
            Channel::Push  => "push",
            Channel::Sms   => "sms",
        }
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ── API shapes ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelDefault {
    pub channel: Channel,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetChannelDefaultsRequest {
    pub defaults: Vec<ChannelDefault>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventPref {
    pub event_type: String,
    pub channel: Channel,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetEventPrefsRequest {
    pub prefs: Vec<EventPref>,
}

/// What the orchestrator gets back when it asks "should I notify this user?"
#[derive(Debug, Serialize)]
pub struct ResolvedChannels {
    pub user_id: String,
    pub event_type: String,
    pub channels: Vec<Channel>, // channels that should fire
}
