use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::collections::HashMap;
use crate::preferences::models::{Channel, ChannelDefault, EventPref};

// ── Channel defaults ──────────────────────────────────────────────────────────

pub async fn upsert_channel_defaults(
    pool: &SqlitePool,
    user_id: &str,
    defaults: &[ChannelDefault],
) -> sqlx::Result<()> {
    if defaults.is_empty() {
        return Ok(());
    }

    if user_id.is_empty() {
        return Err(sqlx::Error::Protocol("user_id cannot be empty".into()));
    }

    let mut qb = QueryBuilder::<Sqlite>::new(
        "INSERT INTO notification_channel_defaults (user_id, channel, enabled) ",
    );

    qb.push_values(defaults.iter(), |mut b, d| {
        b.push_bind(user_id)
            .push_bind(d.channel.as_str())
            .push_bind(d.enabled);
    });

    qb.push(
        " ON CONFLICT(user_id, channel) DO UPDATE SET \
         enabled = excluded.enabled, \
         updated_at = datetime('now')",
    );

    qb.build().execute(pool).await?;
    Ok(())
}

pub async fn get_channel_defaults(
    pool: &SqlitePool,
    user_id: &str,
) -> sqlx::Result<Vec<ChannelDefault>> {
    if user_id.is_empty() {
        return Err(sqlx::Error::Protocol("user_id cannot be empty".into()));
    }

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
    if prefs.is_empty() {
        return Ok(());
    }

    if user_id.is_empty() {
        return Err(sqlx::Error::Protocol("user_id cannot be empty".into()));
    }

    let mut qb = QueryBuilder::<Sqlite>::new(
        "INSERT INTO notification_event_prefs (user_id, event_type, channel, enabled) ",
    );

    qb.push_values(prefs.iter(), |mut b, p| {
        b.push_bind(user_id)
            .push_bind(&p.event_type)
            .push_bind(p.channel.as_str())
            .push_bind(p.enabled);
    });

    qb.push(
        " ON CONFLICT(user_id, event_type, channel) DO UPDATE SET \
         enabled = excluded.enabled, \
         updated_at = datetime('now')",
    );

    qb.build().execute(pool).await?;
    Ok(())
}

pub async fn get_event_prefs(
    pool: &SqlitePool,
    user_id: &str,
    event_type: &str,
) -> sqlx::Result<Vec<EventPref>> {
    if user_id.is_empty() || event_type.is_empty() {
        return Err(sqlx::Error::Protocol(
            "user_id and event_type cannot be empty".into(),
        ));
    }

    let rows = sqlx::query!(
        r#"
        SELECT event_type,
               channel AS "channel: Channel",
               enabled AS "enabled: bool"
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
///   1. Start with all channels enabled by default.
///   2. Apply global channel defaults.
///   3. Apply per-event-type overrides on top.
pub async fn resolve_channels(
    pool: &SqlitePool,
    user_id: &str,
    event_type: &str,
) -> sqlx::Result<Vec<Channel>> {
    if user_id.is_empty() || event_type.is_empty() {
        return Err(sqlx::Error::Protocol(
            "user_id and event_type cannot be empty".into(),
        ));
    }

    let defaults = get_channel_defaults(pool, user_id).await?;
    let overrides = get_event_prefs(pool, user_id, event_type).await?;

    let mut channel_map: HashMap<&str, bool> = Channel::all()
        .iter()
        .map(|c| (c.as_str(), true))
        .collect();

    for d in &defaults {
        channel_map.insert(d.channel.as_str(), d.enabled);
    }

    for o in &overrides {
        channel_map.insert(o.channel.as_str(), o.enabled);
    }

    Ok(Channel::all()
        .iter()
        .filter(|c| *channel_map.get(c.as_str()).unwrap_or(&true))
        .cloned()
        .collect())
}

/// Batch resolve multiple event types for a single user efficiently.
///
/// Loads global defaults and all event prefs in two queries total,
/// regardless of how many event types are requested.
/// We fetch all user prefs rather than filtering by event_type list to avoid
/// a dynamic IN clause — users typically have few prefs total.
pub async fn resolve_channels_batch(
    pool: &SqlitePool,
    user_id: &str,
    event_types: &[String],
) -> sqlx::Result<HashMap<String, Vec<Channel>>> {
    if user_id.is_empty() {
        return Err(sqlx::Error::Protocol("user_id cannot be empty".into()));
    }

    if event_types.is_empty() {
        return Ok(HashMap::new());
    }

    let defaults = get_channel_defaults(pool, user_id).await?;

    // Build base channel map from global defaults
    let base_map: HashMap<&str, bool> = {
        let mut map: HashMap<&str, bool> = Channel::all()
            .iter()
            .map(|c| (c.as_str(), true))
            .collect();
        for d in &defaults {
            map.insert(d.channel.as_str(), d.enabled);
        }
        map
    };

    // Load all event prefs for this user in one query
    let rows = sqlx::query!(
        r#"
        SELECT event_type,
               channel AS "channel: Channel",
               enabled AS "enabled: bool"
        FROM notification_event_prefs
        WHERE user_id = ?
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let mut all_prefs: HashMap<String, Vec<EventPref>> = HashMap::new();
    for row in rows {
        all_prefs
            .entry(row.event_type.clone())
            .or_default()
            .push(EventPref {
                event_type: row.event_type,
                channel: row.channel,
                enabled: row.enabled,
            });
    }

    // Resolve each requested event type against base map + its overrides
    let mut results = HashMap::new();

    for event_type in event_types {
        let mut channel_map = base_map.clone();

        if let Some(overrides) = all_prefs.get(event_type) {
            for o in overrides {
                channel_map.insert(o.channel.as_str(), o.enabled);
            }
        }

        let enabled: Vec<Channel> = Channel::all()
            .iter()
            .filter(|c| *channel_map.get(c.as_str()).unwrap_or(&true))
            .cloned()
            .collect();

        results.insert(event_type.clone(), enabled);
    }

    Ok(results)
}
