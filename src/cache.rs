/**
 * Directory cache
 *
 * This module provise cache with directory structure.
 *
 * The cache is stored in the following format:
 * cache_dir/{key1}/{key2}/../{key_n}/data
 *
 */
use std::{fs, io::Write, path::PathBuf};
pub mod errors;

pub struct Client {
    cache_dir: PathBuf,
    filename: String,
}

impl Client {
    pub fn new(cache_dir: PathBuf, filename: String) -> Client {
        Client {
            cache_dir,
            filename,
        }
    }

    fn dirpath_by_keys(&self, keys: &[&str]) -> PathBuf {
        let mut path = self.cache_dir.clone();
        for key in keys {
            path.push(key);
        }
        path
    }

    fn filepath_by_keys(&self, keys: &[&str]) -> PathBuf {
        let mut path = self.dirpath_by_keys(keys);
        path.push(&self.filename);
        path
    }

    fn ensure_dir(&self, keys: &[&str]) -> Result<(), errors::Error> {
        let path = self.dirpath_by_keys(keys);
        fs::create_dir_all(path)?;
        Ok(())
    }

    /**
     * Save data to cache with specified keys.
     */
    pub fn save(&self, keys: &[&str], data: &[u8]) -> Result<(), errors::Error> {
        let path = self.filepath_by_keys(keys);
        self.ensure_dir(keys)?;
        let file = fs::File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);
        writer.write_all(data)?;
        Ok(())
    }

    /**
     * Load data from cache with specified keys.
     */
    pub fn load(&self, keys: &[&str]) -> Result<Vec<u8>, errors::Error> {
        let path = self.filepath_by_keys(keys);
        Ok(fs::read(path)?)
    }

    /**
     * Remove cache with specified keys.
     */
    pub fn remove(&self, keys: &[&str]) -> Result<(), errors::Error> {
        let path = self.filepath_by_keys(keys);
        fs::remove_file(path)?;
        Ok(())
    }

    /**
     * Check if cache exists with specified keys.
     */
    pub fn has(&self, keys: &[&str]) -> bool {
        let path = self.filepath_by_keys(keys);
        path.exists()
    }
}
