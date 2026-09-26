//! Владелец диспетчера сообщений региона GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/regionmessage.cpp`. Достигнутый `0x8F801` завершает
//! локальный `CPlayer::ChangeRegion`: проверяет действующий контекст игрока и
//! региона, снимает `m_bInChangingRegion`, переносит IP клиента, добавляет
//! игрока в пространственный реестр назначения, публикует снимки и погоду,
//! затем завершает состояния, спутников и WarSoul прямым владельцем.
//! `0x8F805` проверяет
//! прямое ребро канонического `RegionRouter`, диапазон перехода и
//! `CS_CHANGEREGION`, после чего входит в тот же полный владелец
//! `CPlayer::ChangeRegion` с игровыми, пространственными и локальными/World
//! wire-эффектами. Диагностический исход публикуется через `tracing` после
//! синхронного выполнения эффектов.
//! Локальный клиент `Miracle game.exe` (`Nebokrai.exe`, сборка по
//! `server/rust/src/manifest/_nebokrai_client_manifest.toml`) дополнительно пишет
//! перед аргументом `8F801` C-строку: VA `0x414CD0/0x414CDD`, вызов после `BF401`
//! в `0x54A9E0`. `VERIFIED_DISASSEMBLY`: это account — отправитель читает
//! `std::string` глобального app по `+0x80` (`0x414CA5..0x414CCB`), ту же строку
//! первым полем пишут оба сайта `0x2FD09` (`0x4E54FC..0x4E5517` и accessor
//! `0x4040D0` через `0x4E55E3`), а case `0x2FD09` парного `loginserver.exe`
//! VA `0x48038E` читает это поле как account через `GetStr(0x14)`;
//! машинная база — docs/reconstruction/client-wire-runtime.md. Game не использует
//! строку для выбора игрока: принимается строка с NUL и ровно четырьмя байтами
//! аргумента после неё.
//! Оригинальный Game читает один long в VA `0x5BB37B`; этот четырёхбайтовый
//! вариант также сохраняется.

use nebokrai_shared::protocol::LegacyReader;
use crate::gameserver::appserver::script::function::ScriptFunctionRuntime;
use crate::gameserver::appserver::shape::SHAPE_CHANGE_REGION;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;
use crate::setup::regionrouter::RegionRoutePoint;
use tracing::{debug, trace};

const ENTER_CHANGED_REGION: u32 = 0x0008_f801;
const CHANGE_CONNECTED_REGION: u32 = 0x0008_f805;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameRegionMessageError {
    InvalidPayloadSize { expected: usize, actual: usize },
    MissingEntryStringTerminator,
    MissingArgument,
}

fn read_region_argument(
    message: &mut CMessage,
    message_type: u32,
) -> Result<i32, GameRegionMessageError> {
    let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
    let mut reader = LegacyReader::at(source, *cursor)
        .map_err(|_| GameRegionMessageError::MissingArgument)?;
    let actual = reader.remaining();
    if message_type == ENTER_CHANGED_REGION && actual > 4 {
        // Не ищем NUL внутри завершающего i32: это отдельное поле.
        reader
            .read_c_string(actual - 4)
            .map_err(|_| GameRegionMessageError::MissingEntryStringTerminator)?;
    }
    if reader.remaining() != 4 {
        return Err(GameRegionMessageError::InvalidPayloadSize {
            expected: 4,
            actual: reader.remaining(),
        });
    }
    let argument = reader
        .read_i32()
        .map_err(|_| GameRegionMessageError::MissingArgument)?;
    *cursor = reader.position();
    Ok(argument)
}

pub(crate) fn dispatch_game_region_message<Context>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
) -> Option<Result<(), GameRegionMessageError>>
where
    Context: ScriptFunctionRuntime,
{
    let message_type = message.message_type() as u32;
    if !matches!(message_type, ENTER_CHANGED_REGION | CHANGE_CONNECTED_REGION) {
        return None;
    }
    let argument = match read_region_argument(message, message_type) {
        Ok(argument) => argument,
        Err(error) => return Some(Err(error)),
    };
    message.resolve_player_context(game);
    let player_id = message.player_id();
    let region_id = message.region_id();
    if message_type == ENTER_CHANGED_REGION {
        let applied = match (player_id, region_id) {
            (Some(player_id), Some(region_id)) => game.enter_changed_player_region(
                player_id,
                region_id,
                argument,
                message.ip(),
                message.socket_id(),
                context,
            ),
            _ => false,
        };
        debug!(?player_id, ?region_id, applied, "завершён вход игрока в сменённый регион");
    } else {
        let target_region_id = argument;
        if let Some(player_id) = player_id {
            let player_facts = game.find_player(player_id).map(|player| {
                (
                    player.server_region_id(),
                    player.shape().get_tile_x(),
                    player.shape().get_tile_y(),
                    player.shape().get_direction(),
                    player.shape().change_state(),
                )
            });
            if let Some((Some(source_region_id), x, y, direction, change_state)) = player_facts {
                match x.and_then(|x| y.map(|y| RegionRoutePoint { x, y })) {
                    Err(block) => {
                        trace!(player_id, source_region_id, target_region_id, ?block, "смена связанного региона отклонена координатами");
                    }
                    Ok(current) => {
                        let destination = game.region_router().connected_region_destination(
                            source_region_id,
                            target_region_id,
                            current,
                        );
                        if let Some(destination) = destination {
                            if change_state == SHAPE_CHANGE_REGION {
                                trace!(player_id, source_region_id, target_region_id, "игрок уже меняет регион");
                            } else {
                                let _ = game.change_player_region(
                                    player_id,
                                    target_region_id,
                                    destination.x,
                                    destination.y,
                                    direction,
                                    0,
                                    0,
                                    0,
                                );
                                debug!(player_id, source_region_id, target_region_id, destination_x = destination.x, destination_y = destination.y, "запущена смена связанного региона");
                            }
                        } else {
                            trace!(player_id, source_region_id, target_region_id, "между регионами нет прямого перехода");
                        }
                    }
                }
            }
        }
    }
    Some(Ok(()))
}

// IMPLEMENTED: `CServerRegion::OnMessage` opcodes `0x8F801/0x8F805`
// материализованы выше; покрытый RAW и технический STL/SEH шум удалены.
