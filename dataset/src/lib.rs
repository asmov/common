pub mod error;
pub mod dataset;

pub use error::{Error, Result};

pub mod model {
    pub mod id;
    pub mod meta;
}

pub mod driver {
    #[cfg(feature = "postgresql")]
    pub mod postgresql;

    #[cfg(feature = "sqlite")]
    pub mod sqlite;
}

pub mod ext {
    pub use chrono;
}

pub mod consts {
    pub use crate::model::id::consts::*;
}

pub mod prelude {}

pub use crate::model::{
    meta::{Meta, MetaModel, MetaBuilder, MetaBuilderError},
    id::{ID, init_local_id_generator, generate_local_id},
    dataset::{
        Dataset, DatasetModel,
        MemoryDataset,
        StrategicDataset, StrategicDatasetModel, StrategicGet
    }
};

