use lazy_static::lazy_static;
use std::{fs, path::PathBuf};

lazy_static! {
    pub static ref ROOT_DIR: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pub static ref DATA_DIR: PathBuf = ROOT_DIR.join("data");
    pub static ref FIXTURES_DIR: PathBuf = DATA_DIR.join("fixtures");
    pub static ref HEADER_DIR: PathBuf = DATA_DIR.join("include");
    pub static ref GRAMMARS_DIR: PathBuf = FIXTURES_DIR.join("grammars");
    pub static ref TEST_GRAMMARS_DIR: PathBuf = FIXTURES_DIR.join("test_grammars");
    pub static ref SCRATCH_DIR: PathBuf = {
        let result = ROOT_DIR
            .join(option_env!("CARGO_TARGET_DIR").unwrap_or("target"))
            .join("scratch");
        fs::create_dir_all(&result).unwrap();
        result
    };
}
