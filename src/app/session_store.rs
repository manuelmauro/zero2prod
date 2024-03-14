use std::fmt::Debug;

use async_trait::async_trait;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use time::OffsetDateTime;
use tower_sessions::{
    session::{Id, Record},
    session_store, SessionStore,
};

#[derive(Debug, thiserror::Error)]
pub enum RedisStoreError {
    #[error(transparent)]
    Redis(#[from] redis::RedisError),

    #[error(transparent)]
    Decode(#[from] rmp_serde::decode::Error),

    #[error(transparent)]
    Encode(#[from] rmp_serde::encode::Error),
}

impl From<RedisStoreError> for session_store::Error {
    fn from(err: RedisStoreError) -> Self {
        match err {
            RedisStoreError::Redis(inner) => session_store::Error::Backend(inner.to_string()),
            RedisStoreError::Decode(inner) => session_store::Error::Decode(inner.to_string()),
            RedisStoreError::Encode(inner) => session_store::Error::Encode(inner.to_string()),
        }
    }
}

/// A Redis session store.
#[derive(Debug, Clone, Default)]
pub struct RedisStore<C: Send + Sync> {
    client: C,
}

impl RedisStore<bb8::Pool<RedisConnectionManager>> {
    /// Create a new Redis store with the provided client.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    ///
    /// ```
    pub fn new(client: bb8::Pool<RedisConnectionManager>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl SessionStore for RedisStore<bb8::Pool<RedisConnectionManager>> {
    async fn save(&self, record: &Record) -> session_store::Result<()> {
        self.client
            .get()
            .await
            .map_err(|_| {
                session_store::Error::Backend("Can't get a connection from the pool".to_owned())
            })?
            .set(
                record.id.to_string(),
                rmp_serde::to_vec(&record)
                    .map_err(RedisStoreError::Encode)?
                    .as_slice(),
            )
            .await
            .map_err(RedisStoreError::Redis)?;

        self.client
            .get()
            .await
            .map_err(|_| {
                session_store::Error::Backend("Can't get a connection from the pool".to_owned())
            })?
            .expire_at(
                record.id.to_string(),
                OffsetDateTime::unix_timestamp(record.expiry_date),
            )
            .await
            .map_err(RedisStoreError::Redis)?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let data = self
            .client
            .get()
            .await
            .map_err(|_| {
                session_store::Error::Backend("Can't get a connection from the pool".to_owned())
            })?
            .get::<String, Option<Vec<u8>>>(session_id.to_string())
            .await
            .map_err(RedisStoreError::Redis)?;

        if let Some(data) = data {
            Ok(Some(
                rmp_serde::from_slice(&data).map_err(RedisStoreError::Decode)?,
            ))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        self.client
            .get()
            .await
            .map_err(|_| {
                session_store::Error::Backend("Can't get a connection from the pool".to_owned())
            })?
            .del(session_id.to_string())
            .await
            .map_err(RedisStoreError::Redis)?;
        Ok(())
    }
}
