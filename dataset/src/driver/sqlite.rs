use std::borrow::Cow;
use sqlx::{self, sqlite, Row};
use dataset::Dataset;
use crate::*;


pub struct SqliteDataset {
    local: bool, // harcoded to true
    pool: sqlite::SqlitePool,
}

impl SqliteDataset {
    pub fn new(pool: sqlite::SqlitePool) -> Self {
        Self { local: true, pool }
    }

    pub fn pool(&self) -> &sqlite::SqlitePool {
        &self.pool
    }
     
    /// Performs a SELECT query for a single row.  
    /// Expects a valid ID.  
    /// Expects an authorative ID if this dataset is authorative (currently impossible for SQLite).
    pub async fn standard_get<'d:'m,'m, M>(&'d self, id: ID) -> Result<Option<Cow<'m, M>>>  
    where
        M: DatasetModel<Self> + 'm,
        for<'r> M: sqlx::FromRow<'r, sqlite::SqliteRow> + Unpin + 'r,
    {
        let table = M::SCHEMA_NAME;
        let querystr;

        match id.into_option() {
            OptionID::None => { panic!("Cannot SQLite SELECT for an invalid ID"); },
            OptionID::Some(ID::Authorative(_)) | OptionID::Some(ID::Mutual(_, _)) => {
                querystr = format!("SELECT * FROM {table} WHERE id = ? LIMIT 1");
            },
            OptionID::Some(ID::Local(_)) if !self.local => panic!("Cannot SQLite SELECT for a local ID from a non-local dataset"),
            OptionID::Some(ID::Local(_)) => { querystr = format!("SELECT * FROM {table} WHERE local_id = ? LIMIT 1"); },
            OptionID::Reserved => panic!("Cannot SQLite INSERT or UPDATE for a reserved ID"),
        }

        let result: sqlx::Result<M> = sqlx::query_as::<_, M>(&querystr)
            .bind(id.sql())
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

    /// Performs an UPDATE or INSERT operation depending on whether the model has a valid ID or not, respectively.
    /// Expects the SQLite table to have a column named "local_id" if this dataset is local (currently always true for SQLite).
    /// Returns an ID if INSERTed.
    pub async fn standard_insert_or_update<'d:'m,'m:'q,'q, M>(&'d self, model: &'m M) -> Result<Option<ID>>  
    where
        M: DatasetModel<Self> + ToArguments<sqlx::sqlite::Sqlite>
    {
        let table = M::SCHEMA_NAME;
        let (arguments, num_args) = model.to_arguments().map_err(|e| Error::Database(e.to_string()))?;
        let placement = (0..num_args).map(|_| "?").collect::<Vec<&str>>().join(", ");
        let mut need_id = false;
        let querystr;

        match model.id().into_option() {
            OptionID::None => {
                querystr = format!("INSERT INTO {table} VALUES ({placement})");
                println!("querystr: {}", querystr);
                need_id = true;
            },
            OptionID::Some(ID::Authorative(_)) | OptionID::Some(ID::Mutual(_, _)) => {
                querystr = format!("INSERT INTO {table} SET VALUES ({placement}) ON CONFLICT(id) DO UPDATE SET VALUES ({placement}) WHERE id = ? LIMIT 1");
                println!("querystr: {}", querystr);
            },
            #[cfg(debug_assertions)]
            OptionID::Some(ID::Local(_)) if !self.local => panic!("Cannot SQLite UPDATE for a local ID from a non-local dataset"),
            OptionID::Some(ID::Local(_)) => {
                querystr = format!("INSERT INTO {table} SET VALUES ({placement}) ON CONFLICT(id) DO UPDATE SET VALUES ({placement}) WHERE id = ? LIMIT 1");
                println!("querystr: {}", querystr);
            },
            OptionID::Reserved => panic!("Cannot SQLite INSERT or UPDATE for a reserved ID"),
        }

        let query = sqlx::query_with(&querystr, arguments);
        match query.execute(&self.pool).await {
            Ok(_) => {
                if need_id {
                    sqlx::query("SELECT last_insert_rowid() as id")
                        .fetch_one(&self.pool)
                        .await
                        .map(|row| row.try_get::<i64, _>("id"))
                        .map_err(|e| Error::from(e))?
                        .map(|id| Ok(Some(ID::Local(id as u64))))?
                } else {
                    Ok(None)
                }
            }
            Err(e ) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                _ => Err(Error::Database(e.to_string()))
            }
        }
    }

}

impl Dataset for SqliteDataset {}


