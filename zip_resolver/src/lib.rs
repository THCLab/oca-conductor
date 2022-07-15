use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::io::Read;

#[derive(Debug)]
struct ResolvedFile {
    meta: serde_json::Value,
    files: HashMap<String, String>,
}

pub fn resolve_from_zip(path: &str) -> String {
    let fname = std::path::Path::new(path);
    let file = fs::File::open(&fname).unwrap();
    let reader = BufReader::new(file);

    let mut archive = zip::ZipArchive::new(reader).unwrap();

    let mut resolved_file = ResolvedFile {
        meta: serde_json::Value::Null,
        files: HashMap::new(),
    };

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => {
                println!("Entry {} has a suspicious path", file.name());
                continue;
            }
        };

        if (*file.name()).ends_with('/') {
            println!(
                "Entry {} is a directory with name \"{}\"",
                i,
                outpath.display()
            );
        } else {
            let mut buffer = String::new();

            file.read_to_string(&mut buffer).unwrap();
            if file.name().to_string().eq(&String::from("meta.json")) {
                resolved_file.meta = serde_json::from_str(buffer.as_str()).unwrap();
            } else {
                resolved_file.files.insert(file.name().to_string(), buffer);
            }
        }
    }

    if let serde_json::Value::String(root_sai) = resolved_file.meta.get("root").unwrap() {
        let root_filename = format!("{}.json", root_sai);
        let root_file_content = resolved_file.files.remove(&root_filename).unwrap();
        let data = format!(
            r#"{{"capture_base": {}, "overlays": [{}] }}"#,
            root_file_content,
            resolved_file
                .files
                .values()
                .cloned()
                .collect::<Vec<String>>()
                .join(",")
        );

        let oca_builder = oca_rust::controller::load_oca(&mut data.as_bytes()).unwrap();
        let oca = oca_builder.finalize();
        println!("{:?}", oca.capture_base.attributes);
    }

    path.to_string()
}
