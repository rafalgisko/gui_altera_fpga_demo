use std::fs;
use std::path::Path;

fn main() {
    // Ścieżka do folderu 'sofs' w katalogu źródłowym
    let source_path = Path::new("sofs");

    // Ścieżka docelowa (target/release/sofs) – folder w którym generowane są pliki wykonywalne
    let target_path = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("target")
        .join("release")
        .join("sofs");

    // Sprawdzenie, czy folder 'sofs' istnieje w katalogu źródłowym
    if source_path.exists() {
        if let Err(e) = copy_dir_all(&source_path, &target_path) {
            panic!("Failed to copy 'sofs' directory: {}", e);
        }
    } else {
        panic!("Directory 'sofs' does not exist in the source folder.");
    }

    let images_source_path = Path::new("images");
    let images_target_path = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("target")
        .join("release")
        .join("images");
    if images_source_path.exists() {
        if let Err(e) = copy_dir_all(&images_source_path, &images_target_path) {
            panic!("Failed to copy 'images' directory: {}", e);
        }
    } else {
        panic!("Directory 'images' does not exist in the source folder.");
    }

    // Informacja dla Cargo, że musimy wykonać krok build.rs, gdy zmieni się folder 'sofs'
    println!("cargo:rerun-if-changed=sofs");
}

// Funkcja do kopiowania całego folderu
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if entry_path.is_dir() {
            copy_dir_all(&entry_path, &dst_path)?;
        } else {
            fs::copy(&entry_path, &dst_path)?;
        }
    }
    Ok(())
}
