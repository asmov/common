pub mod id;
pub mod meta;

use std::hash::{self, Hash, Hasher};

pub type Timestamp = chrono::DateTime<chrono::Utc>;

pub trait TimestampTrait {
    fn now() -> Timestamp {
        chrono::Utc::now()
    }
}

impl TimestampTrait for Timestamp {}

pub type Hashcode = u64;

pub trait HashcodeTrait {
    fn calculate_hash<T: crate::MetaModel>(model: &T) -> Hashcode {
        let mut hasher = hash::DefaultHasher::new();
        model.hashcode().hash(&mut hasher);
        hasher.finish()
    }
}

impl HashcodeTrait for Hashcode {}

