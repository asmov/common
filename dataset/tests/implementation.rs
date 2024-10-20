//! This module is used to design boilerplate macros for implementation.
//! Do NOT alter without also updating the macros.
//! All impl code should be written in a way that allows for it to be copied and pasted into the macros:
//!  - use root-level namespacing
//!  - use the same macro variable names without the '$'. eg:
//!    - $ImplModel, $ImplMemoryDataset, etc.
//!    

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use tokio;
    use chrono;
    use asmov_common_dataset::{self as dataset, prelude::*};

    #[derive(Debug, PartialEq, Eq, Clone, Hash, dataset::Builder)]
    struct ImplModel {
        meta: dataset::Meta,
        text: String,
        num: u64,
        toggle: bool,
        timestamp: chrono::DateTime<chrono::Utc>
    }

    // for testing assertions
    impl PartialEq<Cow<'_, ImplModel>> for ImplModel {
        fn eq(&self, other: &Cow<'_, ImplModel>) -> bool {
            match other {
                Cow::Borrowed(b) => &self == b,
                Cow::Owned(o) => self == o
            }
        }
    }

    // for testing assertions
    impl PartialEq<ImplModel> for Cow<'_, ImplModel> {
        fn eq(&self, other: &ImplModel) -> bool {
            match self {
                Cow::Borrowed(b) => b == &other,
                Cow::Owned(o) => o == other
            }
        }
    }

    // MACRORULE: imprint_meta_for_model!($ImplModel, $schema_name, $schema_name_plural)
    impl ::asmov_common_dataset::MetaModel for ImplModel {
        const SCHEMA_NAME: &'static str = "$schema_name";
        const SCHEMA_NAME_PLURAL: &'static str = "$schema_name_plural";

        fn meta(&self) -> &::asmov_common_dataset::Meta {
            &self.meta
        }
    
        fn meta_mut(&mut self) -> &mut ::asmov_common_dataset::Meta {
            &mut self.meta
        }
    }

    impl ::asmov_common_dataset::MetaModelMut for ImplModel {}

    #[derive(Debug, Default)]
    struct ImplMemoryDataset {
        impl_model_variable: ::rustc_hash::FxHashMap<::asmov_common_dataset::ID, ImplModel>
    }

    impl ::asmov_common_dataset::Dataset for ImplMemoryDataset {}
    impl ::asmov_common_dataset::DatasetMut for ImplMemoryDataset {}
    impl ::asmov_common_dataset::MemoryDataset for ImplMemoryDataset {}
    
    // MACRORULE: imprint_memory_dataset_for_model!($ImplModel, $ImplMemoryDataset, $impl_model_variable)
    impl ::asmov_common_dataset::DatasetModel<ImplMemoryDataset> for ImplModel {
        async fn dataset_get<'d:'m,'m>(dataset: &'d ImplMemoryDataset, id: ::asmov_common_dataset::ID) -> ::asmov_common_dataset::Result<::std::option::Option<::std::borrow::Cow<'m, Self>>> where Self: 'm {
            Ok(dataset.impl_model_variable.get(&id).and_then(|m| Some(::std::borrow::Cow::Borrowed(m))))
        }

        async fn dataset_put<'m>(dataset: &'m mut ImplMemoryDataset, model: Self) -> ::asmov_common_dataset::Result<::asmov_common_dataset::ID> where Self: 'm {
            let id = model.meta().id();
            dataset.impl_model_variable.insert(id, model);
            Ok(id)
        }
    }

    impl ::asmov_common_dataset::DatasetModelMut<ImplMemoryDataset> for ImplModel {
        async fn dataset_take<'d:'m, 'm>(dataset: &'d mut ImplMemoryDataset, id: ::asmov_common_dataset::ID) -> ::asmov_common_dataset::Result<::std::option::Option<Self>> {
            Ok(dataset.impl_model_variable.remove(&id))
        }
    }

    #[tokio::test]
    async fn test_memory() {
        let mut memory_dataset = ImplMemoryDataset::default();

        assert!(matches!(memory_dataset.get::<ImplModel>(dataset::ID::Local(101)).await, Ok(None)),
            "Memory dataset should return Ok(None) for a missing model");

        let model = ImplModelBuilder::default()
            .meta(MetaBuilder::default()
                .id(dataset::ID::Local(101))
                .user_id(dataset::ID::LOCAL_USER)
                .time_created(chrono::Utc::now())
                .time_modified(chrono::Utc::now())
                .build().unwrap())
            .text("Hello, world!".to_string())
            .num(42)
            .toggle(true)
            .timestamp(chrono::Utc::now())
            .build().unwrap()
            .rehashed();

        assert!(matches!(memory_dataset.put(model.clone()).await, Ok(dataset::ID::Local(101))),
            "Memory dataset should return with the expected ID after writing model");

        assert!(matches!(memory_dataset.get::<ImplModel>(dataset::ID::Local(101)).await, Ok(Some(m)) if m == model),
            "Memory dataset should return an exact copy of the model inserted");

        // take() the model, alter it, put() it back, then get() to compare
        let mut model_taken = memory_dataset.take::<ImplModel>(dataset::ID::Local(101)).await.unwrap().unwrap();
        model_taken.text = "Hello, universe!".to_string();
        memory_dataset.put(model_taken).await.unwrap();
        let model_updated = memory_dataset.get::<ImplModel>(dataset::ID::Local(101)).await.unwrap().unwrap();
        assert_eq!(model_updated.text, "Hello, universe!".to_string(),
            "Memory dataset should return the updated model after taking, altering, and putting it back");
    }
}