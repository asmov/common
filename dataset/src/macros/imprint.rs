pub use crate::*;

/// Implements the [DatasetModel] trait with [StrategicDataset] for a model.
// Attempting to create a generic version of this runs into a Rust issue: https://github.com/rust-lang/rust/issues/100011
#[macro_export]
macro_rules! imprint_strategic_dataset_for_model {
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
                        #[cfg(feature = "postgres")]
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
                        #[cfg(feature = "postgres")]
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
                #[cfg(feature = "postgres")]
                let postgres = dataset.postgres_dataset.as_mut().unwrap();
                todo!()

                //let order = dataset.options.strategic_order.clone();
                //if order[-2] == DatasetType::Memory {
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

/// Implements the [DatasetModel] trait with [MemoryDataset] for a model. 
#[macro_export]
macro_rules! imprint_memory_dataset_for_model {
    ($MemoryDataset:ty, $Model:ty, $model_variable:ident) => {
        impl DatasetModel<$MemoryDataset> for $Model {
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

/// Implements the [DatasetModel] trait for a model.  
/// Calls [$Dataset::standard_get] for [DatasetModel::dataset_get].
#[macro_export]
macro_rules! imprint_sql_dataset_for_model {
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