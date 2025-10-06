use cocoatly_core::error::{CocoatlyError, Result};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::fs::{File, create_dir_all};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use walkdir::WalkDir;

pub struct TarGzCompressor;

impl TarGzCompressor {
    pub fn compress<P: AsRef<Path>, Q: AsRef<Path>>(
        source_dir: P,
        output_file: Q,
    ) -> Result<u64> {
        let tar_gz = File::create(output_file.as_ref())?;
        let enc = GzEncoder::new(tar_gz, Compression::best());
        let mut tar = Builder::new(enc);

        let source_path = source_dir.as_ref();

        for entry in WalkDir::new(source_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                let relative_path = path.strip_prefix(source_path)
                    .map_err(|e| CocoatlyError::IoError(
                        io::Error::new(io::ErrorKind::Other, e.to_string())
                    ))?;

                tar.append_path_with_name(path, relative_path)?;
            }
        }

        let mut enc = tar.into_inner()?;
        enc.flush()?;
        let file_size = enc.get_ref().metadata()?.len();

        Ok(file_size)
    }

    pub fn decompress<P: AsRef<Path>, Q: AsRef<Path>>(
        archive_path: P,
        output_dir: Q,
    ) -> Result<Vec<PathBuf>> {
        let tar_gz = File::open(archive_path.as_ref())?;
        let dec = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(dec);

        let output_path = output_dir.as_ref();
        create_dir_all(output_path)?;

        let mut extracted_files = Vec::new();

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let output_file_path = output_path.join(&*path);

            if let Some(parent) = output_file_path.parent() {
                create_dir_all(parent)?;
            }

            entry.unpack(&output_file_path)?;
            extracted_files.push(output_file_path);
        }

        Ok(extracted_files)
    }

    pub fn list_contents<P: AsRef<Path>>(archive_path: P) -> Result<Vec<String>> {
        let tar_gz = File::open(archive_path.as_ref())?;
        let dec = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(dec);

        let mut contents = Vec::new();

        for entry in archive.entries()? {
            let entry = entry?;
            let path = entry.path()?;
            contents.push(path.to_string_lossy().to_string());
        }

        Ok(contents)
    }
}

pub fn compress_directory<P: AsRef<Path>, Q: AsRef<Path>>(
    source: P,
    destination: Q,
) -> Result<u64> {
    TarGzCompressor::compress(source, destination)
}

pub fn extract_archive<P: AsRef<Path>, Q: AsRef<Path>>(
    archive: P,
    destination: Q,
) -> Result<Vec<PathBuf>> {
    TarGzCompressor::decompress(archive, destination)
}
