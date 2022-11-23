use rocket::serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum FileKind {
    Directory,
    File,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct FileMetadata {
    id: String,
    name: String,
    size: u64,
    kind: FileKind,
    mime: String,
    path: Vec<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct FileMetadatas {
    files: Vec<FileMetadata>,
}

fn extract_mime(filename: &str) -> String {
    let parts: Vec<&str> = filename.split('.').collect();
    let res = match parts.last() {
        Some(v) => match *v {
            "png" => "image/png",
            "jpg" => "image/jpeg",
            &_ => "application/octet-stream",
        },
        None => "application/octet-stream",
    };
    return String::from(res);
}

fn create_id(filename: &String) -> String {
    base64::encode_config(filename, base64::URL_SAFE_NO_PAD)
}

fn parse_id(value: &str) -> Result<String, String> {
    match base64::decode_config(value, base64::URL_SAFE_NO_PAD) {
        Ok(r) => match std::str::from_utf8(&r) {
            Ok(r_str) => return Ok(String::from(r_str)),
            Err(e) => return Err(e.to_string()),
        },
        Err(e) => return Err(e.to_string()),
    }
}

fn path_to_vec(path: &PathBuf) -> Vec<String> {
    let srcdir = std::path::PathBuf::from(path);
    const VERBATIM_PREFIX: &str = r#"\\?\"#;
    let resolved_path = fs::canonicalize(&srcdir);
    let resolve_path_unwrap = resolved_path.unwrap();
    let mut str = resolve_path_unwrap.to_str().unwrap();
    if str.starts_with(VERBATIM_PREFIX) {
        str = &str[VERBATIM_PREFIX.len()..];
    }
    let replaced = str.replace("\\", "/");
    let splitted = replaced.split("/");
    let vec = splitted.map(str::to_string).collect();
    return vec;
}

fn get_absolute_path(root_path: &PathBuf, path: &PathBuf) -> Vec<String> {
    let root_vec = path_to_vec(root_path);
    let mut path_vec = path_to_vec(path);
    for _ in root_vec {
        if 0 < path_vec.len() {
            path_vec.remove(0);
        }
    }
    return path_vec;
}

pub fn get(root: &str, id: &str) -> Result<FileMetadatas, String> {
    println!("Getting files:{}, id:{}", root, id);
    let path: &std::path::Path;
    let path_str: String;
    if 0 < id.len() {
        path_str = match parse_id(&id) {
            Ok(result) => result,
            Err(error) => {
                return Err(error);
            }
        };
        path = std::path::Path::new(&path_str);
    } else {
        path = std::path::Path::new(&root);
    }
    if !path.exists() {
        return Err(String::from("Path does not exists"));
    }
    let mut result: Vec<FileMetadata> = vec![];
    let entries = match fs::read_dir(path) {
        Ok(result) => result,
        Err(error) => {
            return Err(error.to_string());
        }
    };
    for entry in entries {
        let entry_file = entry.expect("Error");
        let metadata = entry_file.metadata().expect("Error reading metadata");
        let file_name = entry_file
            .file_name()
            .into_string()
            .expect("Error converting str");
        let file_metadata: FileMetadata;
        let file_path_buf = entry_file.path();
        let file_path = file_path_buf.to_str();
        let file_path_str = file_path.unwrap();
        let file_path_string = String::from(file_path_str);
        let id = create_id(&file_path_string);
        if metadata.is_dir() {
            file_metadata = FileMetadata {
                id: id,
                name: file_name,
                size: metadata.len(),
                kind: FileKind::Directory,
                mime: String::new(),
                path: vec![],
            }
        } else {
            let mime = extract_mime(&file_name);
            file_metadata = FileMetadata {
                id: id,
                name: file_name,
                size: metadata.len(),
                kind: FileKind::File,
                mime: mime,
                path: vec![],
            }
        }
        result.push(file_metadata);
    }
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result.sort_by(|a, b| {
        if 0 == a.size && 0 < b.size {
            std::cmp::Ordering::Less
        } else if 0 == a.size && 0 == b.size {
            std::cmp::Ordering::Equal
        } else {
            std::cmp::Ordering::Greater
        }
    });
    return Ok(FileMetadatas { files: result });
}

pub fn search(root: &str, id: &str, query: &str) -> Result<FileMetadatas, String> {
    let files_result = get(root, id);
    let query_lower = query.to_lowercase();
    match files_result {
        Ok(files) => {
            let search_files = files
                .files
                .into_iter()
                .filter(|file| file.name.to_lowercase().contains(&query_lower))
                .collect();
            return Ok(FileMetadatas {
                files: search_files,
            });
        }
        Err(e) => Err(e),
    }
}

pub fn get_file(root: &str, id: &str) -> Result<FileMetadata, String> {
    println!("Getting file:{}, id:{}", root, id);
    let path: &std::path::Path;
    let path_str: String;
    if 0 < id.len() {
        path_str = match parse_id(&id) {
            Ok(result) => result,
            Err(error) => {
                return Err(error);
            }
        };
    } else {
        path_str = String::from(root);
    }
    let root_path = std::path::Path::new(root);
    path = std::path::Path::new(path_str.as_str());
    if !path.exists() {
        return Err(String::from("Path does not exists"));
    }
    let entry_file = match fs::metadata(path) {
        Ok(result) => result,
        Err(error) => {
            return Err(error.to_string());
        }
    };
    let file_name_os = match path.file_name() {
        None => {
            return Err(String::from("Error getting file name"));
        }
        Some(result) => result,
    };
    let file_name = match file_name_os.to_str() {
        None => {
            return Err(String::from("Error getting file name from OsStr"));
        }
        Some(result) => result,
    };
    let file_metadata: FileMetadata;
    let file_path_metadata = get_absolute_path(&root_path.to_path_buf(), &path.to_path_buf());
    let id = create_id(&path_str);
    if entry_file.is_dir() {
        file_metadata = FileMetadata {
            id: id,
            name: file_name.to_string(),
            size: entry_file.len(),
            kind: FileKind::Directory,
            mime: String::new(),
            path: file_path_metadata,
        }
    } else {
        let mime = extract_mime(&file_name);
        file_metadata = FileMetadata {
            id: id,
            name: file_name.to_string(),
            size: entry_file.len(),
            kind: FileKind::File,
            mime: mime,
            path: file_path_metadata,
        }
    }
    return Ok(file_metadata);
}
