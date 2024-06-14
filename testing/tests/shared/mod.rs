use std::path::PathBuf;
use lazy_static::lazy_static;
use asmov_testing::{self as testing, UseCase};

lazy_static!{
    pub(crate) static ref NAMEPATH: testing::Namepath =
        testing::Namepath::module(testing::UseCase::Integration, "shared".to_string());

    pub(crate) static ref FIXTURE_DIR: PathBuf = 
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testing")
            .join("fixtures")
            .join(UseCase::Integration.to_str())
            .join("shared")
            .canonicalize()
            .unwrap();
}

