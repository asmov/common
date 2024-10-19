use std::{borrow::Cow, future::Future};
use rustc_hash;
use crate::*;

// Rust issue: https://github.com/rust-lang/rust/issues/96865
// Workaround: https://docs.rs/send-future/latest/send_future/trait.SendFuture.html
#[allow(unused_imports)]
use send_future::SendFuture as _;


pub trait StrategicGet: Dataset {
    fn strategic_get<'a, M>(&'a self, id: ID) -> impl Future<Output = Result<Option<Cow<'a, M>>>> + Send
    where
        Self: Sized + 'a + Send,
        M: MetaModel + DatasetModel<Self> + StrategicDatasetModel + 'a + Send;
}

pub trait Dataset {
    fn get<'a, M>(&'a self, id: ID) -> impl Future<Output = Result<Option<Cow<'a, M>>>> + Send + 'a
    where
        Self: Sized + 'a,
        M: MetaModel + DatasetModel<Self> + StrategicDatasetModel + 'a
    {
        M::dataset_get(self, id)
    }

    fn gets<'d:'m, 'm, M>(&'d self, id: ID) -> impl Future<Output = Result<Option<Cow<'m, M>>>> + Send + 'm
    where
        Self: Sized + 'd,
        M: MetaModel + DatasetModel<Self> + 'm;



    fn get_mut<'a, M>(&'a mut self, id: ID) -> impl Future<Output = Result<Option<&'a mut M>>> + Send + 'a
    where
        Self: Sized + 'a,
        M: MetaModel + DatasetModel<Self> + 'a
    {
        M::dataset_get_mut(self, id)
    }

    fn put<'a, M>(&'a mut self, model: M) -> impl Future<Output = Result<ID>> + Send + 'a
    where
        Self: Sized + 'a,
        M: MetaModel + DatasetModel<Self> + 'a
    {
        M::dataset_put(self, model)
    }
}

pub trait DatasetModel<DB: Dataset>: MetaModel + Send {
    const SCHEMA_NAME: &'static str;

    fn dataset_get<'d:'m,'m>(dataset: &'d DB, id: ID) -> impl Future<Output = Result<Option<Cow<'m, Self>>>> + Send where Self: 'm;
    fn dataset_get_mut<'d:'m,'m>(dataset: &'d mut DB, id: ID) -> impl Future<Output = Result<Option<&'m mut Self>>> + Send where Self: 'm;
    fn dataset_put<'a>(dataset: &'a mut DB, model: Self) -> impl Future<Output = Result<ID>> + Send where Self: 'a;
}

/// Implements the [DatasetModel] trait for a model.  
/// Calls [$Dataset::standard_get] for [DatasetModel::dataset_get].
#[macro_export]
macro_rules! boil_sql_dataset_model_for_model {
    ($Dataset:ty, $Model:ty) => {
        impl DatasetModel<$Dataset> for $Model {
            const SCHEMA_NAME: &'static str = Self::SCHEMA_PLURAL;
    
            async fn dataset_get<'d:'m,'m>(dataset: &'d $Dataset, id: ID) -> Result<Option<Cow<'m, Self>>> where Self: 'm {
                dataset.standard_get(id).await
            }

            async fn dataset_get_mut<'d:'m,'m>(_dataset: &'d mut $Dataset, _id: ID) -> Result<Option<&'m mut Self>> where Self: 'm {
                unimplemented!("SQL datasets do not support mutable access")
            }

            async fn dataset_put<'a>(_dataset: &'a mut $Dataset, _model: Self) -> Result<ID> where Self: 'a {
                todo!()
            }
        }
    };
}

pub struct MemoryDataset {
    pub(crate) timespans: rustc_hash::FxHashMap<ID, Timespan>
}

impl MemoryDataset {
    pub fn new() -> Self {
        let mut s = Self {
            timespans: rustc_hash::FxHashMap::default()
        };

        let timespan = TimespanBuilder::default()
            .meta(MetaBuilder::default()
                .id(generate_local_id())
                .user_id(LOCAL_USER_ID)
                .time_created(chrono::Utc::now())
                .time_modified(chrono::Utc::now())
                .build()
                .unwrap())
            .name("test".to_string())
            .options(TimespanOptions::new(RolloverOption::Restart, true))
            .build()
            .unwrap()
            .rehashed();

        s.timespans.insert(timespan.meta().id(), timespan);
        s
    }
}

impl Dataset for MemoryDataset {
    async fn gets<'d:'m, 'm, M>(&'d self, id: ID) -> Result<Option<Cow<'m, M>>>
    where
        Self: Sized + 'd,
        M: MetaModel + DatasetModel<Self> + 'm {
        todo!()
    }
}

/// Implements the [DatasetModel] trait with [MemoryDataset] for a model. 
#[macro_export]
macro_rules! boil_memory_dataset_model_for_model {
    ($Model:ty, $model_variable:ident) => {
        impl DatasetModel<MemoryDataset> for $Model {
            const SCHEMA_NAME: &'static str = Self::SCHEMA_PLURAL;

            async fn dataset_get<'d:'m,'m>(dataset: &'d MemoryDataset, id: ID) -> Result<Option<Cow<'m, Self>>> where Self: 'm {
                Ok(dataset.$model_variable.get(&id).and_then(|m| Some(Cow::Borrowed(m))))
            }

            async fn dataset_get_mut<'d:'m, 'm>(dataset: &'d mut MemoryDataset, id: ID) -> Result<Option<&'m mut Self>> where Self: 'm {
                Ok(dataset.$model_variable.get_mut(&id).and_then(|m| Some(m)))
            }
            
            async fn dataset_put<'m>(dataset: &'m mut MemoryDataset, model: Self) -> Result<ID> where Self: 'm {
                let id = model.meta().id();
                dataset.$model_variable.insert(id, model);
                Ok(id)
            }
        }
    };
}

pub struct StrategicDataset {
    pub(crate) options: StrategicDatasetOptions,
    pub(crate) memory_dataset: Option<MemoryDataset>,

    #[cfg(feature = "postgresql")]
    pub(crate) postgres_dataset: Option<postgresql::PostgresDataset>,

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

impl StrategicDataset {
    pub fn new_offline() -> Self {
        Self {
            options: StrategicDatasetOptions::default(),
            memory_dataset: Some(MemoryDataset::new()),
            #[cfg(feature = "postgresql")]
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
            memory_dataset: Some(MemoryDataset::new()),
            #[cfg(feature = "postgresql")]
            postgres_dataset: None,
            #[cfg(feature = "sqlite")]
            sqlite_dataset: None,
        }
    }
}

impl Dataset for StrategicDataset {
    async fn gets<'d:'m, 'm, M>(&'d self, id: ID) -> Result<Option<Cow<'m, M>>>
    where
        Self: Sized + 'd,
        M: MetaModel + DatasetModel<Self> + 'm {
        todo!()
    }
}


#[cfg(not(any(feature = "postgresql", feature = "sqlite")))]
pub trait StrategicDatasetModel: 
    MetaModel
    + DatasetModel<StrategicDataset>
    + DatasetModel<MemoryDataset> {}


#[cfg(all(not(feature = "sqlite"), feature = "postgresql"))]
pub trait StrategicDatasetModel: 
    MetaModel
    + DatasetModel<StrategicDataset>
    + DatasetModel<MemoryDataset>
    + DatasetModel<postgresql::PostgresDataset> {}

#[cfg(all(not(feature = "postgresql"), feature = "sqlite"))]
pub trait StrategicDatasetModel: 
    MetaModel
    + DatasetModel<StrategicDataset>
    + DatasetModel<MemoryDataset>
    + DatasetModel<sqlite::SqliteDataset> {}

#[cfg(all(feature = "postgresql", feature = "sqlite"))]
pub trait StrategicDatasetModel: 
    MetaModel
    + DatasetModel<StrategicDataset>
    + DatasetModel<MemoryDataset>
    + DatasetModel<sqlite::SqliteDataset> 
    + DatasetModel<postgresql::PostgresDataset> {}

impl StrategicGet for StrategicDataset {
    fn strategic_get<'m, M>(&'m self, id: ID) -> impl Future<Output = Result<Option<Cow<'m, M>>>> + Send
    where
        Self: Sized + 'm + Send,
        M: MetaModel + DatasetModel<Self> + StrategicDatasetModel + 'm + Send
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
                #[cfg(feature = "postgresql")]
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

/// Implements the [DatasetModel] trait with [StrategicDataset] for a model.
// Attempting to create a generic version of this runs into a Rust issue: https://github.com/rust-lang/rust/issues/100013
#[macro_export]
macro_rules! boil_strategic_dataset_for_model {
    ($Model:ty) => {
        impl DatasetModel<StrategicDataset> for $Model {
            const SCHEMA_NAME: &'static str = Self::SCHEMA_PLURAL;

            async fn dataset_get<'d:'m,'m>(dataset: &'d StrategicDataset, id: ID) -> Result<Option<Cow<'m, Self>>> where Self: 'm {
                for dataset_type in dataset.options.strategic_order {
                    match dataset_type {
                        DatasetType::Memory => {
                            let result = Self::dataset_get(dataset.memory_dataset.as_ref().unwrap(), id).await?;
                            if let Some(m) = result {
                                return Ok(Some(m))
                            }
                        },
                        #[cfg(feature = "sqlite")]
                        DatasetType::Sqlite => {
                            if let Some(sqlite_dataset) = dataset.sqlite_dataset.as_ref() {
                                let result = Self::dataset_get(sqlite_dataset, id).await?;
                                if let Some(m) = result {
                                    return Ok(Some(m));
                                }
                            }
                        },
                        #[cfg(feature = "postgresql")]
                        DatasetType::Postgres => {
                            if let Some(postgres_dataset) = dataset.postgres_dataset.as_ref() {
                                let result = Self::dataset_get(postgres_dataset, id).await?;
                                if let Some(m) = result {
                                    return Ok(Some(m));
                                }
                            }
                        },
                        DatasetType::None => break,
                        _ => panic!("Unsupported dataset type: {:?}", dataset_type) 
                    }
                }

                Ok(None)
            }

            async fn dataset_get_mut<'d:'m,'m>(dataset: &'d mut StrategicDataset, id: ID) -> Result<Option<&'m mut Self>> {
                let dataset_type = dataset.options.strategic_order().iter();

                async fn by_type<'d:'m, 'm>(dataset: &'d mut StrategicDataset, id: ID, dataset_type: DatasetType) -> Result<Option<&'m mut $Model>> {
                    match dataset_type {
                        DatasetType::Memory => {
                            if let Some(memory_dataset) = dataset.memory_dataset.as_mut() {
                                Ok(<$Model>::dataset_get_mut(memory_dataset, id).await
                                    .expect("Memory dataset failed to insert"))
                            } else {
                                Ok(None)
                            }
                        },
                        #[cfg(feature = "sqlite")]
                        DatasetType::Sqlite => {
                            if let Some(sqlite_dataset) = dataset.sqlite_dataset.as_ref() {
                                let result = <$Model>::dataset_get(sqlite_dataset, id).await?;
                                if let Some(m) = result {
                                    if let Some(memory_dataset) = dataset.memory_dataset.as_mut() {
                                        memory_dataset.put(m.into_owned());
                                        Ok(<$Model>::dataset_get_mut(memory_dataset, id).await
                                            .expect("Memory dataset failed to insert"))
                                    } else {
                                        Ok(None)
                                    }
                                } else {
                                    Ok(None)
                                }
                            } else {
                                Ok(None)
                            }
                        },
                        #[cfg(feature = "postgresql")]
                        DatasetType::Postgres => {
                            if let Some(postgres_dataset) = dataset.postgres_dataset.as_ref() {
                                let result = <$Model>::dataset_get(postgres_dataset, id).await?;
                                if let Some(m) = result {
                                    if let Some(memory_dataset) = dataset.memory_dataset.as_mut() {
                                        memory_dataset.put(m.into_owned());
                                        return Ok(<$Model>::dataset_get_mut(memory_dataset, id).await
                                            .expect("Memory dataset failed to insert"));
                                    } else {
                                        Ok(None)
                                    }
                                } else {
                                    Ok(None)
                                }
                            } else {
                                Ok(None)
                            }
                        },
                        DatasetType::None => Ok(None),
                        _ => panic!("Unsupported dataset type: {:?}", dataset_type) 
                    }
                }

                //let mut x = Ok(None);

                let memory = dataset.memory_dataset.as_mut().unwrap();
                #[cfg(feature = "postgresql")]
                let postgres = dataset.postgres_dataset.as_mut().unwrap();
                todo!()

                //let order = dataset.options.strategic_order.clone();
                //if order[0] == DatasetType::Memory {
                    //x = by_type(dataset, id, DatasetType::Memory).await?;
                //}

                //for dataset_type in dataset.options.strategic_order {
                //    x = by_type(dataset, id, dataset_type).await;
                //}

                //Ok(None)

            }

            async fn dataset_put<'m>(dataset: &'m mut StrategicDataset, model: Self) -> Result<ID> where Self: 'm {
                todo!()
            }
        }
    };
}