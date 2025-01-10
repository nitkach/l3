use std::sync::{Arc, Mutex};

use anyhow::Result;
use bcrypt::DEFAULT_COST;
use tokio_postgres::Row;
use ulid::{Generator, Ulid};

use crate::model::{Claims, CreatePostRequest, LoginRequest, RegisterRequest};

#[derive(Clone)]
pub(crate) struct Repository {
    pool: deadpool_postgres::Pool,
    ulid: Arc<Mutex<Generator>>,
}

#[derive(Debug)]
pub(crate) struct DatabaseUser {
    pub(crate) id: Ulid,
    pub(crate) login: String,
    pub(crate) password_hash: String,
}

impl TryFrom<Row> for DatabaseUser {
    type Error = anyhow::Error;

    fn try_from(row: Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            login: row.try_get("login")?,
            password_hash: row.try_get("password_hash")?,
        })
    }
}

#[derive(Debug)]
pub(crate) struct DatabasePost {
    pub(crate) id: Ulid,
    pub(crate) user_id: Ulid,
    pub(crate) content: String,
    pub(crate) likes: u32,
}

impl TryFrom<Row> for DatabasePost {
    type Error = anyhow::Error;

    fn try_from(row: Row) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            content: row.try_get("content")?,
            likes: row.try_get("likes")?,
        })
    }
}

impl Repository {
    pub(crate) async fn initialize() -> Result<Self> {
        let mut config = deadpool_postgres::Config::new();

        config.host = Some(std::env::var("DB_HOST")?);
        config.port = Some(std::env::var("DB_PORT")?.parse::<u16>()?);
        config.user = Some(std::env::var("DB_USER")?);
        config.password = Some(std::env::var("DB_USER_PASSWORD")?);
        config.dbname = Some(std::env::var("DB_NAME")?);

        let pool = config.create_pool(
            Some(deadpool_postgres::Runtime::Tokio1),
            tokio_postgres::NoTls,
        )?;

        let ulid = Arc::new(Mutex::new(ulid::Generator::new()));

        pool.get()
            .await?
            .batch_execute(
                "
                create table if not exists users (
                               id uuid primary key,
                            login text      unique not null,
                    password_hash text             not null
                );
                create table if not exists posts (
                         id   uuid primary key,
                    user_id   uuid references users (id) on delete cascade,
                    content   text not null,
                      likes serial not null
                );
                ",
            )
            .await?;

        Ok(Self { pool, ulid })
    }

    pub(crate) async fn register_user(&self, user_data: RegisterRequest) -> Result<Ulid> {
        let ulid = {
            let mut generator = self.ulid.lock().unwrap();
            loop {
                if let Ok(ulid) = generator.generate() {
                    break ulid;
                }
                std::thread::yield_now();
            }
        };

        let conn = self.pool.get().await?;

        let RegisterRequest { login, password } = user_data;
        let password_hash = bcrypt::hash(password, DEFAULT_COST)?;

        let query = "insert into users (id, login, password_hash) values ($1, $2, $3);";
        conn.execute(query, &[&ulid.to_string(), &login, &password_hash])
            .await?;

        Ok(ulid)
    }

    pub(crate) async fn login_user(
        &self,
        user_data: &LoginRequest,
    ) -> Result<Option<DatabaseUser>> {
        let conn = self.pool.get().await?;

        let query = "select id from users where login = $1 and password = $2;";
        let Some(row) = conn
            .query_opt(query, &[&user_data.login, &user_data.password])
            .await?
        else {
            return Ok(None);
        };

        let user = DatabaseUser::try_from(row)?;

        Ok(Some(user))
    }

    pub(crate) async fn create_post(
        &self,
        post: CreatePostRequest,
        claims: Claims,
    ) -> Result<Ulid> {
        let ulid = {
            let mut generator = self.ulid.lock().unwrap();
            loop {
                if let Ok(ulid) = generator.generate() {
                    break ulid;
                }
                std::thread::yield_now();
            }
        };

        let conn = self.pool.get().await?;

        let query = "insert into posts (id, user_id, content, likes) values ($1, $2, $3, $4);";
        conn.execute(
            query,
            &[&ulid.to_string(), &claims.sub, &post.content, &"0"],
        )
        .await?;

        Ok(ulid)
    }

    pub(crate) async fn get_post(&self, ulid: Ulid) -> Result<Option<DatabasePost>> {
        let conn = self.pool.get().await?;

        let query = "select id, user_id, content, likes from posts where id = $1;";
        let Some(row) = conn.query_opt(query, &[&ulid.to_string()]).await? else {
            return Ok(None);
        };

        let post = DatabasePost::try_from(row)?;

        Ok(Some(post))
    }

    pub(crate) async fn delete_post(&self, post_id: Ulid, claims: Claims) -> Result<bool> {
        let conn = self.pool.get().await?;

        let query = "delete from posts where id = $1 and user_id = $2;";
        let rows_deleted = conn.execute(query, &[&post_id, &claims.sub]).await?;

        Ok(rows_deleted == 1)
    }

    pub(crate) async fn like_post(&self, post_id: Ulid) -> Result<bool> {
        let conn = self.pool.get().await?;

        let query = "update posts set likes = likes + 1 where id = $1;";
        let rows_updated = conn.execute(query, &[&post_id]).await?;

        Ok(rows_updated == 1)
    }
}
