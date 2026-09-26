//! Переносимая часть appserver/script/script.cpp: Check, Count и ComputeVar.
//! GameServer/gameserver.exe + GameServer/GameServer.pdb; RVA 0x26780, 0x255e0, 0x26d20.
//! Идентификаторы пары (SHA-256): `server/rust/src/manifest/_gameserver_export_manifest.toml`.
//! Для GetArrayNum вычисляется только контекстно-свободная арифметика.

use super::ini::decimal_i32;

/// Вычисляет без состояния поддержанное подмножество выражений размера массива.
pub(super) fn evaluate_array_length(expression: &[u8]) -> Option<i32> {
    evaluate(expression)
}

fn evaluate(expression: &[u8]) -> Option<i32> {
    let expression = trim_space_tab(expression);
    if expression.is_empty() {
        return None;
    }

    if let Some(inner) = strip_outer_parentheses(expression) {
        return evaluate(inner);
    }

    let mut depth = 0usize;
    let mut weakest_operator: Option<(&'static [u8], u8, usize, usize)> = None;
    let mut offset = 0usize;
    while offset < expression.len() {
        match expression[offset] {
            b'(' => depth = depth.checked_add(1)?,
            b')' => {
                depth = depth.checked_sub(1)?;
            }
            _ if depth == 0 => {
                if let Some((operator, precedence)) = operator_at(expression, offset) {
                    // Count сворачивает первый оператор на текущем старшем уровне;
                    // правое разбиение сохраняет порядок слева направо.
                    let replace = weakest_operator.is_none_or(
                        |(_, current_precedence, current_offset, _)| {
                            precedence < current_precedence
                                || (precedence == current_precedence && offset > current_offset)
                        },
                    );
                    if replace {
                        weakest_operator = Some((operator, precedence, offset, operator.len()));
                    }
                    offset += operator.len();
                    continue;
                }
            }
            _ => {}
        }
        offset += 1;
    }
    if depth != 0 {
        return None;
    }

    if let Some((operator, _, position, length)) = weakest_operator {
        let left = evaluate(&expression[..position])?;
        let right = evaluate(&expression[position + length..])?;
        return match operator {
            b"+" => Some(left.wrapping_add(right)),
            b"-" => Some(left.wrapping_sub(right)),
            b"*" => Some(left.wrapping_mul(right)),
            b"/" => left.checked_div(right),
            b"%" => left.checked_rem(right),
            _ => None,
        };
    }

    expression
        .iter()
        .all(u8::is_ascii_digit)
        .then(|| decimal_i32(expression))
}

fn trim_space_tab(mut expression: &[u8]) -> &[u8] {
    while matches!(expression.first(), Some(b' ' | b'\t')) {
        expression = &expression[1..];
    }
    while matches!(expression.last(), Some(b' ' | b'\t')) {
        expression = &expression[..expression.len() - 1];
    }
    expression
}

fn strip_outer_parentheses(expression: &[u8]) -> Option<&[u8]> {
    if expression.first() != Some(&b'(') || expression.last() != Some(&b')') {
        return None;
    }
    let mut depth = 0usize;
    for (offset, byte) in expression.iter().enumerate() {
        match byte {
            b'(' => depth = depth.checked_add(1)?,
            b')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 && offset + 1 != expression.len() {
                    return None;
                }
            }
            _ => {}
        }
    }
    (depth == 0).then_some(&expression[1..expression.len() - 1])
}

fn operator_at(expression: &[u8], offset: usize) -> Option<(&'static [u8], u8)> {
    const OPERATORS: &[(&[u8], u8)] = &[(b"+", 7), (b"-", 7), (b"*", 8), (b"/", 8), (b"%", 8)];
    OPERATORS
        .iter()
        .find(|(operator, _)| expression[offset..].starts_with(operator))
        .map(|(operator, precedence)| (*operator, *precedence))
}
