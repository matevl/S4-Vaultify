use crate::backend::file_manager::file_handling::get_name;
pub use crate::backend::file_manager::file_handling::{read_bytes, save_binary};
use crate::backend::file_manager::mapping::update_map;
use exif::Reader;
use ffmpeg_next;
use lopdf::Document;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Write};
use std::path::Path;
use std::{env, fs};
use tempfile::NamedTempFile;
use zip::ZipArchive;

#[derive(Serialize, Deserialize)]
struct FileMap {
    original_filename: String,
    binary: String,
    metadata: String,
    file_type: String,
}

#[derive(Debug, Clone, Copy)]
pub enum FType {
    Jpg,
    Png,
    Gif,
    Webp,
    Cr2,
    Tif,
    Bmp,
    Heif,
    Avif,
    Jxr,
    Psd,
    Ico,
    Ora,
    Djvu,
    Mp4,
    M4v,
    Mkv,
    Webm,
    Mov,
    Avi,
    Wmv,
    Mpg,
    Flv,
    Mid,
    Mp3,
    M4a,
    Ogg,
    Flac,
    Wav,
    Amr,
    Aac,
    Aiff,
    Dsf,
    Ape,
    Epub,
    Zip,
    Tar,
    Rar,
    Gz,
    Bz2,
    Bz3,
    SevenZ,
    Xz,
    Pdf,
    Swf,
    Rtf,
    Eot,
    Ps,
    Sqlite,
    Nes,
    Crx,
    Cab,
    Deb,
    Ar,
    Z,
    Lz,
    Rpm,
    Dcm,
    Zst,
    Lz4,
    Msi,
    Cpio,
    Par2,
    Mobi,
    Doc,
    Docx,
    Xls,
    Xlsx,
    Ppt,
    Pptx,
    Odt,
    Ods,
    Odp,
    Woff,
    Woff2,
    Ttf,
    Otf,
    Wasm,
    Exe,
    Dll,
    Elf,
    Bc,
    Mach,
    Class,
    Dex,
    Dey,
    Der,
    Obj,
    Unknown,
}

/**
 * Processes a file by reading its bytes, detecting its type,
 * extracting metadata, saving the binary, and updating the map.
 */
pub fn process_file<P: AsRef<Path>>(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let original_name = get_name(file_path);
    println!("DEBUG: File path: {:?}", file_path);
    let buffer = read_bytes(file_path)?;
    println!("DEBUG: {} bytes read.", buffer.len());
    let file_type = detect_type(&buffer);
    println!("DEBUG: Detected file type: {:?}", file_type);
    println!("DEBUG: Processing metadata.");
    md_treatment(&buffer, file_type, original_name)?;
    save_binary(&buffer);
    println!("DEBUG: Binary file saved in binary_files directory.");
    Ok(())
}

/**
 * Detects the file type based on its byte content using MIME type inference.
 */
pub fn detect_type(buffer: &Vec<u8>) -> FType {
    if let Some(kind) = infer::get(buffer) {
        match kind.mime_type() {
            "image/jpeg" => FType::Jpg,
            "image/png" => FType::Png,
            "image/gif" => FType::Gif,
            "image/webp" => FType::Webp,
            "image/x-canon-cr2" => FType::Cr2,
            "image/tiff" => FType::Tif,
            "image/bmp" => FType::Bmp,
            "image/heif" => FType::Heif,
            "image/avif" => FType::Avif,
            "image/vnd.ms-photo" => FType::Jxr,
            "image/vnd.adobe.photoshop" => FType::Psd,
            "image/vnd.microsoft.icon" => FType::Ico,
            "image/openraster" => FType::Ora,
            "image/vnd.djvu" => FType::Djvu,
            "video/mp4" => FType::Mp4,
            "video/x-m4v" => FType::M4v,
            "video/x-matroska" => FType::Mkv,
            "video/webm" => FType::Webm,
            "video/quicktime" => FType::Mov,
            "video/x-msvideo" => FType::Avi,
            "video/x-ms-wmv" => FType::Wmv,
            "video/mpeg" => FType::Mpg,
            "video/x-flv" => FType::Flv,
            "audio/midi" => FType::Mid,
            "audio/mpeg" => FType::Mp3,
            "audio/m4a" => FType::M4a,
            "audio/ogg" => FType::Ogg,
            "audio/x-flac" => FType::Flac,
            "audio/x-wav" => FType::Wav,
            "audio/amr" => FType::Amr,
            "audio/aac" => FType::Aac,
            "audio/x-aiff" => FType::Aiff,
            "audio/x-dsf" => FType::Dsf,
            "audio/x-ape" => FType::Ape,
            "application/epub+zip" => FType::Epub,
            "application/zip" => FType::Zip,
            "application/x-tar" => FType::Tar,
            "application/vnd.rar" => FType::Rar,
            "application/gzip" => FType::Gz,
            "application/x-bzip2" => FType::Bz2,
            "application/vnd.bzip3" => FType::Bz3,
            "application/x-7z-compressed" => FType::SevenZ,
            "application/x-xz" => FType::Xz,
            "application/pdf" => FType::Pdf,
            "application/x-shockwave-flash" => FType::Swf,
            "application/rtf" => FType::Rtf,
            "application/octet-stream" => FType::Eot,
            "application/postscript" => FType::Ps,
            "application/vnd.sqlite3" => FType::Sqlite,
            "application/x-nintendo-nes-rom" => FType::Nes,
            "application/x-google-chrome-extension" => FType::Crx,
            "application/vnd.ms-cab-compressed" => FType::Cab,
            "application/vnd.debian.binary-package" => FType::Deb,
            "application/x-unix-archive" => FType::Ar,
            "application/x-compress" => FType::Z,
            "application/x-lzip" => FType::Lz,
            "application/x-rpm" => FType::Rpm,
            "application/dicom" => FType::Dcm,
            "application/zstd" => FType::Zst,
            "application/x-lz4" => FType::Lz4,
            "application/x-ole-storage" => FType::Msi,
            "application/x-cpio" => FType::Cpio,
            "application/x-par2" => FType::Par2,
            "application/x-mobipocket-ebook" => FType::Mobi,
            "application/msword" => FType::Doc,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                FType::Docx
            }
            "application/vnd.ms-excel" => FType::Xls,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => FType::Xlsx,
            "application/vnd.ms-powerpoint" => FType::Ppt,
            "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
                FType::Pptx
            }
            "application/vnd.oasis.opendocument.text" => FType::Odt,
            "application/vnd.oasis.opendocument.spreadsheet" => FType::Ods,
            "application/vnd.oasis.opendocument.presentation" => FType::Odp,
            "application/font-woff" => FType::Woff,
            "application/font-woff2" => FType::Woff2,
            "application/font-sfnt" => FType::Ttf,
            "application/wasm" => FType::Wasm,
            "application/vnd.microsoft.portable-executable" => FType::Exe,
            "application/x-executable" => FType::Elf,
            "application/llvm" => FType::Bc,
            "application/x-mach-binary" => FType::Mach,
            "application/java" => FType::Class,
            "application/vnd.android.dex" => FType::Dex,
            "application/vnd.android.dey" => FType::Dey,
            "application/x-x509-ca-cert" => FType::Der,
            _ => FType::Elf,
        }
    } else {
        FType::Unknown
    }
}

/**
 * Extracts and separates metadata from various file types and updates the map.
 */
pub fn md_treatment(
    buffer: &Vec<u8>,
    ext: FType,
    original_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    match ext {
        FType::Tif | FType::Jpg | FType::Heif | FType::Png => {
            let exifreader = Reader::new();
            let mut cursor = Cursor::new(&buffer);
            let exif = exifreader.read_from_container(&mut cursor)?;
            for f in exif.fields() {
                println!(
                    "{} {} {}",
                    f.tag,
                    f.ifd_num,
                    f.display_value().with_unit(&exif)
                );
            }
            if let FType::Jpg = ext {
                let (content_buffer, metadata_buffer) = split_jpeg(&buffer);
                let content = save_binary(&content_buffer);
                if !metadata_buffer.is_empty() {
                    let metadata = save_binary(&metadata_buffer);
                    update_map(original_name, content, metadata, "jpg".to_string());
                }
            } else if let FType::Png = ext {
                let (content_buffer, metadata_buffer) = split_png(&buffer);
                let content = save_binary(&content_buffer);
                if !metadata_buffer.is_empty() {
                    let metadata = save_binary(&metadata_buffer);
                    update_map(original_name.clone(), content, metadata, "png".to_string());
                }
            } else if let FType::Tif = ext {
                let (content_buffer, metadata_buffer) = split_tiff(&buffer);
                let content = save_binary(&content_buffer);
                if !metadata_buffer.is_empty() {
                    let metadata = save_binary(&metadata_buffer);
                    update_map(original_name.clone(), content, metadata, "tif".to_string());
                }
            } else if let FType::Heif = ext {
                let (content_buffer, metadata_buffer) = split_tiff(&buffer);
                let content = save_binary(&content_buffer);
                if !metadata_buffer.is_empty() {
                    let metadata = save_binary(&metadata_buffer);
                    update_map(original_name.clone(), content, metadata, "heif".to_string());
                }
            }
        }
        FType::Mp4
        | FType::M4v
        | FType::Mkv
        | FType::Webm
        | FType::Mov
        | FType::Avi
        | FType::Wmv
        | FType::Mpg
        | FType::Flv => {
            ffmpeg_next::init()?;
            let mut temp_file = NamedTempFile::new()?;
            temp_file.write_all(&buffer)?;
            let temp_path = temp_file
                .path()
                .to_str()
                .ok_or("Invalid temporary file path")?;
            let mut ictx = ffmpeg_next::format::input(&temp_path)?;
            println!("Métadonnées du format:");
            for (key, value) in ictx.metadata().iter() {
                println!("  {}: {}", key, value);
            }
            for (index, stream) in ictx.streams().enumerate() {
                println!("\nStream {}:", index);
                println!("  Métadonnées:");
                for (key, value) in stream.metadata().iter() {
                    println!("    {}: {}", key, value);
                }
            }
            let (content_buffer, metadata_buffer) = split_video(&buffer);
            let content = save_binary(&content_buffer);
            if !metadata_buffer.is_empty() {
                let metadata = save_binary(&metadata_buffer);
                update_map(original_name.clone(), content, metadata, "mp4".to_string());
                //add other fmt to map
            }
        }
        FType::Zip => {
            let reader = Cursor::new(buffer);
            let mut archive = ZipArchive::new(reader)?;
            for i in 0..archive.len() {
                let file = archive.by_index(i)?;
                println!(
                    "ZIP Entry {}: {} (size: {} bytes, compressed: {} bytes)",
                    i,
                    file.name(),
                    file.size(),
                    file.compressed_size()
                );
            }
            let (content_buffer, metadata_buffer) = split_zip(&buffer);
            let content = save_binary(&content_buffer);
            if !metadata_buffer.is_empty() {
                let metadata = save_binary(&metadata_buffer);
                update_map(original_name.clone(), content, metadata, "zip".to_string());
            }
        }
        FType::Pdf => {
            let doc = Document::load_mem(&buffer)?;
            let info_obj = doc.trailer.get(b"Info").ok().unwrap();
            let info_dict = doc.get_dictionary(info_obj.as_reference().unwrap())?;
            for (key, value) in info_dict.iter() {
                println!("  {}: {:?}", String::from_utf8_lossy(key), value);
            }
            let (content_buffer, metadata_buffer) = split_pdf(&buffer);
            let content = save_binary(&content_buffer);
            let metadata = if metadata_buffer.is_empty() {
                save_binary(&Vec::new())
            } else {
                save_binary(&metadata_buffer)
            };
            update_map(original_name.clone(), content, metadata, "pdf".to_string());
        }
        _ => {}
    }
    Ok(())
}

/**
 * Splits TIFF file buffer into content and metadata.
 */
fn split_tiff(buffer: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let meta_len = 512.min(buffer.len());
    let metadata = buffer[..meta_len].to_vec();
    let content = buffer[meta_len..].to_vec();
    (content, metadata)
}

/**
 * Splits JPEG file buffer into content and metadata.
 */
fn split_jpeg(buffer: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut content = Vec::new();
    let mut metadata = Vec::new();
    let mut i = 0;
    if buffer.len() >= 2 && buffer[0] == 0xFF && buffer[1] == 0xD8 {
        content.extend_from_slice(&buffer[0..2]);
        i = 2;
    } else {
        return (buffer.to_vec(), Vec::new());
    }
    while i < buffer.len() {
        if buffer[i] == 0xFF {
            if i + 1 >= buffer.len() {
                break;
            }
            let marker = buffer[i + 1];
            if marker == 0xDA {
                content.extend_from_slice(&buffer[i..]);
                break;
            }
            if marker >= 0xE0 && marker <= 0xEF {
                if i + 4 > buffer.len() {
                    break;
                }
                let seg_len = ((buffer[i + 2] as usize) << 8) | (buffer[i + 3] as usize);
                if i + 2 + seg_len <= buffer.len() {
                    let segment = &buffer[i..i + 2 + seg_len];
                    metadata.extend_from_slice(segment);
                    i += 2 + seg_len;
                    continue;
                } else {
                    break;
                }
            }
        }
        content.push(buffer[i]);
        i += 1;
    }
    (content, metadata)
}

/**
 * Splits PNG file buffer into content and metadata.
 */
fn split_png(buffer: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut content = Vec::new();
    let mut metadata = Vec::new();
    if buffer.len() < 8 || &buffer[0..8] != b"\x89PNG\r\n\x1a\n" {
        return (buffer.to_vec(), Vec::new());
    }
    content.extend_from_slice(&buffer[0..8]);
    let mut i = 8;
    while i + 8 <= buffer.len() {
        let length = u32::from_be_bytes(buffer[i..i + 4].try_into().unwrap()) as usize;
        if i + 8 + length > buffer.len() {
            break;
        }
        let chunk_type = &buffer[i + 4..i + 8];
        let chunk_total_len = 4 + 4 + length + 4;
        let chunk = &buffer[i..i + chunk_total_len];
        if chunk_type == b"tEXt"
            || chunk_type == b"iTXt"
            || chunk_type == b"zTXt"
            || chunk_type == b"eXIf"
        {
            metadata.extend_from_slice(chunk);
        } else {
            content.extend_from_slice(chunk);
        }
        i += chunk_total_len;
    }
    (content, metadata)
}

/**
 * Splits video file buffer into content and metadata.
 */
fn split_video(buffer: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut content = Vec::new();
    let mut metadata = Vec::new();
    let mut i = 0;
    while i + 8 <= buffer.len() {
        let box_size = u32::from_be_bytes(buffer[i..i + 4].try_into().unwrap()) as usize;
        if box_size < 8 || i + box_size > buffer.len() {
            break;
        }
        let box_type = &buffer[i + 4..i + 8];
        if box_type == b"moov" {
            metadata.extend_from_slice(&buffer[i..i + box_size]);
        } else {
            content.extend_from_slice(&buffer[i..i + box_size]);
        }
        i += box_size;
    }
    if i < buffer.len() {
        content.extend_from_slice(&buffer[i..]);
    }
    (content, metadata)
}

/**
 * Splits ZIP file buffer into content and metadata.
 */
fn split_zip(buffer: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let signature = b"\x50\x4B\x05\x06";
    if let Some(pos) = buffer.windows(4).rposition(|w| w == signature) {
        let content = buffer[..pos].to_vec();
        let metadata = buffer[pos..].to_vec();
        (content, metadata)
    } else {
        (buffer.to_vec(), Vec::new())
    }
}

/**
 * Splits PDF file buffer into content and metadata.
 */
fn split_pdf(buffer: &[u8]) -> (Vec<u8>, Vec<u8>) {
    if let Some(pos) = buffer.windows(7).position(|w| w == b"trailer") {
        let content = buffer[..pos].to_vec();
        let metadata = buffer[pos..].to_vec();
        (content, metadata)
    } else {
        (buffer.to_vec(), Vec::new())
    }
}

/**
 * Recombines TIFF content and metadata into a full file buffer.
 */
fn recombine_tiff(content: &[u8], metadata: &[u8]) -> Vec<u8> {
    let mut combined = Vec::new();
    combined.extend_from_slice(metadata);
    combined.extend_from_slice(content);
    combined
}

/**
 * Recombines JPEG content and metadata into a full file buffer.
 */
fn recombine_jpeg(content: &[u8], metadata: &[u8]) -> Vec<u8> {
    let mut combined = Vec::new();
    if content.len() >= 2 {
        combined.extend_from_slice(&content[..2]);
        combined.extend_from_slice(metadata);
        combined.extend_from_slice(&content[2..]);
    } else {
        combined.extend_from_slice(content);
        combined.extend_from_slice(metadata);
    }
    combined
}

/**
 * Recombines PNG content and metadata into a full file buffer.
 */
fn recombine_png(content: &[u8], metadata: &[u8]) -> Vec<u8> {
    let mut combined = Vec::new();
    if content.len() >= 8 {
        combined.extend_from_slice(&content[..8]);
        combined.extend_from_slice(metadata);
        combined.extend_from_slice(&content[8..]);
    } else {
        combined.extend_from_slice(content);
        combined.extend_from_slice(metadata);
    }
    combined
}

/**
 * Recombines video content and metadata into a full file buffer.
 */
fn recombine_video(content: &[u8], metadata: &[u8]) -> Vec<u8> {
    //MP4 proof, other format not tested
    let mut combined = Vec::new();
    combined.extend_from_slice(metadata);
    combined.extend_from_slice(content);
    combined
}

/**
 * Recombines ZIP content and metadata into a full file buffer.
 */
fn recombine_zip(content: &[u8], metadata: &[u8]) -> Vec<u8> {
    let mut combined = Vec::new();
    combined.extend_from_slice(content);
    combined.extend_from_slice(metadata);
    combined
}

/**
 * Recombines PDF content and metadata into a full file buffer.
 */
fn recombine_pdf(content: &[u8], metadata: &[u8]) -> Vec<u8> {
    let mut combined = Vec::new();
    combined.extend_from_slice(content);
    combined.extend_from_slice(metadata);
    combined
}

/**
 * Detects the file type enum from a file extension string.
 */
fn detect_type_from_ext(ext: &str) -> FType {
    match ext.to_lowercase().as_str() {
        "jpg" | "jpeg" => FType::Jpg,
        "png" => FType::Png,
        "tif" | "tiff" => FType::Tif,
        "heif" => FType::Heif,
        "mp4" => FType::Mp4,
        "m4v" => FType::M4v,
        "mkv" => FType::Mkv,
        "webm" => FType::Webm,
        "mov" => FType::Mov,
        "avi" => FType::Avi,
        "wmv" => FType::Wmv,
        "mpg" => FType::Mpg,
        "flv" => FType::Flv,
        "zip" => FType::Zip,
        "pdf" => FType::Pdf,
        _ => FType::Unknown,
    }
}

/**
 * Reconstructs the original file from its binary and metadata parts using the mapping file.
 * Writes the recombined file back to the filesystem.
 */
pub fn refusion_from_map(filename: &str) -> std::io::Result<()> {
    let map_path = env::current_dir()?.join("binary_files").join("map.json");
    let content = fs::read_to_string(&map_path)?;
    let entries: Vec<FileMap> = serde_json::from_str(&content)?;

    let entry = entries
        .into_iter()
        .find(|e| e.original_filename == filename);

    if let Some(entry) = entry {
        let binary_path = env::current_dir()?.join("binary_files").join(&entry.binary);
        let metadata_path = env::current_dir()?
            .join("binary_files")
            .join(&entry.metadata);

        let binary_buffer = read_bytes(&binary_path)?;
        let metadata_buffer = read_bytes(&metadata_path)?;

        let ext = entry.file_type.as_str();
        let combined = match detect_type_from_ext(ext) {
            FType::Jpg => recombine_jpeg(&binary_buffer, &metadata_buffer),
            FType::Png => recombine_png(&binary_buffer, &metadata_buffer),
            FType::Tif | FType::Heif => recombine_tiff(&binary_buffer, &metadata_buffer),
            FType::Mp4
            | FType::M4v
            | FType::Mkv
            | FType::Webm
            | FType::Mov
            | FType::Avi
            | FType::Wmv
            | FType::Mpg
            | FType::Flv => recombine_video(&binary_buffer, &metadata_buffer),
            FType::Zip => recombine_zip(&binary_buffer, &metadata_buffer),
            FType::Pdf => recombine_pdf(&binary_buffer, &metadata_buffer),
            _ => {
                let mut combined = Vec::new();
                combined.extend_from_slice(&binary_buffer);
                combined.extend_from_slice(&metadata_buffer);
                combined
            }
        };

        let output_path = env::current_dir()?.join(filename);
        fs::write(output_path, combined)?;
        println!("Recombined file written to {:?}", filename);
    } else {
        println!("No entry found in map for filename: {}", filename);
    }

    Ok(())
}
