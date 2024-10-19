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
        M: MetaModel + DatasetModel<Self> + 'a
    {
        M::dataset_get(self, id)
    }

    /*fn gets<'d:'m, 'm, M>(&'d self, id: ID) -> impl Future<Output = Result<Option<Cow<'m, M>>>> + Send + 'm
    where
        Self: Sized + 'd,
        M: MetaModel + DatasetModel<Self> + 'm;*/



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
