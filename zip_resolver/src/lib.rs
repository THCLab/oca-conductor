use oca_rust::state::oca::OCA;
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::io::Read;

#[derive(Debug)]
struct ResolvedFile {
    meta: serde_json::Value,
    files: HashMap<String, String>,
}

pub fn resolve_from_zip(path: &str) -> Result<OCA, String> {
    let fname = std::path::Path::new(path);
    let file =
        fs::File::open(&fname).map_err(|e| format!("Error while loading {} file. {}", path, e))?;
    let reader = BufReader::new(file);

    let mut archive = zip::ZipArchive::new(reader).map_err(|err| err.to_string())?;

    let mut resolved_file = ResolvedFile {
        meta: serde_json::Value::Null,
        files: HashMap::new(),
    };

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.enclosed_name().is_none() {
            return Err(format!("Entry {} has a suspicious path", file.name()));
        };

        if (*file.name()).contains('/') {
            continue;
        } else {
            let mut buffer = String::new();

            file.read_to_string(&mut buffer)
                .map_err(|err| err.to_string())?;
            if file.name().to_string().eq(&String::from("meta.json")) {
                resolved_file.meta =
                    serde_json::from_str(buffer.as_str()).map_err(|err| err.to_string())?;
            } else {
                resolved_file.files.insert(file.name().to_string(), buffer);
            }
        }
    }

    if let serde_json::Value::Null = resolved_file.meta {
        return Err(format!(
            "Malformed OCA Bundle ({}). Missing meta.json file.",
            path
        ));
    }

    let mut oca_option: Option<OCA> = None;
    if let serde_json::Value::String(root_sai) = resolved_file
        .meta
        .get("root")
        .ok_or("Missing 'root' attribute in meta.json file")
        .map_err(|e| e.to_string())?
    {
        let root_filename = format!("{}.json", root_sai);
        let root_file_content = resolved_file.files.remove(&root_filename).ok_or(format!(
            "Malformed OCA Bundle ({}). Missing {} file.",
            path, &root_filename
        ))?;
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
        oca_option = Some(oca_builder.finalize());
    }

    oca_option
        .ok_or("Error while loading OCA Bundle")
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assets_dir_path() -> String {
        format!("{}/assets", env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn resolve_from_proper_flat_oca_bundle_is_ok() {
        let common_assets_dir_path = format!("{}/../assets", env!("CARGO_MANIFEST_DIR"));
        let path = format!("{}/oca_bundle.zip", common_assets_dir_path);
        let oca_result = resolve_from_zip(path.as_str());
        assert!(oca_result.is_ok());
    }

    #[test]
    fn resolve_from_proper_oca_bundle_with_dir_is_ok() {
        let path = format!("{}/oca_bundle_with_dir.zip", assets_dir_path());
        let oca_result = resolve_from_zip(path.as_str());
        assert!(oca_result.is_ok());
    }

    #[test]
    fn resolve_from_missing_oca_bundle_is_err() {
        let path = format!("{}/missing_oca.zip", assets_dir_path());
        let oca_result = resolve_from_zip(path.as_str());
        assert!(oca_result.is_err());
    }

    #[test]
    fn resolve_from_malformed_oca_bundle_is_err() {
        let assets_dir_path = assets_dir_path();
        let paths = vec![
            format!("{}/missing_meta_file.zip", &assets_dir_path),
            format!("{}/missing_root_file.zip", &assets_dir_path),
        ];
        for path in paths {
            let oca_result = resolve_from_zip(path.as_str());
            assert!(oca_result.is_err());
            if let Err(e) = oca_result {
                assert!(e.contains("Malformed OCA Bundle"));
            }
        }
    }
}
