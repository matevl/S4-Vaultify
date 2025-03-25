use s4_vaultify::backend::file_manager::file_handling::{
    clear_binary, open_file_binary, save_binary,
};
use s4_vaultify::backend::file_manager::metadata_handing::{
    detect_type, md_treatment_buffer, read_bytes,
};
use std::env;
use std::error::Error;
use std::path::Path;
//fn main() -> Result<(), Box<dyn Error>> {
//    println!("DEBUG: Début de la fonction main.");
//
//    // Récupération du chemin du fichier depuis la ligne de commande ou utilisation d'un chemin par défaut.
//    let args: Vec<String> = env::args().collect();
//    let file_path = if args.len() > 1 {
//        &args[1]
//    } else {
//        //"/Users/lothaire/RustroverProjects/S4-Vaultify/test-files/3198359_14_articlemedium_jnpobyrne_20210731_-2_1_.jpg"
//        //"/Users/lothaire/RustroverProjects/S4-Vaultify/test-files/file_example_TIFF_1MB.tiff"
//        //"/Users/lothaire/Document/photos/IMG_1204.HEIC"
//        //"/Users/lothaire/Desktop/vidéo-collée.png"
//        //"/Users/lothaire/Desktop/Enregistrement de l’écran 2024-10-13 à 15.41.25.mov"
//        //"/Users/lothaire/RustroverProjects/S4-Vaultify/test-files/file_example_MP4_480_1_5MG.mp4"
//        "/Users/lothaire/RustroverProjects/S4-Vaultify/test-files/sunflower-street-drumloop-85bpm-163900.ogg"
//    };
//    println!("DEBUG: Chemin du fichier : {}", file_path);
//
//    // Lecture des premiers octets du fichier.
//    println!("DEBUG: Lecture des premiers octets du fichier.");
//    let buffer = read_bytes(file_path)?;
//    println!("DEBUG: {} octets lus.", buffer.len());
//
//    // Détection du type de fichier à partir du buffer.
//    println!("DEBUG: Détection du type de fichier.");
//    let file_type = detect_type(&buffer);
//    println!("DEBUG: Type de fichier détecté : {:?}", file_type);
//
//    // Traitement des métadonnées (si le type correspond à une image supportée).
//    println!("DEBUG: Traitement des métadonnées.");
//    md_treatment_buffer(&buffer,file_type)?;
//    println!("DEBUG: Fin du traitement des métadonnées.");
//
//    println!("DEBUG: Fin du traitement du fichier.");
//    Ok(())
//}

fn main() {
    // Définir le chemin du fichier directement dans le code
    let file_path =
        Path::new("/Users/lothaire/Desktop/Enregistrement de l’écran 2024-10-13 à 15.41.25.mov");

    if !file_path.exists() {
        eprintln!("Erreur : Le fichier spécifié n'existe pas.");
        std::process::exit(1);
    }

    // Lire le fichier en binaire
    let binary_data = open_file_binary(file_path);
    println!(
        "Le fichier a été lu avec succès ({} octets).",
        binary_data.len()
    );

    // Sauvegarder sous un nouveau nom
    save_binary(&binary_data);
    println!("Le fichier a été sauvegardé avec un nouveau nom.");
    clear_binary();
}
