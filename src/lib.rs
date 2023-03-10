use std::fs::OpenOptions;
use std::{env, fs, io};
use std::io::{Read, Write};
use std::iter::Zip;
use std::path::Path;
use quick_xml::Error;
use uuid::Uuid;
use crate::DocxError::{IoError, XmlError, ZipError};
// use tempfile::tempdir;

mod zip_utils;
mod xml_utils;

// struct Docx {
//     file_bytes: Vec<u8>
// }

// impl Docx {
//     pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Self {
//         return Self { file_bytes: bytes.as_ref().to_vec() }
//     }
//
//     pub fn from_path(path: &Path) -> Result<Self, io::Error> {
//         let mut file = std::fs::File::open(path).unwrap();
//         let mut file_bytes = Vec::<u8>::new();
//         file.read_to_end(&mut file_bytes)?;
//         Ok(Self { file_bytes })
//     }
//
//     pub fn make_docx_readonly() -> Result<Vec<u8>, ()> {
//         // make temporary dir
//         let tmp_dir = env::temp_dir();
//         let unique_name: String = Uuid::new_v4();
//         let unique_tmp_dir = tmp_dir.join(unique_name);
//         // let tmp_dir_path: &Path = tmp_dir.path();
//         // unpack docx to this temporary dir
//         zip_utils::extract_archive((), tmp_dir_path)?;
//         // let mut settings_xml_file = std::fs::File::open(tmp_dir_path.join("word/settings.xml"))?;
//         let mut settings_xml_file = OpenOptions::new()
//             .read(true)
//             .write(true)
//             .create(true)
//             .open(tmp_dir_path.join("word/settings.xml"))?;
//         let mut settings_xml_bytes = Vec::<u8>::new();
//         settings_xml_file.read_to_end(&mut settings_xml_bytes)?;
//
//         let settings_xml_bytes = xml_utils::apply_settings_readonly(
//             settings_xml_bytes.as_ref())?;
//         settings_xml_file.write_all(settings_xml_bytes.as_ref())?;
//
//         // repack docx
//         let repacked_docx_path = "readonly.docx";
//         zip_utils::build_archive_by_dir((), tmp_dir_path)?;
//
//         Ok(())
//     }
// }

#[derive(Debug)]
pub enum DocxError {
    ZipError(zip::result::ZipError),
    XmlError(quick_xml::Error),
    IoError(io::Error),
}

pub fn make_docx_readonly(src_docx_path: &Path, dst_docx_path: &Path) -> Result<(), DocxError> {
    // make temporary dir
    let tmp_dir = env::temp_dir();
    let unique_name: String = Uuid::new_v4().to_string();
    let unique_tmp_dir = tmp_dir.join(unique_name);
    println!("unique_tmp_dir - {:?}", unique_tmp_dir);
    println!("settings xml path - {:?}", unique_tmp_dir.join("word/settings.xml"));
    // let tmp_dir_path: &Path = tmp_dir.path();
    // unpack docx to this temporary dir
    match zip_utils::extract_archive(src_docx_path, &unique_tmp_dir) {
        Ok(_) => {},
        Err(e) => { return Err(IoError(e)) }
    };
    // let mut settings_xml_file = std::fs::File::open(tmp_dir_path.join("word/settings.xml"))?;
    let mut settings_xml_bytes = Vec::<u8>::new();
    let mut settings_xml_file = match fs::File::open(unique_tmp_dir.join("word/settings.xml")) {
        Ok(f) => f,
        Err(e) => { println!("{}", e);
            return Err(IoError(e)) }
    };
    match settings_xml_file.read_to_end(&mut settings_xml_bytes) {
        Ok(_) => {},
        Err(e) => { return Err(XmlError(e.into())) }
    };

    let settings_xml_bytes = match xml_utils::apply_settings_readonly(
        settings_xml_bytes.as_ref()) {
        Ok(s) => s,
        Err(e) => { return Err(XmlError(e.into())) }
    };
    let mut settings_xml_file = match fs::File::create(unique_tmp_dir.join("word/settings.xml")) {
        Ok(f) => f,
        Err(e) => { return Err(IoError(e)) }
    };
    match settings_xml_file.write_all(settings_xml_bytes.as_ref()) {
        Ok(_) => {}
        Err(e) => { return Err(IoError(e)) }
    };

    // repack docx
    let repacked_docx_path = "readonly.docx";
    match zip_utils::build_archive_by_dir(dst_docx_path, &unique_tmp_dir) {
        Ok(_) => {}
        Err(e) => { return Err(ZipError(e)) }
    };

    Ok(())
}



// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }

    #[test]
    fn make_docx_readonly_test() {
        let source_docx_path = Path::new("test_artifacts/source.docx");
        let readonly_dir = Path::new("test_artifacts/readonly/readonly11.docx");
        make_docx_readonly(source_docx_path, readonly_dir).unwrap();
    }

    #[test]
    fn show_temp_dir() {
        let dir = env::temp_dir();
        println!("Temporary directory: {}", dir.display());
    }
}
