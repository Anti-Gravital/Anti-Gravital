//! Sequential line prompts: the degraded mode for terminals smaller than the
//! full-screen minimum.
//!
//! It asks the same questions as the wizard and builds the same
//! [`NewProjectRequest`]. Reader and writer are generic so the flow is tested
//! without a terminal.

use std::io::{self, BufRead, Write};

use super::text;
use crate::commands::project::validate_project_name;
use crate::request::{NewProjectRequest, Template};

/// Reads one trimmed line. `None` means end of input.
fn read_line<R: BufRead>(input: &mut R) -> io::Result<Option<String>> {
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim().to_owned()))
}

/// Asks for the missing inputs one line at a time. Returns `None` when the
/// user declines or input ends; nothing is written in that case.
pub(crate) fn prompt_new_project<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    name: Option<String>,
    template: Option<Template>,
    name_taken: fn(&str) -> bool,
) -> io::Result<Option<NewProjectRequest>> {
    let mut candidate = name;
    let name = loop {
        let current = match candidate.take() {
            Some(given) => given,
            None => {
                write!(output, "{}", text::LINE_NAME_PROMPT)?;
                output.flush()?;
                match read_line(input)? {
                    Some(line) => line,
                    None => return Ok(None),
                }
            }
        };
        let problem = match validate_project_name(&current) {
            Err(problem) => Some(problem),
            Ok(()) if name_taken(&current) => Some(text::NAME_TAKEN.to_owned()),
            Ok(()) => None,
        };
        match problem {
            Some(problem) => writeln!(output, "{}{problem}", text::ERROR_PREFIX)?,
            None => break current,
        }
    };

    let template = match template {
        Some(given) => given,
        None => {
            writeln!(output, "{}", text::TEMPLATE_LABEL)?;
            for (index, option) in Template::ALL.iter().enumerate() {
                writeln!(
                    output,
                    "  {}. {:<10} {}",
                    index + 1,
                    option.as_str(),
                    option.description()
                )?;
            }
            loop {
                write!(output, "{}", text::LINE_TEMPLATE_PROMPT)?;
                output.flush()?;
                let Some(line) = read_line(input)? else {
                    return Ok(None);
                };
                if line.is_empty() {
                    break Template::ALL[0];
                }
                let by_number = line
                    .parse::<usize>()
                    .ok()
                    .and_then(|n| n.checked_sub(1))
                    .and_then(|i| Template::ALL.get(i).copied());
                match by_number.or_else(|| Template::parse(&line)) {
                    Some(chosen) => break chosen,
                    None => writeln!(output, "{}", text::LINE_TEMPLATE_INVALID)?,
                }
            }
        }
    };

    let request = NewProjectRequest { name, template };
    writeln!(output, "{}: ./{}", text::REVIEW_DIRECTORY, request.name)?;
    writeln!(output, "{}: {}", text::REVIEW_TEMPLATE, request.template)?;
    for file in request.template.files() {
        writeln!(output, "  {}/{file}", request.name)?;
    }
    write!(output, "{}", text::LINE_CONFIRM_PROMPT)?;
    output.flush()?;
    match read_line(input)? {
        Some(answer) if answer.is_empty() || answer.eq_ignore_ascii_case("y") => Ok(Some(request)),
        Some(answer) if answer.eq_ignore_ascii_case("yes") => Ok(Some(request)),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn free(_: &str) -> bool {
        false
    }

    fn run(
        typed: &str,
        name: Option<&str>,
        template: Option<Template>,
    ) -> (Option<NewProjectRequest>, String) {
        let mut input = typed.as_bytes();
        let mut output = Vec::new();
        let result = prompt_new_project(
            &mut input,
            &mut output,
            name.map(str::to_owned),
            template,
            free,
        )
        .expect("io");
        (result, String::from_utf8(output).expect("utf8"))
    }

    #[test]
    fn answers_build_the_request() {
        let (request, _) = run("shop\n3\ny\n", None, None);
        assert_eq!(
            request,
            Some(NewProjectRequest {
                name: "shop".into(),
                template: Template::Fullstack,
            })
        );
    }

    #[test]
    fn empty_answers_take_the_defaults() {
        let (request, _) = run("api\n\n\n", None, None);
        assert_eq!(
            request,
            Some(NewProjectRequest {
                name: "api".into(),
                template: Template::Rest,
            })
        );
    }

    #[test]
    fn invalid_name_is_asked_again() {
        let (request, output) = run("bad name\ngood\n1\ny\n", None, None);
        assert_eq!(request.map(|r| r.name), Some("good".to_owned()));
        assert!(output.contains(text::ERROR_PREFIX));
    }

    #[test]
    fn template_is_accepted_by_name_and_rejected_when_unknown() {
        let (request, output) = run("9\nrealtime\ny\n", Some("x"), None);
        assert_eq!(request.map(|r| r.template), Some(Template::Realtime));
        assert!(output.contains(text::LINE_TEMPLATE_INVALID));
    }

    #[test]
    fn declining_or_end_of_input_creates_nothing() {
        assert_eq!(run("api\n1\nn\n", None, None).0, None);
        assert_eq!(run("", None, None).0, None);
        assert_eq!(run("api\n", None, None).0, None);
    }

    #[test]
    fn given_inputs_are_not_asked_again() {
        let (request, output) = run("y\n", Some("given"), Some(Template::Realtime));
        assert_eq!(
            request,
            Some(NewProjectRequest {
                name: "given".into(),
                template: Template::Realtime,
            })
        );
        assert!(!output.contains(text::LINE_NAME_PROMPT));
        assert!(!output.contains(text::LINE_TEMPLATE_PROMPT));
    }
}
