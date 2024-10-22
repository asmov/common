pub mod memory;
pub mod strategy;

use std::{borrow::Cow, future::Future};
use crate::*;


// Rust issue: https://github.com/rust-lang/rust/issues/96865
// Workaround: https://docs.rs/send-future/latest/send_future/trait.SendFuture.html
#[allow(unused_imports)]
use send_future::SendFuture as _;

pub trait Dataset {
    fn get<'a, M>(&'a self, id: ID) -> impl Future<Output = Result<Option<Cow<'a, M>>>> + Send + 'a
    where
        Self: Sized + 'a,
        M: DatasetModel<Self> + 'a
    {
        M::dataset_get(self, id)
    }

    fn put<'a, M>(&'a mut self, model: M) -> impl Future<Output = Result<ID>> + Send + 'a
    where
        Self: Sized + 'a,
        M: DatasetModel<Self> + 'a
    {
        M::dataset_put(self, model)
    }

    fn delete<'a, M>(&'a mut self, id: ID) -> impl Future<Output = Result<()>> + Send + 'a
    where
        Self: Sized + 'a,
        M: DatasetModel<Self> + 'a
    {
        M::dataset_delete(self, id)
    }
}

pub trait DatasetDirect: Dataset {
    /// After mutation, [MutDataset::update] must be called to process changes properly.
    fn take<'d:'m,'m,M>(&'d mut self, id: ID) -> impl Future<Output = Result<Option<M>>> + Send + 'm
    where
        Self: Sized + 'd,
        M: DatasetModelDirect<Self> + 'm
    {
        M::dataset_take(self, id)
    }
}

pub trait DatasetModel<DB: Dataset>: MetaModel + Send {
    fn dataset_get<'d:'m,'m>(dataset: &'d DB, id: ID) -> impl Future<Output = Result<Option<Cow<'m, Self>>>> + Send where Self: 'm;
    fn dataset_put<'d:'m,'m>(dataset: &'d mut DB, model: Self) -> impl Future<Output = Result<ID>> + Send where Self: 'm;
    fn dataset_delete<'d:'m,'m>(dataset: &'d mut DB, id: ID) -> impl Future<Output = Result<()>> + Send where Self: 'm;
}

pub trait DatasetModelDirect<DB: Dataset>: DatasetModel<DB> {
    fn dataset_take<'d:'m,'m>(dataset: &'d mut DB, id: ID) -> impl Future<Output = Result<Option<Self>>> + Send + 'm where Self: 'm;
}
