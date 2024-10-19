use std::borrow::Cow;
use sqlx::{self, sqlite};
use dataset::Dataset;
use crate::*;


pub struct SqliteDataset {
    pool: sqlite::SqlitePool,
}

impl SqliteDataset {
    pub fn pool(&self) -> &sqlite::SqlitePool {
        &self.pool
    }
     
    pub async fn standard_get<'d:'m,'m, M>(&'d self, id: ID) -> Result<Option<Cow<'m, M>>>  
    where
        M: DatasetModel<Self> + 'm,
        for<'r> M: sqlx::FromRow<'r, sqlite::SqliteRow> + Unpin + 'r,
    {
        let table = M::SCHEMA_NAME;
        let result: sqlx::Result<M> = sqlx::query_as::<_, M>(&format!("SELECT * FROM {table} WHERE id = ? LIMIT 1"))
            .bind(id.bind_online())
            .fetch_one(&self.pool)
            .await;

        match result {
            Ok(m) => Ok(Some(Cow::Owned(m))),
            Err(e ) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                _ => Err(Error::Database(e.to_string()))
            }
        }
    }
}

impl Dataset for SqliteDataset {
    async fn gets<'d:'m, 'm, M>(&'d self, id: ID) -> Result<Option<Cow<'m, M>>>
    where
        Self: Sized + 'd,
        M: MetaModel + DatasetModel<Self> + 'm {
        todo!()
    }
}


