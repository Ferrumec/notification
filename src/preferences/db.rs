use sqlx::SqlitePool;
use crate::preferences::models::{Channel, ChannelDefault, EventPref};

// ── Channel defaults ──────────────────────────────────────────────────────────

pub async fn upsert_channel_defaults(
    pool: &SqlitePool,
    user_id: &str,
    defaults: &[ChannelDefault],
) -> sqlx::Result<()> {
    for d in defaults {
let channel = d.channel.as_str();
        sqlx::query!(
            r#"
            INSERT INTO notification_channel_defaults (user_id, channel, enabled, updated_at)
            VALUES (?, ?, ?, datetime('now'))
            ON CONFLICT(user_id, channel) DO UPDATE SET
                enabled    = excluded.enabled,
                updated_at = excluded.updated_at
            "#,
            user_id,
            channel,
            d.enabled,
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn get_channel_defaults(
    pool: &SqlitePool,
    user_id: &str,
) -> sqlx::Result<Vec<ChannelDefault>> {
    let rows = sqlx::query!(
        r#"
        SELECT channel AS "channel: Channel", enabled AS "enabled: bool"
        FROM notification_channel_defaults
        WHERE user_id = ?
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ChannelDefault { channel: r.channel, enabled: r.enabled })
        .collect())
}

// ── Event-type overrides ──────────────────────────────────────────────────────

pub async fn upsert_event_prefs(
    pool: &SqlitePool,
    user_id: &str,
    prefs: &[EventPref],
) -> sqlx::Result<()> {
    for p in prefs {
let channel = p.channel.as_str();
        sqlx::query!(
            r#"
            INSERT INTO notification_event_prefs (user_id, event_type, channel, enabled, updated_at)
            VALUES (?, ?, ?, ?, datetime('now'))
            ON CONFLICT(user_id, event_type, channel) DO UPDATE SET
                enabled    = excluded.enabled,
                updated_at = excluded.updated_at
            "#,
            user_id,
            p.event_type,
            channel,
            p.enabled,
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn get_event_prefs(
    pool: &SqlitePool,
    user_id: &str,
    event_type: &str,
) -> sqlx::Result<Vec<EventPref>> {
    let rows = sqlx::query!(
        r#"
        SELECT event_type,
               channel   AS "channel: Channel",
               enabled   AS "enabled: bool"
        FROM notification_event_prefs
        WHERE user_id = ? AND event_type = ?
        "#,
        user_id,
        event_type
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| EventPref {
            event_type: r.event_type,
            channel: r.channel,
            enabled: r.enabled,
        })
        .collect())
}

// ── Resolution logic ──────────────────────────────────────────────────────────

/// Returns which channels should fire for (user, event_type).
///
/// Resolution order:
///   1. Start with global channel defaults (missing = enabled by default).
///   2. Apply per-event-type overrides on top.
pub async fn resolve_channels(
    pool: &SqlitePool,
    user_id: &str,
    event_type: &str,
) -> sqlx::Result<Vec<Channel>> {
    use std::collections::HashMap;

    // 1. Load global defaults → seed map
    let defaults = get_channel_defaults(pool, user_id).await?;
    let mut channel_map: HashMap<String, bool> = Channel::all()
        .iter()
        .map(|c| (c.as_str().to_string(), true)) // default: all enabled
        .collect();

    for d in defaults {
        channel_map.insert(d.channel.as_str().to_string(), d.enabled);
    }

    // 2. Apply event-type overrides
    let overrides = get_event_prefs(pool, user_id, event_type).await?;
    for o in overrides {
        channel_map.insert(o.channel.as_str().to_string(), o.enabled);
    }

    // 3. Collect enabled channels
    let enabled = Channel::all()
        .iter()
        .filter(|c| *channel_map.get(c.as_str()).unwrap_or(&true))
        .cloned()
        .collect();

    Ok(enabled)
}
