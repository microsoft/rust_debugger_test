use regex::Regex;

enum OutputParsingStyle {
    LiteralMatch(String),
    PatternMatch(Regex),
}

const PATTERN_PREFIX: &str = "pattern:";

/// Parse the output of a debugger and verify that the expected contents
/// are found. If content was expected in the debugger output that is not
/// found, stop and return an error.
pub fn parse(debugger_output: String, expected_contents: Vec<&str>) -> anyhow::Result<()> {
    // If there are no check statements, return early.
    if expected_contents.len() == 0 {
        log::info!("No expected contents found.");
        return anyhow::Ok(());
    }

    // Trim whitespace at the beginning and end of output lines.
    let debugger_output_lines = debugger_output
        .trim()
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<&str>>();

    // Trim whitespace at the beginning and end of expected contents.
    let expected_contents = expected_contents
        .iter()
        .filter_map(|line| {
            let str = line.trim();
            match str.is_empty() {
                false => Some(str),
                true => None,
            }
        })
        .map(|line| line.trim())
        .collect::<Vec<&str>>();

    let mut index = 0;

    for expected in expected_contents {
        let parsing_style = get_output_parsing_style(expected)?;
        loop {
            if index >= debugger_output_lines.len() {
                let error_msg = format_error_message(&parsing_style);
                anyhow::bail!(
                    "Unable to find expected content in the debugger output. {}",
                    error_msg
                );
            }

            let debugger_output_line = debugger_output_lines[index];
            index += 1;

            // Search for the expected line or pattern within the current debugger output line.
            match &parsing_style {
                OutputParsingStyle::LiteralMatch(literal_str) => {
                    let str = literal_str.as_str();
                    if debugger_output_line.contains(&str) {
                        log::info!(
                            "Expected content found: `{}` at line `{}`",
                            str,
                            debugger_output_line
                        );
                        break;
                    }
                }
                OutputParsingStyle::PatternMatch(re) => {
                    if re.is_match(&debugger_output_line) {
                        log::info!("Expected pattern found: `{}`", debugger_output_line);
                        break;
                    }
                }
            }
        }
    }

    anyhow::Ok(())
}

fn format_error_message(parsing_style: &OutputParsingStyle) -> String {
    match parsing_style {
        OutputParsingStyle::LiteralMatch(literal_string) => {
            format!("Missing line: `{}`", literal_string)
        }
        OutputParsingStyle::PatternMatch(pattern) => {
            format!("Found 0 matches for pattern: `{}`", pattern.to_string())
        }
    }
}

/// Get the parsing style for the given expected statement.
fn get_output_parsing_style(expected_output: &str) -> anyhow::Result<OutputParsingStyle> {
    let parsing_style = if expected_output.starts_with(PATTERN_PREFIX) {
        let re_pattern = expected_output
            .strip_prefix(PATTERN_PREFIX)
            .expect("string starts with `pattern:`");
        let re = match Regex::new(re_pattern) {
            Ok(re) => re,
            Err(error) => anyhow::bail!("Invalid regex pattern: {}\n{}", re_pattern, error),
        };

        OutputParsingStyle::PatternMatch(re)
    } else {
        OutputParsingStyle::LiteralMatch(String::from(expected_output))
    };

    Ok(parsing_style)
}
