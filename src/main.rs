use s4_vaultify::backend::file_manager::file_handling::{open_file_binary, save_binary};
use std::fs;
use std::path::Path;

fn main() {
    // 🔹 Fichier à sauvegarder en binaire
    let file_to_save = "test_file.png"; // Remplace par le fichier que tu veux sauvegarder
    let binary_dir = Path::new("binary_files");

    // 🔹 Vérifier si le fichier existe avant de continuer
    if !Path::new(file_to_save).exists() {
        println!("❌ Le fichier '{}' n'existe pas.", file_to_save);
        return;
    }

    // 🔹 Lire le contenu du fichier original
    let contents = fs::read(file_to_save).expect("Erreur lors de la lecture du fichier");

    // 🔹 Sauvegarde du fichier en binaire
    save_binary(&contents);
    println!("✅ Fichier '{}' sauvegardé en binaire.", file_to_save);

    // 🔹 Vérification en lisant un fichier sauvegardé
    let saved_files =
        fs::read_dir(binary_dir).expect("Erreur : impossible de lire le dossier binaire");

    // Prendre un des fichiers créés pour tester l'ouverture
    if let Some(file) = saved_files.flatten().next() {
        let saved_path = file.path();
        println!("📂 Lecture du fichier sauvegardé : {:?}", saved_path);

        let loaded_data = open_file_binary(&saved_path);
        if loaded_data == contents {
            println!("✅ Test réussi : les données sont intactes !");
        } else {
            println!("❌ Erreur : les données ne correspondent pas !");
        }
    } else {
        println!("❌ Aucun fichier binaire trouvé !");
    }
}
