//! Диспетчер клиентского управления питомцами GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/petmessage.cpp`. `0x90401/02` проходят через владельцев
//! player, region и monster: режим AI, действие и цель, состояние отзыва,
//! локализованные уведомления, `0xC0202` и рассылка вокруг `0xBF504`
//! сохраняют исходный порядок. Выполненные отправки не дублируются в отчёте;
//! сведения о результате публикуются через `tracing` на месте.

use crate::gameserver::gameserver::game::{CGame, colored_player_notice_message};
use crate::nets::netserver::message::CMessage;
use tracing::{debug, trace};

const PET_MODE: u32 = 0x0009_0401;
const PET_COMMAND: u32 = 0x0009_0402;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePetMessageError {
    MissingField,
}

fn notify(game: &CGame, player_id: i32, string_id: &[u8]) {
    let _ = colored_player_notice_message(
        0xffff_0000,
        0xffff_ffff,
        game.get_string_by_id(string_id),
    )
    .send_to_player(game.net_server(), player_id);
}

pub(crate) fn dispatch_game_pet_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> Option<Result<(), GamePetMessageError>> {
    let source_type = message.message_type() as u32;
    if !matches!(source_type, PET_MODE | PET_COMMAND) {
        return None;
    }
    let player_id = message.map_id();
    let Some(player) = game.find_player(player_id) else {
        trace!(player_id, "pet-команда пропущена: игрок не найден");
        return Some(Ok(()));
    };
    if player.in_changing_server()
        || player.in_changing_region()
        || player
            .server_region_id()
            .and_then(|region_id| game.find_region(region_id))
            .is_none()
    {
        trace!(player_id, "pet-команда пропущена: контекст игрока меняется");
        return Some(Ok(()));
    }
    if source_type == PET_MODE {
        let Some(mode) = message.base_mut().get_long() else {
            return Some(Err(GamePetMessageError::MissingField));
        };
        if (0..=2).contains(&mode) {
            let Some(affected) = game.set_player_pet_mode(player_id, mode) else {
                return Some(Ok(()));
            };
            notify(
                game,
                player_id,
                match mode {
                    0 => b"GS0050",
                    1 => b"GS0051",
                    _ => b"GS0052",
                },
            );
            debug!(player_id, mode, affected_pets = affected, "изменён режим питомцев");
        }
        let mode = game
            .find_player(player_id)
            .map_or(0, |player| player.current_pets_mode());
        let mut response = CMessage::new(0x000c_0202);
        response.add_long(mode);
        let _ = response.send_to_player(game.net_server(), player_id);
        return Some(Ok(()));
    }

    let Some(command) = message.base_mut().get_long() else {
        return Some(Err(GamePetMessageError::MissingField));
    };
    match command {
        0 => {
            let Some(target_type) = message.base_mut().get_long() else {
                return Some(Err(GamePetMessageError::MissingField));
            };
            let Some(target_id) = message.base_mut().get_long() else {
                return Some(Err(GamePetMessageError::MissingField));
            };
            let Some(affected) = game.set_player_pets_target(player_id, target_type, target_id)
            else {
                return Some(Ok(()));
            };
            notify(game, player_id, b"GS0053");
            debug!(player_id, target_type, target_id, affected_pets = affected, "питомцам назначена цель");
        }
        1 | 2 => {
            let Some(affected) = game.set_player_pets_action(player_id, command) else {
                return Some(Ok(()));
            };
            notify(
                game,
                player_id,
                if command == 1 { b"GS0054" } else { b"GS0055" },
            );
            debug!(player_id, command, affected_pets = affected, "изменено действие питомцев");
        }
        3 => {
            let Some(pet_type) = message.base_mut().get_long() else {
                return Some(Err(GamePetMessageError::MissingField));
            };
            let Some(pet_id) = message.base_mut().get_long() else {
                return Some(Err(GamePetMessageError::MissingField));
            };
            let Some(_delivery) = game.dismiss_player_pet(player_id, pet_type, pet_id) else {
                return Some(Ok(()));
            };
            notify(game, player_id, b"GS0056");
            debug!(player_id, pet_type, pet_id, "питомец отозван");
        }
        _ => {}
    }
    Some(Ok(()))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\petmessage.cpp

// ============================================================================
// FUNCTION: OnPetMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\petmessage.cpp:14
// RVA: 0x000865D0
// ADDRESS: 004865d0
// PROTOTYPE: void __cdecl OnPetMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
