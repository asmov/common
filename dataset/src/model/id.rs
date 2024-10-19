use std::sync::atomic::{AtomicU64, Ordering};
use std::hash::{Hash, Hasher};

/// Represents either a local (purely offline) or online (database) ID.
/// Once an online ID is assigned, the former Local ID should be converted to Transitional. 
/// Transitional IDs should never be stored persistant.
/// Valid between [ID_NONE] and [ID_RESERVED] non-inclusive.
#[derive(Debug, Clone, Copy, Eq, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
pub enum ID {
    Local(u64),
    Online(u64),
    Transitional(u64, u64)
}

impl ID {
    pub const ONLINE_NONE: ID = ID::Online(u64::MIN);
    pub const ONLINE_RESERVED: ID = ID::Online(u64::MAX);
    pub const LOCAL_USER: ID = ID::Local(1);
}

pub enum OptionID {
    None,
    Reserved,
    Some(u64) 
}

impl PartialEq<ID> for ID {
    fn eq(&self, other: &ID) -> bool {
        match (self, other) {
            (ID::Local(a), ID::Local(b)) => a == b,
            (ID::Online(a), ID::Online(b)) => a == b,
            (ID::Transitional(a, aa), ID::Transitional(b, bb)) => a == b || aa == bb,
            (ID::Local(a), ID::Transitional(b, _)) => a == b,
            (ID::Online(a), ID::Transitional(_, b)) => a == b,
            (ID::Transitional(a, _), ID::Local(b)) => a == b,
            (ID::Transitional(_, a), ID::Online(b)) => a == b,
            _ => false
        }
    }
}

impl Hash for ID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ID::Local(id) => { 1.hash(state) ; id.hash(state) },
            ID::Online(id) => { 2.hash(state) ; id.hash(state) },
            ID::Transitional(_, id) => { 2.hash(state) ; id.hash(state) },
        }
    }
}

impl ID {
    /// For use with (postgre)SQL. Panics if the ID is not online.
    pub fn bind_online(self) -> i64 {
        match self {
            ID::Local(id) => id as i64,
            ID::Transitional(_, id) => id as i64,
            _ => panic!("Cannot convert local ID to online ID"),
        }
    }

    /// For use with SQL(ite). Panics if the ID is not local.
    pub fn bind_local(self) -> i64 {
        match self {
            ID::Local(id) => id as i64,
            ID::Transitional(id, _) => id as i64,
            _ => panic!("Cannot convert online ID to local ID"),
        }
    }

    pub fn value(self) -> u64 {
        match self {
            ID::Local(id) => id,
            ID::Online(id) => id,
            ID::Transitional(_, id) => id,
        }
    }

    pub fn value_option(&self) -> OptionID {
        let value = self.value();
        match value {
            u64::MIN => OptionID::None,
            u64::MAX => OptionID::Reserved,
            _ => OptionID::Some(value)
        }
    }
}

static LOCAL_SERIAL: AtomicU64 = AtomicU64::new(101);

pub fn init_local_id_generator(last_serial: u64) -> &'static AtomicU64 {
    LOCAL_SERIAL.store(last_serial, Ordering::Relaxed);
    &LOCAL_SERIAL
}

pub fn generate_local_id() -> ID {
    let serial = LOCAL_SERIAL.fetch_add(1, Ordering::Relaxed);
    ID::Local(serial)
}