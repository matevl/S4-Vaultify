use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const FILE_TREE_FILE_NAME: &str = "file_tree.json";

/// Node in the File Tree
///
/// @field file_name - the virtual name of the file
///
/// @field binary_file_name - the name of the file on the disk
///
/// @file_type - .jpeg, .png, .pdf ...
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileNode {
    pub file_name: String,
    pub binary_file_name: String,
    pub file_type: String,
}

impl FileNode {
    pub fn new(file_name: String, binary_file_name: String, file_type: String) -> Self {
        Self {
            file_name,
            binary_file_name,
            file_type,
        }
    }

    /// transform to public FileNode
    pub fn to_public(&self) -> PubFileNode {
        PubFileNode::new(self.file_name.clone(), self.file_type.clone())
    }
}

/// A FileType can be either a File or a Directory
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FileType {
    File(FileNode),
    Dir(Directory),
}

/// A Virtual Directory
///
/// @field name - name of the directory
///
/// @field files - hashmap file or directory name -> Filetype
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Directory {
    name: String,
    pub(crate) files: HashMap<String, FileType>,
}

impl Directory {
    pub fn new(name: String) -> Self {
        Self {
            name,
            files: HashMap::new(),
        }
    }

    /// to_public
    pub fn to_public(&self) -> PubDirectory {
        let mut res = PubDirectory::new(self.name.clone());
        for (key, value) in self.files.iter() {
            match value {
                FileType::File(file) => {
                    res.files
                        .insert(key.clone(), PubFileType::File(file.to_public()));
                }
                FileType::Dir(dir) => {
                    res.files
                        .insert(key.clone(), PubFileType::Dir(dir.to_public()));
                }
            }
        }
        res
    }

    pub fn add_dir(&mut self, name: &str) {
        let final_name = generate_unique_name(&self.files, name);
        self.files.insert(
            final_name.clone(),
            FileType::Dir(Directory::new(final_name)),
        );
    }

    pub fn add_file(&mut self, file_name: &str, binary_name: String, file_type: String) {
        let final_name = generate_unique_name(&self.files, file_name);
        let file_node = FileNode::new(final_name.clone(), binary_name, file_type);
        self.files.insert(final_name, FileType::File(file_node));
    }

    pub fn rename(&mut self, old_name: &str, new_name: &str) -> Result<(), String> {
        if let Some(node) = self.files.remove(old_name) {
            let new_name = generate_unique_name(&self.files, new_name);
            match &node {
                FileType::File(f) => {
                    let mut new_file = f.clone();
                    new_file.file_name = new_name.clone();
                    self.files.insert(new_name, FileType::File(new_file));
                }
                FileType::Dir(d) => {
                    let mut new_dir = d.clone();
                    new_dir.name = new_name.clone();
                    self.files.insert(new_name, FileType::Dir(new_dir));
                }
            }
            Ok(())
        } else {
            Err(format!("File or directory '{}' not found", old_name))
        }
    }

    pub fn get_mut_directory_from_path(&mut self, path: &str) -> Result<&mut Directory, String> {
        let mut current_dir = self;

        // Skip empty path (return root)
        if path.is_empty() {
            return Ok(current_dir);
        }

        for part in path.split('/') {
            match current_dir.files.get_mut(part) {
                Some(FileType::Dir(sub_dir)) => {
                    current_dir = sub_dir;
                }
                Some(_) => {
                    return Err(format!(
                        "Path component '{}' is a file, not a directory",
                        part
                    ));
                }
                None => {
                    return Err(format!("Directory '{}' not found in path", part));
                }
            }
        }

        Ok(current_dir)
    }
}

/// Public Node in the File Tree
///
/// @field file_name - the virtual name of the file
///
/// @file_type - .jpeg, .png, .pdf ...
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PubFileNode {
    file_name: String,
    file_type: String,
}

impl PubFileNode {
    pub fn new(file_name: String, file_type: String) -> Self {
        Self {
            file_name,
            file_type,
        }
    }
}

/// A Public FileType can be either a File or a Directory
#[derive(Serialize, Deserialize, Debug, Clone)]
enum PubFileType {
    File(PubFileNode),
    Dir(PubDirectory),
}

/// A Public Virtual Directory
///
/// @field name - name of the directory
///
/// @field files - hashmap file or directory name -> Filetype
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PubDirectory {
    name: String,
    files: HashMap<String, PubFileType>,
}

impl PubDirectory {
    pub fn new(name: String) -> Self {
        Self {
            name,
            files: HashMap::new(),
        }
    }
}

fn generate_unique_name(existing: &HashMap<String, FileType>, base: &str) -> String {
    if !existing.contains_key(base) {
        return base.to_string();
    }

    let mut count = 1;
    loop {
        let candidate = format!("{} ({})", base, count);
        if !existing.contains_key(&candidate) {
            return candidate;
        }
        count += 1;
    }
}

use std::fs;
use std::path::Path;

/// Recursively removes a directory from memory and deletes all files from disk
pub fn remove_directory_recursively(
    directory: &mut Directory,
    name: &str,
    vault_path: &str,
) -> Result<(), String> {
    match directory.files.remove(name) {
        Some(FileType::Dir(mut dir)) => {
            // For each item inside the directory being deleted
            let keys: Vec<String> = dir.files.keys().cloned().collect();

            for child_name in keys {
                match dir.files.remove(&child_name) {
                    Some(FileType::File(file_node)) => {
                        let disk_path = format!("{}/{}", vault_path, file_node.binary_file_name);
                        if let Err(err) = std::fs::remove_file(&disk_path) {
                            return Err(format!("Failed to delete file '{}': {}", disk_path, err));
                        }
                    }
                    Some(FileType::Dir(_)) => {
                        // Recursive call on the subdirectory
                        remove_directory_recursively(&mut dir, &child_name, vault_path)?;
                    }
                    None => continue,
                }
            }
            Ok(())
        }
        Some(_) => Err("Target is not a directory".to_string()),
        None => Err("Directory not found".to_string()),
    }
}

/// Removes a file from memory and deletes it from disk
pub fn remove_file_from_directory(
    directory: &mut Directory,
    name: &str,
    vault_path: &str,
) -> Result<(), String> {
    match directory.files.remove(name) {
        Some(FileType::File(file_node)) => {
            let disk_path = format!("{}/{}", vault_path, file_node.binary_file_name);
            fs::remove_file(&disk_path)
                .map_err(|e| format!("Failed to delete file '{}': {}", disk_path, e))?;
            Ok(())
        }
        Some(_) => Err("Target is not a file".to_string()),
        None => Err("File not found".to_string()),
    }
}
