use derive_builder;
use bincode;
use serde;
use chrono;
use derivative;
use asmov_common_dataset_fieldenum::DatasetFieldEnum;
use crate::*;

/// Database meta information that is associated with every standalone data class / database table.
#[derive(Debug, Clone, derive_builder::Builder, derivative::Derivative, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
#[derivative(PartialEq, Eq, Hash)]
pub struct Meta {
    id: ID,
    user_id: ID,
    #[bincode(with_serde)]
    time_created: Timestamp,
    #[bincode(with_serde)]
    time_modified: Timestamp,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    #[builder(default)]
    hashcode: Hashcode,
}

impl Meta {
    pub fn id(&self) -> ID {
        self.id
    }

    pub fn user_id(&self) -> ID {
        self.user_id
    }

    pub fn time_created(&self) -> Timestamp {
        self.time_created
    }

    pub fn time_modified(&self) -> Timestamp {
        self.time_modified
    }
}

#[derive(asmov_common_dataset_fieldenum_derive::DatasetFieldEnum, strum::EnumIter, strum::EnumCount)]
pub enum MetaField {
    Id,
    UserId,
    TimeCreated,
    TimeModified,
    Hashcode,
}

pub trait MetaModel: Sized + std::hash::Hash + Clone + ToOwned<Owned = Self> {
    type FieldEnum: asmov_common_dataset_fieldenum::DatasetFieldEnum;

    const SCHEMA_NAME: &'static str;
    const SCHEMA_NAME_PLURAL: &'static str;

    fn meta(&self) -> &Meta;

    fn meta_mut(&mut self) -> &mut Meta;

    fn schema_name() -> &'static str {
        Self::SCHEMA_NAME
    }

    fn schema_name_plural() -> &'static str {
        Self::SCHEMA_NAME_PLURAL
    }

    fn id(&self) -> ID {
        self.meta().id
    }

    fn user_id(&self) -> ID {
        self.meta().user_id
    }

    fn time_created(&self) -> Timestamp {
        self.meta().time_created
    }

    fn time_modified(&self) -> Timestamp {
        self.meta().time_modified
    }

    fn hashcode(&self) -> Hashcode {
        self.meta().hashcode
    }

    fn rehash(&mut self) {
        self.meta_mut().hashcode = Hashcode::calculate_hash(self);
    }

    fn rehashed(mut self) -> Self {
        self.rehash();
        self
    }

    fn fields() -> Self::FieldEnum::Iterator {
        MetaField::iter()
    }
}

pub trait MetaModelMut: MetaModel {
    fn modify_now(&mut self) {
        self.meta_mut().time_modified = chrono::Utc::now();
    }
}

impl MetaModelMut for Meta {
    fn modify_now(&mut self) {
        self.time_modified = chrono::Utc::now();
    }
}

impl MetaModel for Meta {
    type FieldEnum = MetaField;

    const SCHEMA_NAME: &'static str = "meta";
    const SCHEMA_NAME_PLURAL: &'static str = "meta";

    fn meta(&self) -> &Meta {
        self
    }

    fn meta_mut(&mut self) -> &mut Meta {
        self
    }
}

// schema constants
impl Meta {
    pub const SCHEMA: &'static str = "meta";
    pub const SCHEMA_PLURAL: &'static str = "meta";
    pub const SCHEMA_FIELD_USER_ID: &'static str = "user_id";
    pub const SCHEMA_FIELD_TIME_CREATED: &'static str = "time_created";
    pub const SCHEMA_FIELD_TIME_MODIFIED: &'static str = "time_modified";
    pub const SCHEMA_FIELD_HASHCODE: &'static str = "hashcode";
}

#[cfg(feature = "postgres")]
mod postgresql  {
    use super::*;
    use sqlx::{self, Row, postgres};

    impl sqlx::FromRow<'_, postgres::PgRow> for Meta {
        fn from_row(row: &postgres::PgRow) -> sqlx::Result<Self> {
            Ok(Self {
                id: ID::Authorative(row.try_get::<i64, _>("id")? as u64),
                user_id: ID::Authorative(row.try_get::<i64, _>("user_id")? as u64),
                time_created: row.try_get("time_created")?,
                time_modified: row.try_get("time_modified")?,
                hashcode: row.try_get::<i64, _>("hashcode")? as u64,
            })
        }
    }
}

#[cfg(feature = "sqlite")]
mod sqlite  {
    use super::*;
    use sqlx::{self, Row, sqlite};

    impl sqlx::FromRow<'_, sqlite::SqliteRow> for Meta {
        fn from_row(row: &sqlite::SqliteRow) -> sqlx::Result<Self> {
            Ok(Self {
                id: ID::Authorative(row.try_get::<i64, _>("id")? as u64),
                user_id: ID::Authorative(row.try_get::<i64, _>("user_id")? as u64),
                time_created: row.try_get("time_created")?,
                time_modified: row.try_get("time_modified")?,
                hashcode: row.try_get::<i64, _>("hashcode")? as u64,
            })
        }
    }
}