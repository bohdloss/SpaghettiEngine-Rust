use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind, Read};
use std::path::Path;
use image::{DynamicImage, open, RgbaImage};
use crate::utils::Logger;

pub fn path_to_bin(path: &Path) -> io::Result<Vec<u8>> {
    let current = std::env::current_dir()?;
    let full = current.join(path);
    absolute_path_to_bin(&full)
}

pub fn absolute_path_to_bin(path: &Path) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    file_to_bin(file)
}

pub fn file_to_bin(mut file: File) -> io::Result<Vec<u8>> {
    let metadata = file.metadata()?;
    let mut buffer: Vec<u8> = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn path_to_image(path: &Path) -> io::Result<DynamicImage> {
    let current = std::env::current_dir()?;
    let full = current.join(path);
    absolute_path_to_image(&full)
}

pub fn absolute_path_to_image(path: &Path) -> io::Result<DynamicImage> {
    let file = File::open(path)?;
    file_to_image(file)
}

pub fn file_to_image(file: File) -> io::Result<DynamicImage> {
    let bytes = file_to_bin(file)?;
    match image::load_from_memory(&bytes) {
        Ok(img) => {
            Ok(img)
        },
        Err(error) => {
            Logger::warning_err("Error loading image", &error);
            Err(Error::from(ErrorKind::Other))
        }
    }
}