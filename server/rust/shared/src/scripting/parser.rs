//! Структурный байтовый разбор команд, используемый Zone-исполнителем `CScript`.
//!
//! `nom` отвечает за безопасное продвижение входного среза, распознавание
//! комментариев, границ команд и токенов, меток и внешнюю форму вызова с
//! аргументами. Возвращаемые аргументы остаются заимствованными ленивыми
//! хвостами: вычислитель обрабатывает их позднее и в порядке GameServer. Модуль
//! `parser` не строит предварительное синтаксическое дерево и не касается
//! переменных, диспетчера или игровых эффектов.
//!
//! Пара GameServer: `gameserver.exe` SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E` и
//! `GameServer.pdb` SHA-256
//! `B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016`;
//! CodeView GUID `5bee6dd1-bf90-49b8-8be9-eb25c4038d53`, age `2`. В `RunStep`
//! (`0x00428d80`) `ReadCmd` предшествует `GetFunctionName`; машинный код последнего
//! (`0x00425000`) завершает имя на `(`, пробеле, TAB, LF, CR или `;`. Тело
//! `ReadCmd` и полнота грамматики остаются `PARTIAL`/`UNKNOWN`.
//! Исходный владелец PDB: `server/gameserver/appserver/script/script.cpp`.

use nom::Parser;
use nom::bytes::complete::{tag, take, take_till, take_until, take_while1};
use nom::character::complete::multispace0;
use nom::error::{Error, ErrorKind};
use nom::sequence::terminated;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScriptParseError {
    pub offset: usize,
    pub kind: ErrorKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedCommand {
    pub bytes: Vec<u8>,
    pub next_point: usize,
}

pub fn next_command(
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

pub fn command_name(command: &[u8]) -> Result<&[u8], ScriptParseError> {
    let (_, name) =
        take_while1::<_, _, Error<&[u8]>>(|byte: u8| !is_function_name_terminator(byte))
            .parse(command)
            .map_err(|error| map_error(command, error))?;
    Ok(name)
}

pub fn label(command: &[u8]) -> Option<&[u8]> {
    let mut parser = terminated(
        take_while1::<_, _, Error<&[u8]>>(|byte: u8| byte != b':'),
        tag(&b":"[..]),
    );
    let (remaining, name) = parser.parse(command).ok()?;
    remaining.is_empty().then_some(trim_ascii(name))
}

pub fn function(expression: &[u8]) -> Result<(&[u8], Vec<&[u8]>), ScriptParseError> {
    let expression = trim_ascii(expression);
    let (input, name) =
        take_while1::<_, _, Error<&[u8]>>(|byte: u8| !byte.is_ascii_whitespace() && byte != b'(')
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

fn is_function_name_terminator(value: u8) -> bool {
    matches!(value, b'(' | b' ' | b'\t' | b'\n' | b'\r' | b';')
}
