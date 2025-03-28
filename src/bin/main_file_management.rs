use std::env;
use std::error::Error;

// Import metadata-handling functions.
use s4_vaultify::backend::file_manager::metadata_handling::{process_file, refusion_from_map};

// Import binary file utilities from our module (assume binary_utils.rs is in your project).
use s4_vaultify::backend::file_manager::file_handling::clear_binary;

fn main() -> Result<(), Box<dyn Error>> {
    println!("DEBUG: Starting main function.");

    // Optionally clear the binary_files directory before running the test.
    println!("DEBUG: Clearing binary files directory.");
    clear_binary();

    // Retrieve the file path from the command line or use a default HEIF image path.
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "/Users/lothaire/Document/photos/IMG_1204.HEIC"
    };
    println!("DEBUG: File path: {}", file_path);

    // Delegate processing to process_file
    process_file(file_path.as_ref())?;
    refusion_from_map("mcq-2027-s4-02copy.pdf").expect("failed to process file");

    // // Re-fusion logic: duplicate HEIC/HEIF file to output.heif
    // let buffer = read_bytes(file_path)?;
    // let file_type = detect_type(&buffer);
    // if format!("{:?}", file_type).to_lowercase().contains("heic")
    //     || format!("{:?}", file_type).to_lowercase().contains("heif")
    // {
    //     let output_path = env::current_dir()?.join("output.heif");
    //     println!(
    //         "DEBUG: Detected HEIF image. Writing output to {:?}",
    //         output_path
    //     );
    //     fs::write(&output_path, &buffer)?;
    // } else {
    //     println!("DEBUG: File is not a HEIF image; skipping HEIF output generation.");
    // }
    //
    // println!("DEBUG: End of main function.");
    Ok(())
}
