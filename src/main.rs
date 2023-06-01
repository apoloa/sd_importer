use std::fs;
use std::env;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use walkdir::WalkDir;
use chrono::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: sd_importer <SD path> <Destination directory>");
        std::process::exit(1);
    }

    let sd_card_path = &args[1];
    let destination_dir = &args[2];
    

    for entry in WalkDir::new(sd_card_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .filter(|e| match e.path().extension() {
            Some(ext) => ext == "ARW" || ext == "MP4",
            None => false,
        }) {
        let metadata = match entry.metadata() {
            Ok(md) => md,
            Err(e) => {
                eprintln!("Error al obtener metadatos para {}: {}", entry.path().display(), e);
                return;
            },
        }; 
        let creation_time_sec = metadata.ctime();
        let naive_datetime = NaiveDateTime::from_timestamp_opt(creation_time_sec as i64, 0);
        let datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime.unwrap(), Utc);
        
        let year = datetime.format("%Y").to_string();
        let full_date = datetime.format("%Y-%m-%d").to_string();

        // Construct the destination path
        let destination_path = Path::new(destination_dir)
            .join(year)
            .join(full_date)
            .join(entry.path().file_name().unwrap());
        
        // Create directories if they don't exist
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directories");
        }
        match fs::copy(entry.path(), &destination_path) {
            Ok(_) => println!("Copiado {} a {}", entry.path().display(), destination_path.display()),
            Err(e) => eprintln!("Error al copiar el archivo {}: {}", entry.path().display(), e),
        }    
    }



}
