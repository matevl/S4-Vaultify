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
struct FileNode {
    file_name: String,
    binary_file_name: String,
    file_type: String,
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
enum FileType {
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
    files: HashMap<String, FileType>,
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
}

/// Public Node in the File Tree
///
/// @field file_name - the virtual name of the file
///
/// @file_type - .jpeg, .png, .pdf ...
#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
enum PubFileType {
    File(PubFileNode),
    Dir(PubDirectory),
}

/// A Public Virtual Directory
///
/// @field name - name of the directory
///
/// @field files - hashmap file or directory name -> Filetype
#[derive(Serialize, Deserialize, Debug)]
struct PubDirectory {
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
