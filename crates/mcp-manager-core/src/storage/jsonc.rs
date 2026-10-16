//! In-place edits to JSON and JSONC files.
//!
//! Client config files are often hand-edited: they carry comments, trailing
//! commas, and keys in an order their owner chose. Parsing them into a value
//! and printing it back would lose all of that, so these helpers find the byte
//! range of the member being changed and splice only that range. Everything
//! outside the entries MCP Manager manages stays byte-for-byte the same.

use serde_json::{Map, Value};

/// One `"key": value` member of an object, as byte offsets into the text.
struct Member {
    key: String,
    /// Offset of the key's opening quote.
    start: usize,
    value_start: usize,
    value_end: usize,
}

struct Object {
    open: usize,
    close: usize,
    members: Vec<Member>,
}

/// Indentation used when nothing in the file suggests another.
const DEFAULT_INDENT: &str = "  ";

fn error_at(text: &str, offset: usize, message: &str) -> String {
    let line = text[..offset.min(text.len())].matches('\n').count() + 1;
    format!("{message} at line {line}")
}

/// Skips whitespace, `//` comments and `/* */` comments.
fn skip_trivia(text: &str, mut i: usize) -> Result<usize, String> {
    let bytes = text.as_bytes();
    loop {
        match bytes.get(i) {
            Some(b' ' | b'\t' | b'\n' | b'\r') => i += 1,
            Some(b'/') if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            Some(b'/') if bytes.get(i + 1) == Some(&b'*') => {
                let end = text[i + 2..]
                    .find("*/")
                    .ok_or_else(|| error_at(text, i, "unterminated comment"))?;
                i += 2 + end + 2;
            }
            _ => return Ok(i),
        }
    }
}

/// Returns the decoded string and the offset just past its closing quote.
fn scan_string(text: &str, start: usize) -> Result<(String, usize), String> {
    let bytes = text.as_bytes();
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => {
                let decoded = serde_json::from_str::<String>(&text[start..=i])
                    .map_err(|_| error_at(text, start, "invalid string"))?;
                return Ok((decoded, i + 1));
            }
            _ => i += 1,
        }
    }
    Err(error_at(text, start, "unterminated string"))
}

/// Returns the offset just past the value starting at `start`.
fn skip_value(text: &str, start: usize) -> Result<usize, String> {
    match text.as_bytes().get(start) {
        Some(b'{') => Ok(parse_object(text, start)?.close + 1),
        Some(b'[') => skip_array(text, start),
        Some(b'"') => Ok(scan_string(text, start)?.1),
        Some(_) => {
            let bytes = text.as_bytes();
            let mut i = start;
            while i < bytes.len()
                && !matches!(
                    bytes[i],
                    b',' | b'}' | b']' | b' ' | b'\t' | b'\n' | b'\r' | b'/'
                )
            {
                i += 1;
            }
            if i == start {
                Err(error_at(text, start, "expected a value"))
            } else {
                Ok(i)
            }
        }
        None => Err(error_at(text, start, "unexpected end of file")),
    }
}

fn skip_array(text: &str, open: usize) -> Result<usize, String> {
    let mut i = open + 1;
    loop {
        i = skip_trivia(text, i)?;
        match text.as_bytes().get(i) {
            Some(b']') => return Ok(i + 1),
            Some(b',') => i += 1,
            Some(_) => i = skip_value(text, i)?,
            None => return Err(error_at(text, open, "unterminated array")),
        }
    }
}

/// Parses the object whose `{` is at `open`. Stray and trailing commas are
/// accepted, as JSONC readers do.
fn parse_object(text: &str, open: usize) -> Result<Object, String> {
    let bytes = text.as_bytes();
    let mut members = Vec::new();
    let mut i = open + 1;
    loop {
        i = skip_trivia(text, i)?;
        match bytes.get(i) {
            Some(b'}') => {
                return Ok(Object {
                    open,
                    close: i,
                    members,
                })
            }
            Some(b',') => i += 1,
            Some(b'"') => {
                let start = i;
                let (key, after_key) = scan_string(text, i)?;
                i = skip_trivia(text, after_key)?;
                if bytes.get(i) != Some(&b':') {
                    return Err(error_at(text, i, "expected `:`"));
                }
                let value_start = skip_trivia(text, i + 1)?;
                let value_end = skip_value(text, value_start)?;
                members.push(Member {
                    key,
                    start,
                    value_start,
                    value_end,
                });
                i = value_end;
            }
            Some(_) => return Err(error_at(text, i, "expected a key")),
            None => return Err(error_at(text, open, "unterminated object")),
        }
    }
}

/// Offset of the root object's `{`, after an optional byte order mark.
fn root_object(text: &str) -> Result<Object, String> {
    let bom = if text.starts_with('\u{feff}') { 3 } else { 0 };
    let open = skip_trivia(text, bom)?;
    if text.as_bytes().get(open) != Some(&b'{') {
        return Err("does not contain a JSON object".to_string());
    }
    let root = parse_object(text, open)?;
    let rest = skip_trivia(text, root.close + 1)?;
    if rest != text.len() {
        return Err(error_at(
            text,
            rest,
            "unexpected content after the root object",
        ));
    }
    Ok(root)
}

/// The whitespace before `offset` when it is the first thing on its line.
fn line_indent(text: &str, offset: usize) -> Option<&str> {
    let line_start = text[..offset].rfind('\n').map_or(0, |i| i + 1);
    let prefix = &text[line_start..offset];
    prefix
        .chars()
        .all(|ch| ch == ' ' || ch == '\t')
        .then_some(prefix)
}

/// The indentation unit the file uses, taken from the root object's members.
fn indent_unit(text: &str, root: &Object) -> String {
    root.members
        .first()
        .and_then(|member| line_indent(text, member.start))
        .filter(|indent| !indent.is_empty())
        .unwrap_or(DEFAULT_INDENT)
        .to_string()
}

/// Leading whitespace of the line `offset` is on.
fn leading_whitespace(text: &str, offset: usize) -> &str {
    let line_start = text[..offset].rfind('\n').map_or(0, |i| i + 1);
    let line = &text[line_start..];
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}

/// Indentation for members of `object`, and for its closing brace.
fn member_and_close_indent(text: &str, object: &Object, unit: &str) -> (String, String) {
    let first_member = object
        .members
        .first()
        .and_then(|member| line_indent(text, member.start));
    let close = line_indent(text, object.close)
        .or_else(|| first_member.and_then(|indent| indent.strip_suffix(unit)))
        .unwrap_or_else(|| leading_whitespace(text, object.open))
        .to_string();
    let member = first_member
        .map(str::to_string)
        .unwrap_or_else(|| format!("{close}{unit}"));
    (member, close)
}

/// Pretty-prints `value` for a member whose line starts with `indent`.
fn render(value: &Value, indent: &str, unit: &str) -> String {
    let mut out = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(unit.as_bytes());
    let mut serializer = serde_json::Serializer::with_formatter(&mut out, formatter);
    serde::Serialize::serialize(value, &mut serializer).expect("serialize JSON value");
    let pretty = String::from_utf8(out).expect("serializer writes UTF-8");
    pretty.replace('\n', &format!("\n{indent}"))
}

fn render_member(key: &str, value: &Value, indent: &str, unit: &str) -> String {
    format!(
        "{indent}{}: {}",
        serde_json::to_string(key).expect("serialize key"),
        render(value, indent, unit)
    )
}

struct Edit {
    start: usize,
    end: usize,
    text: String,
}

fn apply_edits(text: &str, mut edits: Vec<Edit>) -> String {
    // Back to front, so earlier offsets stay valid. At the same start the
    // wider edit goes first, so an insertion lands after a deletion there.
    edits.sort_by_key(|edit| std::cmp::Reverse((edit.start, edit.end)));
    let mut out = text.to_string();
    for edit in edits {
        out.replace_range(edit.start..edit.end, &edit.text);
    }
    out
}

/// The range that removes `member` together with the comma after it and, when
/// it sits on lines of its own, those lines.
fn member_with_trailing_comma(text: &str, member: &Member, comma: usize) -> (usize, usize) {
    let mut start = member.start;
    let mut end = comma + 1;
    if let Some(indent) = line_indent(text, member.start) {
        let rest = &text[end..];
        let spaces = rest.len() - rest.trim_start_matches([' ', '\t']).len();
        let after_spaces = &rest[spaces..];
        if after_spaces.starts_with('\n') || after_spaces.starts_with("\r\n") {
            start -= indent.len();
            end += spaces + if after_spaces.starts_with('\n') { 1 } else { 2 };
        }
    }
    (start, end)
}

/// Offset of the comma after `member`, if one follows it.
fn comma_after(text: &str, member: &Member) -> Result<Option<usize>, String> {
    let next = skip_trivia(text, member.value_end)?;
    Ok((text.as_bytes().get(next) == Some(&b',')).then_some(next))
}

/// Replaces, removes and adds entries of the object stored under `field` in
/// the root object, leaving the rest of `text` untouched.
///
/// Entries named in `remove_keys` and not in `new_entries` are removed. Each
/// entry in `new_entries` replaces the existing one in place (after
/// `carry_over` has merged anything from the old value it should keep) or is
/// appended.
pub fn merge_object_entries(
    text: &str,
    field: &str,
    new_entries: &Map<String, Value>,
    remove_keys: &[String],
    carry_over: impl Fn(&Value, &mut Value),
) -> Result<String, String> {
    if text.trim().is_empty() {
        let mut root = Map::new();
        root.insert(field.to_string(), Value::Object(new_entries.clone()));
        return Ok(render(&Value::Object(root), "", DEFAULT_INDENT));
    }

    let root = root_object(text)?;
    let unit = indent_unit(text, &root);
    let Some(host) = root.members.iter().rev().find(|member| member.key == field) else {
        if new_entries.is_empty() {
            return Ok(text.to_string());
        }
        return Ok(append_members(
            text,
            &root,
            &[(field.to_string(), Value::Object(new_entries.clone()))],
            &unit,
        ));
    };
    if text.as_bytes()[host.value_start] != b'{' {
        return Err(format!("field {field} does not contain a JSON object"));
    }
    let object = parse_object(text, host.value_start)?;
    let (member_indent, _) = member_and_close_indent(text, &object, &unit);

    let mut edits = Vec::new();
    let mut written = std::collections::BTreeSet::new();
    let mut removed = vec![false; object.members.len()];
    for (index, member) in object.members.iter().enumerate() {
        if let Some(new_value) = new_entries.get(&member.key) {
            if written.insert(member.key.clone()) {
                let mut value = new_value.clone();
                if let Ok(old) =
                    crate::parser::parse_jsonc(&text[member.value_start..member.value_end])
                {
                    carry_over(&old, &mut value);
                }
                let indent = line_indent(text, member.start).unwrap_or(&member_indent);
                edits.push(Edit {
                    start: member.value_start,
                    end: member.value_end,
                    text: render(&value, indent, &unit),
                });
                continue;
            }
            // A duplicate of an entry already rewritten above.
            removed[index] = true;
        } else if remove_keys.contains(&member.key) {
            removed[index] = true;
        }
    }

    let appended = new_entries
        .iter()
        .filter(|(key, _)| !written.contains(*key))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<Vec<_>>();

    let Some(last_kept) = removed.iter().rposition(|removed| !*removed) else {
        // Nothing of the old object survives: rewrite its inside.
        let (member_indent, close_indent) = member_and_close_indent(text, &object, &unit);
        let inner = if appended.is_empty() {
            String::new()
        } else {
            let members = appended
                .iter()
                .map(|(key, value)| render_member(key, value, &member_indent, &unit))
                .collect::<Vec<_>>()
                .join(",\n");
            format!("\n{members}\n{close_indent}")
        };
        return Ok(apply_edits(
            text,
            vec![Edit {
                start: object.open + 1,
                end: object.close,
                text: inner,
            }],
        ));
    };

    // Removed entries before the last kept one take their own trailing comma.
    for (index, member) in object.members.iter().enumerate().take(last_kept) {
        if removed[index] {
            let comma = comma_after(text, member)?
                .ok_or_else(|| error_at(text, member.value_end, "expected `,`"))?;
            let (start, end) = member_with_trailing_comma(text, member, comma);
            edits.push(Edit {
                start,
                end,
                text: String::new(),
            });
        }
    }
    // Removed entries after it go from the end of the kept one to their own
    // end, which takes the commas between them and leaves a trailing comma,
    // if the file had one, where it was.
    let kept_end = object.members[last_kept].value_end;
    if let Some(last) = object.members[last_kept + 1..].last() {
        edits.push(Edit {
            start: kept_end,
            end: last.value_end,
            text: String::new(),
        });
    }

    if !appended.is_empty() {
        let members = appended
            .iter()
            .map(|(key, value)| format!(",\n{}", render_member(key, value, &member_indent, &unit)))
            .collect::<String>();
        edits.push(Edit {
            start: kept_end,
            end: kept_end,
            text: members,
        });
    }

    Ok(apply_edits(text, edits))
}

/// Appends members to the end of `object`.
fn append_members(text: &str, object: &Object, members: &[(String, Value)], unit: &str) -> String {
    let (member_indent, close_indent) = member_and_close_indent(text, object, unit);
    let Some(last) = object.members.last() else {
        let inner = members
            .iter()
            .map(|(key, value)| render_member(key, value, &member_indent, unit))
            .collect::<Vec<_>>()
            .join(",\n");
        return apply_edits(
            text,
            vec![Edit {
                start: object.open + 1,
                end: object.close,
                text: format!("\n{inner}\n{close_indent}"),
            }],
        );
    };
    let added = members
        .iter()
        .map(|(key, value)| format!(",\n{}", render_member(key, value, &member_indent, unit)))
        .collect::<String>();
    apply_edits(
        text,
        vec![Edit {
            start: last.value_end,
            end: last.value_end,
            text: added,
        }],
    )
}

/// Sets `field` on the root object to `value`, replacing its old value in
/// place or appending it.
pub fn set_root_member(text: &str, field: &str, value: &Value) -> Result<String, String> {
    if text.trim().is_empty() {
        let mut root = Map::new();
        root.insert(field.to_string(), value.clone());
        return Ok(render(&Value::Object(root), "", DEFAULT_INDENT));
    }
    let root = root_object(text)?;
    let unit = indent_unit(text, &root);
    match root.members.iter().rev().find(|member| member.key == field) {
        Some(member) => {
            let indent = line_indent(text, member.start).unwrap_or("");
            Ok(apply_edits(
                text,
                vec![Edit {
                    start: member.value_start,
                    end: member.value_end,
                    text: render(value, indent, &unit),
                }],
            ))
        }
        None => Ok(append_members(
            text,
            &root,
            &[(field.to_string(), value.clone())],
            &unit,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{merge_object_entries, set_root_member};
    use crate::parser::parse_jsonc;
    use serde_json::{json, Map, Value};

    fn entries(value: Value) -> Map<String, Value> {
        value.as_object().expect("object").clone()
    }

    fn keep_nothing(_: &Value, _: &mut Value) {}

    fn merge(text: &str, new: Value, remove: &[&str]) -> String {
        let remove = remove.iter().map(|key| key.to_string()).collect::<Vec<_>>();
        merge_object_entries(text, "mcp", &entries(new), &remove, keep_nothing).expect("merge")
    }

    const OPENCODE: &str = r#"{
  // Theme picked by hand.
  "$schema": "https://opencode.ai/config.json",
  "theme": "tokyonight",
  "mcp": {
    /* shared with the team */
    "github": {
      "type": "remote",
      "url": "https://api.githubcopilot.com/mcp/", // keep this host
    },
    "managed": {
      "type": "local",
      "command": ["old"],
    },
  },
  "model": "anthropic/claude", // trailing comment
}
"#;

    #[test]
    fn replaces_an_entry_in_place_and_leaves_the_rest_byte_for_byte() {
        let out = merge(
            OPENCODE,
            json!({"managed": {"type": "local", "command": ["new"]}}),
            &["managed"],
        );
        // Only the rewritten entry changes; it is printed fresh, keys sorted.
        let expected = OPENCODE.replace(
            "{\n      \"type\": \"local\",\n      \"command\": [\"old\"],\n    }",
            "{\n      \"command\": [\n        \"new\"\n      ],\n      \"type\": \"local\"\n    }",
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn removes_the_last_entry_and_keeps_the_trailing_comma_style() {
        let out = merge(OPENCODE, json!({}), &["managed"]);
        assert!(out.contains("// keep this host"), "{out}");
        assert!(out.contains("/* shared with the team */"), "{out}");
        assert!(!out.contains("managed"), "{out}");
        let parsed = parse_jsonc(&out).expect("valid JSONC");
        assert_eq!(parsed["mcp"].as_object().unwrap().len(), 1);
        assert_eq!(parsed["model"], "anthropic/claude");
    }

    #[test]
    fn appends_after_the_last_entry_with_its_indentation() {
        let text = "{\n    \"mcp\": {\n        \"a\": {\"command\": \"x\"}\n    }\n}\n";
        let out = merge(text, json!({"b": {"command": "y"}}), &[]);
        assert_eq!(
            out,
            "{\n    \"mcp\": {\n        \"a\": {\"command\": \"x\"},\n        \"b\": {\n            \"command\": \"y\"\n        }\n    }\n}\n"
        );
    }

    #[test]
    fn adds_the_field_when_the_file_has_none() {
        let text = "{\n  // only a theme\n  \"theme\": \"dark\"\n}\n";
        let out = merge(text, json!({"a": {"command": "x"}}), &[]);
        assert_eq!(
            out,
            "{\n  // only a theme\n  \"theme\": \"dark\",\n  \"mcp\": {\n    \"a\": {\n      \"command\": \"x\"\n    }\n  }\n}\n"
        );
    }

    #[test]
    fn fills_an_empty_object_and_an_empty_file() {
        assert_eq!(
            merge("{}", json!({"a": {"command": "x"}}), &[]),
            "{\n  \"mcp\": {\n    \"a\": {\n      \"command\": \"x\"\n    }\n  }\n}"
        );
        assert_eq!(
            merge("  \n", json!({"a": {"command": "x"}}), &[]),
            "{\n  \"mcp\": {\n    \"a\": {\n      \"command\": \"x\"\n    }\n  }\n}"
        );
        assert_eq!(
            merge("{\"mcp\": {}}", json!({"a": {"command": "x"}}), &[]),
            "{\"mcp\": {\n  \"a\": {\n    \"command\": \"x\"\n  }\n}}"
        );
    }

    #[test]
    fn removing_every_entry_leaves_an_empty_object() {
        let out = merge(OPENCODE, json!({}), &["github", "managed"]);
        let parsed = parse_jsonc(&out).expect("valid JSONC");
        assert_eq!(parsed["mcp"], json!({}));
        assert!(out.contains("// Theme picked by hand."));
    }

    #[test]
    fn a_file_it_cannot_parse_is_an_error_not_an_overwrite() {
        for broken in [
            "{\"mcp\": {",
            "[1, 2]",
            "{\"a\": 1} trailing",
            "{\"a\" 1}",
            "/* open",
        ] {
            assert!(
                merge_object_entries(broken, "mcp", &Map::new(), &[], keep_nothing).is_err(),
                "{broken}"
            );
        }
    }

    #[test]
    fn a_field_that_is_not_an_object_is_an_error() {
        assert!(
            merge_object_entries("{\"mcp\": []}", "mcp", &Map::new(), &[], keep_nothing).is_err()
        );
    }

    #[test]
    fn strings_that_look_like_syntax_are_not_parsed_as_syntax() {
        let text =
            r#"{"note": "a // b /* c */ { } , \" ]", "mcp": {"a": {"url": "https://x/{id}"}}}"#;
        let out = merge(text, json!({"b": {"command": "y"}}), &["a"]);
        let parsed = parse_jsonc(&out).expect("valid JSON");
        assert_eq!(parsed["note"], "a // b /* c */ { } , \" ]");
        assert_eq!(parsed["mcp"], json!({"b": {"command": "y"}}));
    }

    #[test]
    fn keeps_a_byte_order_mark() {
        let out = merge("\u{feff}{\"mcp\": {}}", json!({}), &[]);
        assert!(out.starts_with('\u{feff}'));
    }

    #[test]
    fn duplicate_keys_collapse_into_the_rewritten_entry() {
        let out = merge(
            r#"{"mcp": {"a": {"command": "1"}, "b": 2, "a": {"command": "2"}}}"#,
            json!({"a": {"command": "new"}}),
            &["a"],
        );
        let parsed = parse_jsonc(&out).expect("valid JSON");
        assert_eq!(parsed["mcp"], json!({"a": {"command": "new"}, "b": 2}));
        assert_eq!(out.matches("\"a\"").count(), 1, "{out}");
    }

    #[test]
    fn carry_over_sees_the_old_value_even_with_comments() {
        let text = "{\"mcp\": {\"a\": {\"command\": \"x\", // note\n \"timeout\": 5,}}}";
        let out = merge_object_entries(
            text,
            "mcp",
            &entries(json!({"a": {"command": "y"}})),
            &["a".to_string()],
            |old, new| {
                new["timeout"] = old["timeout"].clone();
            },
        )
        .expect("merge");
        let parsed = parse_jsonc(&out).expect("valid JSON");
        assert_eq!(parsed["mcp"]["a"], json!({"command": "y", "timeout": 5}));
    }

    /// Every way of keeping, replacing and removing three entries, with and
    /// without trailing commas, gives valid JSONC with exactly the expected
    /// entries and leaves comments outside the touched entries in place.
    #[test]
    fn every_keep_replace_remove_combination_is_valid() {
        let layouts = [
            "{\n  // top\n  \"mcp\": {\n    \"a\": {\"v\": 1},\n    // before b\n    \"b\": {\"v\": 2},\n    \"c\": {\"v\": 3}\n  },\n  \"z\": 0\n}\n",
            "{\n  // top\n  \"mcp\": {\n    \"a\": {\"v\": 1},\n    // before b\n    \"b\": {\"v\": 2},\n    \"c\": {\"v\": 3},\n  },\n  \"z\": 0,\n}\n",
            "{ // top\n\"mcp\":{\"a\":{\"v\":1},\"b\":{\"v\":2},\"c\":{\"v\":3}},\"z\":0}",
        ];
        for layout in layouts {
            // 0 = keep, 1 = replace, 2 = remove, for each of a, b, c.
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
                    let out = merge_object_entries(layout, "mcp", &new, &remove, keep_nothing)
                        .unwrap_or_else(|error| panic!("{actions:?} {add_new}: {error}"));
                    let parsed = parse_jsonc(&out)
                        .unwrap_or_else(|error| panic!("{actions:?} {add_new}: {error}\n{out}"));
                    assert_eq!(parsed["mcp"], Value::Object(expected), "{actions:?}\n{out}");
                    assert_eq!(parsed["z"], 0);
                    assert!(out.contains("// top"), "{out}");
                    if actions[1] == 0 && layout.contains("// before b") {
                        assert!(out.contains("// before b"), "{actions:?}\n{out}");
                    }
                }
            }
        }
    }

    #[test]
    fn sets_a_root_member_in_place() {
        let text = "{\n  // keep\n  \"mcpServers\": {\"old\": 1},\n  \"theme\": \"dark\"\n}\n";
        let out = set_root_member(text, "mcpServers", &json!({"new": 2})).expect("set");
        assert_eq!(
            out,
            "{\n  // keep\n  \"mcpServers\": {\n    \"new\": 2\n  },\n  \"theme\": \"dark\"\n}\n"
        );
        let added =
            set_root_member("{\"theme\": \"dark\"}", "mcpServers", &json!({})).expect("set");
        assert_eq!(parse_jsonc(&added).unwrap()["mcpServers"], json!({}));
    }
}
