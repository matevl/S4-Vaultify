use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug)]
pub enum FType {
    // Image
    Jpg,    // image/jpeg
    Png,    // image/png
    Gif,    // image/gif
    Webp,   // image/webp
    Cr2,    // image/x-canon-cr2
    Tif,    // image/tiff
    Bmp,    // image/bmp
    Heif,   // image/heif
    Avif,   // image/avif
    Jxr,    // image/vnd.ms-photo
    Psd,    // image/vnd.adobe.photoshop
    Ico,    // image/vnd.microsoft.icon
    Ora,    // image/openraster
    Djvu,   // image/vnd.djvu

    // Vidéo
    Mp4,    // video/mp4
    M4v,    // video/x-m4v
    Mkv,    // video/x-matroska
    Webm,   // video/webm
    Mov,    // video/quicktime
    Avi,    // video/x-msvideo
    Wmv,    // video/x-ms-wmv
    Mpg,    // video/mpeg
    Flv,    // video/x-flv

    // Audio
    Mid,    // audio/midi
    Mp3,    // audio/mpeg
    M4a,    // audio/m4a
    Ogg,    // audio/ogg
    Flac,   // audio/x-flac
    Wav,    // audio/x-wav
    Amr,    // audio/amr
    Aac,    // audio/aac
    Aiff,   // audio/x-aiff
    Dsf,    // audio/x-dsf
    Ape,    // audio/x-ape

    // Archive
    Epub,   // application/epub+zip
    Zip,    // application/zip
    Tar,    // application/x-tar
    Rar,    // application/vnd.rar
    Gz,     // application/gzip
    Bz2,    // application/x-bzip2
    Bz3,    // application/vnd.bzip3
    SevenZ, // application/x-7z-compressed
    Xz,     // application/x-xz
    Pdf,    // application/pdf
    Swf,    // application/x-shockwave-flash
    Rtf,    // application/rtf
    Eot,    // application/octet-stream
    Ps,     // application/postscript
    Sqlite, // application/vnd.sqlite3
    Nes,    // application/x-nintendo-nes-rom
    Crx,    // application/x-google-chrome-extension
    Cab,    // application/vnd.ms-cab-compressed
    Deb,    // application/vnd.debian.binary-package
    Ar,     // application/x-unix-archive
    Z,      // application/x-compress
    Lz,     // application/x-lzip
    Rpm,    // application/x-rpm
    Dcm,    // application/dicom
    Zst,    // application/zstd
    Lz4,    // application/x-lz4
    Msi,    // application/x-ole-storage
    Cpio,   // application/x-cpio
    Par2,   // application/x-par2

    // Book
    Mobi,   // application/x-mobipocket-ebook

    // Documents
    Doc,    // application/msword
    Docx,   // application/vnd.openxmlformats-officedocument.wordprocessingml.document
    Xls,    // application/vnd.ms-excel
    Xlsx,   // application/vnd.openxmlformats-officedocument.spreadsheetml.sheet
    Ppt,    // application/vnd.ms-powerpoint
    Pptx,   // application/vnd.openxmlformats-officedocument.presentationml.presentation
    Odt,    // application/vnd.oasis.opendocument.text
    Ods,    // application/vnd.oasis.opendocument.spreadsheet
    Odp,    // application/vnd.oasis.opendocument.presentation

    // Font
    Woff,   // application/font-woff
    Woff2,  // application/font-woff
    Ttf,    // application/font-sfnt
    Otf,    // application/font-sfnt

    // Application
    Wasm,   // application/wasm
    Exe,    // application/vnd.microsoft.portable-executable
    Dll,    // application/vnd.microsoft.portable-executable
    Elf,    // application/x-executable
    Bc,     // application/llvm
    Mach,   // application/x-mach-binary
    Class,  // application/java
    Dex,    // application/vnd.android.dex
    Dey,    // application/vnd.android.dey
    Der,    // application/x-x509-ca-cert
    Obj,    // application/x-executable
    Unknown,
}

fn read_initial_bytes<P: AsRef<Path>>(file_path: P, num_bytes: usize) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = vec![0; num_bytes];
    let useful_bytes = file.read(&mut buffer)?;
    buffer.truncate(useful_bytes);
    Ok(buffer)
}


pub fn detect_type<P: AsRef<Path>>(file_path: P) -> FType {

    if let Ok(buffer) = read_initial_bytes(file_path, 16) {
        if let Some(kind) = infer::get(&buffer) {
            match kind.mime_type() {
                // Image
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

                // Vidéo
                "video/mp4" => FType::Mp4,
                "video/x-m4v" => FType::M4v,
                "video/x-matroska" => FType::Mkv,
                "video/webm" => FType::Webm,
                "video/quicktime" => FType::Mov,
                "video/x-msvideo" => FType::Avi,
                "video/x-ms-wmv" => FType::Wmv,
                "video/mpeg" => FType::Mpg,
                "video/x-flv" => FType::Flv,

                // Audio
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

                // Archive
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

                // Book
                "application/x-mobipocket-ebook" => FType::Mobi,

                // Documents
                "application/msword" => FType::Doc,
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => FType::Docx,
                "application/vnd.ms-excel" => FType::Xls,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => FType::Xlsx,
                "application/vnd.ms-powerpoint" => FType::Ppt,
                "application/vnd.openxmlformats-officedocument.presentationml.presentation" => FType::Pptx,
                "application/vnd.oasis.opendocument.text" => FType::Odt,
                "application/vnd.oasis.opendocument.spreadsheet" => FType::Ods,
                "application/vnd.oasis.opendocument.presentation" => FType::Odp,

                // Font
                "application/font-woff" => FType::Woff,
                "application/font-woff2" => FType::Woff2,
                "application/font-sfnt" => FType::Ttf,

                // Application
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
    } else {
        FType::Unknown
    }


}

