use anyhow::{Context, Result};
use sqlx::SqlitePool;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use tokio::sync::OnceCell;

static POOL: tokio::sync::OnceCell<SqlitePool> = OnceCell::const_new();
const DB_URL: &str = "sqlite:///tmp/corchat.db";

async fn db() -> &'static SqlitePool {
    POOL.get_or_init(|| async {
        if !sqlx::Sqlite::database_exists(DB_URL).await.unwrap() {
            sqlx::Sqlite::create_database(DB_URL).await.unwrap();
        }
        let pool = SqlitePoolOptions::new()
            .connect(DB_URL)
            .await
            .expect("failed to open database");
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS channels (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            );
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel INTEGER NOT NULL,
                user TEXT NOT NULL,
                content TEXT NOT NULL,

                FOREIGN KEY(channel) REFERENCES channels(id)
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("failed to run migrations");

        pool
    })
    .await
}

pub async fn drop() -> Result<()> {
    if sqlx::Sqlite::database_exists(DB_URL).await? {
        sqlx::Sqlite::drop_database(DB_URL).await?;
    }
    Ok(())
}

#[derive(sqlx::FromRow, Clone)]
pub struct Channel {
    id: i32,
    name: String,
}

impl Channel {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn send_message(&self, username: &str, content: &str) -> Result<()> {
        let db = db().await;
        sqlx::query(r"INSERT INTO messages (channel, user, content) VALUES (?, ?, ?)")
            .bind(self.id)
            .bind(username)
            .bind(content)
            .execute(db)
            .await
            .context("sending message")?;
        Ok(())
    }

    pub async fn get_messages(&self, since: Option<i32>) -> Result<Vec<Message>> {
        let db = db().await;
        let results = if let Some(since) = since {
            sqlx::query_as(
                r"SELECT id, channel, user, content FROM messages WHERE channel = ? AND id > ?",
            )
            .bind(self.id)
            .bind(since)
            .fetch_all(db)
            .await?
        } else {
            sqlx::query_as(r#"SELECT id, channel, user, content FROM messages WHERE channel = ?"#)
                .bind(self.id)
                .fetch_all(db)
                .await?
        };
        Ok(results)
    }
}

#[derive(sqlx::FromRow, Clone)]
pub struct Message {
    id: i32,
    #[allow(dead_code)]
    channel: i32,
    user: String,
    content: String,
}

impl Message {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

pub async fn get_channels() -> Result<Vec<Channel>> {
    let db = db().await;
    let results = sqlx::query_as(r"SELECT id, name FROM channels")
        .fetch_all(db)
        .await
        .context("getting channels")?;
    Ok(results)
}

/// Add a new channel, returning the inserted channel ID and name
pub async fn add_channel(name: &str) -> Result<Channel> {
    let db = db().await;
    let result = sqlx::query_as(r"INSERT INTO channels (name) VALUES (?) RETURNING *")
        .bind(name)
        .fetch_one(db)
        .await
        .context("adding channel")?;
    Ok(result)
}
