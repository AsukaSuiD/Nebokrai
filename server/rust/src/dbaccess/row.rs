//! Чтение именованных полей существующей SQL-схемы через Tiberius.

use tiberius::{FromSql, Row};

/// Имена полей старой схемы различаются регистром с именами в DB-обработчиках.
/// Tiberius сравнивает их буквально, поэтому разрешаем ASCII-имя по метаданным
/// строки. Отсутствующее поле, NULL и несовместимый тип сохраняют обычный результат
/// `try_get`; значения и правила их преобразования остаются у вызывающего кода.
pub(crate) fn get_value<'a, T: FromSql<'a>>(
    row: &'a Row,
    column: &str,
) -> Result<Option<T>, tiberius::error::Error> {
    match row.columns().iter().position(|field| field.name().eq_ignore_ascii_case(column)) {
        Some(index) => row.try_get(index),
        None => row.try_get(column),
    }
}

/// Читает целочисленные типы SQL без сужения. Диапазон игрового поля проверяет
/// вызывающий код; этот же путь используется для abilities, предметов и JJC.
pub(crate) fn get_integer(
    row: &Row,
    column: &str,
) -> Result<Option<i64>, tiberius::error::Error> {
    let first_error = match get_value::<i32>(row, column) {
        Ok(value) => return Ok(value.map(i64::from)),
        Err(error) => error,
    };
    if let Ok(value) = get_value::<u8>(row, column) {
        return Ok(value.map(i64::from));
    }
    if let Ok(value) = get_value::<i16>(row, column) {
        return Ok(value.map(i64::from));
    }
    if let Ok(value) = get_value::<i64>(row, column) {
        return Ok(value);
    }
    Err(first_error)
}
