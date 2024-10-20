use crate::*;

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