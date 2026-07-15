//! Web output download authorization behavior.
//!
//! This module mirrors `web_server.py` output download helpers for
//! fixture-backed URL path validation, registered output authorization, and
//! route response metadata while legacy Python remains the runtime owner.

/// File fixture used by the output-download model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadFileSpec {
    pub path: String,
    pub body: String,
}

/// Synthetic symlink used by canonical-path fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadSymlinkSpec {
    pub path: String,
    pub target: String,
}

/// Modeled response for `GET /api/download/<path>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadResponse {
    pub status_code: u16,
    pub json_error: Option<String>,
    pub json_success: Option<bool>,
    pub body: Option<String>,
    pub download_name: Option<String>,
}

/// Resolves a URL path to a project-relative file path, rejecting traversal.
pub fn safe_requested_download_path(project_root: &str, filepath: &str) -> Option<String> {
    if filepath.is_empty() || filepath.contains('\0') || filepath.contains('\\') {
        return None;
    }
    if has_windows_drive_prefix(filepath) {
        return None;
    }

    let parts = filepath.split('/').collect::<Vec<_>>();
    if filepath.starts_with('/') || parts.contains(&"..") {
        return None;
    }

    Some(normalize_posix_path(&join_posix(project_root, filepath)))
}

/// Returns the requested file only if it is registered as a task output.
pub fn authorized_output_file(
    project_root: &str,
    filepath: &str,
    existing_files: &[DownloadFileSpec],
    registered_outputs: &[String],
) -> Option<String> {
    authorized_output_file_with_symlinks(
        project_root,
        filepath,
        existing_files,
        &[],
        registered_outputs,
    )
}

/// Returns the requested file only when it matches a registered output after synthetic symlink resolution.
pub fn authorized_output_file_with_symlinks(
    project_root: &str,
    filepath: &str,
    existing_files: &[DownloadFileSpec],
    symlinks: &[DownloadSymlinkSpec],
    registered_outputs: &[String],
) -> Option<String> {
    let requested_path = safe_requested_download_path(project_root, filepath)?;
    let requested_path = canonicalize_fixture_path(&requested_path, symlinks);
    if !file_exists(existing_files, symlinks, &requested_path) {
        return None;
    }

    registered_outputs.iter().find_map(|output| {
        let output_path = if is_absolute_posix(output) {
            normalize_posix_path(output)
        } else {
            normalize_posix_path(&join_posix(project_root, output))
        };
        let output_path = canonicalize_fixture_path(&output_path, symlinks);
        (output_path == requested_path).then(|| requested_path.clone())
    })
}

/// Simulates the download route result for fixture-backed files and outputs.
pub fn download_route_response(
    project_root: &str,
    filepath: &str,
    existing_files: &[DownloadFileSpec],
    registered_outputs: &[String],
) -> DownloadResponse {
    download_route_response_with_symlinks(
        project_root,
        filepath,
        existing_files,
        &[],
        registered_outputs,
    )
}

/// Simulates the download route result with synthetic symlink canonicalization.
pub fn download_route_response_with_symlinks(
    project_root: &str,
    filepath: &str,
    existing_files: &[DownloadFileSpec],
    symlinks: &[DownloadSymlinkSpec],
    registered_outputs: &[String],
) -> DownloadResponse {
    let Some(path) = authorized_output_file_with_symlinks(
        project_root,
        filepath,
        existing_files,
        symlinks,
        registered_outputs,
    ) else {
        return DownloadResponse {
            status_code: 404,
            json_error: Some("File not found".to_string()),
            json_success: None,
            body: None,
            download_name: None,
        };
    };

    let body = existing_files
        .iter()
        .find(|file| canonicalize_fixture_path(&file.path, symlinks) == path)
        .map(|file| file.body.clone())
        .unwrap_or_default();
    DownloadResponse {
        status_code: 200,
        json_error: None,
        json_success: None,
        body: Some(body),
        download_name: Some(file_name(&path).to_string()),
    }
}

/// Simulates Flask route decoding before `download_file(filepath)` is invoked.
pub fn download_route_response_from_url_path(
    project_root: &str,
    url_filepath: &str,
    existing_files: &[DownloadFileSpec],
    registered_outputs: &[String],
) -> DownloadResponse {
    download_route_response_from_url_path_with_symlinks(
        project_root,
        url_filepath,
        existing_files,
        &[],
        registered_outputs,
    )
}

/// Simulates Flask route decoding with synthetic symlink canonicalization.
pub fn download_route_response_from_url_path_with_symlinks(
    project_root: &str,
    url_filepath: &str,
    existing_files: &[DownloadFileSpec],
    symlinks: &[DownloadSymlinkSpec],
    registered_outputs: &[String],
) -> DownloadResponse {
    let filepath = percent_decode_route_path(url_filepath);
    if filepath.starts_with('/') {
        return app_level_404_response();
    }
    download_route_response_with_symlinks(
        project_root,
        &filepath,
        existing_files,
        symlinks,
        registered_outputs,
    )
}

/// Simulates app-level Flask 404 behavior for unmatched download routes.
pub fn app_level_404_response() -> DownloadResponse {
    DownloadResponse {
        status_code: 404,
        json_error: Some("Resource not found".to_string()),
        json_success: Some(false),
        body: None,
        download_name: None,
    }
}

fn file_exists(
    existing_files: &[DownloadFileSpec],
    symlinks: &[DownloadSymlinkSpec],
    path: &str,
) -> bool {
    existing_files
        .iter()
        .any(|file| canonicalize_fixture_path(&file.path, symlinks) == path)
}

fn canonicalize_fixture_path(path: &str, symlinks: &[DownloadSymlinkSpec]) -> String {
    let mut current = normalize_posix_path(path);
    for _ in 0..16 {
        let Some(link) = symlinks
            .iter()
            .find(|link| normalize_posix_path(&link.path) == current)
        else {
            return current;
        };
        current = normalize_posix_path(&link.target);
    }
    current
}

fn has_windows_drive_prefix(filepath: &str) -> bool {
    let bytes = filepath.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
}

fn is_absolute_posix(path: &str) -> bool {
    path.starts_with('/')
}

fn join_posix(base: &str, child: &str) -> String {
    if child.is_empty() {
        return normalize_posix_path(base);
    }
    if is_absolute_posix(child) {
        return normalize_posix_path(child);
    }
    let base = base.trim_end_matches('/');
    if base.is_empty() {
        normalize_posix_path(&format!("/{child}"))
    } else {
        normalize_posix_path(&format!("{base}/{child}"))
    }
}

fn normalize_posix_path(path: &str) -> String {
    let absolute = path.starts_with('/');
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if !parts.is_empty() {
                    parts.pop();
                } else if !absolute {
                    parts.push(part);
                }
            }
            _ => parts.push(part),
        }
    }
    if absolute {
        if parts.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", parts.join("/"))
        }
    } else if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn percent_decode_route_path(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
            {
                decoded.push(high << 4 | low);
                index += 3;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, Value, json};

    const FIXTURES: &str = include_str!("../../../../fixtures/web_output_download_security.jsonl");

    fn replace_case_placeholder(value: &str, case_dir: &str) -> String {
        value.replace("__case__", case_dir)
    }

    fn restore_case_placeholder(value: Value, case_dir: &str) -> Value {
        match value {
            Value::String(value) => Value::String(value.replace(case_dir, "__case__")),
            Value::Array(values) => Value::Array(
                values
                    .into_iter()
                    .map(|value| restore_case_placeholder(value, case_dir))
                    .collect(),
            ),
            Value::Object(values) => Value::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, restore_case_placeholder(value, case_dir)))
                    .collect(),
            ),
            value => value,
        }
    }

    fn file_specs(case: &Value, project_root: &str, case_dir: &str) -> Vec<DownloadFileSpec> {
        case["files"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|file| {
                let path = replace_case_placeholder(file["path"].as_str().unwrap(), case_dir);
                let path = if path.starts_with('/') {
                    normalize_posix_path(&path)
                } else {
                    normalize_posix_path(&join_posix(project_root, &path))
                };
                DownloadFileSpec {
                    path,
                    body: file["body"].as_str().unwrap().to_string(),
                }
            })
            .collect()
    }

    fn registered_outputs(case: &Value, case_dir: &str) -> Vec<String> {
        case["registered_outputs"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|output| replace_case_placeholder(output.as_str().unwrap(), case_dir))
            .collect()
    }

    fn symlink_specs(case: &Value, project_root: &str, case_dir: &str) -> Vec<DownloadSymlinkSpec> {
        case["symlinks"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|symlink| {
                let path = replace_case_placeholder(symlink["path"].as_str().unwrap(), case_dir);
                let target =
                    replace_case_placeholder(symlink["target"].as_str().unwrap(), case_dir);
                DownloadSymlinkSpec {
                    path: if path.starts_with('/') {
                        normalize_posix_path(&path)
                    } else {
                        normalize_posix_path(&join_posix(project_root, &path))
                    },
                    target: if target.starts_with('/') {
                        normalize_posix_path(&target)
                    } else {
                        normalize_posix_path(&join_posix(project_root, &target))
                    },
                }
            })
            .collect()
    }

    fn response_to_value(response: DownloadResponse) -> Value {
        let mut value = Map::new();
        value.insert("status_code".to_string(), json!(response.status_code));
        if response.status_code == 200 {
            value.insert("body".to_string(), json!(response.body.unwrap()));
            value.insert(
                "download_name".to_string(),
                json!(response.download_name.unwrap()),
            );
        } else {
            let mut json_payload = Map::new();
            if let Some(success) = response.json_success {
                json_payload.insert("success".to_string(), json!(success));
            }
            json_payload.insert("error".to_string(), json!(response.json_error.unwrap()));
            value.insert("json".to_string(), Value::Object(json_payload));
        }
        Value::Object(value)
    }

    #[test]
    fn web_output_download_follows_parity_fixture_table() {
        for line in FIXTURES.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let case_dir = format!("/tmp/{case_id}");
            let project_root = format!("{case_dir}/project");
            let expected = &case["expect"];

            match case["operation"].as_str().unwrap() {
                "helper" => {
                    let filepath =
                        replace_case_placeholder(case["filepath"].as_str().unwrap(), &case_dir);
                    let safe_path = safe_requested_download_path(&project_root, &filepath);
                    let actual = restore_case_placeholder(
                        json!({
                            "safe_path": safe_path,
                            "authorized": authorized_output_file(
                                &project_root,
                                &filepath,
                                &[],
                                &[],
                            ).is_some(),
                        }),
                        &case_dir,
                    );
                    assert_eq!(&actual, expected, "{case_id}");
                }
                "route" => {
                    let filepath =
                        replace_case_placeholder(case["filepath"].as_str().unwrap(), &case_dir);
                    let files = file_specs(&case, &project_root, &case_dir);
                    let symlinks = symlink_specs(&case, &project_root, &case_dir);
                    let outputs = registered_outputs(&case, &case_dir);
                    let actual =
                        response_to_value(download_route_response_from_url_path_with_symlinks(
                            &project_root,
                            &filepath,
                            &files,
                            &symlinks,
                            &outputs,
                        ));
                    assert_eq!(&actual, expected, "{case_id}");
                }
                "route_raw" => {
                    let actual = response_to_value(app_level_404_response());
                    assert_eq!(&actual, expected, "{case_id}");
                }
                operation => panic!("unknown operation {operation}"),
            }
        }
    }
}
