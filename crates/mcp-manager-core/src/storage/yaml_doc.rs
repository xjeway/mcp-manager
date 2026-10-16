//! In-place edits to YAML client config files.
//!
//! There is no comment-preserving YAML editor to build on, so this works on
//! lines, for the layout these files use: a top-level `field:` holding a block
//! mapping of entries. Each managed entry's lines are replaced with a freshly
//! printed entry at the same indentation; every other line stays as written.
//! Anything outside that layout (a flow mapping, a list, tabs) is refused
//! rather than guessed at, and every result is parsed back and compared with
//! the original before it is returned.

use serde_json::{Map, Value};

fn parse(text: &str) -> Result<Value, String> {
    if text.trim().is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    let value: Value = serde_yaml::from_str(text).map_err(|e| e.to_string())?;
    match value {
        Value::Object(_) => Ok(value),
        Value::Null => Ok(Value::Object(Map::new())),
        _ => Err("does not contain a YAML mapping".to_string()),
    }
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

/// Neither blank nor only a comment.
fn is_content(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty() && !trimmed.starts_with('#')
}

/// The key a `key: ...` line starts with, if it does.
fn line_key(line: &str) -> Option<String> {
    let rest = line.trim_start_matches(' ').trim_end_matches(['\n', '\r']);
    let (key, after) = if let Some(quoted) = rest.strip_prefix('"') {
        let end = quoted.find('"')?;
        (quoted[..end].to_string(), &quoted[end + 1..])
    } else if let Some(quoted) = rest.strip_prefix('\'') {
        let end = quoted.find('\'')?;
        (quoted[..end].to_string(), &quoted[end + 1..])
    } else {
        let end = rest.find(':')?;
        let key = rest[..end].trim_end();
        if key.is_empty() || key.starts_with(['-', '#', '?', '{', '[']) {
            return None;
        }
        (key.to_string(), &rest[end..])
    };
    let after = after.trim_start_matches(' ');
    let after = after.strip_prefix(':')?;
    (after.is_empty() || after.starts_with(' ')).then_some(key)
}

/// `key: value` printed as block YAML, every line shifted right by `indent`.
///
/// Lines are only prefixed, never re-indented: nested levels keep the two
/// spaces serde_yaml prints (valid in a file indented differently), and the
/// content of a multi-line string keeps its own leading spaces.
fn render_entry(key: &str, value: &Value, indent: usize, newline: &str) -> String {
    let mut wrapper = Map::new();
    wrapper.insert(key.to_string(), value.clone());
    let printed = serde_yaml::to_string(&Value::Object(wrapper)).expect("print YAML entry");
    let pad = " ".repeat(indent);
    printed
        .lines()
        .map(|line| {
            if line.is_empty() {
                newline.to_string()
            } else {
                format!("{pad}{line}{newline}")
            }
        })
        .collect()
}

/// Lines of an entry: its key line through its last content line.
struct Entry {
    key: String,
    start: usize,
    end: usize,
}

/// Replaces, removes and adds the entries of the mapping under the top-level
/// `field`. Entries named in `remove_keys` and not in `new_entries` are
/// removed; the rest of `new_entries` replace existing entries in place (after
/// `carry_over` has merged anything from the old value it should keep) or are
/// appended.
pub fn merge_mapping_entries(
    text: &str,
    field: &str,
    new_entries: &Map<String, Value>,
    remove_keys: &[String],
    carry_over: impl Fn(&Value, &mut Value),
) -> Result<String, String> {
    let before = parse(text)?;
    if text.contains('\t') {
        return Err("tab characters are not supported".to_string());
    }
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let old_entries = match before.get(field) {
        None | Some(Value::Null) => Map::new(),
        Some(Value::Object(entries)) => entries.clone(),
        Some(_) => return Err(format!("field {field} is not a mapping")),
    };

    let lines = text
        .split_inclusive('\n')
        .map(str::to_string)
        .collect::<Vec<_>>();
    let field_line = lines
        .iter()
        .position(|line| indent_of(line) == 0 && line_key(line).as_deref() == Some(field));

    // What each new entry will hold once the old entry's kept fields are in.
    let merged = new_entries
        .iter()
        .map(|(key, value)| {
            let mut value = value.clone();
            if let Some(old) = old_entries.get(key) {
                carry_over(old, &mut value);
            }
            (key.clone(), value)
        })
        .collect::<Map<_, _>>();
    let rendered = |key: &str, indent: usize| render_entry(key, &merged[key], indent, newline);

    let Some(field_line) = field_line else {
        if new_entries.is_empty() {
            return Ok(text.to_string());
        }
        let mut out = text.to_string();
        if !out.is_empty() && !out.ends_with('\n') {
            out.push_str(newline);
        }
        out.push_str(&format!("{field}:{newline}"));
        for key in new_entries.keys() {
            out.push_str(&rendered(key, 2));
        }
        return check(text, &out, field, &merged, remove_keys);
    };

    // `field:` must open a block mapping: nothing after the colon but a
    // comment, or an empty value (`{}`, `~`, `null`) to fill in.
    let after_colon = lines[field_line]
        .split_once(':')
        .map(|(_, rest)| rest.trim())
        .unwrap_or("");
    let value_part = after_colon.split(" #").next().unwrap_or("").trim();
    let empty_inline = matches!(value_part, "{}" | "~" | "null");
    if !value_part.is_empty() && !after_colon.starts_with('#') && !empty_inline {
        return Err(format!("field {field} is not written as a block mapping"));
    }

    // The field's block: following lines up to the next top-level content.
    let block_end = (field_line + 1..lines.len())
        .find(|&index| is_content(&lines[index]) && indent_of(&lines[index]) == 0)
        .unwrap_or(lines.len());
    let last_content = (field_line + 1..block_end)
        .rev()
        .find(|&index| is_content(&lines[index]));
    let child_indent = (field_line + 1..block_end)
        .find(|&index| is_content(&lines[index]))
        .map(|index| indent_of(&lines[index]));
    if empty_inline && child_indent.is_some() {
        return Err(format!("field {field} is not written as a block mapping"));
    }

    let mut entries = Vec::<Entry>::new();
    if let Some(indent) = child_indent {
        for (index, line) in lines
            .iter()
            .enumerate()
            .take(block_end)
            .skip(field_line + 1)
        {
            if !is_content(line) {
                continue;
            }
            if indent_of(line) == indent {
                let key = line_key(line)
                    .ok_or_else(|| format!("field {field} is not written as a block mapping"))?;
                entries.push(Entry {
                    key,
                    start: index,
                    end: index + 1,
                });
            } else if indent_of(line) < indent {
                return Err(format!("field {field} has unexpected indentation"));
            } else if let Some(entry) = entries.last_mut() {
                entry.end = index + 1;
            }
        }
    }
    let indent = child_indent.unwrap_or(2);

    // The first entry of each key is rewritten; later duplicates and removed
    // entries are dropped. New entries go after the field's last content line.
    let mut replacements = std::collections::BTreeMap::new();
    let mut written = Vec::new();
    for entry in &entries {
        let text = if new_entries.contains_key(&entry.key) && !written.contains(&entry.key) {
            written.push(entry.key.clone());
            rendered(&entry.key, indent)
        } else if new_entries.contains_key(&entry.key) || remove_keys.contains(&entry.key) {
            String::new()
        } else {
            continue;
        };
        replacements.insert(entry.start, (entry.end, text));
    }
    let appended = new_entries
        .keys()
        .filter(|key| !written.contains(key))
        .map(|key| rendered(key, indent))
        .collect::<String>();
    let append_after = last_content.unwrap_or(field_line);

    let mut out = String::new();
    let mut index = 0;
    while index < lines.len() {
        let next = match replacements.get(&index) {
            Some((end, text)) => {
                out.push_str(text);
                *end
            }
            None => {
                out.push_str(&lines[index]);
                index + 1
            }
        };
        if (index..next).contains(&append_after) && !appended.is_empty() {
            if !out.is_empty() && !out.ends_with('\n') {
                out.push_str(newline);
            }
            out.push_str(&appended);
        }
        index = next;
    }
    if empty_inline {
        let field_start = lines[..field_line].concat().len();
        let old_line = &lines[field_line];
        out.replace_range(
            field_start..field_start + old_line.len(),
            &format!("{field}:{newline}"),
        );
    }

    check(text, &out, field, &merged, remove_keys)
}

/// Parses `out` back and makes sure it holds exactly what was intended:
/// everything outside `field` unchanged, untouched entries unchanged, removed
/// entries gone, and every written entry equal to its `merged` value.
fn check(
    text: &str,
    out: &str,
    field: &str,
    merged: &Map<String, Value>,
    remove_keys: &[String],
) -> Result<String, String> {
    let before = parse(text)?;
    let after = parse(out).map_err(|error| format!("edit produced invalid YAML: {error}"))?;
    let strip = |value: &Value| {
        let mut value = value.as_object().cloned().unwrap_or_default();
        value.remove(field);
        value
    };
    let entries = |value: &Value| match value.get(field) {
        Some(Value::Object(entries)) => entries.clone(),
        _ => Map::new(),
    };
    let (old, new) = (entries(&before), entries(&after));
    let untouched_kept = old
        .iter()
        .filter(|(key, _)| !merged.contains_key(*key) && !remove_keys.contains(*key))
        .all(|(key, value)| new.get(key) == Some(value));
    let expected_keys = old
        .keys()
        .filter(|key| !remove_keys.contains(*key))
        .chain(merged.keys())
        .collect::<std::collections::BTreeSet<_>>();
    if strip(&before) != strip(&after)
        || !untouched_kept
        || new.keys().collect::<std::collections::BTreeSet<_>>() != expected_keys
        || merged
            .iter()
            .any(|(key, value)| new.get(key) != Some(value))
    {
        return Err(format!(
            "could not edit field {field} without changing other settings"
        ));
    }
    Ok(out.to_string())
}

#[cfg(test)]
mod tests {
    use super::merge_mapping_entries;
    use serde_json::{json, Map, Value};

    fn entries(value: Value) -> Map<String, Value> {
        value.as_object().expect("object").clone()
    }

    fn keep_nothing(_: &Value, _: &mut Value) {}

    fn merge(text: &str, new: Value, remove: &[&str]) -> Result<String, String> {
        let remove = remove.iter().map(|key| key.to_string()).collect::<Vec<_>>();
        merge_mapping_entries(text, "mcp_servers", &entries(new), &remove, keep_nothing)
    }

    fn yaml(text: &str) -> Value {
        serde_yaml::from_str(text).expect("valid YAML")
    }

    const HERMES: &str = "# Hermes config
model:
  default: anthropic/claude  # picked by hand

mcp_servers:
  # Local files
  filesystem:
    command: npx
    args: [\"-y\", \"@modelcontextprotocol/server-filesystem\", \"/home/me\"]
    timeout: 120

  github:
    command: npx
    args:
      - -y
      - \"@modelcontextprotocol/server-github\"
    env:
      GITHUB_PERSONAL_ACCESS_TOKEN: \"${GITHUB_TOKEN}\"

# Terminal settings
terminal:
  backend: local
";

    #[test]
    fn replaces_an_entry_and_leaves_every_other_line_alone() {
        let out = merge(
            HERMES,
            json!({"filesystem": {"command": "uvx", "args": ["fs-mcp"]}}),
            &["filesystem"],
        )
        .expect("merge");
        assert_eq!(
            out,
            HERMES.replace(
                "  filesystem:\n    command: npx\n    args: [\"-y\", \"@modelcontextprotocol/server-filesystem\", \"/home/me\"]\n    timeout: 120\n",
                "  filesystem:\n    args:\n    - fs-mcp\n    command: uvx\n"
            )
        );
    }

    #[test]
    fn removes_and_appends_entries() {
        let out = merge(
            HERMES,
            json!({"remote": {"url": "https://x/mcp"}}),
            &["github"],
        )
        .expect("merge");
        let parsed = yaml(&out);
        assert_eq!(
            parsed["mcp_servers"]
                .as_object()
                .unwrap()
                .keys()
                .collect::<Vec<_>>(),
            ["filesystem", "remote"]
        );
        assert!(out.contains("# Local files\n"), "{out}");
        assert!(out.contains("# Terminal settings\nterminal:\n"), "{out}");
        assert!(out.contains("  remote:\n    url: https://x/mcp\n"), "{out}");
        assert_eq!(parsed["terminal"]["backend"], "local");
    }

    #[test]
    fn removing_every_entry_and_adding_to_an_empty_field() {
        let out = merge(HERMES, json!({}), &["filesystem", "github"]).expect("merge");
        let parsed = yaml(&out);
        assert!(parsed["mcp_servers"].is_null(), "{out}");
        assert_eq!(parsed["model"]["default"], "anthropic/claude");

        let refilled = merge(&out, json!({"a": {"command": "x"}}), &[]).expect("merge");
        assert_eq!(
            yaml(&refilled)["mcp_servers"],
            json!({"a": {"command": "x"}})
        );

        let inline = merge(
            "mcp_servers: {}\nother: 1\n",
            json!({"a": {"command": "x"}}),
            &[],
        )
        .expect("merge");
        assert_eq!(inline, "mcp_servers:\n  a:\n    command: x\nother: 1\n");
    }

    #[test]
    fn adds_the_field_to_a_file_without_it() {
        assert_eq!(
            merge("model: x  # mine", json!({"a": {"command": "x"}}), &[]).unwrap(),
            "model: x  # mine\nmcp_servers:\n  a:\n    command: x\n"
        );
        assert_eq!(
            merge("", json!({"a": {"command": "x"}}), &[]).unwrap(),
            "mcp_servers:\n  a:\n    command: x\n"
        );
        let untouched = "model: x # mine\n";
        assert_eq!(merge(untouched, json!({}), &["a"]).unwrap(), untouched);
    }

    #[test]
    fn keeps_four_space_indentation_and_crlf() {
        let text = "mcp_servers:\r\n    a:\r\n        command: old\r\nz: 1\r\n";
        let out = merge(
            text,
            json!({"a": {"command": "new"}, "b": {"command": "b"}}),
            &["a"],
        )
        .expect("merge");
        assert_eq!(
            out,
            "mcp_servers:\r\n    a:\r\n      command: new\r\n    b:\r\n      command: b\r\nz: 1\r\n"
        );
    }

    #[test]
    fn refuses_layouts_it_cannot_edit_safely() {
        for text in [
            "mcp_servers: {a: {command: x}}\n",
            "mcp_servers:\n  - a\n",
            "mcp_servers:\n\ta:\n\t\tcommand: x\n",
            "mcp_servers: [\n",
            "- just a list\n",
        ] {
            assert!(
                merge(text, json!({"b": {"command": "y"}}), &[]).is_err(),
                "{text}"
            );
        }
    }

    #[test]
    fn carry_over_sees_the_old_entry() {
        let out = merge_mapping_entries(
            HERMES,
            "mcp_servers",
            &entries(json!({"filesystem": {"command": "uvx"}})),
            &["filesystem".to_string()],
            |old, new| new["timeout"] = old["timeout"].clone(),
        )
        .expect("merge");
        assert_eq!(
            yaml(&out)["mcp_servers"]["filesystem"],
            json!({"command": "uvx", "timeout": 120})
        );
    }

    #[test]
    fn duplicate_keys_collapse_into_the_rewritten_entry() {
        let out = merge(
            "mcp_servers:\n  a: 1\n  b: 2\n  a: 3\n",
            json!({"a": {"command": "x"}}),
            &["a"],
        )
        .expect("merge");
        assert_eq!(out, "mcp_servers:\n  a:\n    command: x\n  b: 2\n");
    }

    /// Every keep/replace/remove combination over three entries, with and
    /// without an appended one, gives exactly the expected entries.
    #[test]
    fn every_keep_replace_remove_combination_is_valid() {
        let text = "# top\nmcp_servers:\n  a:\n    v: 1\n  # before b\n  b:\n    v: 2\n\n  c: {v: 3}\nz: 0\n";
        for plan in 0..27 {
            let actions = [plan % 3, plan / 3 % 3, plan / 9 % 3];
            for add_new in [false, true] {
                let mut new = Map::new();
                let mut remove = Vec::new();
                let mut expected = Map::new();
                for (index, key) in ["a", "b", "c"].into_iter().enumerate() {
                    match actions[index] {
                        0 => {
                            expected.insert(key.into(), json!({"v": index + 1}));
                        }
                        1 => {
                            new.insert(key.into(), json!({"v": 10 + index}));
                            remove.push(key.to_string());
                            expected.insert(key.into(), json!({"v": 10 + index}));
                        }
                        _ => remove.push(key.to_string()),
                    }
                }
                if add_new {
                    new.insert("d".into(), json!({"v": 4}));
                    expected.insert("d".into(), json!({"v": 4}));
                }
                let out = merge_mapping_entries(text, "mcp_servers", &new, &remove, keep_nothing)
                    .unwrap_or_else(|error| panic!("{actions:?} {add_new}: {error}"));
                let parsed = yaml(&out);
                let got = match &parsed["mcp_servers"] {
                    Value::Null => Map::new(),
                    value => value.as_object().unwrap().clone(),
                };
                assert_eq!(got, expected, "{actions:?}\n{out}");
                assert_eq!(parsed["z"], 0);
                assert!(out.starts_with("# top\n"), "{out}");
            }
        }
    }

    #[test]
    fn multi_line_strings_keep_their_leading_spaces() {
        let value = "line one\n   three spaces\n  two spaces\n";
        let text = "mcp_servers:\n    a:\n        command: old\n";
        let out = merge_mapping_entries(
            text,
            "mcp_servers",
            &entries(json!({"a": {"command": "x", "env": {"NOTE": value}}})),
            &["a".to_string()],
            keep_nothing,
        )
        .expect("merge");
        assert_eq!(
            yaml(&out)["mcp_servers"]["a"]["env"]["NOTE"],
            value,
            "{out}"
        );
    }
}
