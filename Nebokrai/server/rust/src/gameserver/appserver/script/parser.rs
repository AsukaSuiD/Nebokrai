//! Структурный byte-parser достигнутого языка `CScript`.
//!
//! `nom` отвечает за безопасное продвижение входного slice, распознавание
//! комментариев, command/token boundaries, labels и внешнюю форму вызова с
//! аргументами. Возвращаемые аргументы остаются заимствованными lazy tails:
//! evaluator вычисляет их позднее и в GameServer-specific порядке. Parser не
//! строит eager AST и не касается переменных, dispatcher-а либо side effects.

use nom::Parser;
use nom::bytes::complete::{tag, take, take_till, take_until, take_while1};
use nom::character::complete::multispace0;
use nom::error::{Error, ErrorKind};
use nom::sequence::terminated;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScriptParseError {
    pub(crate) offset: usize,
    pub(crate) kind: ErrorKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParsedCommand {
    pub(crate) bytes: Vec<u8>,
    pub(crate) next_point: usize,
}

pub(crate) fn next_command(
    source: &[u8],
    point: usize,
) -> Result<Option<ParsedCommand>, ScriptParseError> {
    let Some(mut input) = source.get(point..) else {
        return Err(ScriptParseError {
            offset: source.len(),
            kind: ErrorKind::Eof,
        });
    };
    loop {
        if input.is_empty() {
            return Ok(None);
        }
        if input.starts_with(b"//") {
            let (remaining, _) = line_comment(input).map_err(|error| map_error(source, error))?;
            input = remaining;
            continue;
        }
        if input.starts_with(b"/*") {
            match block_comment(input) {
                Ok((remaining, _)) => {
                    input = remaining;
                    continue;
                }
                Err(_) => return Ok(None),
            }
        }
        if is_command_start(input[0]) {
            break;
        }
        let (remaining, _) = take::<_, _, Error<&[u8]>>(1usize)
            .parse(input)
            .map_err(|error| map_error(source, error))?;
        input = remaining;
    }

    let mut quoted = false;
    let mut end = input.len();
    let mut delimiter = false;
    for (position, byte) in input.iter().copied().enumerate() {
        if byte == b'"' {
            quoted = !quoted;
        }
        if !quoted && matches!(byte, b';' | b'\t' | b'\n' | b'\r') {
            end = position;
            delimiter = true;
            break;
        }
    }
    let (remaining, command) = take::<_, _, Error<&[u8]>>(end)
        .parse(input)
        .map_err(|error| map_error(source, error))?;
    let remaining = if delimiter {
        take::<_, _, Error<&[u8]>>(1usize)
            .parse(remaining)
            .map_err(|error| map_error(source, error))?
            .0
    } else {
        remaining
    };
    Ok(Some(ParsedCommand {
        bytes: command.to_vec(),
        next_point: source.len() - remaining.len(),
    }))
}

pub(crate) fn command_name(command: &[u8]) -> Result<&[u8], ScriptParseError> {
    let (_, name) = take_while1::<_, _, Error<&[u8]>>(|byte: u8| {
        !byte.is_ascii_whitespace() && !matches!(byte, b'(' | b';')
    })
    .parse(command)
    .map_err(|error| map_error(command, error))?;
    Ok(name)
}

pub(crate) fn label(command: &[u8]) -> Option<&[u8]> {
    let mut parser = terminated(
        take_while1::<_, _, Error<&[u8]>>(|byte: u8| byte != b':'),
        tag(&b":"[..]),
    );
    let (remaining, name) = parser.parse(command).ok()?;
    remaining.is_empty().then_some(trim_ascii(name))
}

pub(crate) fn function(expression: &[u8]) -> Result<(&[u8], Vec<&[u8]>), ScriptParseError> {
    let expression = trim_ascii(expression);
    let (input, name) = take_while1::<_, _, Error<&[u8]>>(|byte: u8| {
        !byte.is_ascii_whitespace() && byte != b'('
    })
    .parse(expression)
    .map_err(|error| map_error(expression, error))?;
    let (input, _) = multispace0::<_, Error<&[u8]>>
        .parse(input)
        .map_err(|error| map_error(expression, error))?;
    let (input, _) = tag::<_, _, Error<&[u8]>>(&b"("[..])
        .parse(input)
        .map_err(|error| map_error(expression, error))?;

    let mut parameters = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    let mut quoted = false;
    for (position, byte) in input.iter().copied().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            b'(' if !quoted => depth += 1,
            b')' if !quoted && depth == 0 => {
                let parameter = trim_ascii(&input[start..position]);
                if !parameter.is_empty() {
                    parameters.push(parameter);
                }
                let _ = take::<_, _, Error<&[u8]>>(position + 1)
                    .parse(input)
                    .map_err(|error| map_error(expression, error))?;
                return Ok((trim_ascii(name), parameters));
            }
            b')' if !quoted => depth -= 1,
            b',' if !quoted && depth == 0 => {
                parameters.push(trim_ascii(&input[start..position]));
                start = position + 1;
            }
            _ => {}
        }
    }
    Err(ScriptParseError {
        offset: expression.len(),
        kind: ErrorKind::Eof,
    })
}

fn line_comment(input: &[u8]) -> nom::IResult<&[u8], (), Error<&[u8]>> {
    let (input, _) = tag(&b"//"[..]).parse(input)?;
    let (input, _) = take_till(|byte| matches!(byte, b'\n' | b'\r')).parse(input)?;
    Ok((input, ()))
}

fn block_comment(input: &[u8]) -> nom::IResult<&[u8], (), Error<&[u8]>> {
    let (input, _) = tag(&b"/*"[..]).parse(input)?;
    let (input, _) = take_until(&b"*/"[..]).parse(input)?;
    let (input, _) = tag(&b"*/"[..]).parse(input)?;
    Ok((input, ()))
}

fn map_error(source: &[u8], error: nom::Err<Error<&[u8]>>) -> ScriptParseError {
    let error = match error {
        nom::Err::Error(error) | nom::Err::Failure(error) => error,
        nom::Err::Incomplete(_) => Error::new(&source[source.len()..], ErrorKind::Eof),
    };
    ScriptParseError {
        offset: source.len().saturating_sub(error.input.len()),
        kind: error.code,
    }
}

fn trim_ascii(mut value: &[u8]) -> &[u8] {
    while value.first().is_some_and(u8::is_ascii_whitespace) {
        value = &value[1..];
    }
    while value.last().is_some_and(u8::is_ascii_whitespace) {
        value = &value[..value.len() - 1];
    }
    value
}

fn is_command_start(value: u8) -> bool {
    matches!(value, b'{' | b'}' | b'<' | b'>' | b'#' | b'$') || value.is_ascii_alphabetic()
}
