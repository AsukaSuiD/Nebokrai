//! Общие технические helpers перенесены в Shared runtime.
//! Здесь реэкспорт для переходных потребителей всех направлений.

pub(crate) use nebokrai_shared::runtime::{
    add_game_error_log_text, add_game_log_text, get_line_direction, ini_decode, put_debug_string,
    put_string_to_file,
};
