use std::borrow::Cow;
use sqlx::{self, postgres, Executor};
use dataset::Dataset;
use crate::*;


pub struct PostgresDataset {
    pool: postgres::PgPool,
}

impl PostgresDataset {
    pub fn new(pool: postgres::PgPool) -> Self {
        Self {
            pool,
        }
    }

    pub fn pool(&self) -> &postgres::PgPool {
        &self.pool
    }

    pub async fn standard_get<'d:'m, 'm, M>(&'d self, id: ID) -> Result<Option<Cow<'m, M>>>  
    where
        M: DatasetModel<Self> + 'm,
        for<'r> M: sqlx::FromRow<'r, postgres::PgRow> + Unpin + 'r,
    {
        let table = M::SCHEMA_NAME;
        let result = sqlx::query_as::<_, M>(&format!("SELECT * FROM {table} WHERE id = ? LIMIT 1"))
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

impl Dataset for PostgresDataset {
    async fn gets<'d:'m, 'm, M>(&'d self, id: ID) -> Result<Option<Cow<'m, M>>>
    where
        Self: Sized + 'd,
        M: MetaModel + DatasetModel<Self> + 'm {
        todo!()
    }
}


