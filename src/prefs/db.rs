use super::channel::Channel;
use anyhow::Result;
use moka::future::Cache;
use sqlx::{Pool, Sqlite};

// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct Defaults {
    db: Pool<Sqlite>,
    cache: Cache<String, Channel>, // subject -> Channel
}

impl Defaults {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self {
            db,
            cache: Cache::new(100),
        }
    }

    pub async fn set(&self, subject: &str, channel: Channel) -> Result<()> {
        sqlx::query(
            "INSERT INTO defaults (subject, channel)
             VALUES (?, ?)
             ON CONFLICT(subject) DO UPDATE SET channel = excluded.channel",
        )
        .bind(subject)
        .bind(channel)
        .execute(&self.db)
        .await?;

        self.cache.insert(subject.to_string(), channel).await;
        Ok(())
    }

    pub async fn get(&self, subject: &str) -> Result<Option<Channel>> {
        if let Some(cached) = self.cache.get(subject).await {
            return Ok(Some(cached));
        }

        let result =
            sqlx::query_scalar::<_, Channel>("SELECT channel FROM defaults WHERE subject = ?")
                .bind(subject)
                .fetch_optional(&self.db)
                .await?;

        if let Some(channel) = result {
            self.cache.insert(subject.to_string(), channel).await;
        }

        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Preferences
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct Preferences {
    db: Pool<Sqlite>,
    cache: Cache<(String, String), Channel>, // (user, subject) -> Channel
    defaults: Defaults,
}

impl Preferences {
    pub fn new(db: Pool<Sqlite>, defaults: Defaults) -> Self {
        Self {
            db,
            cache: Cache::new(1000),
            defaults,
        }
    }

    pub async fn set(&self, user: &str, subject: &str, channel: Channel) -> Result<()> {
        sqlx::query(
            "INSERT INTO preferences (user, subject, channel)
             VALUES (?, ?, ?)
             ON CONFLICT(user, subject)
             DO UPDATE SET channel = excluded.channel",
        )
        .bind(user)
        .bind(subject)
        .bind(channel)
        .execute(&self.db)
        .await?;

        self.cache
            .insert((user.to_string(), subject.to_string()), channel)
            .await;

        Ok(())
    }

    pub async fn get(&self, user: &str, subject: &str) -> Result<Option<Channel>> {
        let key = (user.to_string(), subject.to_string());

        if let Some(cached) = self.cache.get(&key).await {
            return Ok(Some(cached));
        }

        let result = sqlx::query_scalar::<_, Channel>(
            "SELECT channel FROM preferences WHERE user = ? AND subject = ?",
        )
        .bind(user)
        .bind(subject)
        .fetch_optional(&self.db)
        .await?;

        if let Some(channel) = result {
            self.cache.insert(key, channel).await;
            return Ok(Some(channel));
        }

        // Fallback to subject default.
        self.defaults.get(subject).await
    }
}
