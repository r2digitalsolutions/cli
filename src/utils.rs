use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    str::FromStr,
};

use serde_json::{json, Error, Map, Value};

pub fn create_dir(idioma: &str) -> bool {
    println!("Creando el idioma {}", idioma);
    let ruta_locales = Path::new("locales");
    let ruta_idioma = ruta_locales.join(idioma);

    if has_exist_locales(idioma.to_string()) {
        eprintln!("El directorio {}  existe", idioma);
        return false;
    }

    fs::create_dir(ruta_idioma).expect("Error al crear el directorio");

    return true;
}

pub fn get_first_lang() -> String {
    let ruta_locales = Path::new("locales");

    // Verificamos que la carpeta 'locales' exista
    if !ruta_locales.exists() || !ruta_locales.is_dir() {
        eprintln!("El directorio 'locales' no existe o no es un directorio.");
        return String::new();
    }

    // Iteramos sobre las subcarpetas (idiomas)
    match fs::read_dir(ruta_locales) {
        Ok(entradas) => {
            for entrada in entradas {
                if let Ok(entrada) = entrada {
                    let ruta_idioma = entrada.path();

                    if ruta_idioma.is_dir() {
                        return ruta_idioma
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();
                    }
                }
            }
        }
        Err(e) => eprintln!("Error al leer el directorio 'locales': {}", e),
    }

    String::new()
}

pub fn get_templates(idioma: String) -> HashMap<String, String> {
    let ruta_locales = Path::new("locales");
    let ruta_idioma = ruta_locales.join(idioma.clone());

    if !has_exist_locales(idioma.clone()) {
        eprintln!("El directorio {} no existe", idioma.clone());
        return HashMap::new();
    }

    eprintln!("Ruta idioma: {}", ruta_idioma.display());

    let mut claves_completas: HashMap<String, String> = HashMap::new();

    match fs::read_dir(ruta_idioma) {
        Ok(entradas) => {
            for entrada in entradas {
                if let Ok(entrada) = entrada {
                    let ruta_archivo = entrada.path();
                    if ruta_archivo.extension().and_then(|ext| ext.to_str()) == Some("json") {
                        let nombre_archivo = ruta_archivo
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();

                        let nombre_archivo = nombre_archivo.replace(".json", "");

                        let file_content = fs::read_to_string(ruta_archivo)
                            .expect("Error al leer el archivo JSON");

                        claves_completas.insert(nombre_archivo, file_content);
                    }
                }
            }
        }
        Err(e) => eprintln!("Error al leer el directorio {}: {}", idioma, e),
    }

    return claves_completas;
}

pub fn create_files(idioma: String, templates: HashMap<String, String>) -> bool {
    println!("Creando archivos para el idioma {}", idioma);

    let ruta_locales = Path::new("locales");
    let ruta_idioma = ruta_locales.join(idioma.clone());

    // Verificamos que la carpeta 'locales' exista
    if !has_exist_locales(idioma.clone()) {
        eprintln!("El directorio 'locales' no existe o no es un directorio.");
        return false;
    }

    eprintln!("Ruta idioma: {}", ruta_idioma.display());

    for (template, content) in templates {
        println!("Creando archivo {}.json", template.clone());

        fs::write(ruta_idioma.join(format!("{}.json", template)), content)
            .expect("Error al crear el archivo JSON");
    }

    return true;
}

pub fn has_exist_locales(idioma: String) -> bool {
    let ruta_locales = Path::new("locales");

    // Verificamos que la carpeta 'locales' exista
    if !ruta_locales.exists() || !ruta_locales.is_dir() {
        eprintln!("El directorio 'locales' no existe o no es un directorio.");
        return false;
    }

    let ruta_idioma = ruta_locales.join(idioma.clone());

    if !ruta_idioma.exists() {
        eprintln!("El directorio {} no existe", idioma.clone());
        return false;
    }

    return true;
}

pub fn has_exist_directory(ruta_locales: &Path) -> bool {
    // Verificamos que la carpeta 'locales' exista
    if !ruta_locales.exists() || !ruta_locales.is_dir() {
        eprintln!("El directorio 'locales' no existe o no es un directorio.");
        return false;
    }

    return true;
}

pub(crate) fn merge_json_files(locale_files: HashMap<String, String>) -> Result<String, Error> {
    let mut combined = Map::new();

    locale_files.iter().for_each(|(_, file)| {
        if let Value::Object(obj1) = serde_json::from_str(file).unwrap() {
            combined.extend(obj1);
        }
    });

    let merged_json = Value::Object(combined);
    return serde_json::to_string_pretty(&merged_json);
}

pub(crate) fn get_translations_keys(locales: HashSet<String>) -> String {
    return locales
        .iter()
        .map(String::as_str)
        .collect::<Vec<&str>>()
        .join(", ");
}

fn recursive_keys(obj: Value, prefix: String) -> Map<String, Value> {
    let mut keys = Map::new();

    if let Value::Object(map) = obj {
        for (key, value) in map {
            let full_key = if prefix.is_empty() {
                key.clone() // Si no hay prefijo, usamos la clave tal cual
            } else {
                format!("{}.{}", prefix, key) // Si hay prefijo, lo concatenamos
            };

            if value.is_object() {
                // Si el valor es un objeto, llamamos recursivamente
                let nested_keys = recursive_keys(value, full_key.clone());
                keys.extend(nested_keys); // Extendemos el mapa actual con los resultados
            } else {
                // Si es un valor simple, lo agregamos al mapa
                keys.insert(full_key.clone(), Value::String(full_key.clone()));
            }
        }
    }

    keys
}

pub(crate) fn generate_keys_file(obj_keys: String) {
    let ruta_locales = Path::new("locales");

    if !has_exist_directory(ruta_locales) {
        eprintln!("El directorio 'locales' no existe o no es un directorio.");
        return;
    }

    let parsed: Value = serde_json::from_str(&obj_keys).expect("Error al parsear el JSON");

    let result = recursive_keys(parsed, "".to_string());

    let mut template = String::from_str("export const I18N_TRANSLATIONS = {").unwrap();
    let template_end = String::from_str("\n} as const;").unwrap();

    let result_keys: Vec<String> = result
        .keys()
        .map(|k| k.to_string())
        .collect::<Vec<String>>();

    for key in result_keys {
        let key_object = key.to_uppercase().replace(".", "_").replace("-", "_");
        template.push_str(&format!("\n\t{}: \"{}\",", key_object, key));
    }

    template.push_str(&template_end);

    fs::write(ruta_locales.join("translations.ts"), template)
        .expect("Error al escribir el archivo");

    println!("Se guardo el archivo 'translations.ts'");
}
