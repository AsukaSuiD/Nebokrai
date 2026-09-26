//! Общие технические helpers перенесены в Shared runtime.
//! Здесь реэкспорт для переходных потребителей всех направлений.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_shared::runtime::{
    add_game_error_log_text, add_game_log_text, get_line_direction, ini_decode, put_debug_string,
    put_string_to_file,
};
