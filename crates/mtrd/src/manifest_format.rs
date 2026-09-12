/// Apply the manifest's compact canonical style to `serde_yaml` output.
pub(crate) fn canonicalize_yaml(yaml: String) -> String {
    inline_yaml_names(inline_yaml_positions(yaml))
}

fn inline_yaml_positions(yaml: String) -> String {
    let lines: Vec<_> = yaml.lines().collect();
    let mut output = String::with_capacity(yaml.len());
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        if line.trim() == "position:"
            && let (Some(x), Some(y)) = (lines.get(index + 1), lines.get(index + 2))
            && let (Some(x), Some(y)) = (x.trim().strip_prefix("- "), y.trim().strip_prefix("- "))
        {
            let indentation = &line[..line.len() - line.trim_start().len()];
            output.push_str(indentation);
            output.push_str("position: [");
            output.push_str(x);
            output.push_str(", ");
            output.push_str(y);
            output.push_str("]\n");
            index += 3;
            continue;
        }

        output.push_str(line);
        output.push('\n');
        index += 1;
    }

    output
}

fn inline_yaml_names(yaml: String) -> String {
    let lines: Vec<_> = yaml.lines().collect();
    let mut output = String::with_capacity(yaml.len());
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        output.push_str(line);
        output.push('\n');
        index += 1;

        if line.trim() != "names:" {
            continue;
        }

        let names_indentation = indentation(line);
        while index < lines.len() {
            let locale = lines[index];
            let locale_indentation = indentation(locale);
            if !locale.trim().is_empty() && locale_indentation <= names_indentation {
                break;
            }

            let Some(first_item) = lines.get(index + 1) else {
                break;
            };
            if locale_indentation != names_indentation + 2
                || !locale.trim_end().ends_with(':')
                || indentation(first_item) != locale_indentation
                || !is_sequence_item(first_item)
            {
                output.push_str(locale);
                output.push('\n');
                index += 1;
                continue;
            }

            let items_start = index + 1;
            let mut items_end = items_start;
            while let Some(item_line) = lines.get(items_end) {
                let item_indentation = indentation(item_line);
                if item_line.trim().is_empty()
                    || item_indentation > locale_indentation
                    || (item_indentation == locale_indentation && is_sequence_item(item_line))
                {
                    items_end += 1;
                } else {
                    break;
                }
            }

            let sequence = lines[items_start..items_end]
                .iter()
                .map(|item| item.get(locale_indentation..).unwrap_or(item))
                .collect::<Vec<_>>()
                .join("\n");
            let names = serde_yaml::from_str::<Vec<String>>(&sequence)
                .expect("serde_yaml emitted a valid localized-name sequence");
            let inline_names =
                inline_name_scalars(&lines[items_start..items_end], locale_indentation, &names);

            output.push_str(locale);
            output.push(' ');
            output.push('[');
            output.push_str(&inline_names.join(", "));
            output.push_str("]\n");
            index = items_end;
        }
    }

    output
}

fn inline_name_scalars(lines: &[&str], indentation: usize, names: &[String]) -> Vec<String> {
    let plain_scalars = (lines.len() == names.len()).then(|| {
        lines
            .iter()
            .map(|line| {
                line.get(indentation..)
                    .and_then(|line| line.strip_prefix("- "))
            })
            .collect::<Option<Vec<_>>>()
    });

    match plain_scalars.flatten() {
        Some(scalars) => scalars
            .into_iter()
            .zip(names)
            .map(|(scalar, name)| {
                if scalar.contains([',', '[', ']', '{', '}', ':']) {
                    serde_json::to_string(name).expect("a string is always valid JSON")
                } else {
                    scalar.to_owned()
                }
            })
            .collect(),
        None => names
            .iter()
            .map(|name| serde_json::to_string(name).expect("a string is always valid JSON"))
            .collect(),
    }
}

fn indentation(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn is_sequence_item(line: &str) -> bool {
    line.trim_start() == "-" || line.trim_start().starts_with("- ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inlines_positions_and_localized_names_only() {
        let yaml = "stations:\n- id: munich\n  names:\n    de:\n    - München\n    en:\n    - Munich\n    - 'yes'\n    fr:\n    - Greater Paris, France\n    zh-TW:\n    - 慕尼黑\n  position:\n  - 754.0\n  - 323.0\nlines:\n- red\n";

        assert_eq!(
            canonicalize_yaml(yaml.to_owned()),
            "stations:\n- id: munich\n  names:\n    de: [München]\n    en: [Munich, 'yes']\n    fr: [\"Greater Paris, France\"]\n    zh-TW: [慕尼黑]\n  position: [754.0, 323.0]\nlines:\n- red\n"
        );
    }
}
