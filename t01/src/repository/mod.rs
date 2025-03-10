use anyhow::Result;
use log::info;
use refinery::embed_migrations;
use serde::Serialize;
use tokio_postgres::Row;

use crate::{error::AppError, utils::PasswordHash};

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

        info!("Transaction for register started");

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

        info!("Transaction for register successfully ended");

        Ok(user_id)
    }

    pub(crate) async fn get_login_credentials(
        &self,
        username: &str,
    ) -> Result<Option<DatabaseUser>> {
        info!("Verifying user credentials");
        let mut connection = self.pool.get().await?;

        info!("Transaction for login credentials started");

        let transaction = connection.transaction().await?;

        let query = "
            select user_id, username, password_hash, created_at
            from users
            where username = $1;
        ";
        let Some(row) = transaction.query_opt(query, &[&username]).await? else {
            info!("User not found in database with username: '{username}'");
            return Ok(None);
        };

        info!("User found in database with username: '{username}'");
        let user = DatabaseUser::try_from(row)?;

        transaction.commit().await?;
        info!("Transaction for user credentials successfully ended");

        Ok(Some(user))
    }

    pub(crate) async fn get_posts(&self) -> Result<Vec<DatabasePost>> {
        let mut connection = self.pool.get().await?;

        info!("Transaction for listing posts started");

        let transaction = connection.transaction().await?;

        let query = "
            select
                p.post_id,
                p.user_id,
                u.username,
                p.title,
                p.content,
                p.created_at,
                count(l.like_id) as likes_count
                from posts p
            join users u on p.user_id = u.user_id
            left join likes l on p.post_id = l.post_id
            group by p.post_id, p.user_id, u.username, p.title, p.content, p.created_at
            order by p.created_at desc;
        ";
        let rows = transaction.query(query, &[]).await?;

        let posts = rows
            .into_iter()
            .map(DatabasePost::try_from)
            .collect::<Result<Vec<_>>>()?;

        transaction.commit().await?;

        info!("Transaction for listing posts successfully ended");

        Ok(posts)
    }

    pub(crate) async fn get_post(&self, post_id: i32) -> Result<Option<DatabasePost>> {
        let mut connection = self.pool.get().await?;

        info!("Transaction for getting post with id = {post_id} started");

        let transaction = connection.transaction().await?;

        let query = "
        select
                p.post_id,
                p.user_id,
                u.username,
                p.title,
                p.content,
                p.created_at,
                count(l.like_id) as likes_count
                from posts p
                join users u on p.user_id = u.user_id
            left join likes l on p.post_id = l.post_id
            where p.post_id = $1
            group by p.post_id, p.user_id, u.username, p.title, p.content, p.created_at;
        ";
        let row = transaction.query_opt(query, &[&post_id]).await?;

        let post = row.map(DatabasePost::try_from).transpose()?;

        transaction.commit().await?;

        info!("Transaction for getting post with id = {post_id} successfully ended");

        Ok(post)
    }

    pub(crate) async fn create_post(
        &self,
        user_id: i32,
        title: &str,
        content: &str,
    ) -> Result<i32> {
        let mut connection = self.pool.get().await?;

        info!("Transaction for creating post by user with id = {user_id} started");

        let transaction = connection.transaction().await?;

        let query = "
            insert into posts (user_id, title, content)
            values ($1, $2, $3)
            returning post_id;
        ";
        let row = transaction
            .query_one(query, &[&user_id, &title, &content])
            .await?;

        let post_id: i32 = row.try_get(0)?;

        transaction.commit().await?;

        info!("Transaction for creating post by user with id = {user_id} successfully ended");

        Ok(post_id)
    }

    pub(crate) async fn like_post(&self, user_id: i32, post_id: i32) -> Result<Like> {
        let mut connection = self.pool.get().await?;

        info!("Transaction for liking/disliking post (id = {post_id}) by user (id = {user_id}) started");

        let transaction = connection.transaction().await?;

        let query = "
            select like_id
            from likes
            where user_id = $1 and post_id = $2;
        ";
        let row = transaction.query_opt(query, &[&user_id, &post_id]).await?;

        if row.is_some() {
            let delete_query = "
                delete from likes
                where user_id = $1 and post_id = $2;
            ";
            transaction
                .execute(delete_query, &[&user_id, &post_id])
                .await?;
            transaction.commit().await?;

            info!("Transaction for liking/disliking post (id = {post_id}) by user (id = {user_id}) successfully ended");

            Ok(Like::Removed)
        } else {
            let insert_query = "
                insert into likes (user_id, post_id)
                values ($1, $2);
            ";
            transaction
                .execute(insert_query, &[&user_id, &post_id])
                .await?;

            transaction.commit().await?;

            info!("Transaction for liking/disliking post (id = {post_id}) by user (id = {user_id}) successfully ended");

            Ok(Like::Added)
        }
    }

    pub(crate) async fn get_like_count(&self, post_id: i32) -> Result<i64> {
        let mut connection = self.pool.get().await?;

        info!("Transaction for obtaining likes count for post (id = {post_id}) started");

        let transaction = connection.transaction().await?;

        let query = "
            select count(*)
            from likes
            where post_id = $1;
        ";

        let row = transaction.query_one(query, &[&post_id]).await?;

        let likes_count: i64 = row.try_get(0)?;

        transaction.commit().await?;

        info!("Transaction for obtaining likes count for post (id = {post_id}) successfully ended");

        Ok(likes_count)
    }

    pub(crate) async fn delete_post(&self, post_id: i32, user_id: i32) -> Result<PostDeleteResult> {
        let mut connection = self.pool.get().await?;

        info!("Transaction for deleting post (id = {post_id}) by user (id = {user_id}) started");

        let transaction = connection.transaction().await?;

        let query = "
            select user_id
            from posts
            where post_id=$1;
        ";
        let row = transaction.query_opt(query, &[&post_id]).await?;
        let rows_owned = match row {
            Some(row) => {
                let post_user_id: i32 = row.try_get("user_id")?;
                if post_user_id == user_id {
                    1 // post exists and is owned by the user
                } else {
                    0 // post exists but is not owned by the user
                }
            }
            None => 0, // post does not exist
        };

        let query = "
            delete from posts
            where post_id=$1 and user_id=$2
            returning post_id;
        ";
        let row = transaction.query_opt(query, &[&post_id, &user_id]).await?;
        let rows_deleted = match row {
            Some(_) => 1, // Post was deleted
            None => 0,    // Post was not deleted
        };

        let delete_result = match (rows_owned, rows_deleted) {
            (0, 0) => PostDeleteResult::NotFound,
            (0, 1) => unreachable!("Wrong state: post is not found, but was deleted"),
            (1, 0) => PostDeleteResult::NotOwned,
            (1, 1) => PostDeleteResult::Deleted,
            (_, _) => unreachable!(),
        };

        transaction.commit().await?;

        info!("Transaction for deleting post (id = {post_id}) by user (id = {user_id}) successfully ended");

        Ok(delete_result)
    }

    pub(crate) async fn get_user_posts(&self, user_id: i32) -> Result<Vec<DatabasePost>> {
        let mut connection = self.pool.get().await?;

        info!("Transaction for listing user posts started");

        let transaction = connection.transaction().await?;

        let query = "
            select
                p.post_id,
                p.user_id,
                u.username,
                p.title,
                p.content,
                p.created_at,
                count(l.like_id) as likes_count
                from posts p
            join users u on p.user_id = u.user_id
            left join likes l on p.post_id = l.post_id
            where p.user_id = $1
            group by p.post_id, p.user_id, u.username, p.title, p.content, p.created_at
            order by p.created_at desc;
        ";
        let rows = transaction.query(query, &[&user_id]).await?;

        let posts = rows
            .into_iter()
            .map(DatabasePost::try_from)
            .collect::<Result<Vec<_>>>()?;

        transaction.commit().await?;

        info!("Transaction for listing user posts successfully ended");

        Ok(posts)
    }

    pub(crate) async fn get_username_by_user_id(&self, user_id: i32) -> Result<Option<String>> {
        let mut connection = self.pool.get().await?;

        info!("Transaction for listing user posts started");

        let transaction = connection.transaction().await?;

        let query = "
            select username
            from users
            where user_id = $1;
        ";
        let Some(row) = transaction.query_opt(query, &[&user_id]).await? else {
            return Ok(None);
        };

        let username: String = row.try_get("username")?;

        transaction.commit().await?;

        info!("Transaction for listing user posts successfully ended");

        Ok(Some(username))
    }
}

#[derive(Debug, Serialize)]
pub(crate) enum Like {
    Added,
    Removed,
}

pub(crate) enum PostDeleteResult {
    Deleted,
    NotFound,
    NotOwned,
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

#[derive(Debug, Serialize)]
pub(crate) struct DatabasePost {
    pub(crate) post_id: i32,
    pub(crate) user_id: i32,
    pub(crate) username: String,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) created_at: chrono::NaiveDateTime,
    pub(crate) likes_count: i64,
}

impl TryFrom<Row> for DatabasePost {
    type Error = anyhow::Error;

    fn try_from(row: Row) -> Result<Self> {
        Ok(Self {
            post_id: row.try_get("post_id")?,
            user_id: row.try_get("user_id")?,
            username: row.try_get("username")?,
            title: row.try_get("title")?,
            content: row.try_get("content")?,
            created_at: row.try_get("created_at")?,
            likes_count: row.try_get("likes_count")?,
        })
    }
}
