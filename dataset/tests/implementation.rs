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
    use asmov_common_dataset::{self as dataset};

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

    //MACRO: imprint_meta_for_model!($ImplModel, $schema_name, $schema_name_plural)
    impl ::asmov_common_dataset::MetaModel for ImplModel {
        const SCHEMA_NAME: &'static str = "schema_name"; //MACRO: replace "schema_name" with $schema_name
        const SCHEMA_NAME_PLURAL: &'static str = "schema_name_plural"; //MACRO: replace "schema_name_plural" with "$schema_name_plural"

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
    impl ::asmov_common_dataset::DatasetDirect for ImplMemoryDataset {}
    impl ::asmov_common_dataset::MemoryDataset for ImplMemoryDataset {}
    
    // MACRORULE: imprint_memory_dataset_for_model!($ImplModel, $ImplMemoryDataset, $impl_model_variable)
    impl ::asmov_common_dataset::DatasetModel<ImplMemoryDataset> for ImplModel {
        async fn dataset_get<'d:'m,'m>(dataset: &'d ImplMemoryDataset, id: ::asmov_common_dataset::ID) -> ::asmov_common_dataset::Result<::std::option::Option<::std::borrow::Cow<'m, Self>>> where Self: 'm {
            Ok(dataset.impl_model_variable.get(&id).and_then(|m| Some(::std::borrow::Cow::Borrowed(m))))
        }

        async fn dataset_put<'d:'m,'m>(dataset: &'d mut ImplMemoryDataset, model: Self) -> ::asmov_common_dataset::Result<::std::option::Option<::asmov_common_dataset::ID>> where Self: 'm {
            let id = <Self as ::asmov_common_dataset::MetaModel>::meta(&model).id();
            dataset.impl_model_variable.insert(id, model);
            Ok(Some(id))
        }

        async fn dataset_delete<'d:'m,'m>(dataset: &'d mut ImplMemoryDataset, id: ::asmov_common_dataset::ID) -> ::asmov_common_dataset::Result<()> where Self: 'm {
            let _ = dataset.impl_model_variable.remove(&id);
            Ok(())
        }
    }

    impl ::asmov_common_dataset::DatasetModelDirect<ImplMemoryDataset> for ImplModel {
        async fn dataset_take<'d:'m, 'm>(dataset: &'d mut ImplMemoryDataset, id: ::asmov_common_dataset::ID) -> ::asmov_common_dataset::Result<::std::option::Option<Self>> {
            Ok(dataset.impl_model_variable.remove(&id))
        }
    }

    fn fixture_model() -> ImplModel {
        use asmov_common_dataset::{self as dataset, prelude::*};
         ImplModelBuilder::default()
            .meta(MetaBuilder::default()
                .id(dataset::ID::Local(101))
                .user_id(dataset::ID::LOCAL_USER)
                .time_created(dataset::Timestamp::now())
                .time_modified(dataset::Timestamp::now())
                .build().unwrap())
            .text("Hello, world!".to_string())
            .num(42)
            .toggle(true)
            .timestamp(chrono::Utc::now())
            .build().unwrap()
            .rehashed()
    }

    #[tokio::test]
    async fn test_memory() {
        use asmov_common_dataset::{self as dataset, prelude::*};

        let mut memory_dataset = ImplMemoryDataset::default();

        assert!(matches!(memory_dataset.get::<ImplModel>(dataset::ID::Local(101)).await.unwrap(), None),
            "Memory dataset should return Ok(None) for a missing model");

        let model = fixture_model();

        // put() the model into memory and make sure that we can get() it back
        assert!(matches!(memory_dataset.put(model.clone()).await.unwrap(), Some(dataset::ID::Local(101))),
            "Memory dataset should return with the expected ID after writing model");
        assert!(matches!(memory_dataset.get::<ImplModel>(dataset::ID::Local(101)).await.unwrap(), Some(m) if m == model),
            "Memory dataset should return an exact copy of the model inserted");

        // take() the model from memory, alter it, put() it back, then get() to compare
        let mut model_taken = memory_dataset.take::<ImplModel>(dataset::ID::Local(101)).await.unwrap().unwrap();
        model_taken.text = "Hello, universe!".to_string();
        assert!(matches!(memory_dataset.put(model_taken).await.unwrap(), Some(id) if id == dataset::ID::Local(101)),
            "Memory dataset should return the same ID after modifying it");
        let model_updated = memory_dataset.get::<ImplModel>(dataset::ID::Local(101)).await.unwrap().unwrap();
        assert_eq!(model_updated.text, "Hello, universe!".to_string(),
            "Memory dataset should return the updated model after taking, altering, and putting it back");

        // delete() the model from memory and make sure that we can't get() it afterwards
        assert!(matches!(memory_dataset.delete::<ImplModel>(dataset::ID::Local(101)).await, Ok(())),
            "Memory dataset should deleting the model and return Ok");
        assert!(matches!(memory_dataset.get::<ImplModel>(dataset::ID::Local(101)).await, Ok(None)),
            "Memory dataset should return Ok(None) for a missing model after deletion");
    }

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use tokio;
        use sqlx::{self, Row, Executor, Arguments, sqlite};
        use super::*;

        impl sqlx::FromRow<'_, sqlite::SqliteRow> for ImplModel {
            fn from_row(row: &sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
                Ok(Self {
                    meta: dataset::Meta::from_row(row)?,
                    text: row.try_get("text")?,
                    num: row.try_get("num")?,
                    toggle: row.try_get("num")?,
                    timestamp: row.try_get("timestamp")?,
                })
            }
        }
     
        use dataset::SqlxArgumentsExtended;
        
        impl ::asmov_common_dataset::ToArguments<::sqlx::sqlite::Sqlite> for ImplModel {
            fn to_arguments<'m:'q,'q>(&'m self) -> ::sqlx::Result<(<::sqlx::sqlite::Sqlite as ::sqlx::Database>::Arguments<'q>, usize)> {
                ::sqlx::sqlite::SqliteArguments::start_args(8)?
                    .arg(&self.text)?
                    .arg(self.num as i64)?
                    .arg(self.toggle)?
                    .arg(self.timestamp)?
                    .finish_args(self, 8)
            }
        }

        impl ::asmov_common_dataset::DatasetModel<::asmov_common_dataset::SqliteDataset> for ImplModel {
            async fn dataset_get<'d:'m,'m>(dataset: &'d ::asmov_common_dataset::SqliteDataset, id: asmov_common_dataset::ID) -> ::asmov_common_dataset::Result<std::option::Option<std::borrow::Cow<'m, Self>>> where Self: 'm {
                ::asmov_common_dataset::SqliteDataset::standard_get(dataset, id).await
            }
        
            async fn dataset_put<'d:'m,'m>(dataset: &'d mut ::asmov_common_dataset::SqliteDataset, model: Self) -> ::asmov_common_dataset::Result<::std::option::Option<::asmov_common_dataset::ID>> where Self: 'm {
                ::asmov_common_dataset::SqliteDataset::standard_insert_or_update(dataset, &model).await
            }
        
            async fn dataset_delete<'d:'m,'m>(dataset: &'d mut ::asmov_common_dataset::SqliteDataset, id: ::asmov_common_dataset::ID) -> ::asmov_common_dataset::Result<()> where Self: 'm {
                //::asmov_common_dataset::SqliteDataset::standard_delete(dataset, id).await
                todo!()
            }
        }

        #[tokio::test]
        async fn test_sqlite() {
            use asmov_common_dataset::{self as dataset, prelude::*};

            let sqlite_pool = sqlite::SqlitePool::connect(":memory:").await.unwrap();
            sqlite_pool.execute("CREATE TABLE schema_name (local_id INTEGER PRIMARY KEY, id INTEGER UNIQUE, user_id INTEGER, time_created TEXT, time_modified TEXT, hashcode INTEGER, text TEXT, num INTEGER, toggle INTEGER, timestamp TEXT)").await.unwrap();
            let mut sqlite_dataset = dataset::SqliteDataset::new(sqlite_pool);

            assert!(matches!(sqlite_dataset.get::<ImplModel>(dataset::ID::Local(101)).await.unwrap(), None),
                "SQLite dataset should return Ok(None) for a missing model");

            let model = fixture_model();

            // put() the model into memory and make sure that we can get() it back
            assert!(matches!(sqlite_dataset.put(model.clone()).await.unwrap(), Some(dataset::ID::Local(101))),
                "SQLite dataset should return with the expected ID after writing model");
            assert!(matches!(sqlite_dataset.get::<ImplModel>(dataset::ID::Local(101)).await, Ok(Some(m)) if m == model),
                "SQLite dataset should return an exact copy of the model inserted");

            // delete() the model from memory and make sure that we can't get() it afterwards
            assert!(matches!(sqlite_dataset.delete::<ImplModel>(dataset::ID::Local(101)).await.unwrap(), ()),
                "SQLite dataset should deleting the model and return Ok");
            assert!(matches!(sqlite_dataset.get::<ImplModel>(dataset::ID::Local(101)).await.unwrap(), None),
                "SQLite dataset should return Ok(None) for a missing model after deletion");

        }
    }
}