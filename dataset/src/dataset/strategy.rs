use std::{future::Future,borrow::Cow};
use crate::*;

// Rust issue: https://github.com/rust-lang/rust/issues/96865
// Workaround: https://docs.rs/send-future/latest/send_future/trait.SendFuture.html
#[allow(unused_imports)]
use send_future::SendFuture as _;

pub trait DatasetStrategy<MEM>: Dataset where MEM: MemoryDataset {
    fn strategic_get<'a, M>(&'a self, id: ID) -> impl Future<Output = Result<Option<Cow<'a, M>>>> + Send
        where
            Self: Sized + 'a + Send,
            M: MetaModel + DatasetModel<Self> + StrategicDatasetModel<MEM> + 'a + Send;
}

/*pub trait StrategicGet: Dataset {
    fn strategic_get<'a, M>(&'a self, id: ID) -> impl Future<Output = Result<Option<Cow<'a, M>>>> + Send
    where
        Self: Sized + 'a + Send,
        M: MetaModel + DatasetModel<Self> + StrategicDatasetModel<MEM> + 'a + Send;
}*/



pub struct StrategicDataset<MEM: MemoryDataset> {
    pub(crate) options: StrategicDatasetOptions,
    pub(crate) memory_dataset: Option<MEM>,

    #[cfg(feature = "postgres")]
    pub(crate) postgres_dataset: Option<postgres::PostgresDataset>,

    #[cfg(feature = "sqlite")]
    pub(crate) sqlite_dataset: Option<sqlite::SqliteDataset>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetType {
    None,
    Memory,
    Sqlite,
    Backend,
    Postgres
}

pub type StrategicOrder = [DatasetType; 4];

#[derive(Debug)]
pub struct StrategicDatasetOptions {
    pub(crate) online: bool,
    pub(crate) strategic_order: StrategicOrder,
}

impl Default for StrategicDatasetOptions {
    fn default() -> Self {
        Self {
            online: false,
            strategic_order: [DatasetType::Memory, DatasetType::None, DatasetType::None, DatasetType::None]
        }
    }
}

impl StrategicDatasetOptions {
    pub fn strategic_order(&self) -> StrategicOrder {
        self.strategic_order
    }

    pub fn is_online(&self) -> bool {
        self.online
    }
}

impl<MEM: MemoryDataset> StrategicDataset<MEM> {
    pub fn new_offline() -> Self {
        Self {
            options: StrategicDatasetOptions::default(),
            memory_dataset: Some(MEM::default()),
            #[cfg(feature = "postgres")]
            postgres_dataset: None,
            #[cfg(feature = "sqlite")]
            sqlite_dataset: None,
        }
    }

    pub fn new_online() -> Self {
        Self {
            options: StrategicDatasetOptions {
                online: true,
                ..Default::default()
            },
            memory_dataset: Some(MEM::default()),
            #[cfg(feature = "postgres")]
            postgres_dataset: None,
            #[cfg(feature = "sqlite")]
            sqlite_dataset: None,
        }
    }
}

impl<MEM: MemoryDataset> Dataset for StrategicDataset<MEM> {}


#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
pub trait StrategicDatasetModel: 
    MetaModel
    + DatasetModel<StrategicDataset>
    + DatasetModel<MemoryDataset> {}


#[cfg(all(not(feature = "sqlite"), feature = "postgres"))]
pub trait StrategicDatasetModel: 
    MetaModel
    + DatasetModel<StrategicDataset>
    + DatasetModel<MemoryDataset>
    + DatasetModel<postgres::PostgresDataset> {}

#[cfg(all(not(feature = "postgres"), feature = "sqlite"))]
pub trait StrategicDatasetModel: 
    MetaModel
    + DatasetModel<StrategicDataset>
    + DatasetModel<MemoryDataset>
    + DatasetModel<sqlite::SqliteDataset> {}

#[cfg(all(feature = "postgres", feature = "sqlite"))]
pub trait StrategicDatasetModel<MEM: MemoryDataset>:
    MetaModel
    + DatasetModel<StrategicDataset<MEM>>
    + DatasetModel<MEM>
    + DatasetModel<sqlite::SqliteDataset> 
    + DatasetModel<postgres::PostgresDataset> {}

impl<MEM: MemoryDataset> DatasetStrategy<MEM> for StrategicDataset<MEM> {
    fn strategic_get<'m, M>(&'m self, id: ID) -> impl Future<Output = Result<Option<Cow<'m, M>>>> + Send
    where
        Self: Sized + 'm + Send,
        M: MetaModel + DatasetModel<Self> + StrategicDatasetModel<MEM> + 'm + Send
    {
        let memory_dataset = self.memory_dataset.as_ref().unwrap();
        M::dataset_get(memory_dataset, id)
        /*for dataset_type in self.options.strategic_order {
            match dataset_type {
                DatasetType::Memory => {
                    if let Some(memory_dataset) = self.memory_dataset.as_ref() {
                        //let result = M::dataset_get(memory_dataset, id).await?;

                        return async move { M::dataset_get(memory_dataset, id).send().await };
                        //return Ok(None);

                        //if let Some(m) = result {
                        //    return Ok(Some(m))
                        //}
                    }
                },
                /*#[cfg(feature = "sqlite")]
                DatasetType::Sqlite => {
                    if let Some(sqlite_dataset) = self.sqlite_dataset.as_ref() {
                        let result = M::dataset_get(sqlite_dataset, id).await?;
                        if let Some(m) = result {
                            return Ok(Some(m));
                        }
                    }
                },
                #[cfg(feature = "postgres")]
                DatasetType::Postgres => {
                    if let Some(postgres_dataset) = self.postgres_dataset.as_ref() {
                        let result = M::dataset_get(postgres_dataset, id).await?;
                        if let Some(m) = result {
                            return Ok(Some(m));
                        }
                    }
                },*/
                DatasetType::None => break,
                _ => panic!("Unsupported dataset type: {:?}", dataset_type) 
            }
        }

        Ok(None)*/
    }
}
