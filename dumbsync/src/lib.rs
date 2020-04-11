#![forbid(unsafe_code)]

use blake3::{Hash, Hasher};
use pathdiff::diff_paths;
use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;
use ureq;
use url::Url;
use walkdir::WalkDir;

#[derive(Copy, Clone, Debug)]
pub enum DumbItem {
    Local,
    Remote,
    Uptodate,
    Outdated,
}

fn hash_file(path: &PathBuf) -> Result<Hash, Box<dyn Error>> {
    let mut hasher = Hasher::new();
    let mut reader = BufReader::with_capacity(65536, File::open(path)?);
    std::io::copy(&mut reader, &mut hasher)?;
    Ok(hasher.finalize())
}

fn download_file(url: &str, mut file: File) -> Result<File, Box<dyn Error>> {
    let res = ureq::get(url).call();
    io::copy(&mut res.into_reader(), &mut file)?;
    Ok(file)
}

fn process_item(
    path: &PathBuf,
    item: &DumbItem,
    base_url: &Url,
    target: &PathBuf,
    purge: &bool,
) -> Result<(), Box<dyn Error>> {
    match item {
        DumbItem::Uptodate => println!("Uptodate: {}", path.to_string_lossy()),
        DumbItem::Local => {
            if *purge {
                println!("Purging {}", path.to_string_lossy());
                fs::remove_file(path)?;
            }
        }
        _ => {
            // Outdated or Remote
            println!("Downloading {}", path.to_string_lossy());
            let file_url = base_url.join(path.to_string_lossy().as_ref())?;
            let output_path = target.join(path);
            match output_path.parent() {
                Some(path) => fs::create_dir_all(path)?,
                None => (),
            };
            let output_file = File::create(&output_path)?;
            download_file(file_url.as_str(), output_file)?;
        }
    }
    Ok(())
}

pub fn aggregate(
    url: &str,
    target: &PathBuf,
) -> Result<HashMap<PathBuf, DumbItem>, Box<dyn Error>> {
    let mut hash_map = HashMap::new();
    let mut local_files = Vec::new();
    let mut url = String::from(url);
    if !url.ends_with("/") {
        url.push('/');
    }
    let base_url = Url::parse(&url)?;

    for entry in WalkDir::new(&target) {
        let raw_path = entry?.into_path();
        if raw_path.is_file() {
            let path = match diff_paths(&raw_path, target) {
                Some(p) => p,
                None => {
                    println!(
                        "Failed to create path of {} relative to {}",
                        &raw_path.to_string_lossy(),
                        target.to_string_lossy()
                    );
                    raw_path
                }
            };
            local_files.push(path);
        }
    }

    let dumb_url = base_url.join(".dumbsync")?;
    let res = ureq::get(dumb_url.as_str()).call();
    let dumb_file = String::from(res.into_string()?);
    for line in dumb_file.lines() {
        if line.contains(" ") {
            let splitn: Vec<&str> = line.splitn(2, " ").collect();
            let remote_hash = *splitn.get(0).unwrap();
            let path = PathBuf::from(*splitn.get(1).unwrap());

            if local_files.contains(&path) {
                local_files.retain(|l| !path.eq(l));

                let local_hash = hash_file(&target.join(&path))?;
                if remote_hash.eq(local_hash.to_hex().as_ref()) {
                    hash_map.insert(path, DumbItem::Uptodate);
                } else {
                    hash_map.insert(path, DumbItem::Outdated);
                }
            } else {
                hash_map.insert(path, DumbItem::Remote);
            }
        }
    }

    for path in local_files {
        hash_map.insert(path, DumbItem::Local);
    }

    Ok(hash_map)
}

/// Download from URL into target. Optionally purge files that don't exist on the remote.
pub fn download(
    url: &str,
    target: &PathBuf,
    aggregated: &HashMap<PathBuf, DumbItem>,
    purge: &bool,
) -> Result<(), Box<dyn Error>> {
    let mut url = String::from(url);
    if !url.ends_with("/") {
        url.push('/');
    }
    let base_url = Url::parse(&url)?;

    aggregated.par_iter().for_each(|(path, item)| {
        process_item(path, item, &base_url, target, purge)
            .expect(&format!("Failed to process {}", path.to_string_lossy()))
    });

    Ok(())
}

/// Hash files/folders recursively and write to file
pub fn generate(dir: &PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    let outfile = dir.join(".dumbsync");
    let mut file_content = String::new();

    for entry in WalkDir::new(dir) {
        let raw_path = entry?.into_path();
        let path = match diff_paths(&raw_path, dir) {
            Some(p) => p,
            None => {
                println!(
                    "Failed to create path of {} relative to {}",
                    raw_path.to_string_lossy(),
                    dir.to_string_lossy()
                );
                raw_path
            }
        };
        if path.is_file() {
            let hash = hash_file(&path)?;
            file_content = file_content + &format!("{} {}\n", hash.to_hex(), path.to_string_lossy())
        }
    }

    fs::write(outfile.as_path(), file_content)?;
    Ok(outfile)
}
