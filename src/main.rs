use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::fs::{self};
use std::path::Path;
mod utils;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lista todos los archivos JSON en los idiomas
    Listar,
    /// Crea un nuevo archivo JSON para un idioma
    Crear {
        #[clap(value_parser)]
        idioma: String,
    },
    CreateLang {
        #[clap(value_parser)]
        lang: String,
    },
    /// Genera archivos JSON a partir de una plantilla
    Generar,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Listar => listar_archivos(),
        Commands::Crear { idioma } => crear_archivo(idioma),
        Commands::CreateLang { lang } => create_lang(lang),
        Commands::Generar => generar_archivos(),
    }
}

fn listar_archivos() {
    let ruta_locales = Path::new("locales");

    // Verificamos que la carpeta 'locales' exista
    if !ruta_locales.exists() || !ruta_locales.is_dir() {
        eprintln!("El directorio 'locales' no existe o no es un directorio.");
        return;
    }

    // Iteramos sobre las subcarpetas (idiomas)
    match fs::read_dir(ruta_locales) {
        Ok(entradas) => {
            for entrada in entradas {
                if let Ok(entrada) = entrada {
                    let ruta_idioma = entrada.path();

                    if ruta_idioma.is_dir() {
                        println!(
                            "Idioma: {}",
                            ruta_idioma.file_name().unwrap().to_string_lossy()
                        );

                        // Listamos los archivos JSON dentro de la carpeta del idioma
                        match fs::read_dir(&ruta_idioma) {
                            Ok(archivos) => {
                                for archivo in archivos {
                                    if let Ok(archivo) = archivo {
                                        let ruta_archivo = archivo.path();
                                        if ruta_archivo.extension().and_then(|ext| ext.to_str())
                                            == Some("json")
                                        {
                                            println!(
                                                "  - {}",
                                                ruta_archivo.file_name().unwrap().to_string_lossy()
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => eprintln!(
                                "Error al leer los archivos en {}: {}",
                                ruta_idioma.display(),
                                e
                            ),
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Error al leer el directorio 'locales': {}", e),
    }
}

fn create_lang(idioma: &str) {
    println!("Creando archivo el idioma {}", idioma);
    let ruta_locales = Path::new("locales");

    if !utils::has_exist_directory(ruta_locales) {
        eprintln!("El directorio 'locales' no existe o no es un directorio.");
        return;
    }

    let one_lang = utils::get_first_lang();

    if !utils::create_dir(idioma) {
        return;
    }

    let templates = utils::get_templates(one_lang.clone());

    if templates.is_empty() {
        println!("No hay plantillas para el idioma {}", one_lang);
        return;
    }

    if !utils::create_files(idioma.to_string(), templates) {
        println!("No se pudieron crear archivos para el idioma {}", one_lang);
        return;
    }

    generar_archivos();
}

fn crear_archivo(_idioma: &str) {}

fn generar_archivos() {
    // generar ficheros JSON UNIDOS en uno solo con el nuevo JSON
    let locales_path = Path::new("locales");

    if !utils::has_exist_directory(locales_path) {
        println!("No hay directorio locales");
        return;
    }

    // Combinar los objetos JSON
    let mut file_data = String::new();
    let mut locales_data: HashSet<String> = HashSet::new();
    let mut obj_keys = String::new();

    match fs::read_dir(locales_path) {
        Ok(locales) => {
            for locale in locales {
                if let Ok(locale) = locale {
                    let locale_path = locale.path();
                    if locale_path.is_dir() {
                        let locale_name = locale_path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();

                        let locale_files = utils::get_templates(locale_name.clone());

                        if locale_files.is_empty() {
                            continue;
                        }

                        locales_data.insert(locale_name.clone());

                        // UNION JSON FILES INTO ONE
                        let union_json = utils::merge_json_files(locale_files).unwrap();

                        // SAVE JSON FILE
                        let new_file =
                            fs::write(locale_path.join("_index.json"), union_json.clone());

                        if new_file.is_err() {
                            println!("Error al guardar el archivo 'all.json'");
                            return;
                        }

                        if obj_keys.is_empty() {
                            obj_keys = union_json.clone();
                        }

                        let imports = format!(
                            "import {} from './{}/_index.json';\n",
                            locale_name, locale_name
                        );

                        file_data.push_str(&imports);
                    }
                }
            }

            let translations_keys = utils::get_translations_keys(locales_data.clone());

            let export_data = format!("\nexport const translations = {{ {} }};", translations_keys);

            file_data.push_str(&export_data);
            file_data.push_str("\n");

            if !file_data.is_empty() {
                println!("Generando index.ts {}", file_data);
                match fs::write(locales_path.join("index.ts"), file_data) {
                    Ok(_) => {
                        println!("Se guardo el archivo 'index.ts'");
                        utils::generate_keys_file(obj_keys);
                    }
                    Err(e) => eprintln!("Error al guardar el archivo 'index.ts': {}", e),
                }
            }
        }
        Err(e) => eprintln!("Error al leer el directorio 'locales': {}", e),
    }
}
