use std::{fs, io};
use std::io::Error;
// use std::io::Read;
use std::path::Path;
use std::io::prelude::*;
use walkdir::WalkDir;
use zip::read::ZipFile;
use zip::result::ZipError;

const DOCX_SETTINGS_PATH: &str = "word/settings.xml";

pub fn extract_settings(path: &Path) -> Option<Vec<u8>> {
    let zipfile = std::fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(zipfile).ok()?;

    let mut settings_xml: ZipFile = archive.by_name(DOCX_SETTINGS_PATH).ok()?;
    let mut settings_xml_bytes = Vec::<u8>::new();

    settings_xml.read_to_end(&mut settings_xml_bytes).ok()?;
    Some(settings_xml_bytes)
}

pub fn extract_archive(path: &Path, dst_dir: &Path) -> Result<(), io::Error> {
    let zipfile = std::fs::File::open(path).unwrap();
    let mut archive = zip::ZipArchive::new(zipfile).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();

        // extract only path-safe files
        let outpath = match file.enclosed_name() {
            Some(path) => {
                let mut p = dst_dir.to_path_buf();
                p.push(path);
                p
            },
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            // println!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    }

    Ok(())
}

pub fn write_settings(path: &Path, new_settings: Vec<u8>) -> Result<(), std::io::Error> {
    /**
    Doesnt work if file already exist in archive (cannot overwrite)
     */
    // let zipfile = std::fs::File::open(path)?;
    // println!("1");
    // let mut archive = zip::ZipWriter::new_append(zipfile)?;
    // println!("2");
    //
    // archive.start_file(DOCX_SETTINGS_PATH, Default::default())?;
    // println!("3");
    // archive.write_all(new_settings.as_ref())?;
    // println!("4");
    // archive.finish()?;
    // println!("5");
    Ok(())
}

pub fn build_archive_by_dir(outfile_path: &Path, dir_path: &Path) -> Result<(), ZipError> {
    // check `dir_path` is dit
    if !Path::new(dir_path).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let file = std::fs::File::create(outfile_path).unwrap();
    let mut archive = zip::ZipWriter::new(file);

    let mut buffer = Vec::new();
    let walkdir = WalkDir::new(dir_path);

    // let it = walkdir.into_iter();
    for file in walkdir.into_iter().filter_map(|f| f.ok()) {
        let file_path = file.path();
        let name = file_path.strip_prefix(dir_path).unwrap();

        if file_path.is_file() {
            archive.start_file(name.to_str().unwrap(), Default::default())?;
            let mut f = std::fs::File::open(file_path)?;
            f.read_to_end(&mut buffer)?;
            archive.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            archive.add_directory(name.to_str().unwrap(), Default::default())?;
        }
    }
    archive.finish()?;

    Ok(())
}

fn repack_zip_buf_with_custom_function<Buf: AsRef<[u8]>>(
    zip_buf: Buf,
    buf_modifier_func: fn(&[u8]) -> Vec<u8>
) -> Result<Vec<u8>, ZipError> {

    let mut archive = zip::ZipArchive::new(io::Cursor::new(zip_buf)).unwrap();

    let mut repacked_archive_buf = Vec::<u8>::new();
    {
        let mut repacked_archive = zip::ZipWriter::new(std::io::Cursor::new(&mut repacked_archive_buf));

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            println!("file {}", file.name());

            if file.is_file() {
                match file.name() {
                    DOCX_SETTINGS_PATH => {
                        // this file should be overwritten
                        let mut settings_xml_bytes = Vec::<u8>::new();
                        file.read_to_end(&mut settings_xml_bytes).unwrap();
                        let settings_xml_bytes = buf_modifier_func(&settings_xml_bytes);

                        repacked_archive.start_file(file.name(), Default::default())?;
                        repacked_archive.write_all(&settings_xml_bytes)?;
                    }
                    _ => {
                        // repacked_archive.start_file(file.name(), Default::default())?;
                        // repacked_archive.write_all()?;
                        repacked_archive.raw_copy_file(file)?;
                    },
                }
            } else {
                repacked_archive.add_directory(file.name(), Default::default())?;
            }
        }
        repacked_archive.finish()?;
    }

    Ok(repacked_archive_buf)
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{Read, Write};
    use std::path::Path;
    use crate::zip_utils;

    #[test]
    fn extract_settings_test() {
        let source_docx_path = Path::new("test_artifacts/source.docx");
        let docx_settings = zip_utils::extract_settings(source_docx_path);
        // println!("{:?}", docx_settings);

        assert!(docx_settings.is_some())
    }

    #[test]
    fn extract_archive_test() {
        let source_docx_path = Path::new("test_artifacts/source.docx");
        let test_dst = Path::new("test_artifacts/test");
        zip_utils::extract_archive(source_docx_path, test_dst).expect("Oops archive extraction error");
    }

    #[test]
    fn build_archive_by_dir_test() {
        let test_docx_dst = Path::new("test_artifacts/source_restored.docx");
        let dir_path = Path::new("test_artifacts/test");
        zip_utils::build_archive_by_dir(test_docx_dst, dir_path).expect("Oops archive build error");
    }

    #[test]
    fn write_settings_test() {
        let readonly_settings_path = Path::new("test_artifacts/settings_readonly_pass_123.xml");
        let mut file = File::open(readonly_settings_path).unwrap();
        let mut settings_bytes = Vec::<u8>::new();
        file.read_to_end(&mut settings_bytes).unwrap();

        let docx_path = Path::new("test_artifacts/insertable.docx");
        let docx_settings = zip_utils::write_settings(docx_path, settings_bytes).unwrap();
        // println!("{:?}", docx_settings);
        // assert!(docx_settings.is_ok())
    }

    #[test]
    fn repack_zip_buf_with_custom_function_test() {
        fn abc(b: &[u8]) -> Vec<u8> {
            let v: Vec<u8> = b.into();
            println!("{:?}", &v[10..20]);
            v
        }

        let source_docx_path = Path::new("test_artifacts/source.docx");
        let mut file = File::open(source_docx_path).unwrap();
        let mut docx_buf = Vec::<u8>::new();
        file.read_to_end(&mut docx_buf).unwrap();

        let result = zip_utils::repack_zip_buf_with_custom_function(docx_buf, abc).expect("Oops archive repack error");
        let repacked_docx_path = Path::new("test_artifacts/source_repacked.docx");
        let mut file = File::create(repacked_docx_path).unwrap();
        file.write_all(&result).unwrap();
    }
}
