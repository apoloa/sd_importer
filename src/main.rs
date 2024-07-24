use std::fs;
use std::env;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use walkdir::WalkDir;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::collections::HashSet;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::{Mutex, Arc};
use std::ffi::OsString;
use std::fs::File;
use std::io::BufReader;
use exif::{Reader, Tag, In, Value};
use chrono::{NaiveDateTime, DateTime, Utc};


fn get_date_time_exif(path: &Path) -> Option<DateTime<Utc>> {
    let file = File::open(path).ok()?;
    let mut bufreader = BufReader::new(&file);
    let exif = Reader::new().read_from_container(&mut bufreader).ok()?;

    if let Some(field) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
        if let Value::Ascii(ref vec) = field.value {
            if !vec.is_empty() {
                let datetime_str = std::str::from_utf8(&vec[0]).ok()?;
                let naive_datetime = NaiveDateTime::parse_from_str(datetime_str, "%Y:%m:%d %H:%M:%S").ok()?;
                let datetime = DateTime::from_utc(naive_datetime, Utc);
                return Some(datetime);
            }
        }
    }
    None
}

fn extract_date_time(path: &Path) -> DateTime<Utc> {
       return get_date_time_exif(path).unwrap_or_else(|| {
            let metadata = fs::metadata(&path).unwrap();
            let creation_time_sec = metadata.ctime();
            let naive_datetime = NaiveDateTime::from_timestamp_opt(creation_time_sec as i64, 0);
            let datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime.unwrap(), Utc);
            datetime
        })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: sd_importer <SD path> <Destination directory>");
        std::process::exit(1);
    }

    let sd_card_path = &args[1];
    let destination_dir = &args[2];

    let mut unaccepted_extensions: HashSet<OsString> = HashSet::new();


    let all_files : Vec<_> = WalkDir::new(sd_card_path).into_iter().collect();
    println!("Scanning {} files", all_files.len());

    let files: Vec<_> = WalkDir::new(sd_card_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| match e.path().extension() {
            Some(ext) => {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ext_str == "arw" || ext_str == "mp4" || ext_str == "jpg" || ext_str == "heic" {
                    true
                } else {
                    unaccepted_extensions.insert(ext.to_os_string());
                    false
                }
            },
            None => false,
        })
        .collect();
    
    if files.is_empty() {
        eprintln!("Not found any element inside the path provided");
        std::process::exit(1);
    }
    println!("It will import {}", files.len());

    println!("Unaccepted extensions:");
    for ext in unaccepted_extensions {
        println!("{}", ext.to_string_lossy());
    }
    
    let progress_bar = ProgressBar::new(files.len() as u64);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} [{per_sec}] ({eta_precise})")
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "));

    // Create a thread pool with a specific number of threads
    let pool = ThreadPoolBuilder::new()
        .num_threads(6)  // set the number of threads here
        .build()
        .unwrap();

    let directories_created = Arc::new(Mutex::new(HashSet::new()));

    pool.install(|| {
        files.par_iter().for_each(|entry| {
            let path = entry.path();
            
            // Get file creation date
            let datetime = extract_date_time(path);
            
            let year = datetime.format("%Y").to_string();
            let full_date = datetime.format("%Y-%m-%d").to_string();

            // Construct the destination path
            let destination_path = Path::new(destination_dir)
                .join(year)
                .join(full_date)
                .join(path.file_name().unwrap());
                    

            // Create directories if they don't exist
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent).expect("Failed to create directories");
                directories_created.lock().unwrap().insert(parent.to_path_buf());
            }

            match fs::copy(path, &destination_path) {
                Ok(_) => progress_bar.inc(1),
                Err(e) => eprintln!("Error al copiar el archivo {}: {}", path.display(), e),
            }
        });
        progress_bar.finish_with_message("Completed copy");
    });

    for path in directories_created.lock().unwrap().iter() {
        println!("{}", path.display());
    }

}
