use crate::*;

pub trait MemoryDataset: Dataset + Default {}

/// Implements the [DatasetModel] trait with [MemoryDataset] for a model. 
#[macro_export]
macro_rules! boil_memory_dataset_for_model {
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

