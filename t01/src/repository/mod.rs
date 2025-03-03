use anyhow::Result;
use log::info;
use refinery::embed_migrations;
use tokio_postgres::Row;
use ulid::Ulid;

use crate::{
    error::AppError,
    model::{Claims, CreatePostRequest, LoginRequest},
    utils::PasswordHash,
};

#[derive(Clone)]
pub(crate) struct Repository {
    pool: deadpool_postgres::Pool,
}

embed_migrations!("migrations");

impl Repository {
    pub(crate) async fn initialize(config: deadpool_postgres::Config) -> Result<Self> {
        info!("Initializing repository");
        info!("Trying to connect to database...");
        let pool = config.create_pool(
            Some(deadpool_postgres::Runtime::Tokio1),
            tokio_postgres::NoTls,
        )?;
        info!("Connection successfull");

        info!("Applying database migrations...");
        let mut connection = pool.get().await?;
        let migration_report = migrations::runner().run_async(&mut **connection).await?;
        info!("Migrations applied successfully: {:?}", migration_report);

        Ok(Self { pool })
    }

    pub(crate) async fn register_user(
        &self,
        username: &str,
        password_hash: PasswordHash,
    ) -> Result<i32, AppError> {
        let mut connection = self.pool.get().await?;

        let transaction = connection.transaction().await?;

        let check_query = "
            select user_id
            from users
            where username = $1;
        ";
        let existing_user = transaction.query_opt(check_query, &[&username]).await?;

        if existing_user.is_some() {
            return Err(AppError::user_already_exist());
        }

        let query = "
            insert into users (username, password_hash)
            values ($1, $2)
            returning user_id;
        ";
        let row = transaction
            .query_one(query, &[&username, &password_hash.as_str()])
            .await?;

        let user_id: i32 = row.try_get(0)?;

        transaction.commit().await?;

        Ok(user_id)
    }

    pub(crate) async fn get_login_credentials(
        &self,
        username: &str,
    ) -> Result<Option<DatabaseUser>> {
        let mut connection = self.pool.get().await?;

        let transaction = connection.transaction().await?;

        let query = "
            select (user_id, username, password_hash, created_at)
            from users
            where username = $1;
        ";
        let Some(row) = transaction.query_opt(query, &[&username]).await? else {
            return Ok(None);
        };

        let user = DatabaseUser::try_from(row)?;

        transaction.commit().await?;

        Ok(Some(user))
    }

    pub(crate) async fn create_post(
        &self,
        post: CreatePostRequest,
        claims: Claims,
    ) -> Result<Ulid> {
        // let conn = self.pool.get().await?;

        // let query = "insert into posts (id, user_id, content, likes) values ($1, $2, $3, $4);";
        // conn.execute(
        //     query,
        //     &[&ulid.to_string(), &claims.sub, &post.content, &"0"],
        // )
        // .await?;

        // Ok(ulid)
        todo!()
    }

    pub(crate) async fn get_post(&self, ulid: Ulid) -> Result<Option<DatabasePost>> {
        // let conn = self.pool.get().await?;

        // let query = "select id, user_id, content, likes from posts where id = $1;";
        // let Some(row) = conn.query_opt(query, &[&ulid.to_string()]).await? else {
        //     return Ok(None);
        // };

        // let post = DatabasePost::try_from(row)?;

        // Ok(Some(post))
        todo!()
    }

    pub(crate) async fn delete_post(&self, post_id: Ulid, claims: Claims) -> Result<bool> {
        // let conn = self.pool.get().await?;

        // let query = "delete from posts where id = $1 and user_id = $2;";
        // let rows_deleted = conn.execute(query, &[&post_id, &claims.sub]).await?;

        // Ok(rows_deleted == 1)
        todo!()
    }

    pub(crate) async fn like_post(&self, post_id: Ulid) -> Result<bool> {
        // let conn = self.pool.get().await?;

        // let query = "update posts set likes = likes + 1 where id = $1;";
        // let rows_updated = conn.execute(query, &[&post_id]).await?;

        // Ok(rows_updated == 1)
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct DatabaseUser {
    pub(crate) user_id: i32,
    pub(crate) username: String,
    pub(crate) password_hash: PasswordHash,
    pub(crate) created_at: chrono::NaiveDateTime,
}

impl TryFrom<Row> for DatabaseUser {
    type Error = anyhow::Error;

    fn try_from(row: Row) -> Result<Self> {
        Ok(Self {
            user_id: row.try_get("user_id")?,
            username: row.try_get("username")?,
            password_hash: PasswordHash::try_from(&row)?,
            created_at: row.try_get("created_at")?,
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
