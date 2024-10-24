pub mod error;
pub mod dataset;

pub mod macros {
    pub mod imprint;
    pub use imprint::*;
}

pub use error::{Error, Result};

pub mod model;

pub mod ext {
    pub use derive_builder;
}

pub use crate::{
    model::{
        Timestamp, TimestampTrait, Hashcode, HashcodeTrait,
        meta::{Meta, MetaModel, MetaModelMut, MetaBuilder, MetaBuilderError},
        id::{ID, OptionID, init_local_id_generator, generate_local_id},
    },
    dataset::{
        Dataset, DatasetModel, DatasetDirect, DatasetModelDirect,
        memory::MemoryDataset,
        strategy::{StrategicDataset, StrategicDatasetModel}
    },
    ext::derive_builder::Builder
};

pub mod prelude {
    pub use crate::{MetaModel, MetaModelMut, MetaBuilder, Dataset, DatasetDirect, DatasetModel,
        DatasetModelDirect, MemoryDataset, StrategicDataset, StrategicDatasetModel,
        TimestampTrait, HashcodeTrait};
}

pub mod driver {
    #[cfg(feature = "postgres")]
    pub mod postgres;

    #[cfg(feature = "sqlite")]
    pub mod sqlite;
}

#[cfg(feature = "postgres")]
pub use crate::driver::postgres::{self, PostgresDataset};

#[cfg(feature = "sqlite")]
pub use crate::driver::sqlite::{self, SqliteDataset};


pub trait ToArguments<DB: sqlx::Database>: crate::MetaModel + Send {
    fn to_arguments<'m:'q,'q>(&'m self) -> sqlx::Result<<DB as sqlx::Database>::Arguments<'q>>;
}

