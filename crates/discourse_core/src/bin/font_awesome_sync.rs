use discourse_core::svg_sprite::font_awesome_sync::FontAwesomeSync;
use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: font_awesome_sync [--sync | --check]");
        process::exit(1);
    }

    let command = &args[1];

    // Determine rails root (usually current dir since it's executed from root)
    let rails_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let package_path = rails_root.join("node_modules/@fortawesome/fontawesome-free");
    let vendor_path = rails_root.join("vendor/assets/svg-icons");

    let sync = FontAwesomeSync::new(package_path, vendor_path);

    match command.as_str() {
        "--sync" => {
            let stale_files = match sync.stale_files() {
                Ok(files) => files,
                Err(e) => {
                    eprintln!("Error checking for stale files: {}", e);
                    process::exit(1);
                }
            };

            if let Err(e) = sync.write() {
                eprintln!("Error writing sprites: {}", e);
                process::exit(1);
            }

            if stale_files.is_empty() {
                println!("Sprites are already up to date");
            } else {
                for path in stale_files {
                    let relative = path.strip_prefix(&rails_root).unwrap_or(&path);
                    println!("Updated {}", relative.display());
                }
            }
        }
        "--check" => {
            let stale_files = match sync.stale_files() {
                Ok(files) => files,
                Err(e) => {
                    eprintln!("Error checking for stale files: {}", e);
                    process::exit(1);
                }
            };

            if stale_files.is_empty() {
                println!("Sprites are up to date");
            } else {
                for path in stale_files {
                    let relative = path.strip_prefix(&rails_root).unwrap_or(&path);
                    println!("Out of date: {}", relative.display());
                }
                eprintln!("Vendored icon sprites don't match node_modules, run `bin/rake svgicons:update`");
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            process::exit(1);
        }
    }
}
