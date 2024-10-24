use std::sync::atomic::{AtomicU64, Ordering};
use std::hash::{Hash, Hasher};

/// Represents either a local (purely offline) or authorative (remote database) ID.
/// Once an authorative ID is assigned, the former Local ID should be converted to Synchronized
/// or Authorative (with migration). 
/// Valid between [ID_NONE] and [ID_RESERVED] non-inclusive.
#[derive(Debug, Clone, Copy, Eq, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
pub enum ID {
    Local(u64),
    Authorative(u64),
    Mutual(u64, u64)
}

impl ID {
    pub const AUTHORATIVE_NONE: ID = ID::Authorative(u64::MIN);
    pub const AUTHORATIVE_RESERVED: ID = ID::Authorative(u64::MAX);
    pub const LOCAL_RESERVED: ID = ID::Local(u64::MAX);
    pub const LOCAL_NONE: ID = ID::Local(u64::MIN);

    pub const LOCAL_USER: ID = ID::Local(1);
}

pub enum OptionID {
    /// Database equivalent of NULL
    None,
    /// Intended for use by application developers. Logic only. Not valid for database storage.
    Reserved,
    /// Valid ID. Not NULL and not Reserved.
    Some(ID) 
}

impl PartialEq<ID> for ID {
    fn eq(&self, other: &ID) -> bool {
        match (self, other) {
            (ID::Local(a), ID::Local(b)) => a == b,
            (ID::Authorative(a), ID::Authorative(b)) => a == b,
            (ID::Mutual(a, aa), ID::Mutual(b, bb)) => a == b || aa == bb,
            (ID::Local(a), ID::Mutual(b, _)) => a == b,
            (ID::Authorative(a), ID::Mutual(_, b)) => a == b,
            (ID::Mutual(a, _), ID::Local(b)) => a == b,
            (ID::Mutual(_, a), ID::Authorative(b)) => a == b,
            _ => false
        }
    }
}

impl Hash for ID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ID::Local(id) => { 1.hash(state) ; id.hash(state) },
            ID::Authorative(id) => { 2.hash(state) ; id.hash(state) },
            ID::Mutual(_, id) => { 2.hash(state) ; id.hash(state) },
        }
    }
}

impl ID {
    /// Returns the authorative ID value if available.
    pub fn authorative(&self) -> Option<u64> {
        match self {
            ID::Authorative(id) => Some(*id),
            ID::Mutual(_, id) => Some(*id), 
            _ => None
        }
    }

    /// Returns the local ID value if available.
    pub fn local(&self) -> Option<u64> {
        match self {
            ID::Local(id) => Some(*id),
            ID::Mutual(id, _) => Some(*id), 
            _ => None
        }
    }

    /// Returns the best ID value available, prioritizing authorative over local.
    pub fn best(&self) -> u64 {
        match self {
            ID::Local(id) => *id,
            ID::Authorative(id) => *id,
            ID::Mutual(_, id) => *id,
        }
    }

    /// Returns a SQL compatible ID value.
    pub fn sql(&self) -> i64 {
        self.best() as i64
    }

    pub fn valid(&self) -> bool {
        match *self {
            Self::Authorative(u64::MIN) => false,
            Self::Authorative(u64::MAX) => false,
            Self::Authorative(_) => true,
            Self::Local(u64::MIN) => false,
            Self::Local(u64::MAX) => false,
            Self::Local(_) => true,

            #[cfg(debug_assertions)]
            Self::Mutual(l, a) if l == u64::MIN || l == u64::MAX || a == u64::MIN || a == u64::MAX 
                => panic!("Mutual ID should have valid local and authorative IDs"),

            Self::Mutual(_, _) => true
        }
    }

    pub fn best_valid_sql(&self) -> Option<i64> {
        match *self {
            Self::Authorative(u64::MIN) => None,
            Self::Authorative(u64::MAX) => None,
            Self::Authorative(id) => Some(id as i64),
            Self::Local(u64::MIN) => None,
            Self::Local(u64::MAX) => None,
            Self::Local(id) => Some(id as i64),

            #[cfg(debug_assertions)]
            Self::Mutual(l, a) if l == u64::MIN || l == u64::MAX || a == u64::MIN || a == u64::MAX 
                => panic!("Mutual ID should have valid local and authorative IDs"),

            Self::Mutual(_, id) => Some(id as i64) 
        }
    }

    /// Converts this ID into an OptionID.
    pub fn into_option(self) -> OptionID {
        match self {
            Self::Authorative(u64::MIN) => OptionID::None,
            Self::Authorative(u64::MAX) => OptionID::Reserved,
            Self::Authorative(_) => OptionID::Some(self),
            Self::Local(u64::MIN) => OptionID::None,
            Self::Local(u64::MAX) => OptionID::Reserved,
            Self::Local(_) => OptionID::Some(self),

            #[cfg(debug_assertions)]
            Self::Mutual(l, a) if l == u64::MIN || l == u64::MAX || a == u64::MIN || a == u64::MAX 
                => panic!("Mutual ID should have valid local and authorative IDs"),

            Self::Mutual(_, _) => OptionID::Some(self)
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