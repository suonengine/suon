use serde::Serialize;
use toml::{Table, Value};

#[derive(Clone, Copy)]
pub struct DocumentedStruct {
    pub docs: &'static [&'static str],
    pub fields: &'static [DocumentedField],
}

#[derive(Clone, Copy)]
pub struct DocumentedField {
    pub name: &'static str,
    pub docs: &'static [&'static str],
    pub kind: DocumentedFieldKind,
}

#[derive(Clone, Copy)]
pub enum DocumentedFieldKind {
    Value,
    Table {
        docs: fn() -> DocumentedStruct,
    },
    OptionalTable {
        docs: fn() -> DocumentedStruct,
        sample: fn() -> Value,
    },
    ArrayOfTables {
        docs: fn() -> DocumentedStruct,
        sample: fn() -> Value,
    },
}

pub trait DocumentedToml: Serialize + Default {
    fn documented_toml() -> DocumentedStruct;

    fn default_toml_value() -> Value
    where
        Self: Sized,
    {
        Value::try_from(Self::default()).expect("documented TOML defaults must serialize")
    }
}

pub fn write_documented_toml<T>(value: &T) -> anyhow::Result<String>
where
    T: DocumentedToml,
{
    let value = Value::try_from(value)?;
    let table = value
        .as_table()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("documented TOML root must serialize as a table"))?;

    let docs = T::documented_toml();
    let mut output = String::new();

    write_doc_lines(&mut output, docs.docs);
    write_table_fields(&mut output, &[], &table, docs)?;

    Ok(output.trim_end().to_string() + "\n")
}

fn write_table_fields(
    output: &mut String,
    path: &[&str],
    table: &Table,
    docs: DocumentedStruct,
) -> anyhow::Result<()> {
    let mut sections: Vec<(bool, String)> = Vec::new();

    for &field in docs.fields {
        if !matches!(field.kind, DocumentedFieldKind::Value) {
            continue;
        }

        let mut section = String::new();
        write_value_field(&mut section, table, field);
        if !section.trim().is_empty() {
            sections.push((true, section));
        }
    }

    for &field in docs.fields {
        if matches!(field.kind, DocumentedFieldKind::Value) {
            continue;
        }

        let mut section = String::new();

        match field.kind {
            DocumentedFieldKind::Value => {}
            DocumentedFieldKind::Table { docs: child } => {
                write_table_field(&mut section, path, table, field, child)?;
            }
            DocumentedFieldKind::OptionalTable {
                docs: child,
                sample,
            } => write_optional_table_field(&mut section, path, table, field, child, sample)?,
            DocumentedFieldKind::ArrayOfTables {
                docs: child,
                sample,
            } => write_array_table_field(&mut section, path, table, field, child, sample)?,
        }

        if !section.trim().is_empty() {
            sections.push((false, section));
        }
    }

    for (index, (is_value, section)) in sections.iter().enumerate() {
        if index > 0 {
            let previous_is_value = sections[index - 1].0;
            if *is_value && previous_is_value {
                // Keep scalar fields tightly grouped for readability.
            } else {
                output.push('\n');
            }
        }
        output.push_str(section);
    }

    Ok(())
}

fn write_value_field(output: &mut String, table: &Table, field: DocumentedField) {
    write_doc_lines(output, field.docs);

    if let Some(value) = table.get(field.name) {
        output.push_str(field.name);
        output.push_str(" = ");
        output.push_str(&value.to_string());
        output.push('\n');
    }
}

fn write_table_field(
    output: &mut String,
    path: &[&str],
    table: &Table,
    field: DocumentedField,
    child: fn() -> DocumentedStruct,
) -> anyhow::Result<()> {
    write_doc_lines(output, field.docs);

    let Some(value) = table.get(field.name).and_then(Value::as_table) else {
        return Ok(());
    };

    let next_path = extend_path(path, field.name);
    write_table_header(output, &next_path, child().docs);
    write_table_fields(output, &next_path, value, child())
}

fn write_optional_table_field(
    output: &mut String,
    path: &[&str],
    table: &Table,
    field: DocumentedField,
    child: fn() -> DocumentedStruct,
    sample: fn() -> Value,
) -> anyhow::Result<()> {
    write_doc_lines(output, field.docs);

    let next_path = extend_path(path, field.name);

    if let Some(value) = table.get(field.name).and_then(Value::as_table) {
        write_table_header(output, &next_path, child().docs);
        return write_table_fields(output, &next_path, value, child());
    }

    output.push_str(
        "# Optional override. Uncomment this section to replace the shared/default settings.\n",
    );

    for line in render_commented_table(&next_path, child(), sample())?.lines() {
        output.push_str("# ");
        output.push_str(line);
        output.push('\n');
    }

    Ok(())
}

fn write_array_table_field(
    output: &mut String,
    path: &[&str],
    table: &Table,
    field: DocumentedField,
    child: fn() -> DocumentedStruct,
    sample: fn() -> Value,
) -> anyhow::Result<()> {
    write_doc_lines(output, field.docs);

    let next_path = extend_path(path, field.name);

    if let Some(values) = table.get(field.name).and_then(Value::as_array) {
        if values.is_empty() {
            for line in render_commented_array_table(&next_path, child(), sample())?.lines() {
                output.push_str("# ");
                output.push_str(line);
                output.push('\n');
            }

            return Ok(());
        }

        for (index, value) in values.iter().enumerate() {
            let Some(table) = value.as_table() else {
                continue;
            };

            if index > 0 {
                output.push('\n');
            }

            write_array_table(output, &next_path, table, child())?;
        }

        return Ok(());
    }

    for line in render_commented_array_table(&next_path, child(), sample())?.lines() {
        output.push_str("# ");
        output.push_str(line);
        output.push('\n');
    }

    Ok(())
}

fn write_table_header(output: &mut String, path: &[&str], docs: &'static [&'static str]) {
    write_doc_lines(output, docs);
    output.push('[');
    output.push_str(&path.join("."));
    output.push_str("]\n");
}

fn write_array_table(
    output: &mut String,
    path: &[&str],
    table: &Table,
    docs: DocumentedStruct,
) -> anyhow::Result<()> {
    write_doc_lines(output, docs.docs);
    output.push_str("[[");
    output.push_str(&path.join("."));
    output.push_str("]]\n");
    write_table_fields(output, path, table, docs)
}

fn render_commented_table(
    path: &[&str],
    docs: DocumentedStruct,
    sample: Value,
) -> anyhow::Result<String> {
    let table = sample
        .as_table()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("documented TOML table samples must serialize as tables"))?;

    let mut output = String::new();
    write_table_header(&mut output, path, docs.docs);
    write_table_fields(&mut output, path, &table, docs)?;
    Ok(output.trim_end().to_string())
}

fn render_commented_array_table(
    path: &[&str],
    docs: DocumentedStruct,
    sample: Value,
) -> anyhow::Result<String> {
    let table = sample.as_table().cloned().ok_or_else(|| {
        anyhow::anyhow!("documented TOML array-table samples must serialize as tables")
    })?;

    let mut output = String::new();
    write_array_table(&mut output, path, &table, docs)?;
    Ok(output.trim_end().to_string())
}

fn write_doc_lines(output: &mut String, docs: &'static [&'static str]) {
    for line in docs
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
    {
        output.push_str("# ");
        output.push_str(line);
        output.push('\n');
    }
}

fn extend_path<'a>(path: &[&'a str], field_name: &'a str) -> Vec<&'a str> {
    let mut next_path = path.to_vec();
    next_path.push(field_name);
    next_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Default, Serialize)]
    struct ChildSettings {
        enabled: bool,
    }

    impl DocumentedToml for ChildSettings {
        fn documented_toml() -> DocumentedStruct {
            DocumentedStruct {
                docs: &["Child section."],
                fields: &[DocumentedField {
                    name: "enabled",
                    docs: &["Whether the child section is enabled."],
                    kind: DocumentedFieldKind::Value,
                }],
            }
        }
    }

    #[derive(Default, Serialize)]
    struct RootSettings {
        value: usize,
        child: Option<ChildSettings>,
        rules: Vec<ChildSettings>,
    }

    impl DocumentedToml for RootSettings {
        fn documented_toml() -> DocumentedStruct {
            DocumentedStruct {
                docs: &["Root section."],
                fields: &[
                    DocumentedField {
                        name: "value",
                        docs: &["Top-level value."],
                        kind: DocumentedFieldKind::Value,
                    },
                    DocumentedField {
                        name: "child",
                        docs: &["Optional child section."],
                        kind: DocumentedFieldKind::OptionalTable {
                            docs: ChildSettings::documented_toml,
                            sample: ChildSettings::default_toml_value,
                        },
                    },
                    DocumentedField {
                        name: "rules",
                        docs: &["Dynamic rules."],
                        kind: DocumentedFieldKind::ArrayOfTables {
                            docs: ChildSettings::documented_toml,
                            sample: ChildSettings::default_toml_value,
                        },
                    },
                ],
            }
        }
    }

    #[test]
    fn should_render_documented_toml_with_commented_optional_sections() {
        let rendered = write_documented_toml(&RootSettings {
            value: 7,
            child: None,
            rules: Vec::new(),
        })
        .expect("documented TOML should render");

        assert!(rendered.contains("# Root section."));
        assert!(rendered.contains("value = 7"));
        assert!(rendered.contains("# [child]"));
        assert!(rendered.contains("# [[rules]]"));
    }
}
