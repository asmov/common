use std::borrow::Cow;
use sqlx::{self, postgres};
use dataset::Dataset;
use crate::*;

pub struct PostgresDataset {
    authorative: bool, //harcoded to true
    pool: postgres::PgPool,
}

const FIELD_ID: &str = "id";
const FIELD_LOCAL_ID: &str = "local_id";

impl PostgresDataset {
    pub fn new(pool: postgres::PgPool) -> Self {
        Self { authorative: true, pool }
    }

    pub fn pool(&self) -> &postgres::PgPool {
        &self.pool
    }

    /// Performs a SELECT query for a single row.  
    /// Expects ID to be valid.  
    /// Expects an authorative ID if this dataset is authorative (currently always true for PostgreSQL).
    pub async fn standard_get<'d:'m, 'm, M>(&'d self, id: ID) -> Result<Option<Cow<'m, M>>>  
    where
        M: DatasetModel<Self> + 'm,
        for<'r> M: sqlx::FromRow<'r, postgres::PgRow> + Unpin + 'r,
    {
        let table = M::SCHEMA_NAME;
        let (id_field, id) = match id.into_option() {
            OptionID::None => panic!("Cannot PostgreSQL SELECT for an invalid ID"),
            OptionID::Some(ID::Authorative(id)) | OptionID::Some(ID::Mutual(_, id)) => (FIELD_ID, id as i64),
            OptionID::Some(ID::Local(_)) if self.authorative => panic!("Cannot PostgreSQL SELECT for a local ID from an authorative dataset"),
            OptionID::Some(ID::Local(id)) => (FIELD_LOCAL_ID, id as i64),
            OptionID::Reserved => panic!("Cannot PostgreSQL INSERT or UPDATE for a reserved ID"),
        };

        let result = sqlx::query_as::<_, M>(&format!("SELECT * FROM {table} WHERE {id_field} = ? LIMIT 1"))
            .bind(id)
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

impl Dataset for PostgresDataset {}


