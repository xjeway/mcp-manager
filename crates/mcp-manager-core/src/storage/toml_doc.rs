//! In-place edits to TOML client config files.
//!
//! Like `jsonc`, these keep everything outside the managed entries as the user
//! wrote it: comments, blank lines, key order and the other settings in the
//! file. A managed entry itself is cleared and refilled where it stands.

use serde_json::{Map, Value};
use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table};

fn parse(text: &str) -> Result<DocumentMut, String> {
    text.parse::<DocumentMut>().map_err(|e| e.to_string())
}

fn to_toml_value(value: &Value) -> Option<toml_edit::Value> {
    Some(match value {
        Value::Null => return None,
        Value::Bool(value) => (*value).into(),
        Value::Number(number) => match number.as_i64() {
            Some(integer) => integer.into(),
            None => number.as_f64()?.into(),
        },
        Value::String(value) => value.as_str().into(),
        Value::Array(items) => {
            let mut array = Array::new();
            for item in items.iter().filter_map(to_toml_value) {
                array.push(item);
            }
            toml_edit::Value::Array(array)
        }
        Value::Object(map) => {
            let mut table = InlineTable::new();
            for (key, value) in map {
                if let Some(value) = to_toml_value(value) {
                    table.insert(key, value);
                }
            }
            toml_edit::Value::InlineTable(table)
        }
    })
}

fn value_to_json(value: &toml_edit::Value) -> Value {
    match value {
        toml_edit::Value::String(value) => Value::String(value.value().clone()),
        toml_edit::Value::Integer(value) => Value::from(*value.value()),
        toml_edit::Value::Float(value) => serde_json::Number::from_f64(*value.value())
            .map(Value::Number)
            .unwrap_or(Value::Null),
        toml_edit::Value::Boolean(value) => Value::Bool(*value.value()),
        toml_edit::Value::Datetime(value) => Value::String(value.value().to_string()),
        toml_edit::Value::Array(array) => Value::Array(array.iter().map(value_to_json).collect()),
        toml_edit::Value::InlineTable(table) => Value::Object(
            table
                .iter()
                .map(|(key, value)| (key.to_string(), value_to_json(value)))
                .collect(),
        ),
    }
}

fn table_to_json(table: &Table) -> Value {
    Value::Object(
        table
            .iter()
            .map(|(key, item)| (key.to_string(), item_to_json(item)))
            .collect(),
    )
}

fn item_to_json(item: &Item) -> Value {
    match item {
        Item::None => Value::Null,
        Item::Value(value) => value_to_json(value),
        Item::Table(table) => table_to_json(table),
        Item::ArrayOfTables(tables) => Value::Array(tables.iter().map(table_to_json).collect()),
    }
}

/// Replaces the contents of `table` with `entry`, keeping the table's header,
/// its comments and its place in the file. Nested objects become inline
/// tables. `first` is written before the other keys.
fn refill(table: &mut Table, entry: &Map<String, Value>, first: Option<&str>) {
    table.clear();
    let first = first.and_then(|key| entry.get_key_value(key));
    for (key, value) in first.into_iter().chain(
        entry
            .iter()
            .filter(|(key, _)| Some(key.as_str()) != first.map(|(first, _)| first.as_str())),
    ) {
        if let Some(value) = to_toml_value(value) {
            table.insert(key, Item::Value(value));
        }
    }
}

fn new_table(entry: &Map<String, Value>, first: Option<&str>) -> Table {
    let mut table = Table::new();
    refill(&mut table, entry, first);
    table
}

/// `new` with whatever `carry_over` keeps from `old`.
fn merged(
    old: &Value,
    new: &Value,
    carry_over: &impl Fn(&Value, &mut Value),
) -> Map<String, Value> {
    let mut value = new.clone();
    carry_over(old, &mut value);
    value.as_object().cloned().unwrap_or_default()
}

/// Replaces, removes and adds the `[field.<key>]` tables, as Codex keeps its
/// `[mcp_servers.<name>]` entries.
pub fn merge_table_entries(
    text: &str,
    field: &str,
    new_entries: &Map<String, Value>,
    remove_keys: &[String],
    carry_over: impl Fn(&Value, &mut Value),
) -> Result<String, String> {
    let mut doc = parse(text)?;
    if !doc.contains_key(field) {
        if new_entries.is_empty() {
            return Ok(text.to_string());
        }
        let mut host = Table::new();
        host.set_implicit(true);
        doc.insert(field, Item::Table(host));
    }
    let item = doc.get_mut(field).expect("field inserted above");
    if item.is_inline_table() {
        let inline = std::mem::take(item);
        *item = Item::Table(inline.into_table().expect("inline table converts"));
    }
    let Some(host) = item.as_table_mut() else {
        return Err(format!("field {field} does not contain a TOML table"));
    };

    let existing = host
        .iter()
        .map(|(key, _)| key.to_string())
        .collect::<Vec<_>>();
    for key in &existing {
        if let Some(new_value) = new_entries.get(key) {
            let old = item_to_json(&host[key.as_str()]);
            let entry = merged(&old, new_value, &carry_over);
            match host.get_mut(key).and_then(Item::as_table_mut) {
                Some(table) => refill(table, &entry, None),
                None => {
                    host.insert(key, Item::Table(new_table(&entry, None)));
                }
            }
        } else if remove_keys.contains(key) {
            host.remove(key);
        }
    }
    for (key, value) in new_entries {
        if !existing.contains(key) {
            let entry = value.as_object().cloned().unwrap_or_default();
            host.insert(key, Item::Table(new_table(&entry, None)));
        }
    }
    Ok(doc.to_string())
}

/// Replaces, removes and adds the `[[field]]` entries identified by their
/// `key` value, as Mistral Vibe keeps `[[mcp_servers]]` entries by `name`.
/// `new_entries` is keyed by that value and need not repeat it.
pub fn merge_array_entries(
    text: &str,
    field: &str,
    key: &str,
    new_entries: &Map<String, Value>,
    remove_keys: &[String],
    carry_over: impl Fn(&Value, &mut Value),
) -> Result<String, String> {
    let mut doc = parse(text)?;
    if !doc.contains_key(field) {
        if new_entries.is_empty() {
            return Ok(text.to_string());
        }
        doc.insert(field, Item::ArrayOfTables(ArrayOfTables::new()));
    }
    let item = doc.get_mut(field).expect("field inserted above");
    if item.is_array() {
        let inline = std::mem::take(item);
        *item = inline
            .into_array_of_tables()
            .map(Item::ArrayOfTables)
            .map_err(|_| format!("field {field} is not a list of tables"))?;
    }
    let Some(tables) = item.as_array_of_tables_mut() else {
        return Err(format!("field {field} is not a list of tables"));
    };

    let with_key = |id: &str, entry: &Value| {
        let mut entry = entry.as_object().cloned().unwrap_or_default();
        entry.insert(key.to_string(), Value::String(id.to_string()));
        entry
    };
    let mut written = Vec::new();
    let mut remove = Vec::new();
    for (index, table) in tables.iter_mut().enumerate() {
        let Some(id) = table.get(key).and_then(Item::as_str).map(str::to_string) else {
            continue;
        };
        match new_entries.get(&id) {
            Some(new_value) if !written.contains(&id) => {
                let old = table_to_json(table);
                let entry = merged(&old, &Value::Object(with_key(&id, new_value)), &carry_over);
                refill(table, &entry, Some(key));
                written.push(id);
            }
            // A second entry with a name already rewritten above.
            Some(_) => remove.push(index),
            None if remove_keys.contains(&id) => remove.push(index),
            None => {}
        }
    }
    for index in remove.into_iter().rev() {
        tables.remove(index);
    }
    for (id, value) in new_entries {
        if !written.contains(id) {
            tables.push(new_table(&with_key(id, value), Some(key)));
        }
    }
    Ok(doc.to_string())
}

#[cfg(test)]
mod tests {
    use super::{merge_array_entries, merge_table_entries};
    use serde_json::{json, Map, Value};

    fn entries(value: Value) -> Map<String, Value> {
        value.as_object().expect("object").clone()
    }

    fn keep_nothing(_: &Value, _: &mut Value) {}

    fn keep_timeout(old: &Value, new: &mut Value) {
        if let Some(timeout) = old.get("startup_timeout_sec") {
            new["startup_timeout_sec"] = timeout.clone();
        }
    }

    const CODEX: &str = r#"# Model picked by hand.
model = "gpt-5"

[mcp_servers.docs] # team docs
command = "npx"
args = ["docs-mcp"]
startup_timeout_sec = 30

[mcp_servers.docs.env]
TOKEN = "x"

# Kept for later.
[mcp_servers.keep]
url = "https://keep/mcp"

[profiles.fast]
model = "gpt-5-mini"
"#;

    #[test]
    fn rewrites_a_codex_table_in_place_and_keeps_the_rest() {
        let out = merge_table_entries(
            CODEX,
            "mcp_servers",
            &entries(json!({"docs": {"command": "uvx", "args": ["docs-mcp@2"]}})),
            &["docs".to_string()],
            keep_timeout,
        )
        .expect("merge");
        assert_eq!(
            out,
            r#"# Model picked by hand.
model = "gpt-5"

[mcp_servers.docs] # team docs
args = ["docs-mcp@2"]
command = "uvx"
startup_timeout_sec = 30

# Kept for later.
[mcp_servers.keep]
url = "https://keep/mcp"

[profiles.fast]
model = "gpt-5-mini"
"#
        );
    }

    #[test]
    fn removes_and_adds_codex_tables() {
        let out = merge_table_entries(
            CODEX,
            "mcp_servers",
            &entries(json!({"fresh": {"url": "https://fresh/mcp", "http_headers": {"A": "1"}}})),
            &["docs".to_string()],
            keep_nothing,
        )
        .expect("merge");
        assert!(!out.contains("docs"), "{out}");
        assert!(out.contains("# Model picked by hand."), "{out}");
        assert!(out.contains("# Kept for later."), "{out}");
        assert!(
            out.contains(
                "[mcp_servers.fresh]\nhttp_headers = { A = \"1\" }\nurl = \"https://fresh/mcp\"\n"
            ),
            "{out}"
        );
        let parsed: toml::Table = toml::from_str(&out).expect("valid TOML");
        assert_eq!(
            parsed["profiles"]["fast"]["model"].as_str(),
            Some("gpt-5-mini")
        );
    }

    #[test]
    fn creates_the_codex_field_in_an_empty_file_and_leaves_files_without_it_alone() {
        let out = merge_table_entries(
            "",
            "mcp_servers",
            &entries(json!({"a": {"command": "x"}})),
            &[],
            keep_nothing,
        )
        .expect("merge");
        assert_eq!(out, "[mcp_servers.a]\ncommand = \"x\"\n");
        let untouched = "model = \"gpt-5\" # mine\n";
        assert_eq!(
            merge_table_entries(
                untouched,
                "mcp_servers",
                &Map::new(),
                &["a".into()],
                keep_nothing
            )
            .unwrap(),
            untouched
        );
    }

    #[test]
    fn invalid_toml_is_an_error() {
        assert!(merge_table_entries("[a", "mcp_servers", &Map::new(), &[], keep_nothing).is_err());
        assert!(merge_array_entries(
            "x = ",
            "mcp_servers",
            "name",
            &Map::new(),
            &[],
            keep_nothing
        )
        .is_err());
    }

    const VIBE: &str = r#"# Vibe config
active_model = "devstral"

[[mcp_servers]]
name = "fetch_server"
transport = "stdio"
command = "uvx"
args = ["mcp-server-fetch"]
tool_timeout_sec = 120

[[mcp_servers]]
name = "linear" # OAuth, added by `/mcp add`
transport = "streamable-http"
url = "https://mcp.linear.app/mcp"

[mcp_servers.auth]
type = "oauth"
scopes = []

[tools.fetch_server_get]
permission = "always"
"#;

    #[test]
    fn rewrites_a_vibe_entry_in_place_keyed_by_name() {
        let out = merge_array_entries(
            VIBE,
            "mcp_servers",
            "name",
            &entries(json!({"fetch_server": {"transport": "stdio", "command": "uvx", "args": ["mcp-server-fetch@2"]}})),
            &["fetch_server".to_string()],
            |old: &Value, new: &mut Value| {
                new["tool_timeout_sec"] = old["tool_timeout_sec"].clone();
            },
        )
        .expect("merge");
        assert_eq!(
            out,
            VIBE.replace(
                "name = \"fetch_server\"\ntransport = \"stdio\"\ncommand = \"uvx\"\nargs = [\"mcp-server-fetch\"]\ntool_timeout_sec = 120\n",
                "name = \"fetch_server\"\nargs = [\"mcp-server-fetch@2\"]\ncommand = \"uvx\"\ntool_timeout_sec = 120\ntransport = \"stdio\"\n",
            )
        );
    }

    #[test]
    fn removes_and_appends_vibe_entries() {
        let out = merge_array_entries(
            VIBE,
            "mcp_servers",
            "name",
            &entries(json!({"docs": {"transport": "streamable-http", "url": "https://docs/mcp", "auth": {"type": "static", "headers": {"A": "1"}}}})),
            &["fetch_server".to_string()],
            keep_nothing,
        )
        .expect("merge");
        let parsed: toml::Table = toml::from_str(&out).expect("valid TOML");
        let servers = parsed["mcp_servers"].as_array().unwrap();
        let names = servers
            .iter()
            .map(|server| server["name"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(names, ["linear", "docs"]);
        // The untouched entry keeps its sub-table and comment.
        assert_eq!(servers[0]["auth"]["type"].as_str(), Some("oauth"));
        assert!(out.contains("# OAuth, added by `/mcp add`"), "{out}");
        assert_eq!(servers[1]["auth"]["headers"]["A"].as_str(), Some("1"));
        assert_eq!(
            parsed["tools"]["fetch_server_get"]["permission"].as_str(),
            Some("always")
        );
        assert!(out.starts_with("# Vibe config\n"), "{out}");
    }

    #[test]
    fn duplicate_vibe_names_collapse_and_an_empty_file_gets_the_list() {
        let text = "[[mcp_servers]]\nname = \"a\"\ncommand = \"1\"\n\n[[mcp_servers]]\nname = \"a\"\ncommand = \"2\"\n";
        let out = merge_array_entries(
            text,
            "mcp_servers",
            "name",
            &entries(json!({"a": {"command": "new"}})),
            &["a".to_string()],
            keep_nothing,
        )
        .expect("merge");
        let parsed: toml::Table = toml::from_str(&out).expect("valid TOML");
        assert_eq!(parsed["mcp_servers"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["mcp_servers"][0]["command"].as_str(), Some("new"));

        let fresh = merge_array_entries(
            "",
            "mcp_servers",
            "name",
            &entries(json!({"a": {"transport": "stdio", "command": "x"}})),
            &[],
            keep_nothing,
        )
        .expect("merge");
        assert_eq!(
            fresh,
            "[[mcp_servers]]\nname = \"a\"\ncommand = \"x\"\ntransport = \"stdio\"\n"
        );
    }
}
