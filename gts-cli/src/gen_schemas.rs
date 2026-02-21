use anyhow::{Result, bail};
use gts::{GtsInstanceId, GtsSchemaId};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Directories that are automatically ignored (e.g., trybuild `compile_fail` tests)
const AUTO_IGNORE_DIRS: &[&str] = &["compile_fail"];

/// Reason why a file was skipped
#[derive(Debug, Clone, Copy)]
enum SkipReason {
    ExcludePattern,
    AutoIgnoredDir,
    IgnoreDirective,
}

impl std::fmt::Display for SkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExcludePattern => write!(f, "matched --exclude pattern"),
            Self::AutoIgnoredDir => write!(f, "in auto-ignored directory (compile_fail)"),
            Self::IgnoreDirective => write!(f, "has // gts:ignore directive"),
        }
    }
}

/// Parsed macro attributes from `#[struct_to_gts_schema(...)]`
#[derive(Debug, Clone)]
struct MacroAttrs {
    dir_path: String,
    schema_id: String,
    description: Option<String>,
    properties: Option<String>,
    base: BaseAttr,
    /// Parsed const field overrides (`field_name`, `json_value`)
    const_values: Vec<(String, serde_json::Value)>,
}

/// Base attribute type
#[derive(Debug, Clone)]
enum BaseAttr {
    /// `base = true` - this is a base type
    IsBase,
    /// `base = ParentStruct` - this type inherits from `ParentStruct`
    Parent(String),
}

/// Generate GTS schemas from Rust source code with `#[struct_to_gts_schema]` annotations
///
/// # Arguments
/// * `source` - Source directory or file to scan
/// * `output` - Optional output directory override
/// * `exclude_patterns` - Patterns to exclude (supports simple glob matching)
/// * `verbose` - Verbosity level (0 = normal, 1+ = show skipped files)
///
/// # Errors
///
/// Returns an error if:
/// - The source path does not exist
/// - The output path is outside the source repository
/// - File I/O operations fail
pub fn generate_schemas_from_rust(
    source: &str,
    output: Option<&str>,
    exclude_patterns: &[String],
    verbose: u8,
) -> Result<()> {
    println!("Scanning Rust source files in: {source}");

    let source_path = Path::new(source);
    if !source_path.exists() {
        bail!("Source path does not exist: {source}");
    }

    // Canonicalize source path to detect path traversal attempts
    let source_canonical = source_path.canonicalize()?;

    let mut schemas_generated = 0;
    let mut files_scanned = 0;
    let mut files_skipped = 0;

    // Walk through all .rs files
    for entry in WalkDir::new(source_path)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        // Check if path should be excluded
        if should_exclude_path(path, exclude_patterns) {
            files_skipped += 1;
            if verbose > 0 {
                println!(
                    "  Skipped: {} ({})",
                    path.display(),
                    SkipReason::ExcludePattern
                );
            }
            continue;
        }

        // Check for auto-ignored directories (e.g., compile_fail)
        if is_in_auto_ignored_dir(path) {
            files_skipped += 1;
            if verbose > 0 {
                println!(
                    "  Skipped: {} ({})",
                    path.display(),
                    SkipReason::AutoIgnoredDir
                );
            }
            continue;
        }

        files_scanned += 1;
        if let Ok(content) = fs::read_to_string(path) {
            // Check for gts:ignore directive
            if has_ignore_directive(&content) {
                files_skipped += 1;
                if verbose > 0 {
                    println!(
                        "  Skipped: {} ({})",
                        path.display(),
                        SkipReason::IgnoreDirective
                    );
                }
                continue;
            }

            // Parse the file and extract schema information
            let results = extract_and_generate_schemas(&content, output, &source_canonical, path)?;
            schemas_generated += results.len();
            for (schema_id, file_path) in results {
                println!("  Generated schema: {schema_id} @ {file_path}");
            }
        }
    }

    println!("\nSummary:");
    println!("  Files scanned: {files_scanned}");
    println!("  Files skipped: {files_skipped}");
    println!("  Schemas generated: {schemas_generated}");

    if schemas_generated == 0 {
        println!(
            "\n- No schemas found. Make sure your structs are annotated with `#[struct_to_gts_schema(...)]`"
        );
    }

    Ok(())
}

/// Check if a path matches any of the exclude patterns
fn should_exclude_path(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();

    for pattern in patterns {
        if matches_glob_pattern(&path_str, pattern) {
            return true;
        }
    }

    false
}

/// Simple glob pattern matching
/// Supports: * (any characters), ** (any path segments)
fn matches_glob_pattern(path: &str, pattern: &str) -> bool {
    // Convert glob pattern to regex
    let regex_pattern = pattern
        .replace('.', r"\.")
        .replace("**", "<<DOUBLESTAR>>")
        .replace('*', "[^/]*")
        .replace("<<DOUBLESTAR>>", ".*");

    if let Ok(re) = Regex::new(&format!("(^|/){regex_pattern}($|/)")) {
        re.is_match(path)
    } else {
        // Fallback to simple contains check
        path.contains(pattern)
    }
}

/// Check if path is in an auto-ignored directory (e.g., `compile_fail`)
fn is_in_auto_ignored_dir(path: &Path) -> bool {
    path.components().any(|component| {
        if let Some(name) = component.as_os_str().to_str() {
            AUTO_IGNORE_DIRS.contains(&name)
        } else {
            false
        }
    })
}

/// Check if file content starts with the gts:ignore directive
fn has_ignore_directive(content: &str) -> bool {
    // Check first few lines for the directive
    for line in content.lines().take(10) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Check for the directive (case-insensitive)
        if trimmed.to_lowercase().starts_with("// gts:ignore") {
            return true;
        }
        // If we hit a non-comment, non-empty line, stop looking
        if !trimmed.starts_with("//") && !trimmed.starts_with("#!") {
            break;
        }
    }
    false
}

/// Unescape a Rust string literal's content (already stripped of outer `"`) so that
/// the CLI sees the same string the proc-macro sees after the Rust lexer decodes it.
/// Only handles the escapes that are relevant in `const_values` strings:
///   `\\` → `\`, `\'` → `'`, `\"` → `"`, `\n` → newline, `\t` → tab.
/// Any unrecognised `\X` is left as-is.
fn unescape_rust_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') | None => out.push('\\'),
                Some('\'') => out.push('\''),
                Some('"') => out.push('"'),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some(x) => {
                    out.push('\\');
                    out.push(x);
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

///
/// - Unquoted: `true`/`false` → bool, integer literal → i64, float → f64, else → String.
/// - Single-quoted: contents become a JSON String (commas inside allowed, `\'` → `'`).
/// - Malformed entries (missing `=`) emit a warning to stderr and are skipped.
fn parse_const_values_cli(s: &str) -> Vec<(String, serde_json::Value)> {
    let mut result = Vec::new();
    let mut remaining = s.trim();
    while !remaining.is_empty() {
        // Determine whether the next token is well-formed (has `=` before any `,`).
        let comma_pos = remaining.find(',');
        let eq_pos = remaining.find('=');
        let is_malformed = match (eq_pos, comma_pos) {
            (None, _) => true,                       // no `=` anywhere
            (Some(eq), Some(cm)) if cm < eq => true, // comma before `=`
            _ => false,
        };
        if is_malformed {
            // warn instead of silently dropping the malformed entry.
            let bad_token = remaining.split(',').next().unwrap_or(remaining).trim();
            eprintln!(
                "gts-cli warning: malformed const_values entry '{bad_token}': \
                 missing '='. Expected format: field=value or field='value with spaces'. Entry skipped."
            );
            // Skip only this one token and continue with the rest.
            remaining = comma_pos.map_or("", |p| {
                remaining[p..].trim_start_matches(|c: char| c == ',' || c.is_whitespace())
            });
            continue;
        }
        let Some(eq_pos) = eq_pos else { continue };
        let key = remaining[..eq_pos].trim().to_owned();
        remaining = remaining[eq_pos + 1..].trim_start();
        if key.is_empty() {
            break;
        }
        let val: serde_json::Value;
        if remaining.starts_with('\'') {
            let chars: Vec<char> = remaining.chars().collect();
            let mut val_str = String::new();
            let mut i = 1usize;
            loop {
                match chars.get(i) {
                    None => break,
                    Some('\'') => {
                        i += 1;
                        break;
                    }
                    Some('\\') => {
                        // In the raw .rs file `\'` = single backslash + `'` = escaped quote.
                        // `\\'` = two backslashes + `'` = literal backslash, then closing quote.
                        // Peek at the next char to decide.
                        i += 1;
                        match chars.get(i) {
                            Some('\'') => {
                                // `\'` → insert a literal `'` and consume it.
                                val_str.push('\'');
                                i += 1;
                            }
                            Some('\\') => {
                                // `\\` → insert a literal `\` and leave the next char for the
                                // next iteration (it may be `'` which closes the string).
                                val_str.push('\\');
                                i += 1;
                            }
                            Some(c) => {
                                val_str.push('\\');
                                val_str.push(*c);
                                i += 1;
                            }
                            None => break,
                        }
                    }
                    Some(c) => {
                        val_str.push(*c);
                        i += 1;
                    }
                }
            }
            let consumed: usize = chars[..i].iter().map(|c| c.len_utf8()).sum();
            remaining = &remaining[consumed..];
            val = serde_json::Value::String(val_str);
        } else {
            let comma_pos = remaining.find(',').unwrap_or(remaining.len());
            let val_str = remaining[..comma_pos].trim();
            val = if val_str == "true" {
                serde_json::Value::Bool(true)
            } else if val_str == "false" {
                serde_json::Value::Bool(false)
            } else if let Ok(n) = val_str.parse::<i64>() {
                serde_json::Value::Number(n.into())
            } else if let Ok(f) = val_str.parse::<f64>() {
                // Fall back to a string instead of silently coercing to 0.
                if let Some(n) = serde_json::Number::from_f64(f) {
                    serde_json::Value::Number(n)
                } else {
                    serde_json::Value::String(val_str.to_owned())
                }
            } else {
                serde_json::Value::String(val_str.to_owned())
            };
            remaining = &remaining[comma_pos..];
        }
        result.push((key, val));
        remaining = remaining.trim_start_matches(|c: char| c == ',' || c.is_whitespace());
    }
    result
}

/// Parse the attribute body of `#[struct_to_gts_schema(...)]` to extract individual attributes
fn parse_macro_attrs(attr_body: &str) -> Option<MacroAttrs> {
    // Patterns for extracting individual attributes
    let dir_path_re = Regex::new(r#"dir_path\s*=\s*"([^"]+)""#).ok()?;
    let schema_id_re = Regex::new(r#"schema_id\s*=\s*"([^"]+)""#).ok()?;
    let description_re = Regex::new(r#"description\s*=\s*"([^"]+)""#).ok()?;
    let properties_re = Regex::new(r#"properties\s*=\s*"([^"]*)""#).ok()?;
    let const_values_re = Regex::new(r#"const_values\s*=\s*"([^"]*)""#).ok()?;
    let base_true_re = Regex::new(r"\bbase\s*=\s*true\b").ok()?;
    let base_parent_re = Regex::new(r"\bbase\s*=\s*([A-Z]\w*)").ok()?;

    // Extract required fields
    let dir_path = dir_path_re.captures(attr_body)?.get(1)?.as_str().to_owned();
    let schema_id = schema_id_re
        .captures(attr_body)?
        .get(1)?
        .as_str()
        .to_owned();

    // Extract optional fields
    let description = description_re
        .captures(attr_body)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_owned()));
    let properties = properties_re
        .captures(attr_body)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_owned()));
    let const_values = const_values_re
        .captures(attr_body)
        .and_then(|c| {
            c.get(1).map(|m| {
                // The CLI reads raw .rs source bytes, but the proc-macro receives the
                // Rust-lexer-decoded string (e.g. `\\'` on disk → `\'` → `'`). Unescape
                // Rust string escapes so both code paths see the same content.
                let unescaped = unescape_rust_str(m.as_str());
                parse_const_values_cli(&unescaped)
            })
        })
        .unwrap_or_default();

    // Parse base attribute
    let base = if base_true_re.is_match(attr_body) {
        BaseAttr::IsBase
    } else if let Some(cap) = base_parent_re.captures(attr_body) {
        BaseAttr::Parent(cap.get(1)?.as_str().to_owned())
    } else {
        // base is required but not found
        return None;
    };

    Some(MacroAttrs {
        dir_path,
        schema_id,
        description,
        properties,
        base,
        const_values,
    })
}

/// Extract schema metadata from Rust source and generate JSON files
/// Returns a vector of (`schema_id`, `file_path`) tuples for each generated schema
fn extract_and_generate_schemas(
    content: &str,
    output_override: Option<&str>,
    source_root: &Path,
    source_file: &Path,
) -> Result<Vec<(String, String)>> {
    // Match #[struct_to_gts_schema(...)] followed by struct definition.
    // Also skips `///` / `//!` doc-comment lines and blank lines between the attribute and struct.
    // The attribute body capture uses `(?:"[^"]*"|[^)"]+)+` so that `)` inside a
    // double-quoted string value does not terminate the capture prematurely.
    // Captures: (1) attribute body, (2) struct name, (3) optional generics, (4) struct body or semicolon for unit structs
    let re = Regex::new(
        r#"(?s)#\[struct_to_gts_schema\(((?:"[^"]*"|[^)"]+)+)\)\]\s*(?:(?:#\[[^\]]+\]|///[^\n]*|//![^\n]*)\s*)*(?:pub\s+)?struct\s+(\w+)(?:<([^>]+)>)?\s*(?:\{([^}]*)\}|;)"#,
    )?;

    // Pre-compile field regex outside the loop
    let field_re = Regex::new(r"(?m)^\s*(?:pub\s+)?(r#)?(\w+)\s*:\s*([^,\n]+)")?;

    // Pass 1: build a map of struct_name -> name of the GtsSchemaId field (for auto const injection)
    let mut gts_schema_id_field: HashMap<String, String> = HashMap::new();
    for cap in re.captures_iter(content) {
        let struct_name = cap[2].to_owned();
        let struct_body = cap.get(4).map_or("", |m| m.as_str());
        for field_cap in field_re.captures_iter(struct_body) {
            let field_name = &field_cap[2];
            let field_type = field_cap[3].trim().trim_end_matches(',');
            if field_type == "GtsSchemaId" {
                gts_schema_id_field.insert(struct_name.clone(), field_name.to_owned());
            }
        }
    }

    let mut results = Vec::new();

    // Pass 2: generate schemas
    for cap in re.captures_iter(content) {
        let attr_body = &cap[1];
        let struct_name = &cap[2];
        let struct_body = cap.get(4).map_or("", |m| m.as_str());

        // Parse macro attributes
        let Some(attrs) = parse_macro_attrs(attr_body) else {
            continue;
        };

        // Determine parent's GtsSchemaId field name for auto const injection
        let parent_schema_id_field: Option<&str> = match &attrs.base {
            BaseAttr::Parent(parent_name) => {
                gts_schema_id_field.get(parent_name).map(String::as_str)
            }
            BaseAttr::IsBase => None,
        };

        // Convert schema_id to filename-safe format
        let schema_file_rel = format!("{}/{}.schema.json", attrs.dir_path, attrs.schema_id);

        // Determine output path
        let output_path = if let Some(output_dir) = output_override {
            Path::new(output_dir).join(&schema_file_rel)
        } else {
            let source_dir = source_file.parent().unwrap_or(source_root);
            source_dir.join(&schema_file_rel)
        };

        // Security check: ensure output path doesn't escape source repository
        let output_canonical = if output_path.exists() {
            output_path.canonicalize()?
        } else {
            let parent = output_path.parent().unwrap_or(Path::new("."));
            fs::create_dir_all(parent)?;
            let parent_canonical = parent.canonicalize()?;
            let file_name = output_path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid output path: no file name"))?;
            parent_canonical.join(file_name)
        };

        if !output_canonical.starts_with(source_root) {
            bail!(
                "Security error in {}:{} - dir_path '{}' attempts to write outside source repository. \
                Resolved to: {}, but must be within: {}",
                source_file.display(),
                struct_name,
                attrs.dir_path,
                output_canonical.display(),
                source_root.display()
            );
        }

        // Parse struct fields
        let mut field_types = HashMap::new();
        for field_cap in field_re.captures_iter(struct_body) {
            let field_name = &field_cap[2];
            let field_type = field_cap[3].trim().trim_end_matches(',');
            field_types.insert(field_name.to_owned(), field_type.to_owned());
        }

        // Build JSON schema
        let schema = build_json_schema(
            &attrs.schema_id,
            struct_name,
            attrs.description.as_deref(),
            attrs.properties.as_deref(),
            &attrs.base,
            &field_types,
            &attrs.const_values,
            parent_schema_id_field,
        );

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&output_path, serde_json::to_string_pretty(&schema)?)?;
        results.push((attrs.schema_id, output_path.display().to_string()));
    }

    Ok(results)
}

/// Build a JSON Schema object from parsed metadata
#[allow(clippy::too_many_arguments)]
fn build_json_schema(
    schema_id: &str,
    struct_name: &str,
    description: Option<&str>,
    properties_list: Option<&str>,
    base: &BaseAttr,
    field_types: &HashMap<String, String>,
    const_values: &[(String, serde_json::Value)],
    parent_schema_id_field: Option<&str>,
) -> serde_json::Value {
    use serde_json::json;

    let mut schema_properties = serde_json::Map::new();
    let mut required = Vec::new();

    // Determine which properties to include
    let property_names: Vec<&str> = if let Some(props) = properties_list {
        props.split(',').map(str::trim).collect()
    } else {
        // If no properties specified, include all fields
        field_types.keys().map(String::as_str).collect()
    };

    for prop in &property_names {
        if let Some(field_type) = field_types.get(*prop) {
            let (is_required, json_type_info) = rust_type_to_json_schema(field_type);

            schema_properties.insert((*prop).to_owned(), json_type_info);

            if is_required {
                required.push((*prop).to_owned());
            }
        }
    }

    // Sort required array for consistent output
    required.sort();

    // Build schema based on whether this has a parent

    match base {
        BaseAttr::IsBase => {
            // Base type - simple flat schema
            let mut s = json!({
                "$id": format!("gts://{schema_id}"),
                "$schema": "http://json-schema.org/draft-07/schema#",
                "title": struct_name,
                "type": "object",
                "additionalProperties": false,
                "properties": schema_properties
            });

            if let Some(desc) = description {
                s["description"] = json!(desc);
            }

            if !required.is_empty() {
                s["required"] = json!(required);
            }

            s
        }
        BaseAttr::Parent(parent_name) => {
            // Child type - use allOf with $ref to parent
            let parent_schema_id = derive_parent_schema_id(schema_id);

            // Auto-inject const for the parent's GtsSchemaId field (e.g. "type")
            if let Some(sid_field) = parent_schema_id_field {
                schema_properties.insert(sid_field.to_owned(), json!({ "const": schema_id }));
            }

            // Inject each const_value as {"const": <value>} in properties
            for (field, val) in const_values {
                schema_properties.insert(field.clone(), json!({ "const": val }));
            }

            let mut own_properties = json!({
                "properties": schema_properties
            });

            if !required.is_empty() {
                own_properties["required"] = json!(required);
            }

            let mut s = json!({
                "$id": format!("gts://{schema_id}"),
                "$schema": "http://json-schema.org/draft-07/schema#",
                "title": format!("{struct_name} (extends {parent_name})"),
                "type": "object",
                "allOf": [
                    { "$ref": format!("gts://{parent_schema_id}") },
                    own_properties
                ]
            });

            if let Some(desc) = description {
                s["description"] = json!(desc);
            }

            s
        }
    }
}

/// Derive parent schema ID from child schema ID
/// e.g., "gts.x.core.events.type.v1~x.core.audit.event.v1~" -> "gts.x.core.events.type.v1~"
fn derive_parent_schema_id(schema_id: &str) -> String {
    // Remove trailing ~ if present for processing
    let s = schema_id.trim_end_matches('~');

    // Find the last ~ and take everything before it, then add ~ back
    if let Some(pos) = s.rfind('~') {
        format!("{}~", &s[..pos])
    } else {
        // No parent segment found, this shouldn't happen for child types
        schema_id.to_owned()
    }
}

/// Convert Rust type string to JSON Schema type
/// Returns (`is_required`, `json_schema_value`)
///
/// This function inlines the actual schema definitions for GTS types (like `GtsInstanceId`)
/// to match what schemars generates, including custom extensions like `x-gts-ref`.
fn rust_type_to_json_schema(rust_type: &str) -> (bool, serde_json::Value) {
    use serde_json::json;

    let rust_type = rust_type.trim();

    // Check if it's an Option type
    let is_optional = rust_type.starts_with("Option<");
    let inner_type = if is_optional {
        rust_type
            .strip_prefix("Option<")
            .and_then(|s| s.strip_suffix('>'))
            .unwrap_or(rust_type)
            .trim()
    } else {
        rust_type
    };

    let json_schema = match inner_type {
        "String" | "str" | "&str" => json!({ "type": "string" }),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" => json!({ "type": "integer" }),
        "f32" | "f64" => json!({ "type": "number" }),
        "bool" => json!({ "type": "boolean" }),
        t if t.starts_with("Vec<") => {
            let item_type = t
                .strip_prefix("Vec<")
                .and_then(|s| s.strip_suffix('>'))
                .unwrap_or("string");
            let (_, item_schema) = rust_type_to_json_schema(item_type);
            json!({
                "type": "array",
                "items": item_schema
            })
        }
        t if t.starts_with("HashMap<") || t.starts_with("BTreeMap<") => {
            json!({ "type": "object" })
        }
        t if t.contains("Uuid") || t.contains("uuid") => {
            json!({ "type": "string", "format": "uuid" })
        }
        // GtsInstanceId - use the canonical schema from the gts crate
        "GtsInstanceId" => GtsInstanceId::json_schema_value(),
        // GtsSchemaId - use the canonical schema from the gts crate
        "GtsSchemaId" => GtsSchemaId::json_schema_value(),
        // Generic type parameter (e.g., P, T, etc.) - treat as object
        t if t.len() <= 2 && t.chars().all(|c| c.is_ascii_uppercase()) => {
            json!({ "type": "object" })
        }
        // Other types - default to object (could be another struct)
        _ => json!({ "type": "object" }),
    };

    // For Option types, add null to the type array
    let final_schema = if is_optional {
        if let Some(type_val) = json_schema.get("type").and_then(|v| v.as_str()) {
            json!({ "type": [type_val, "null"] })
        } else {
            // For $ref types, wrap in oneOf with null
            json!({
                "oneOf": [
                    json_schema,
                    { "type": "null" }
                ]
            })
        }
    } else {
        json_schema
    };

    (!is_optional, final_schema)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_glob_pattern() {
        // Test simple patterns
        assert!(matches_glob_pattern(
            "src/tests/compile_fail/test.rs",
            "compile_fail"
        ));
        assert!(matches_glob_pattern(
            "tests/compile_fail/test.rs",
            "compile_fail"
        ));

        // Test wildcard patterns
        assert!(matches_glob_pattern("src/tests/foo.rs", "tests/*"));
        assert!(matches_glob_pattern("src/examples/bar.rs", "examples/*"));

        // Test double-star patterns
        assert!(matches_glob_pattern("a/b/c/d/test.rs", "**/test.rs"));
    }

    #[test]
    fn test_is_in_auto_ignored_dir() {
        assert!(is_in_auto_ignored_dir(Path::new(
            "tests/compile_fail/test.rs"
        )));
        assert!(is_in_auto_ignored_dir(Path::new("src/compile_fail/foo.rs")));
        assert!(!is_in_auto_ignored_dir(Path::new("src/models.rs")));
        assert!(!is_in_auto_ignored_dir(Path::new("tests/integration.rs")));
    }

    #[test]
    fn test_has_ignore_directive() {
        assert!(has_ignore_directive("// gts:ignore\nuse foo::bar;"));
        assert!(has_ignore_directive("// GTS:IGNORE\nuse foo::bar;"));
        assert!(has_ignore_directive(
            "//! Module doc\n// gts:ignore\nuse foo::bar;"
        ));
        assert!(!has_ignore_directive("use foo::bar;\n// gts:ignore"));
        assert!(!has_ignore_directive("use foo::bar;"));
    }

    #[test]
    fn test_parse_macro_attrs_base_true() {
        let attr_body = r#"
            dir_path = "schemas",
            base = true,
            schema_id = "gts.x.core.events.type.v1~",
            description = "Base event type"
        "#;

        let attrs = parse_macro_attrs(attr_body).unwrap();
        assert_eq!(attrs.dir_path, "schemas");
        assert_eq!(attrs.schema_id, "gts.x.core.events.type.v1~");
        assert_eq!(attrs.description.as_deref(), Some("Base event type"));
        assert!(matches!(attrs.base, BaseAttr::IsBase));
    }

    #[test]
    fn test_parse_macro_attrs_base_parent() {
        let attr_body = r#"
            dir_path = "schemas",
            base = BaseEventV1,
            schema_id = "gts.x.core.events.type.v1~x.core.audit.event.v1~"
        "#;

        let attrs = parse_macro_attrs(attr_body).unwrap();
        assert_eq!(attrs.dir_path, "schemas");
        assert_eq!(
            attrs.schema_id,
            "gts.x.core.events.type.v1~x.core.audit.event.v1~"
        );
        assert!(matches!(attrs.base, BaseAttr::Parent(ref p) if p == "BaseEventV1"));
    }

    #[test]
    fn test_derive_parent_schema_id() {
        assert_eq!(
            derive_parent_schema_id("gts.x.core.events.type.v1~x.core.audit.event.v1~"),
            "gts.x.core.events.type.v1~"
        );
        assert_eq!(
            derive_parent_schema_id(
                "gts.x.core.events.type.v1~x.core.audit.event.v1~x.marketplace.orders.purchase.v1~"
            ),
            "gts.x.core.events.type.v1~x.core.audit.event.v1~"
        );
    }

    #[test]
    fn test_rust_type_to_json_schema() {
        // Basic types
        let (req, schema) = rust_type_to_json_schema("String");
        assert!(req);
        assert_eq!(schema["type"], "string");

        let (req, schema) = rust_type_to_json_schema("i32");
        assert!(req);
        assert_eq!(schema["type"], "integer");

        let (req, schema) = rust_type_to_json_schema("bool");
        assert!(req);
        assert_eq!(schema["type"], "boolean");

        // Optional types
        let (req, schema) = rust_type_to_json_schema("Option<String>");
        assert!(!req);
        assert_eq!(schema["type"][0], "string");
        assert_eq!(schema["type"][1], "null");

        // Vec types
        let (req, schema) = rust_type_to_json_schema("Vec<String>");
        assert!(req);
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "string");

        // GTS types - now inlined with x-gts-ref extension
        let (req, schema) = rust_type_to_json_schema("GtsInstanceId");
        assert!(req);
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["format"], "gts-instance-id");
        assert_eq!(schema["x-gts-ref"], "gts.*");

        // Generic type parameter
        let (req, schema) = rust_type_to_json_schema("P");
        assert!(req);
        assert_eq!(schema["type"], "object");
    }

    #[test]
    fn test_should_exclude_path_matching_pattern() {
        let patterns = vec!["test_*".to_owned(), "**/target/**".to_owned()];
        let path = Path::new("src/test_helper.rs");
        assert!(should_exclude_path(path, &patterns));
    }

    #[test]
    fn test_should_exclude_path_no_match() {
        let patterns = vec!["test_*".to_owned(), "**/compile_fail/**".to_owned()];
        let path = Path::new("src/main.rs");
        assert!(!should_exclude_path(path, &patterns));
    }

    #[test]
    fn test_build_json_schema_base_type() {
        use serde_json::json;

        let mut field_types = HashMap::new();
        field_types.insert("id".to_owned(), "String".to_owned());
        field_types.insert("count".to_owned(), "i32".to_owned());
        field_types.insert("active".to_owned(), "bool".to_owned());

        let schema = build_json_schema(
            "gts.x.test.base.v1~",
            "BaseStruct",
            Some("A base test struct"),
            None, // Include all properties
            &BaseAttr::IsBase,
            &field_types,
            &[],
            None,
        );

        assert_eq!(schema["$id"], "gts://gts.x.test.base.v1~");
        assert_eq!(schema["$schema"], "http://json-schema.org/draft-07/schema#");
        assert_eq!(schema["title"], "BaseStruct");
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["description"], "A base test struct");
        assert_eq!(schema["additionalProperties"], false);

        // Check properties
        assert!(schema["properties"]["id"].is_object());
        assert!(schema["properties"]["count"].is_object());
        assert!(schema["properties"]["active"].is_object());

        // Check required fields (all 3 should be required)
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 3);
        assert!(required.contains(&json!("active")));
        assert!(required.contains(&json!("count")));
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn test_build_json_schema_child_type() {
        let mut field_types = HashMap::new();
        field_types.insert("child_field".to_owned(), "String".to_owned());
        field_types.insert("optional_field".to_owned(), "Option<i32>".to_owned());

        let schema = build_json_schema(
            "gts.x.test.base.v1~x.test.child.v1~",
            "ChildStruct",
            Some("A child test struct"),
            None,
            &BaseAttr::Parent("BaseStruct".to_owned()),
            &field_types,
            &[],
            None,
        );

        assert_eq!(schema["$id"], "gts://gts.x.test.base.v1~x.test.child.v1~");
        assert_eq!(schema["title"], "ChildStruct (extends BaseStruct)");
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["description"], "A child test struct");

        // Check allOf structure
        let all_of = schema["allOf"].as_array().unwrap();
        assert_eq!(all_of.len(), 2);

        // First element should be $ref to parent
        assert_eq!(all_of[0]["$ref"], "gts://gts.x.test.base.v1~");

        // Second element should have child properties
        assert!(all_of[1]["properties"]["child_field"].is_object());
        assert!(all_of[1]["properties"]["optional_field"].is_object());

        // Check required fields (only child_field, not optional_field)
        let required = all_of[1]["required"].as_array().unwrap();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0], "child_field");
    }

    #[test]
    fn test_parse_macro_attrs_with_schema_id() {
        let attr_body = r#"
            dir_path = "schemas",
            base = true,
            schema_id = "gts.x.custom.id.v1~"
        "#;

        let attrs = parse_macro_attrs(attr_body).unwrap();
        assert_eq!(attrs.schema_id, "gts.x.custom.id.v1~");
        assert_eq!(attrs.dir_path, "schemas");
        assert!(matches!(attrs.base, BaseAttr::IsBase));
        assert!(attrs.description.is_none());
    }

    #[test]
    fn test_extract_and_generate_schemas_single_struct() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().canonicalize().unwrap();

        // Create a test Rust file with a struct
        let test_file = temp_path.join("test.rs");
        let content = r#"
use gts::GtsInstanceId;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.person.v1~",
    description = "A test person struct"
)]
pub struct Person {
    pub id: GtsInstanceId,
    pub name: String,
    pub age: i32,
}
"#;
        fs::write(&test_file, content).unwrap();

        // Call extract_and_generate_schemas
        let results = extract_and_generate_schemas(
            content,
            Some(temp_path.to_str().unwrap()),
            &temp_path,
            &test_file,
        )
        .unwrap();

        // Verify results
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "gts.x.test.person.v1~");

        // Verify schema file was created
        let schema_path = Path::new(&results[0].1);
        assert!(schema_path.exists());

        // Verify schema content
        let schema_content = fs::read_to_string(schema_path).unwrap();
        let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();

        assert_eq!(schema["$id"], "gts://gts.x.test.person.v1~");
        assert_eq!(schema["title"], "Person");
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["description"], "A test person struct");
        assert!(schema["properties"]["id"].is_object());
        assert!(schema["properties"]["name"].is_object());
        assert!(schema["properties"]["age"].is_object());
    }

    #[test]
    fn test_extract_and_generate_schemas_with_parent() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().canonicalize().unwrap();

        // Create a test file with parent and child structs
        let test_file = temp_path.join("test.rs");
        let content = r#"
use gts::GtsInstanceId;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.base.v1~",
    description = "Base event"
)]
pub struct BaseEvent {
    pub id: GtsInstanceId,
    pub timestamp: String,
}

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = BaseEvent,
    schema_id = "gts.x.test.base.v1~x.test.child.v1~",
    description = "Child event"
)]
pub struct ChildEvent {
    pub event_type: String,
    pub data: String,
}
"#;
        fs::write(&test_file, content).unwrap();

        // Call extract_and_generate_schemas
        let results = extract_and_generate_schemas(
            content,
            Some(temp_path.to_str().unwrap()),
            &temp_path,
            &test_file,
        )
        .unwrap();

        // Verify results - should have 2 schemas
        assert_eq!(results.len(), 2);

        // Find base and child schemas
        let base_result = results
            .iter()
            .find(|(id, _)| id == "gts.x.test.base.v1~")
            .unwrap();
        let child_result = results
            .iter()
            .find(|(id, _)| id == "gts.x.test.base.v1~x.test.child.v1~")
            .unwrap();

        // Verify base schema
        let base_schema_path = Path::new(&base_result.1);
        assert!(base_schema_path.exists());
        let base_schema: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(base_schema_path).unwrap()).unwrap();
        assert_eq!(base_schema["title"], "BaseEvent");
        assert!(base_schema["properties"]["id"].is_object());

        // Verify child schema
        let child_schema_path = Path::new(&child_result.1);
        assert!(child_schema_path.exists());
        let child_schema: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(child_schema_path).unwrap()).unwrap();
        assert_eq!(child_schema["title"], "ChildEvent (extends BaseEvent)");

        // Verify allOf structure with parent reference
        let all_of = child_schema["allOf"].as_array().unwrap();
        assert_eq!(all_of.len(), 2);
        assert_eq!(all_of[0]["$ref"], "gts://gts.x.test.base.v1~");
        assert!(all_of[1]["properties"]["event_type"].is_object());
    }

    #[test]
    fn test_rust_type_to_json_schema_option_string() {
        let (required, schema) = rust_type_to_json_schema("Option<String>");
        assert!(!required);
        assert_eq!(schema["type"][0], "string");
        assert_eq!(schema["type"][1], "null");
    }

    #[test]
    fn test_rust_type_to_json_schema_hashmap() {
        let (required, schema) = rust_type_to_json_schema("HashMap<String, i32>");
        assert!(required);
        assert_eq!(schema["type"], "object");
    }

    #[test]
    fn test_rust_type_to_json_schema_vec_bool() {
        let (required, schema) = rust_type_to_json_schema("Vec<bool>");
        assert!(required);
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "boolean");
    }

    #[test]
    fn test_rust_type_to_json_schema_unknown_type() {
        let (required, schema) = rust_type_to_json_schema("CustomStruct");
        assert!(required);
        assert_eq!(schema["type"], "object");
    }

    #[test]
    fn test_should_exclude_path_with_patterns() {
        let patterns = vec!["target/**".to_owned(), "*.tmp".to_owned()];

        assert!(should_exclude_path(
            Path::new("target/debug/foo"),
            &patterns
        ));
        assert!(should_exclude_path(Path::new("file.tmp"), &patterns));
        assert!(!should_exclude_path(Path::new("src/main.rs"), &patterns));
    }

    #[test]
    fn test_should_exclude_path_empty_patterns() {
        let patterns = vec![];
        assert!(!should_exclude_path(Path::new("anything.rs"), &patterns));
    }

    #[test]
    fn test_is_in_auto_ignored_dir_specific_paths() {
        assert!(is_in_auto_ignored_dir(Path::new(
            "tests/compile_fail/test.rs"
        )));
        assert!(is_in_auto_ignored_dir(Path::new("src/compile_fail/foo.rs")));
        assert!(!is_in_auto_ignored_dir(Path::new("target/debug")));
        assert!(!is_in_auto_ignored_dir(Path::new("node_modules/pkg")));
        assert!(!is_in_auto_ignored_dir(Path::new("src/main.rs")));
        assert!(!is_in_auto_ignored_dir(Path::new("tests/test.rs")));
    }

    #[test]
    fn test_has_ignore_directive_variations() {
        assert!(has_ignore_directive("// gts:ignore\nstruct Foo {}"));
        assert!(has_ignore_directive("// GTS:IGNORE\nstruct Foo {}"));
        assert!(!has_ignore_directive("struct Foo {}\nfn bar() {}"));
        assert!(!has_ignore_directive("struct Foo {}\n// gts:ignore"));
    }

    #[test]
    fn test_parse_macro_attrs_edge_cases() {
        // With base true
        let attr1 = r#"dir_path = "schemas", base = true, schema_id = "gts.x.test.v1~""#;
        let result1 = parse_macro_attrs(attr1).unwrap();
        assert_eq!(result1.dir_path, "schemas");
        assert_eq!(result1.schema_id, "gts.x.test.v1~");
        assert!(matches!(result1.base, BaseAttr::IsBase));

        // With base parent
        let attr2 = r#"dir_path = "schemas", base = ParentStruct, schema_id = "gts.x.test.v1~""#;
        let result2 = parse_macro_attrs(attr2).unwrap();
        assert!(matches!(result2.base, BaseAttr::Parent(ref p) if p == "ParentStruct"));

        // Missing field
        assert!(parse_macro_attrs(r#"dir_path = "schemas""#).is_none());

        // Malformed
        assert!(parse_macro_attrs(r"invalid syntax here").is_none());
    }

    #[test]
    fn test_generate_schemas_from_rust_with_exclude() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test file that should be excluded
        let test_file = temp_path.join("test_excluded.rs");
        fs::write(&test_file, "// test file").unwrap();

        // Call with exclude pattern
        let result = generate_schemas_from_rust(
            temp_path.to_str().unwrap(),
            None,
            &["test_*.rs".to_owned()],
            1, // verbose
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_schemas_from_rust_with_ignore_directive() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test file with ignore directive
        let test_file = temp_path.join("ignored.rs");
        fs::write(&test_file, "// gts:ignore\nstruct Foo {}").unwrap();

        let result = generate_schemas_from_rust(
            temp_path.to_str().unwrap(),
            None,
            &[],
            1, // verbose
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_schemas_from_rust_nonexistent_path() {
        let result =
            generate_schemas_from_rust("/nonexistent/path/that/does/not/exist", None, &[], 0);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    // -------------------------------------------------------------------------
    // parse_const_values_cli – unit tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_const_values_cli_bool_true() {
        let result = parse_const_values_cli("active=true");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "active");
        assert_eq!(result[0].1, serde_json::Value::Bool(true));
    }

    #[test]
    fn test_parse_const_values_cli_bool_false() {
        let result = parse_const_values_cli("enabled=false");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1, serde_json::Value::Bool(false));
    }

    #[test]
    fn test_parse_const_values_cli_integer() {
        let result = parse_const_values_cli("http_status=400");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1, serde_json::Value::Number(400.into()));
    }

    #[test]
    fn test_parse_const_values_cli_negative_integer() {
        let result = parse_const_values_cli("offset=-1");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1, serde_json::Value::Number((-1_i64).into()));
    }

    #[test]
    fn test_parse_const_values_cli_float() {
        let result = parse_const_values_cli("threshold=0.95");
        assert_eq!(result.len(), 1);
        let n = serde_json::Number::from_f64(0.95_f64).unwrap();
        assert_eq!(result[0].1, serde_json::Value::Number(n));
    }

    #[test]
    fn test_parse_const_values_cli_unquoted_string() {
        let result = parse_const_values_cli("code=ERR_NOT_FOUND");
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].1,
            serde_json::Value::String("ERR_NOT_FOUND".to_owned())
        );
    }

    #[test]
    fn test_parse_const_values_cli_single_quoted_string_with_comma() {
        // Comma inside the quoted value must NOT split the entry.
        let result =
            parse_const_values_cli("message='Invalid input: field required, value missing'");
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].1,
            serde_json::Value::String("Invalid input: field required, value missing".to_owned())
        );
    }

    #[test]
    fn test_parse_const_values_cli_escaped_single_quote() {
        let result = parse_const_values_cli("message='can\\'t stop'");
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].1,
            serde_json::Value::String("can't stop".to_owned())
        );
    }

    #[test]
    fn test_parse_const_values_cli_multiple_entries() {
        let result = parse_const_values_cli("http_status=400,message='bad request',retry=false");
        assert_eq!(result.len(), 3);
        assert_eq!(
            result[0],
            (
                "http_status".to_owned(),
                serde_json::Value::Number(400.into())
            )
        );
        assert_eq!(
            result[1],
            (
                "message".to_owned(),
                serde_json::Value::String("bad request".to_owned())
            )
        );
        assert_eq!(
            result[2],
            ("retry".to_owned(), serde_json::Value::Bool(false))
        );
    }

    #[test]
    fn test_parse_const_values_cli_empty_string() {
        // Empty input produces no entries and must not panic.
        let result = parse_const_values_cli("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_const_values_cli_whitespace_only() {
        let result = parse_const_values_cli("   ");
        assert!(result.is_empty());
    }

    /// Bug fix regression: a malformed entry (missing `=`) must be skipped,
    /// not silently dropped along with all subsequent entries.
    #[test]
    fn test_parse_const_values_cli_malformed_entry_skipped() {
        // "bar" has no `=` – it should be warned about and skipped.
        // The valid entry that follows must still be parsed.
        let result = parse_const_values_cli("foo=1,bar,baz=2");
        // "foo" and "baz" are valid; "bar" is malformed and skipped.
        assert_eq!(result.len(), 2, "expected foo and baz, got {result:?}");
        assert!(result.iter().any(|(k, _)| k == "foo"));
        assert!(result.iter().any(|(k, _)| k == "baz"));
    }

    /// Bug fix regression: a leading malformed entry must not truncate the rest.
    #[test]
    fn test_parse_const_values_cli_leading_malformed_entry_skipped() {
        let result = parse_const_values_cli("bad_entry,good=42");
        assert_eq!(result.len(), 1, "expected only good, got {result:?}");
        assert_eq!(result[0].0, "good");
        assert_eq!(result[0].1, serde_json::Value::Number(42.into()));
    }

    /// Bug fix regression: `NaN` must NOT be silently coerced to 0.
    /// It is not a valid JSON number so it must fall back to a String.
    #[test]
    fn test_parse_const_values_cli_nan_falls_back_to_string() {
        let result = parse_const_values_cli("score=NaN");
        assert_eq!(result.len(), 1);
        // `NaN` parses as f64 but from_f64 returns None → fallback to String.
        assert_eq!(
            result[0].1,
            serde_json::Value::String("NaN".to_owned()),
            "NaN must not be coerced to 0"
        );
    }

    /// Bug fix regression: `inf` must NOT be silently coerced to 0.
    #[test]
    fn test_parse_const_values_cli_inf_falls_back_to_string() {
        let result = parse_const_values_cli("limit=inf");
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].1,
            serde_json::Value::String("inf".to_owned()),
            "inf must not be coerced to 0"
        );
    }

    // -------------------------------------------------------------------------
    // build_json_schema – 3-level hierarchy with const_values (#5)
    // -------------------------------------------------------------------------

    #[test]
    fn test_build_json_schema_three_level_hierarchy() {
        use serde_json::json;

        // L1 – base
        let mut l1_fields = HashMap::new();
        l1_fields.insert("type".to_owned(), "GtsSchemaId".to_owned());
        l1_fields.insert("severity".to_owned(), "i32".to_owned());
        l1_fields.insert("source".to_owned(), "String".to_owned());
        l1_fields.insert("payload".to_owned(), "P".to_owned());

        let l1 = build_json_schema(
            "gts.x.test.three.v1~",
            "ThreeBaseV1",
            Some("L1 base"),
            Some("type,severity,source,payload"),
            &BaseAttr::IsBase,
            &l1_fields,
            &[],
            None,
        );
        assert_eq!(l1["$id"], "gts://gts.x.test.three.v1~");
        assert!(l1["properties"]["type"].is_object());
        assert!(l1["properties"]["payload"].is_object());
        assert!(l1["allOf"].is_null(), "base must NOT have allOf");

        // L2 – child of L1, with const_values
        let mut l2_fields = HashMap::new();
        l2_fields.insert("channel".to_owned(), "String".to_owned());
        l2_fields.insert("data".to_owned(), "D".to_owned());

        let l2 = build_json_schema(
            "gts.x.test.three.v1~x.test.three.alert.v1~",
            "ThreeAlertV1",
            Some("L2 alert"),
            Some("channel,data"),
            &BaseAttr::Parent("ThreeBaseV1".to_owned()),
            &l2_fields,
            &[
                ("severity".to_owned(), json!(3)),
                ("source".to_owned(), json!("alert-svc")),
            ],
            Some("type"), // parent's GtsSchemaId field
        );
        assert_eq!(
            l2["$id"],
            "gts://gts.x.test.three.v1~x.test.three.alert.v1~"
        );
        let l2_allof = l2["allOf"].as_array().expect("L2 must have allOf");
        assert_eq!(l2_allof[0]["$ref"], "gts://gts.x.test.three.v1~");
        let l2_props = l2_allof[1]["properties"]
            .as_object()
            .expect("L2 allOf[1].properties");
        // auto-injected type const from parent's GtsSchemaId field
        assert_eq!(
            l2_props["type"]["const"], "gts.x.test.three.v1~x.test.three.alert.v1~",
            "type const must be auto-injected from parent's GtsSchemaId field"
        );
        // const_values
        assert_eq!(l2_props["severity"]["const"], 3);
        assert_eq!(l2_props["source"]["const"], "alert-svc");
        // own structural properties
        assert!(l2_props["channel"].is_object());
        assert!(l2_props["data"].is_object());

        // L3 – child of L2, with const_values; L2 has no GtsSchemaId field
        let mut l3_fields = HashMap::new();
        l3_fields.insert("to".to_owned(), "String".to_owned());
        l3_fields.insert("subject".to_owned(), "String".to_owned());

        let l3 = build_json_schema(
            "gts.x.test.three.v1~x.test.three.alert.v1~x.test.three.email.v1~",
            "ThreeEmailV1",
            Some("L3 email"),
            Some("to,subject"),
            &BaseAttr::Parent("ThreeAlertV1".to_owned()),
            &l3_fields,
            &[("channel".to_owned(), json!("email"))],
            None, // L2 has no GtsSchemaId field
        );
        assert_eq!(
            l3["$id"],
            "gts://gts.x.test.three.v1~x.test.three.alert.v1~x.test.three.email.v1~"
        );
        let l3_allof = l3["allOf"].as_array().expect("L3 must have allOf");
        // L3 must $ref L2, not L1
        assert_eq!(
            l3_allof[0]["$ref"], "gts://gts.x.test.three.v1~x.test.three.alert.v1~",
            "L3 must reference L2 via derive_parent_schema_id"
        );
        let l3_props = l3_allof[1]["properties"]
            .as_object()
            .expect("L3 allOf[1].properties");
        // const_values
        assert_eq!(
            l3_props["channel"]["const"], "email",
            "channel const must be present in L3"
        );
        // L3 must NOT have auto-injected type const (L2 has no GtsSchemaId field)
        assert!(
            !l3_props.contains_key("type"),
            "L3 must not auto-inject type (L2 has no GtsSchemaId field)"
        );
        // own structural properties
        assert!(l3_props["to"].is_object(), "L3 must have 'to' property");
        assert!(
            l3_props["subject"].is_object(),
            "L3 must have 'subject' property"
        );
        // required
        let l3_required = l3_allof[1]["required"]
            .as_array()
            .expect("L3 must have required");
        assert!(
            l3_required.contains(&json!("to")),
            "L3 required must include 'to'"
        );
        assert!(
            l3_required.contains(&json!("subject")),
            "L3 required must include 'subject'"
        );
    }

    #[test]
    fn test_derive_parent_schema_id_three_segments() {
        // 3-segment ID: derive_parent_schema_id must return the 2-segment parent
        assert_eq!(
            derive_parent_schema_id(
                "gts.x.test.three.v1~x.test.three.alert.v1~x.test.three.email.v1~"
            ),
            "gts.x.test.three.v1~x.test.three.alert.v1~"
        );
    }

    /// `parse_const_values_cli` is called from `parse_macro_attrs`; verify the
    /// round-trip when `const_values` appears in a realistic attribute body.
    #[test]
    fn test_parse_macro_attrs_const_values_round_trip() {
        let attr_body = r#"
            dir_path = "schemas",
            base = true,
            schema_id = "gts.x.test.v1~",
            description = "Test",
            properties = "status",
            const_values = "http_status=400,message='bad request'"
        "#;
        let attrs = parse_macro_attrs(attr_body).unwrap();
        assert_eq!(attrs.const_values.len(), 2);
        assert_eq!(attrs.const_values[0].0, "http_status");
        assert_eq!(
            attrs.const_values[0].1,
            serde_json::Value::Number(400.into())
        );
        assert_eq!(attrs.const_values[1].0, "message");
        assert_eq!(
            attrs.const_values[1].1,
            serde_json::Value::String("bad request".to_owned())
        );
    }
}
