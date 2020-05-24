use crate::typedefs::BoxErr;
use std::{fs::File, io::BufReader, io::Read, path::PathBuf};

lazy_static::lazy_static! {

    static ref CURRENT_EXE_PATH: PathBuf = {
        std::env::current_exe().expect("Unable to determine current exe path").to_path_buf()
    };

    static ref TOP_PATH: PathBuf = {
        calculate_top_path()
    };

    static ref ENGINE_ASSETS_PATH: PathBuf = {
        calculate_engine_assets_path()
    };
}

fn calculate_top_path() -> PathBuf {
    let mut p: PathBuf = CURRENT_EXE_PATH.clone();
    // Ignore the exe file name itself
    p.pop();

    if p.file_name().unwrap().to_str().unwrap() == "debug" {
        log::trace!("The executable file is probably located in \"target/debug\".");

        // Go to root
        p.pop();
        p.pop();

        p
    } else if p.file_name().unwrap().to_str().unwrap() == "test_rel" {
        p
    } else {
        log::error!(
            "We don't know where the relevant folders are relative \
             to the current location of the executable file."
        );
        panic!(
            "Unimplemented: We don't know where the relevant folders are relative \
             to the current location of the executable file."
        )
    }
}

fn calculate_engine_assets_path() -> PathBuf {
    let mut p: PathBuf = TOP_PATH.clone();

    p.push("mat_engine");
    p.push("assets");

    p
}

pub(crate) fn get_engine_assets_path() -> PathBuf {
    ENGINE_ASSETS_PATH.clone()
}

pub fn get_folder_assets_path(folder: &str) -> PathBuf {
    let mut p = TOP_PATH.clone();
    p.push(folder);
    p.push("assets");
    p
}

pub fn read_file_at_path_to_string(path: PathBuf) -> Result<String, BoxErr> {
    let mut file = File::open(path)?;
    read_file_to_string(&mut file)
}

pub fn read_file_at_path_to_bytes(path: PathBuf) -> Result<Vec<u8>, BoxErr> {
    let mut file = File::open(path)?;
    read_file_to_bytes(&mut file)
}

fn read_file_to_string(file: &mut File) -> Result<String, BoxErr> {
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(contents)
}

fn read_file_to_bytes(file: &mut File) -> Result<Vec<u8>, BoxErr> {
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents)?;
    Ok(contents)
}
