use s4_vaultify::backend::file_manager::metadata_handing::{detect_type, md_treatment, read_bytes};
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("DEBUG: Début de la fonction main.");

    // Récupération du chemin du fichier depuis la ligne de commande ou utilisation d'un chemin par défaut.
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        //"/Users/lothaire/RustroverProjects/S4-Vaultify/test-files/3198359_14_articlemedium_jnpobyrne_20210731_-2_1_.jpg"
        //"/Users/lothaire/RustroverProjects/S4-Vaultify/test-files/file_example_TIFF_1MB.tiff"
        //"/Users/lothaire/Document/photos/IMG_1204.HEIC"
        //"/Users/lothaire/Desktop/vidéo-collée.png"
        //"/Users/lothaire/Desktop/Enregistrement de l’écran 2024-10-13 à 15.41.25.mov"
        "/Users/lothaire/RustroverProjects/S4-Vaultify/test-files/file_example_MP4_480_1_5MG.mp4"
    };
    println!("DEBUG: Chemin du fichier : {}", file_path);

    // Lecture des premiers octets du fichier.
    println!("DEBUG: Lecture des premiers octets du fichier.");
    let buffer = read_bytes(file_path)?;
    println!("DEBUG: {} octets lus.", buffer.len());

    // Détection du type de fichier à partir du buffer.
    println!("DEBUG: Détection du type de fichier.");
    let file_type = detect_type(&buffer);
    println!("DEBUG: Type de fichier détecté : {:?}", file_type);

    // Traitement des métadonnées (si le type correspond à une image supportée).
    println!("DEBUG: Traitement des métadonnées.");
    md_treatment(&buffer, file_type)?;
    println!("DEBUG: Fin du traitement des métadonnées.");

    println!("DEBUG: Fin du traitement du fichier.");
    Ok(())
}
