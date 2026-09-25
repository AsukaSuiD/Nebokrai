//! Login lifecycle `OnLogMessage` из `logmessage.cpp`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Переносится стартовой волной: `0x4FB03` restore-role. `OnLogMessage`
//! диспетчер в текущем файле восстанавливает слои напоряд **до wire-ответа
//! LoginServer**, потому что struct- payload хранит reference на очередное
//! взаимодействие.
//!
//! Остальные ветки `0x4FB01..0x4FB07` и `0x5FB01..0x5FB02` у владельца
//! старого пакета до следующей волны; удаление для хода не организовывается.

use crate::app::world_game_view::WorldGameView;
use crate::app::world_message::{CMessage, SendMessageError};

pub const RESTORE_ROLE_REQUEST: i32 = 0x0004_FB03;
pub const RESTORE_ROLE_RESPONSE: i32 = 0x0001_FF04;
pub const RESTORE_ROLE_STATUS: i8 = 0x15;

#[derive(Debug, Eq, PartialEq)]
pub struct WorldRestoreRoleOutcome {
    pub account: Vec<u8>,
    pub player_id: u32,
    pub player_id_complete: bool,
    pub response_type: i32,
    pub status: i8,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

/// Ветвь `0x4FB03` не берёт ни DB, ни cross-owner контактов; список
/// одобрений или `restore` и `deletion` обновляются до отправки, а
/// LoginServer получает restorer отмеченный на это роль. Перенос вербатим
/// без изменения формы payload.
pub fn on_restore_role(
    game: &mut dyn WorldGameView,
    mut message: CMessage,
) -> WorldRestoreRoleOutcome {
    let account = message
        .base_mut()
        .get_str_bytes(0x14)
        .expect("literal 0x14 исключает zero-capacity GetStr");
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0) as u32;

    game.delete_deletion_player(player_id);
    game.append_restore_player(player_id);

    let mut response = CMessage::new(RESTORE_ROLE_RESPONSE);
    response.base_mut().add_char(RESTORE_ROLE_STATUS);
    response.base_mut().add_ulong(player_id);
    response.base_mut().add(&account);
    response.base_mut().add_char(0);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    WorldRestoreRoleOutcome {
        account,
        player_id,
        player_id_complete: decoded_player_id.is_some(),
        response_type: RESTORE_ROLE_RESPONSE,
        status: RESTORE_ROLE_STATUS,
        wire,
        delivery,
    }
}
