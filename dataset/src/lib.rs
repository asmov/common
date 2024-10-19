pub mod error;
pub mod dataset;

pub use error::{Error, Result};

pub mod model {
    pub mod id;
    pub mod meta;
}

pub mod ext {
    pub use chrono;
    pub use derive_builder;
}

pub mod consts {
}

pub use crate::{
    model::{
        meta::{Meta, MetaModel, MetaModelMut, MetaBuilder, MetaBuilderError},
        id::{ID, init_local_id_generator, generate_local_id},
    },
    dataset::{
        Dataset, DatasetModel, DatasetMut, DatasetModelMut,
        memory::MemoryDataset,
        strategy::{StrategicDataset, StrategicDatasetModel}
    },
    ext::derive_builder::Builder
};

pub mod prelude {
    pub use crate::{
        model::meta::{MetaModel, MetaModelMut, MetaBuilder},
        dataset::{Dataset, DatasetMut, DatasetModel, DatasetModelMut, memory::MemoryDataset, strategy::{StrategicDataset, StrategicDatasetModel}}
    };
}

pub mod driver {
    #[cfg(feature = "postgres")]
    pub mod postgres;

    #[cfg(feature = "sqlite")]
    pub mod sqlite;

    #[cfg(feature = "sql")]
    pub mod sql;
}

#[cfg(feature = "postgres")]
pub use crate::driver::postgres::{self, PostgresDataset};

#[cfg(feature = "sqlite")]
pub use crate::driver::sqlite::{self, SqliteDataset};

#[cfg(feature = "sql")]
pub use crate::driver::sql;
