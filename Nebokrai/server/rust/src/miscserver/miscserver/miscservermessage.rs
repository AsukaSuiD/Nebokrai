//! Свободный handler `OnMSG_W2M_AUCTION` из `miscservermessage.cpp`.
//!
//! Статус владельца: `IMPLEMENTED` для всех доказанных ветвей
//! `0x0014ED01` и `0x0014ED04..0x0014ED09`.
//!
//! Точная пара: `MiscServer/miscserver.exe + MiscServer/miscserver.pdb`;
//! SHA-256 EXE
//! `F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65`,
//! SHA-256 PDB
//! `ED5F482DADB3E8B050B37F9911067479D297C5B6D33C1EA2CE99C9CD0FC11FA7`.
//! Исходный путь PDB:
//! `h:\fengyun\fy_russia\src\server\miscserver\miscserver\miscservermessage.cpp`.
//! `OnMSG_W2M_AUCTION` — RVA `0x000036B0`.
//!
//! `0x0014ED01` безусловно wrapping увеличивает `CGame::m_dwAddNewCount`,
//! создаёт один heap `CGoodsNode`, читает его из общего message buffer по
//! текущему `long&` cursor и независимо от результата добавления ничего не
//! отправляет. Готовый `CAuctionRoom::AddItemToAuctionRoom` сохраняет
//! invalid/duplicate отказ, удаление нового owner и `m_dwDelNewCount`.
//!
//! `0x0014ED04` читает signed player ID, затем GUID, и вызывает Misc
//! `PushItemToOptList` с operation `3`. Exact EXE доказал наблюдаемый дефект:
//! helper всегда возвращает `false`, даже когда записал buyer и вставил GUID в
//! opt-list. Поэтому handler всегда строит `0x0015EB06` с двумя 32-битными
//! полями `0` и исходным player ID и отправляет его без приоритета. Ширина
//! нулевого `Add` уже независимо подтверждена exact Misc call target RVA
//! `0x00010F20` в достигнутом `OnOtherMsg`; повторный reverse не выполнялся.
//! Короткий wire сохраняет поведение готовых безопасных getters: отсутствующий
//! long становится `0`, а отсутствующий GUID — `GUID_INVALID`, после чего
//! helper не мутирует комнату, но тот же ответ всё равно отправляется.
//!
//! `0x0014ED05` только при turn-local `m_dwDoneSysnCount == 0` читает map ID,
//! unsigned 32-битный count и столько GUID в ordered `CGUID -> bool(false)`.
//! Значение больше `10000` порождает прежний операторский warning, но не
//! ограничивает цикл. Пока `m_bDoneSyscMsg == false`, handler лишь проверяет
//! строгое wrapping-условие `m_dwStartTime + 120000 < timeGetTime`; даже при
//! срабатывании текущий пакет не синхронизируется. Уже разрешённая ветвь строит
//! доказанный batch `UnityGoods`, игнорирует результаты всех `Send` и только
//! после полного возврата helper-а ставит `m_dwDoneSysnCount = 1`.
//! Constructor не задавал этот count: безопасный вызов вне будущего
//! `GameThreadFunc` turn локально возвращает `BLOCKED_MISSING_FACT` до чтения
//! payload, а не придумывает ноль.
//!
//! `0x0014ED06` читает player ID как unsigned 32-битный шаблон, первым пишет
//! его в `0x0015EB07`, затем вызывает готовый self-list owner и выполняет один
//! неприоритетный `Send`. Signed owner-key получается тем же 32-битным
//! преобразованием, что и в старом вызове `uint -> int`. Safe-ошибка
//! `CGoodsNode::Serialize` локально блокирует ещё не отправленный response:
//! старый `void Serialize` не задаёт байты неинициализированных полей.
//!
//! `0x0014ED07` читает unsigned player ID и page-operation, сначала буквально
//! меняет сохранённую page через `ComputePlayerPage`, затем строит `0x0015EB08`
//! с player ID первым полем, дописывает временную страницу и выполняет один
//! неприоритетный `Send`. Отсутствующий player-search узел оставляет response
//! только с player ID. Любая safe page/filter/serialization граница блокирует
//! ещё не отправленное локальное сообщение, сохраняя уже выполненную page-
//! мутацию, но не назначая результат старого UB.
//!
//! `0x0014ED08` читает один GUID и ничего не делает для `GUID_INVALID`.
//! Живой GUID передаётся готовому Misc `PushItemToOptList` с operation `1` и
//! player ID `0`: этот operation записывает исходный owner ID в buyer перед
//! постановкой GUID в opt-list. Доказанный всегда-`false` return helper-а
//! игнорируется, ответ и `Send` отсутствуют. Короткий wire безопасно становится
//! `GUID_INVALID` по уже принятой замене `GetGUID` и потому тоже не мутирует
//! комнату; недоказанное старое чтение за границей не назначается.
//!
//! `0x0014ED09` создаёт `stPlayerOptNode` с page `0`, затем читает unsigned
//! player ID и пять signed полей low/up/use-self/money/weapon, после чего
//! принимает имя старым `GetStr(..., 0x100)`. Полный узел передаётся готовому
//! `ModifyPlayerSeachCondition`: новый player ID вставляется, существующий
//! получает полную замену значения. Ответ и `Send` отсутствуют. Короткие
//! numeric-поля сохраняют уже принятую legacy-замену нулём; строка остаётся
//! byte-exact в нулевом 256-байтовом буфере.
//!
//! `Box` заменяет `operator new/delete`, а общий wire-срез с `&mut usize` —
//! старые `unsigned char* + long&`; cursor начинается после 16-байтового
//! заголовка и сохраняет каждый успешный сдвиг. Nullable вход исключён:
//! `CMessage::Run` вызывает handler только с живым owned сообщением.
//!
//! Старый `UnSerialize` не имел длины и не возвращал ошибку. Если безопасная
//! граница встречает короткий buffer, переполненную строку либо невыделимый
//! goods-вектор, уже выполненные add-счётчик, `Clear`, присваивания и сдвиги
//! cursor сохраняются, а handler возвращает typed `BLOCKED_MISSING_FACT` до
//! вызова комнаты. Rust-owner освобождается обычным `Drop`, а доменный
//! `m_dwDelNewCount` не меняется, поскольку комната не достигнута. Реакция
//! исходного UB и недостающие байты не назначаются; это не объявляется
//! доказанным fail-closed поведением оригинала.
//!
//! Полное заменённое тело handler-а удалено. Два `$L` были его
//! compiler-generated destructor/unwind cleanup без отдельного наблюдаемого
//! эффекта и также удалены; Rust-массив и `PlayerOptNode` освобождаются `Drop`.

use std::collections::BTreeMap;

use crate::miscserver::miscserver::game::{CGame, legacy_tick_ms};
use crate::nets::netmisc::message::{CMessage, MessageSender, SendMessageError};
use crate::public::aucitionroom::{
    AddAuctionItemMissingGoodsType, AuctionPageBuildError, UnityGoodsBuild,
};
use crate::public::auctionnode::{CGoodsNode, GoodsNodeSerializeError, GoodsNodeUnserializeError};
use crate::public::auctionroom::PlayerOptNode;
use crate::public::guid::CGuid;

const ADD_AUCTION_ITEM: i32 = 0x0014_ED01;
const QUEUE_AUCTION_OPERATION: i32 = 0x0014_ED04;
const UNITY_AUCTION_GOODS: i32 = 0x0014_ED05;
const QUERY_SELF_AUCTION: i32 = 0x0014_ED06;
const QUERY_AUCTION_PAGE: i32 = 0x0014_ED07;
const QUEUE_OWNER_AUCTION_OPERATION: i32 = 0x0014_ED08;
const MODIFY_AUCTION_SEARCH: i32 = 0x0014_ED09;
const AUCTION_OPERATION_RESPONSE: i32 = 0x0015_EB06;
const SELF_AUCTION_RESPONSE: i32 = 0x0015_EB07;
const AUCTION_PAGE_RESPONSE: i32 = 0x0015_EB08;
const WORLD_REQUEST_OPERATION: u32 = 3;
const OWNER_REQUEST_OPERATION: u32 = 1;
const SYNC_WAIT_MILLISECONDS: u32 = 120_000;
const UNITY_COUNT_WARNING_THRESHOLD: u32 = 10_000;
const SEARCH_GOODS_NAME_BYTES: usize = 0x100;

/// Наблюдаемый либо локально заблокированный итог достигнутой ветви handler-а.
#[derive(Debug)]
pub(crate) enum WorldAuctionOutcome {
    /// Opcode ещё не принадлежит реализованной ветви этого владельца.
    Unhandled,
    /// Узел принят комнатой и проиндексирован.
    ItemAdded,
    /// Invalid либо duplicate GUID отклонён с исходным delete-счётчиком.
    ItemRejected,
    /// Безопасное чтение остановилось на недоказанной старой UB-границе.
    UnserializeBlocked(GoodsNodeUnserializeError),
    /// Успешное чтение неожиданно достигло неинициализированного type-поля.
    MissingGoodsType(AddAuctionItemMissingGoodsType),
    /// Теоретический положительный return helper-а подавил response.
    OperationWithoutResponse,
    /// Обязательный из-за exact-дефекта ответ создан и отправлен.
    OperationResponse { send: Result<i32, SendMessageError> },
    /// Handler вызван до обязательного обнуления count в game-thread turn.
    UnityTurnUninitialized,
    /// Sync уже был выполнен в текущем turn; payload не потреблялся.
    UnityAlreadyProcessed,
    /// GUID-map прочитан, но sync-флаг ещё не разрешал вызов комнаты.
    UnityWaiting {
        count_warning: bool,
        enabled_now: bool,
    },
    /// Safe-граница сериализации остановила Unity после сохранённых send-ов.
    UnitySerializeBlocked {
        count_warning: bool,
        sends: Vec<Result<i32, SendMessageError>>,
        error: GoodsNodeSerializeError,
    },
    /// Unity полностью завершён и turn-local count поставлен в `1`.
    UnityCompleted {
        count_warning: bool,
        sends: Vec<Result<i32, SendMessageError>>,
    },
    /// Safe-сериализация self-list остановилась до единственного Send.
    SelfAuctionSerializeBlocked(GoodsNodeSerializeError),
    /// Self-list полностью построен и отправлен без приоритета.
    SelfAuctionResponse { send: Result<i32, SendMessageError> },
    /// Safe page/filter-граница остановила ещё не отправленный response.
    AuctionPageBuildBlocked(AuctionPageBuildError),
    /// Страница построена и отправлена без приоритета.
    AuctionPageResponse { send: Result<i32, SendMessageError> },
    /// Invalid GUID не достиг operation helper-а и не изменил комнату.
    OwnerOperationIgnored,
    /// Operation helper вызван; его всегда-`false` return намеренно отброшен.
    OwnerOperationRequested,
    /// Полное условие поиска вставлено либо заменило прежнее значение.
    SearchConditionModified,
}

/// Обрабатывает доказанные ветви `0x0014ED01` и `0x0014ED04..0x0014ED09`.
pub(crate) fn on_msg_w2m_auction(message: &mut CMessage, game: &mut CGame) -> WorldAuctionOutcome {
    if message.message_type() == QUEUE_AUCTION_OPERATION {
        let player_id = message.base_mut().get_long().unwrap_or(0);
        let guid = message.base_mut().get_guid().unwrap_or(CGuid::GUID_INVALID);
        let pushed =
            game.auction_room_mut()
                .push_item_to_opt_list(guid, WORLD_REQUEST_OPERATION, player_id);
        if pushed {
            return WorldAuctionOutcome::OperationWithoutResponse;
        }

        let mut response = CMessage::new(AUCTION_OPERATION_RESPONSE);
        response.base_mut().add_long(0);
        response.base_mut().add_long(player_id);
        let sender = game.net_client().map(|client| client as &dyn MessageSender);
        return WorldAuctionOutcome::OperationResponse {
            send: response.send(sender, false),
        };
    }

    if message.message_type() == UNITY_AUCTION_GOODS {
        return on_unity_auction_goods(message, game);
    }

    if message.message_type() == QUERY_SELF_AUCTION {
        let player_id = message.base_mut().get_long().unwrap_or(0) as u32;
        let mut response = CMessage::new(SELF_AUCTION_RESPONSE);
        response.base_mut().add_ulong(player_id);
        if let Err(error) = game
            .auction_room()
            .add_byte_auction_self_to_client(&mut response, player_id)
        {
            return WorldAuctionOutcome::SelfAuctionSerializeBlocked(error);
        }
        let sender = game.net_client().map(|client| client as &dyn MessageSender);
        return WorldAuctionOutcome::SelfAuctionResponse {
            send: response.send(sender, false),
        };
    }

    if message.message_type() == QUERY_AUCTION_PAGE {
        let player_id = message.base_mut().get_long().unwrap_or(0) as u32;
        let operation = message.base_mut().get_long().unwrap_or(0) as u32;
        if let Err(error) = game
            .auction_room_mut()
            .compute_player_page(player_id, operation)
        {
            return WorldAuctionOutcome::AuctionPageBuildBlocked(error);
        }

        let mut response = CMessage::new(AUCTION_PAGE_RESPONSE);
        response.base_mut().add_ulong(player_id);
        if let Err(error) = game
            .auction_room()
            .add_byte_at_page_by_time(player_id, &mut response)
        {
            return WorldAuctionOutcome::AuctionPageBuildBlocked(error);
        }
        let sender = game.net_client().map(|client| client as &dyn MessageSender);
        return WorldAuctionOutcome::AuctionPageResponse {
            send: response.send(sender, false),
        };
    }

    if message.message_type() == QUEUE_OWNER_AUCTION_OPERATION {
        let guid = message.base_mut().get_guid().unwrap_or(CGuid::GUID_INVALID);
        if guid == CGuid::GUID_INVALID {
            return WorldAuctionOutcome::OwnerOperationIgnored;
        }

        let _ = game
            .auction_room_mut()
            .push_item_to_opt_list(guid, OWNER_REQUEST_OPERATION, 0);
        return WorldAuctionOutcome::OwnerOperationRequested;
    }

    if message.message_type() == MODIFY_AUCTION_SEARCH {
        let player_id = message.base_mut().get_long().unwrap_or(0) as u32;
        let low_level = message.base_mut().get_long().unwrap_or(0);
        let up_level = message.base_mut().get_long().unwrap_or(0);
        let use_self = message.base_mut().get_long().unwrap_or(0);
        let money_type = message.base_mut().get_long().unwrap_or(0);
        let weapon_type = message.base_mut().get_long().unwrap_or(0);
        let name_bytes = message
            .base_mut()
            .get_str_bytes(SEARCH_GOODS_NAME_BYTES)
            .unwrap_or_default();
        let mut goods_name = [0; SEARCH_GOODS_NAME_BYTES];
        goods_name[..name_bytes.len()].copy_from_slice(&name_bytes);

        let condition = PlayerOptNode::from_search_request(
            player_id,
            low_level,
            up_level,
            use_self,
            money_type,
            weapon_type,
            goods_name,
        );
        game.auction_room_mut()
            .modify_player_seach_condition(condition);
        return WorldAuctionOutcome::SearchConditionModified;
    }

    if message.message_type() != ADD_AUCTION_ITEM {
        return WorldAuctionOutcome::Unhandled;
    }

    game.count_new_auction_item();
    let mut item = Box::new(CGoodsNode::new());
    let unserialize = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        item.unserialize(source, cursor)
    };
    if let Err(error) = unserialize {
        // BLOCKED_MISSING_FACT: MiscServer RVA 0x000036B0 всё равно продолжал
        // после безграничного void `UnSerialize`; результат отсутствующих
        // bytes и последующего `AddItemToAuctionRoom` не доказан.
        return WorldAuctionOutcome::UnserializeBlocked(error);
    }

    match game.add_auction_item(item) {
        Ok(true) => WorldAuctionOutcome::ItemAdded,
        Ok(false) => WorldAuctionOutcome::ItemRejected,
        Err(error) => WorldAuctionOutcome::MissingGoodsType(error),
    }
}

fn on_unity_auction_goods(message: &mut CMessage, game: &mut CGame) -> WorldAuctionOutcome {
    match game.auction_sync_count() {
        None => return WorldAuctionOutcome::UnityTurnUninitialized,
        Some(count) if count != 0 => return WorldAuctionOutcome::UnityAlreadyProcessed,
        Some(0) => {}
        Some(_) => unreachable!("guard охватывает все ненулевые unsigned значения"),
    }

    let map_id = message.base_mut().get_byte().unwrap_or(0);
    let declared_count = message.base_mut().get_long().unwrap_or(0) as u32;
    let count_warning = declared_count > UNITY_COUNT_WARNING_THRESHOLD;
    let mut existing = BTreeMap::new();
    for _ in 0..declared_count {
        let guid = message.base_mut().get_guid().unwrap_or(CGuid::GUID_INVALID);
        existing.insert(guid, false);
    }

    if !game.auction_sync_enabled() {
        let enabled_now = game
            .auction_sync_start_time()
            .wrapping_add(SYNC_WAIT_MILLISECONDS)
            < legacy_tick_ms();
        if enabled_now {
            game.enable_auction_sync();
        }
        return WorldAuctionOutcome::UnityWaiting {
            count_warning,
            enabled_now,
        };
    }

    let UnityGoodsBuild { messages, blocked } = game.auction_room().unity_goods(&existing, map_id);
    let sender = game.net_client().map(|client| client as &dyn MessageSender);
    let sends = messages
        .into_iter()
        .map(|message| message.send(sender, false))
        .collect();
    if let Some(error) = blocked {
        return WorldAuctionOutcome::UnitySerializeBlocked {
            count_warning,
            sends,
            error,
        };
    }

    game.finish_auction_sync_turn();
    WorldAuctionOutcome::UnityCompleted {
        count_warning,
        sends,
    }
}
