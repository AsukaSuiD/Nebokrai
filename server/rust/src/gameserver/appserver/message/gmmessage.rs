//! Владелец GM-сообщений GameServer.
//!
//! Точная пара GameServer EXE/PDB и файл-владелец
//! `server/gameserver/appserver/message/gmmessage.cpp` подтверждают
//! ветви `0x7FC01..0x7FC06`, `0x7FC08..0x7FC13`: ID запросившего
//! игрока читается до выбора ветви, длительность молчания нормализуется
//! к минимуму `1`, а карта игроков обходится дважды в знаковом порядке ID.
//! Ответы `0x5FF0D/0x5FF10`
//! уходят WorldServer. Адресный `0x7FC0F` сохраняет проверки длины,
//! дописывает исходный ` By Game Server {local IP}` и посылает игроку
//! `0xBF806`; обрабатываемая ошибка выделения памяти возвращает `GS0029`.
//! Рассылка `0x7FC0D` сохраняет проверку режима и публикует исходный
//! `0xBF806` либо `0xBF804` через общий `SendAll`.
//! Ответ запросившему игроку `0x7FC0C` выбирает `GS0033/GS0034` и форматирует
//! подтверждённые EXE-вызовом и поставляемой языковой таблицей аргументы
//! `%s/%d/%s`. Безопасный `Vec` заменяет исходный `char[512]`; неизвестный
//! спецификатор формата не воспроизводит неопределённое поведение переменных
//! аргументов и буфера, а остаётся типизированной границей.
//! Рассылка стране `0x7FC13` сохраняет сравнение `unsigned long` с байтом,
//! знаковый порядок ID игроков и отдельный `SendToPlayer` для каждого адресата.
//! Массовое отключение `0x7FC09` оставляет запросившего игрока и ставит
//! `QuitClientByMapID` всем остальным каноническим ID в исходном проходе карты.
//! Отключение региона `0x7FC0A` обходит физическое построчное хранилище областей, сохраняет
//! порядок и повторы `FindShapes(400)`, оставляет запросившего игрока и ставит тот же
//! `QuitClientByMapID` каждому найденному ID игрока. Принадлежащий владельцу
//! `Vec` снимает общий снимок ID перед проходом очереди; это не меняет наблюдаемый порядок, потому что
//! `KickPlayer` только ставит сетевую команду и не меняет реестр региона.
//! Отключение по имени `0x7FC06` сохраняет 24-байтовую границу `GetStr`,
//! выполняет отключение до точного ответа World `0x5FF09` и возвращает
//! исходное имя при обоих исходах. Отключение вокруг `0x7FC07` сохраняет
//! 256-байтовое имя, поле контекста, проход 7×7 с внешним циклом по X,
//! устранение только последовательных повторов, упорядоченные отключения и World
//! `0x5FD02(context,requester,-1,0,GS0030(count))` после побочных эффектов.
//! Ответ о присутствии `0x7FC08` использует ветвь со знаковым `char`, `GS0025/GS0026`
//! с подтверждённым `%s` и адресный `0xBF806(-1,0,text)`.
//! Входящий ответ списка `0x7FC12` сохраняет проверку запросившего игрока,
//! пропускает маршрутный ID карты и публикует каждую строку отдельным
//! адресным `0xBF806`.
//! Запрос `0x7FC11` обходит каноническую карту GM, оставляет только игроков в сети,
//! форматирует пять подтверждённых шаблонов уровней, меняет ID того же пакета на
//! `0x5FF15`, дописывает число и строки и возвращает его WorldServer.
//! Семейство продолжения сценария читает `(value, script ID)` после ID
//! запросившего игрока и продолжает принадлежащий серверу `CScript`; только
//! `0x7FC01` подавляет вызов при отсутствующем игроке, остальные три сохраняют
//! исходный факт пустого владельца.
//! Межсерверная ветвь `0x7FC03` читает ID целевого игрока, свойство с точной
//! границей `0x20`, исходную карту и ID сценария, вычисляет значение через того же
//! владельца `CPlayer` и возвращает WorldServer пакет `0x5FF03`; WorldServer
//! маршрутизирует результат как `0x7FC02` исходному продолжению сценария.
//! Удалённый перенос `0x7FC10` сохраняет чтение запросившего игрока до выбора
//! ветви, ищет цель по имени и только для локального владельца читает регион/X/Y и вызывает полный
//! `ChangeRegion` с текущим направлением и нулевыми use/range/carriage.
//! Эти цепочки имеют статус `IMPLEMENTED`.
//! Ответ списка блокировок `0x7FC14` строго проверяет всю полезную нагрузку,
//! продолжает только ожидающую `ListBanedPlayer 5106` и после этого публикует
//! строки игроку отдельными `0xBF806`.
//! `Vec` заменяет исходное выделение памяти; поле объявленной ёмкости сохраняет
//! исходные `sum(name_len + 2) + 0x40`, включая возможное
//! расхождение между двумя зависящими от времени проходами. Все селекторы
//! `0x7FC01..14` связаны с typed dispatcher-ом.
//! `ParseGMCommand` сохраняет двухуровневую авторизацию, legacy-разбор четырёх
//! параметров, upsert сценарных переменных и отложенный запуск
//! `scripts/gm/{command}.script` через канонический script owner.
//! Для `0x7FC0F` длина `-64` даёт нулевое выделение: exact `GetStr` всё равно
//! потребляет один байт, после чего нативный `strcat` мог читать за буфером.
//! Rust сохраняет cursor, нормализует UB-префикс к пустому и публикует suffix/IP.

use crate::gameserver::appserver::script::function::ScriptFunctionRuntime;
use crate::gameserver::appserver::script::script::{
    ScriptExecutionContext, legacy_atoi,
};
use crate::gameserver::gameserver::game::{
    CGame, GameClockContext, GameKickAroundOutcome, ScriptRegionChangeContext,
    colored_text_message,
};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;
use tracing::trace;

const GM_LIST_RESPONSE_MESSAGE: i32 = 0x0007_FC12;
const GM_LIST_WORLD_RESPONSE_MESSAGE: i32 = 0x0005_FF15;
const GM_SCRIPT_CONTINUE_IF_PRESENT_MESSAGE: i32 = 0x0007_FC01;
const GM_SCRIPT_CONTINUE_MESSAGE_1: i32 = 0x0007_FC02;
const GM_GET_PLAYER_PROPERTY_REQUEST: i32 = 0x0007_FC03;
const GM_SCRIPT_CONTINUE_MESSAGE_2: i32 = 0x0007_FC04;
const GM_SCRIPT_CONTINUE_MESSAGE_3: i32 = 0x0007_FC05;
const GM_KICK_BY_NAME_MESSAGE: i32 = 0x0007_FC06;
const GM_KICK_AROUND_MESSAGE: i32 = 0x0007_FC07;
const GM_PRESENCE_FEEDBACK_MESSAGE: i32 = 0x0007_FC08;
const GM_KICK_OTHERS_MESSAGE: i32 = 0x0007_FC09;
const GM_KICK_REGION_MESSAGE: i32 = 0x0007_FC0A;
const GM_SET_SILENCE_MESSAGE: i32 = 0x0007_FC0B;
const GM_REQUESTER_FEEDBACK_MESSAGE: i32 = 0x0007_FC0C;
const GM_BROADCAST_MESSAGE: i32 = 0x0007_FC0D;
const GM_QUERY_SILENCE_MESSAGE: i32 = 0x0007_FC0E;
const GM_PRIVATE_NOTICE_MESSAGE: i32 = 0x0007_FC0F;
const GM_MOVE_PLAYER_MESSAGE: i32 = 0x0007_FC10;
const GM_LIST_REQUEST_MESSAGE: i32 = 0x0007_FC11;
const GM_COUNTRY_BROADCAST_MESSAGE: i32 = 0x0007_FC13;
const GM_ACTIVE_BAN_LIST_RESPONSE: i32 = 0x0007_FC14;
const GM_SET_SILENCE_RESPONSE: i32 = 0x0005_FF0D;
const GM_QUERY_SILENCE_RESPONSE: i32 = 0x0005_FF10;
const GM_KICK_BY_NAME_RESPONSE: i32 = 0x0005_FF09;
const GM_KICK_AROUND_RESPONSE: i32 = 0x0005_FD02;
const PLAYER_SYSTEM_MESSAGE: i32 = 0x000B_F806;
const PLAYER_ANNOUNCEMENT_MESSAGE: i32 = 0x000B_F804;
const GM_LEGACY_TEXT_LIMIT: usize = 0x100;
const GM_SHORT_NAME_LIMIT: usize = 0x18;
const GM_EMPTY_SILENCE_RESPONSE_LENGTH: u32 = 0x18;
const GM_SILENCE_RESPONSE_SLACK: usize = 0x40;
const GM_SILENCE_NAME_SEPARATOR: [u8; 2] = [0xA3, 0xBB];
const GM_PRIVATE_NOTICE_INVALID_LENGTH: i32 = 0x09FF_FFF9;
const GM_PRIVATE_NOTICE_SLACK: u32 = 0x40;
const GM_PRIVATE_NOTICE_SUFFIX: &[u8] = b" By Game Server ";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GmMessageError {
    MissingRequesterId,
    MissingTargetPlayerId,
    MissingPlayerProperty,
    MissingOriginMapId,
    MissingMoveRegionId,
    MissingMoveTileX,
    MissingMoveTileY,
    MissingScriptContinuationValue,
    MissingScriptContinuationId,
    MissingListReservedField,
    MissingListCount,
    MissingKickPlayerName,
    MissingKickAroundPlayerName,
    MissingKickAroundContext,
    MissingPresenceOutcome,
    MissingPresencePlayerName,
    MissingKickRegionId,
    MissingPlayerName,
    MissingDuration,
    MissingFeedbackPlayerName,
    MissingFeedbackValue,
    MissingFeedbackOutcome,
    MissingFeedbackText,
    UnsupportedFeedbackFormat,
    UnsupportedOnlineGmLevel { level: i32 },
    GmListCountOutsideLegacyRange { count: usize },
    MissingCountryBroadcastText,
    MissingCountry,
    MissingCountryBroadcastFirstField,
    MissingCountryBroadcastSecondField,
    InvalidActiveBanList,
    MissingBroadcastText,
    MissingBroadcastFirstField,
    MissingBroadcastSecondField,
    MissingBroadcastMode,
    MissingPrivateNoticeLength,
    DeclaredLengthOutsideLegacyRange { required: usize },
}

/// Материализует полный selector-owner `OnGMMessage`; `None` означает, что
/// сообщение принадлежит другому диспетчеру.
pub(crate) fn dispatch_gm_message<Runtime: GameClockContext + ScriptRegionChangeContext>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<(), GmMessageError>> {
    let message_type = message.message_type();
    if !matches!(
        message_type,
        GM_LIST_RESPONSE_MESSAGE
            | GM_SCRIPT_CONTINUE_IF_PRESENT_MESSAGE
            | GM_SCRIPT_CONTINUE_MESSAGE_1
            | GM_GET_PLAYER_PROPERTY_REQUEST
            | GM_SCRIPT_CONTINUE_MESSAGE_2
            | GM_SCRIPT_CONTINUE_MESSAGE_3
            | GM_KICK_BY_NAME_MESSAGE
            | GM_KICK_AROUND_MESSAGE
            | GM_PRESENCE_FEEDBACK_MESSAGE
            | GM_KICK_OTHERS_MESSAGE
            | GM_KICK_REGION_MESSAGE
            | GM_SET_SILENCE_MESSAGE
            | GM_REQUESTER_FEEDBACK_MESSAGE
            | GM_BROADCAST_MESSAGE
            | GM_QUERY_SILENCE_MESSAGE
            | GM_PRIVATE_NOTICE_MESSAGE
            | GM_MOVE_PLAYER_MESSAGE
            | GM_LIST_REQUEST_MESSAGE
            | GM_COUNTRY_BROADCAST_MESSAGE
            | GM_ACTIVE_BAN_LIST_RESPONSE
    ) {
        return None;
    }
    let Some(requester_id) = message.base_mut().get_long() else {
        return Some(Err(GmMessageError::MissingRequesterId));
    };

    if message_type == GM_ACTIVE_BAN_LIST_RESPONSE {
        let Some(script_id) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::InvalidActiveBanList));
        };
        let Some(success) = message.base_mut().get_byte().filter(|value| *value <= 1) else {
            return Some(Err(GmMessageError::InvalidActiveBanList));
        };
        let Some(total_count) = message.base_mut().get_long().filter(|value| *value >= 0) else {
            return Some(Err(GmMessageError::InvalidActiveBanList));
        };
        let Some(truncated) = message.base_mut().get_byte().filter(|value| *value <= 1) else {
            return Some(Err(GmMessageError::InvalidActiveBanList));
        };
        let Some(record_count) = message
            .base_mut()
            .get_long()
            .filter(|value| (0..=256).contains(value))
        else {
            return Some(Err(GmMessageError::InvalidActiveBanList));
        };
        let mut published_texts = Vec::with_capacity(record_count as usize);
        for _ in 0..record_count {
            let Some(account) = message.base_mut().get_str_bytes(33) else {
                return Some(Err(GmMessageError::InvalidActiveBanList));
            };
            let Some(ban_until) = message.base_mut().get_str_bytes(20) else {
                return Some(Err(GmMessageError::InvalidActiveBanList));
            };
            if account.is_empty() || account.len() > 32 || ban_until.len() != 19 {
                return Some(Err(GmMessageError::InvalidActiveBanList));
            }
            let mut text = account;
            text.extend_from_slice(b"  ");
            text.extend_from_slice(&ban_until);
            published_texts.push(text);
        }
        if !message.base_mut().unread_bytes().is_empty()
            || (success == 0 && (total_count != 0 || truncated != 0 || record_count != 0))
            || (success != 0
                && (total_count < record_count
                    || truncated != u8::from(total_count > record_count)))
        {
            return Some(Err(GmMessageError::InvalidActiveBanList));
        }
        if truncated != 0 {
            published_texts.push(
                format!("List truncated: shown {} of {}.", record_count, total_count).into_bytes(),
            );
        }
        let continued = game.continue_player_script_function(
            script_id,
            requester_id,
            5106,
            if success != 0 { total_count } else { -1 },
        );
        let published_count = published_texts.len();
        if continued && success != 0 && game.find_player(requester_id).is_some() {
            for text in &published_texts {
                let _delivery = colored_text_message(PLAYER_SYSTEM_MESSAGE, 0xffff_ffff, 0, text)
                    .send_to_player(game.net_server(), requester_id);
            }
        }
        trace!(requester_id, script_id, success = success != 0, total_count, continued, published_count, "Обработан список блокировок GM");
        return Some(Ok(()));
    }

    if message_type == GM_GET_PLAYER_PROPERTY_REQUEST {
        let Some(target_player_id) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingTargetPlayerId));
        };
        let Some(property) = message.base_mut().get_str_bytes(0x20) else {
            return Some(Err(GmMessageError::MissingPlayerProperty));
        };
        let Some(origin_map_id) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingOriginMapId));
        };
        let Some(script_id) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingScriptContinuationId));
        };
        let Some(target) = game.find_player(target_player_id) else {
            trace!(requester_id, target_player_id, "Целевой игрок для свойства GM не найден");
            return Some(Ok(()));
        };
        let value = target.script_value(&property).unwrap_or(0);
        let mut response = CMessage::new(0x0005_ff03);
        response.add_long(requester_id);
        response.add_long(value);
        response.add_long(origin_map_id);
        response.add_long(script_id);
        let delivery = response.send(game, false);
        trace!(requester_id, target_player_id, value, origin_map_id, script_id, ?delivery, "Возвращено свойство игрока для GM-сценария");
        return Some(Ok(()));
    }

    if message_type == GM_MOVE_PLAYER_MESSAGE {
        let Some(player_name) = message.base_mut().get_str_bytes(0x100) else {
            unreachable!("literal 0x100 исключает zero-capacity GetStr");
        };
        let Some(target_player_id) = game
            .find_player_by_name(&player_name)
            .map(|player| player.player_id())
        else {
            trace!(requester_id, "Целевой игрок для переноса GM не найден");
            return Some(Ok(()));
        };
        let Some(region_id) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingMoveRegionId));
        };
        let Some(tile_x) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingMoveTileX));
        };
        let Some(tile_y) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingMoveTileY));
        };
        let direction = game
            .find_player(target_player_id)
            .expect("GM move target проверен до ChangeRegion")
            .shape()
            .get_direction();
        let report = game.change_player_region(
            target_player_id,
            region_id,
            tile_x,
            tile_y,
            direction,
            0,
            0,
            0,
        );
        trace!(requester_id, target_player_id, region_id, tile_x, tile_y, ?report, "Выполнен перенос игрока командой GM");
        return Some(Ok(()));
    }

    if matches!(
        message_type,
        GM_SCRIPT_CONTINUE_IF_PRESENT_MESSAGE
            | GM_SCRIPT_CONTINUE_MESSAGE_1
            | GM_SCRIPT_CONTINUE_MESSAGE_2
            | GM_SCRIPT_CONTINUE_MESSAGE_3
    ) {
        let Some(value) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingScriptContinuationValue));
        };
        let Some(script_id) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingScriptContinuationId));
        };
        let player_present = game.find_player(requester_id).is_some();
        let requested = message_type != GM_SCRIPT_CONTINUE_IF_PRESENT_MESSAGE || player_present;
        if requested {
            let _ = game.continue_player_script(script_id, requester_id, value);
        }
        trace!(requester_id, script_id, value, player_present, requested, "Обработано продолжение GM-сценария");
        return Some(Ok(()));
    }

    if message_type == GM_LIST_REQUEST_MESSAGE {
        let entries = match collect_online_gm_entries(game) {
            Ok(entries) => entries,
            Err(error) => return Some(Err(error)),
        };
        let count = match i32::try_from(entries.len()) {
            Ok(count) => count,
            Err(_) => {
                return Some(Err(GmMessageError::GmListCountOutsideLegacyRange {
                    count: entries.len(),
                }));
            }
        };
        message
            .base_mut()
            .set_message_type(GM_LIST_WORLD_RESPONSE_MESSAGE);
        message.add_long(count);
        for text in &entries {
            add_legacy_c_string(message, text);
        }
        let delivery = message.send(game, false);
        trace!(requester_id, count, ?delivery, "Отправлен список GM в World");
        return Some(Ok(()));
    }

    if message_type == GM_LIST_RESPONSE_MESSAGE {
        let target_player_id = requester_id;
        if game.find_player(target_player_id).is_none() {
            trace!(requester_id, "Получатель списка GM не найден");
            return Some(Ok(()));
        }
        let Some(reserved_field) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingListReservedField));
        };
        let Some(declared_count) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingListCount));
        };
        let mut published_count = 0usize;
        for _ in 0..declared_count.max(0) {
            let text = message
                .base_mut()
                .get_str_bytes(GM_LEGACY_TEXT_LIMIT)
                .expect("ненулевой GM list text limit");
            let mut response = CMessage::new(PLAYER_SYSTEM_MESSAGE);
            response.add_long(-1);
            response.add_long(0);
            add_legacy_c_string(&mut response, &text);
            let _delivery = response.send_to_player(game.net_server(), target_player_id);
            published_count += 1;
        }
        trace!(requester_id, reserved_field, declared_count, published_count, "Опубликован список GM игроку");
        return Some(Ok(()));
    }

    if message_type == GM_KICK_BY_NAME_MESSAGE {
        let Some(player_name) = message.base_mut().get_str_bytes(GM_SHORT_NAME_LIMIT) else {
            return Some(Err(GmMessageError::MissingKickPlayerName));
        };
        let kick = game.kick_player_by_name(&player_name);
        let mut response = CMessage::new(GM_KICK_BY_NAME_RESPONSE);
        response.add_long(requester_id);
        response.add_byte(u8::from(kick));
        add_legacy_c_string(&mut response, &player_name);
        let delivery = response.send(game, false);
        trace!(requester_id, kicked = kick, ?delivery, "Обработано отключение игрока по имени");
        return Some(Ok(()));
    }

    if message_type == GM_KICK_AROUND_MESSAGE {
        let Some(player_name) = message.base_mut().get_str_bytes(GM_LEGACY_TEXT_LIMIT) else {
            return Some(Err(GmMessageError::MissingKickAroundPlayerName));
        };
        let Some(response_context) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingKickAroundContext));
        };
        let matched_players = match game.kick_players_around_name(&player_name) {
            GameKickAroundOutcome::Completed { matched_players } => matched_players,
            outcome => {
                trace!(requester_id, response_context, ?outcome, "Отключение игроков вокруг не выполнено");
                return Some(Ok(()));
            }
        };
        let formatted_text = match format_gm_template(
            game.get_string_by_id(b"GS0030"),
            &[GmFormatArgument::Signed(
                matched_players as i32
            )],
        ) {
            Ok(formatted) => formatted,
            Err(()) => return Some(Err(GmMessageError::UnsupportedFeedbackFormat)),
        };
        let mut response = CMessage::new(GM_KICK_AROUND_RESPONSE);
        response.add_long(response_context);
        response.add_long(requester_id);
        response.add_long(-1);
        response.add_long(0);
        add_legacy_c_string(&mut response, &formatted_text);
        let delivery = response.send(game, false);
        trace!(requester_id, response_context, matched_players, ?delivery, "Завершено отключение игроков вокруг");
        return Some(Ok(()));
    }

    if message_type == GM_PRESENCE_FEEDBACK_MESSAGE {
        let Some(outcome) = message.base_mut().get_char() else {
            return Some(Err(GmMessageError::MissingPresenceOutcome));
        };
        let Some(player_name) = message.base_mut().get_str_bytes(GM_SHORT_NAME_LIMIT) else {
            return Some(Err(GmMessageError::MissingPresencePlayerName));
        };
        let string_id = if outcome < 1 {
            b"GS0026".as_slice()
        } else {
            b"GS0025".as_slice()
        };
        let formatted_text = match format_gm_template(
            game.get_string_by_id(string_id),
            &[GmFormatArgument::Text(&player_name)],
        ) {
            Ok(formatted) => formatted,
            Err(()) => return Some(Err(GmMessageError::UnsupportedFeedbackFormat)),
        };
        let mut response = CMessage::new(PLAYER_SYSTEM_MESSAGE);
        response.add_long(-1);
        response.add_long(0);
        add_legacy_c_string(&mut response, &formatted_text);
        let delivery = response.send_to_player(game.net_server(), requester_id);
        trace!(requester_id, outcome, ?string_id, delivery, "Отправлен ответ о присутствии игрока");
        return Some(Ok(()));
    }

    if message_type == GM_KICK_OTHERS_MESSAGE {
        let kicked = game.kick_players_except(requester_id);
        trace!(requester_id, kicked, "Поставлено массовое отключение игроков");
        return Some(Ok(()));
    }

    if message_type == GM_KICK_REGION_MESSAGE {
        let Some(region_id) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingKickRegionId));
        };
        let region_found = game.find_region(region_id).is_some();
        let kicked = game.kick_players_in_region_except(region_id, requester_id);
        trace!(requester_id, region_id, region_found, kicked, "Поставлено отключение игроков региона");
        return Some(Ok(()));
    }

    if message_type == GM_SET_SILENCE_MESSAGE {
        let Some(player_name) = message.base_mut().get_str_bytes(GM_LEGACY_TEXT_LIMIT) else {
            return Some(Err(GmMessageError::MissingPlayerName));
        };
        let Some(minutes) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingDuration));
        };
        let minutes = minutes.max(1);
        let mut response = CMessage::new(GM_SET_SILENCE_RESPONSE);
        response.add_long(requester_id);
        add_legacy_c_string(&mut response, &player_name);
        response.add_long(minutes);
        let player_id =
            game.silence_player_by_name(&player_name, minutes, || runtime.now_milliseconds());
        let (success, text_id) = if player_id.is_some() {
            (1, b"GS0031".as_slice())
        } else {
            (0, b"GS0032".as_slice())
        };
        let localized = game.get_string_by_id(text_id).to_vec();
        response.add_byte(success);
        add_legacy_c_string(&mut response, &localized);
        let delivery = response.send(game, false);
        trace!(requester_id, minutes, ?player_id, ?delivery, "Обновлено молчание игрока");
        return Some(Ok(()));
    }

    if message_type == GM_REQUESTER_FEEDBACK_MESSAGE {
        let Some(player_name) = message.base_mut().get_str_bytes(GM_LEGACY_TEXT_LIMIT) else {
            return Some(Err(GmMessageError::MissingFeedbackPlayerName));
        };
        let Some(value) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingFeedbackValue));
        };
        let Some(outcome) = message.base_mut().get_char() else {
            return Some(Err(GmMessageError::MissingFeedbackOutcome));
        };
        let Some(detail) = message.base_mut().get_str_bytes(GM_LEGACY_TEXT_LIMIT) else {
            return Some(Err(GmMessageError::MissingFeedbackText));
        };
        let successful = outcome != 0;
        let string_id = if successful {
            b"GS0033".as_slice()
        } else {
            b"GS0034".as_slice()
        };
        let formatted_text = match format_gm_feedback(
            game.get_string_by_id(string_id),
            &player_name,
            value,
            &detail,
        ) {
            Ok(formatted) => formatted,
            Err(()) => return Some(Err(GmMessageError::UnsupportedFeedbackFormat)),
        };
        let mut response = CMessage::new(PLAYER_SYSTEM_MESSAGE);
        response.add_long(-1);
        response.add_long(0);
        add_legacy_c_string(&mut response, &formatted_text);
        let delivery = response.send_to_player(game.net_server(), requester_id);
        trace!(requester_id, value, successful, ?string_id, delivery, "Отправлен итог GM-команды игроку");
        return Some(Ok(()));
    }

    if message_type == GM_COUNTRY_BROADCAST_MESSAGE {
        let Some(text) = message.base_mut().get_str_bytes(GM_LEGACY_TEXT_LIMIT) else {
            return Some(Err(GmMessageError::MissingCountryBroadcastText));
        };
        let Some(country) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingCountry));
        };
        let Some(first_field) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingCountryBroadcastFirstField));
        };
        let Some(second_field) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingCountryBroadcastSecondField));
        };
        let mut response = CMessage::new(PLAYER_SYSTEM_MESSAGE);
        response.add_long(first_field);
        response.add_long(second_field);
        add_legacy_c_string(&mut response, &text);
        let player_ids = game.player_ids_in_country(country as u32);
        let recipient_count = player_ids.len();
        for player_id in player_ids {
            let _delivery = response.send_to_player(game.net_server(), player_id);
        }
        trace!(requester_id, country, first_field, second_field, recipient_count, "Отправлена рассылка GM стране");
        return Some(Ok(()));
    }

    if message_type == GM_PRIVATE_NOTICE_MESSAGE {
        let Some(declared_length) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingPrivateNoticeLength));
        };
        if matches!(declared_length, 0 | GM_PRIVATE_NOTICE_INVALID_LENGTH) {
            trace!(requester_id, declared_length, "Пустое адресное уведомление GM пропущено");
            return Some(Ok(()));
        }
        let buffer_length = (declared_length as u32).wrapping_add(GM_PRIVATE_NOTICE_SLACK) as usize;
        let mut published_text = Vec::new();
        let allocation_failed = published_text.try_reserve_exact(buffer_length).is_err();
        if allocation_failed {
            published_text.extend_from_slice(game.get_string_by_id(b"GS0029"));
        } else {
            published_text.extend_from_slice(
                &message
                    .base_mut()
                    .get_str_bytes(buffer_length)
                    .expect("ненулевой GM notice buffer"),
            );
            published_text.extend_from_slice(GM_PRIVATE_NOTICE_SUFFIX);
            published_text.extend_from_slice(game.net_server().local_ip());
        }
        let mut response = CMessage::new(PLAYER_SYSTEM_MESSAGE);
        response.add_long(-1);
        response.add_long(0);
        add_legacy_c_string(&mut response, &published_text);
        let delivery = response.send_to_player(game.net_server(), requester_id);
        trace!(requester_id, declared_length, allocation_failed, delivery, "Отправлено адресное уведомление GM");
        return Some(Ok(()));
    }

    if message_type == GM_BROADCAST_MESSAGE {
        let Some(text) = message.base_mut().get_str_bytes(GM_LEGACY_TEXT_LIMIT) else {
            return Some(Err(GmMessageError::MissingBroadcastText));
        };
        let Some(first_field) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingBroadcastFirstField));
        };
        let Some(second_field) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingBroadcastSecondField));
        };
        let Some(mode) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingBroadcastMode));
        };
        let mut response = match mode {
            0 => {
                let mut response = CMessage::new(PLAYER_SYSTEM_MESSAGE);
                response.add_long(first_field);
                response.add_long(second_field);
                response
            }
            1 => {
                let mut response = CMessage::new(PLAYER_ANNOUNCEMENT_MESSAGE);
                response.add_long(0);
                response.add_long(-1);
                response.add_long(1);
                response.add_long(1);
                response
            }
            _ => {
                trace!(requester_id, mode, "Неизвестный режим рассылки GM пропущен");
                return Some(Ok(()));
            }
        };
        add_legacy_c_string(&mut response, &text);
        let response_type = response.message_type();
        let delivery = response.send_all(game.current_net_server());
        trace!(requester_id, first_field, second_field, mode, response_type, ?delivery, "Выполнена рассылка GM");
        return Some(Ok(()));
    }

    let first_pass = game.silenced_player_names_pass(|| runtime.now_milliseconds());
    if first_pass.is_empty() {
        let localized = game.get_string_by_id(b"GS0028").to_vec();
        let mut response = CMessage::new(GM_QUERY_SILENCE_RESPONSE);
        response.add_long(requester_id);
        response.add_ulong(GM_EMPTY_SILENCE_RESPONSE_LENGTH);
        add_legacy_c_string(&mut response, &localized);
        let delivery = response.send(game, false);
        trace!(requester_id, declared_capacity = GM_EMPTY_SILENCE_RESPONSE_LENGTH, ?delivery, "Отправлен пустой список молчания");
        return Some(Ok(()));
    }

    let required = first_pass.iter().try_fold(0usize, |total, name| {
        name.len()
            .checked_add(GM_SILENCE_NAME_SEPARATOR.len())
            .and_then(|name_length| total.checked_add(name_length))
    });
    let Some(required) = required.and_then(|length| length.checked_add(GM_SILENCE_RESPONSE_SLACK))
    else {
        return Some(Err(GmMessageError::DeclaredLengthOutsideLegacyRange {
            required: usize::MAX,
        }));
    };
    let Ok(declared_capacity) = u32::try_from(required) else {
        return Some(Err(GmMessageError::DeclaredLengthOutsideLegacyRange {
            required,
        }));
    };

    let published_names = game.silenced_player_names_pass(|| runtime.now_milliseconds());
    let mut names = Vec::with_capacity(required - GM_SILENCE_RESPONSE_SLACK);
    for name in &published_names {
        names.extend_from_slice(name);
        names.extend_from_slice(&GM_SILENCE_NAME_SEPARATOR);
    }
    let mut response = CMessage::new(GM_QUERY_SILENCE_RESPONSE);
    response.add_long(requester_id);
    response.add_ulong(declared_capacity);
    add_legacy_c_string(&mut response, &names);
    let delivery = response.send(game, false);
    trace!(requester_id, first_pass_count = first_pass.len(), published_count = published_names.len(), declared_capacity, ?delivery, "Отправлен список молчания");
    Some(Ok(()))
}

/// Публикует локальную часть `5103 / ListOnlineGM` тому же игроку, который
/// затем запрашивает списки остальных GameServer через `0x5FF14`.
pub(crate) fn publish_local_online_gm_list(
    game: &CGame,
    player_id: i32,
) -> Result<(), GmMessageError> {
    let entries = collect_online_gm_entries(game)?;
    let published_count = entries.len();
    for text in entries {
        let mut response = CMessage::new(PLAYER_SYSTEM_MESSAGE);
        response.add_long(-1);
        response.add_long(0);
        add_legacy_c_string(&mut response, &text);
        let _delivery = response.send_to_player(game.net_server(), player_id);
    }
    trace!(player_id, published_count, "Опубликован локальный список GM");
    Ok(())
}

fn collect_online_gm_entries(game: &CGame) -> Result<Vec<Vec<u8>>, GmMessageError> {
    let mut entries = Vec::new();
    for info in game.gm_list().gm_info().values() {
        if game.find_player_by_name(&info.name).is_none() {
            continue;
        }
        let string_id = match info.level {
            100 => b"GS0035".as_slice(),
            90 => b"GS0036".as_slice(),
            50 => b"GS0037".as_slice(),
            40 => b"GSN0001".as_slice(),
            30 => b"GSN0002".as_slice(),
            level => return Err(GmMessageError::UnsupportedOnlineGmLevel { level }),
        };
        let text = format_gm_template(
            game.get_string_by_id(string_id),
            &[GmFormatArgument::Text(&info.name)],
        )
        .map_err(|()| GmMessageError::UnsupportedFeedbackFormat)?;
        entries.push(text);
    }
    Ok(entries)
}

fn add_legacy_c_string(message: &mut CMessage, value: &[u8]) {
    message.base_mut().add(legacy_c_string_prefix(value));
    message.add_byte(0);
}

fn format_gm_feedback(
    template: &[u8],
    player_name: &[u8],
    value: i32,
    detail: &[u8],
) -> Result<Vec<u8>, ()> {
    let arguments = [
        GmFormatArgument::Text(player_name),
        GmFormatArgument::Signed(value),
        GmFormatArgument::Text(detail),
    ];
    format_gm_template(template, &arguments)
}

enum GmFormatArgument<'a> {
    Text(&'a [u8]),
    Signed(i32),
}

fn format_gm_template(template: &[u8], arguments: &[GmFormatArgument<'_>]) -> Result<Vec<u8>, ()> {
    let template = legacy_c_string_prefix(template);
    let mut output = Vec::with_capacity(template.len());
    let mut argument_index = 0usize;
    let mut offset = 0usize;
    while offset < template.len() {
        if template[offset] != b'%' {
            output.push(template[offset]);
            offset += 1;
            continue;
        }
        let Some(specifier) = template.get(offset + 1).copied() else {
            return Err(());
        };
        if specifier == b'%' {
            output.push(b'%');
            offset += 2;
            continue;
        }
        let Some(argument) = arguments.get(argument_index) else {
            return Err(());
        };
        match (specifier, argument) {
            (b's', GmFormatArgument::Text(text)) => {
                output.extend_from_slice(legacy_c_string_prefix(text));
            }
            (b'd' | b'i', GmFormatArgument::Signed(value)) => {
                output.extend_from_slice(value.to_string().as_bytes());
            }
            (b'u', GmFormatArgument::Signed(value)) => {
                output.extend_from_slice((*value as u32).to_string().as_bytes());
            }
            _ => return Err(()),
        }
        argument_index += 1;
        offset += 2;
    }
    Ok(output)
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    &value[..value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len())]
}

/// Точный reached helper `ParseGMCommand`, вызываемый каналом чата `8`.
/// Возвращает `false` только для неавторизованного/запрещённого запроса или
/// traversal-команды; отсутствие script resource и duplicate уже принадлежат
/// отложенному `CGame::run_script_file` и не отменяют факт принятия команды.
pub(crate) fn parse_gm_command<Runtime: ScriptFunctionRuntime>(
    game: &mut CGame,
    runtime: &mut Runtime,
    player_id: i32,
    region_id: i32,
    command_line: &mut [u8],
) -> bool {
    let Some(player_name) = game
        .find_player(player_id)
        .map(|player| player.player_name().to_vec())
    else {
        return false;
    };
    let player_gm = game.gm_list().player_gm_info().get(&player_name);
    let gm = game.gm_list().gm_info().get(&player_name);
    let (gm_level, restricted) = match (gm, player_gm) {
        (Some(gm), _) => (gm.level, false),
        (None, Some(player_gm)) => (player_gm.level, true),
        (None, None) => return false,
    };

    if restricted {
        command_line.make_ascii_lowercase();
        if !command_line.starts_with(b"silence")
            && !command_line.starts_with(b"move")
            && !command_line.starts_with(b"teleplayer")
        {
            return false;
        }
    }

    let mut command = Vec::with_capacity(0x20);
    let mut parameters: [Vec<u8>; 4] = std::array::from_fn(|_| Vec::with_capacity(0x80));
    let mut field = 0usize;
    for &byte in command_line.iter() {
        if byte == b' ' {
            command.make_ascii_lowercase();
            if command == b"say" && field == 1 {
                if parameters[0].len() < 0x80 {
                    parameters[0].push(byte);
                }
                continue;
            }
            field = field.saturating_add(1);
            continue;
        }
        if field == 0 {
            if command.len() < 0x20 {
                command.push(byte);
            }
        } else if let Some(parameter) = parameters.get_mut(field - 1)
            && parameter.len() < 0x80
        {
            parameter.push(byte);
        }
    }
    command.make_ascii_lowercase();

    let integer_parameters = parameters.each_ref().map(|value| legacy_atoi(value));
    let Some(player) = game.find_player_mut(player_id) else {
        return false;
    };
    let _ = player.add_integer_variable(b"$GMLevel", gm_level);
    for (index, value) in parameters.iter().enumerate() {
        let name = [
            b"#GMParam1".as_slice(),
            b"#GMParam2".as_slice(),
            b"#GMParam3".as_slice(),
            b"#GMParam4".as_slice(),
        ][index];
        let _ = player.add_string_variable(name, value);
    }
    for (index, value) in integer_parameters.into_iter().enumerate() {
        let name = [
            b"$GMParam1".as_slice(),
            b"$GMParam2".as_slice(),
            b"$GMParam3".as_slice(),
            b"$GMParam4".as_slice(),
        ][index];
        let _ = player.add_integer_variable(name, value);
    }

    if command.windows(2).any(|window| window == b"..") {
        return false;
    }
    let mut path = Vec::with_capacity(b"scripts/gm/".len() + command.len() + b".script".len());
    path.extend_from_slice(b"scripts/gm/");
    path.extend_from_slice(&command);
    path.extend_from_slice(b".script");
    let _ = game.run_script_file(
        &path,
        ScriptExecutionContext {
            player_id: Some(player_id),
            region_id: Some(region_id),
            ..ScriptExecutionContext::default()
        },
        runtime,
    );
    true
}
