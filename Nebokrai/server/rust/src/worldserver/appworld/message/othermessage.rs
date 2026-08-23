//! WorldServer dispatcher-owner `OnOtherMessage`.
//!
//! transport leaves `0x5FD02`, `0x5FD06..0x5FD09`, goods-link publish/lookup
//! `0x5FD03/0x5FD04`, increment-log page `0x5FD0A`, copy-number `0x5FD0B`,
//! cursor-only `0x5FD0E`, chat relay `0x5FD01`, player rename `0x5FD05`,
//! LeiTing update `0x5FD10`, honor-reset `0x5FD0C` и eliminate update
//! `0x5FD0D` входят в контракт owner-а. Reset читает один Windows `long`,
//! получает текущий `CGame`
//! и вызывает `ResetHonorElimilateInfo`.
//! Недостаточный payload сохраняет старое поведение numeric getter-а: значение
//! становится нулём без сдвига cursor; отчёт отдельно фиксирует неполноту.
//!
//! Для `0x5FD0D` owner сначала читает player/eliminator,
//! проверяет online player и duplicate ledger, и только для новой пары читает
//! четыре прежних счётчика, прибавляет к каждому единицу, обновляет ranks и
//! отвечает `0x7FA16 + player + char(1)` в исходный socket. Дубликат прекращает
//! ветку до чтения счётчиков и ответа; этот cursor/order контракт сохранён.
//! `0x5FD02` читает target map, переписывает исходный type в `0x7FA05` и
//! маршрутизует то же сообщение; `0x5FD06..09` только переписывают type и
//! делают `SendAll`. `0x5FD0E` ровно один раз читает и отбрасывает signed long.
//! Ownership/tail validation отсутствует в EXE.
//! Полный `switch` завершается общим epilogue после `0x5FD10`: любой
//! иной opcode не читает payload, не отправляет ответ и не передаётся
//! следующему owner-у. Rust materialизует это `NoOp`.
//! `0x5FD0B` отражает первые два signed long, добавляет прежнее значение
//! `s_nCopyNum` и только при ненулевом третьем поле увеличивает global до
//! `SendToSocket`; это сохраняет peek/reserve и side-effect-before-send.
//! `0x5FD10` читает player ID и только для online owner-а делегирует оставшийся
//! buffer/cursor уже действующему `CPlayer::DecodeByteArrayLeiTing`; malformed
//! хвост возвращается typed-ошибкой с сохранением доказанных prefix-мутаций.
//! `0x5FD05` читает signed player ID и `GetStr(..., 0x20)`, оставляет result
//! `1` при отсутствующем map-owner-е, иначе выполняет восемь проверок
//! `CPlayer::ChangeName`. Ответ всегда `0x7FA0E + ID + char(result) + name\0`;
//! parameterized Tiberius query заменяет только старый ADO owner. Только
//! локальная safe-граница недопустимо длинного уже сохранённого имени не
//! получает выдуманного response после исходного stack-overread.
//! `0x5FD0A` читает player/page, требует online owner-а и отвечает
//! `0x7FA12 + player + page` только когда `CIncrementLog` добавил page.
//! Empty/missing player history сохраняет исходную ветку без отправки; page
//! wire и newest-first порядок принадлежат concrete increment-log owner-у.
//!
//! Goods-link publish точно сохраняет три `long`, условную строку type `2`,
//! title/text, positive signed count и два entry-вида. Changed entry владеет
//! декодированным `CGoods`, unchanged хранит `type + uchar amount`; ссылка
//! добавляется до изменения текста. Rewrite удаляет девять байт от `change=`,
//! заменяет участок с offset `+3` до `>` signed-десятичным индексом и продолжает
//! после `</goodslink>`. Lookup возвращает `long(found)` и либо прежний goods,
//! owner/tail/type/count проверки отсутствуют.
//! Malformed goods и невозможные позиции `std::string` остаются typed safe-
//! границами; уже добавленные prefix-ссылки при rewrite-ошибке не откатываются.
//! Chat-ветка сохраняет условное чтение строк: faction name/content читаются
//! только после найденной faction, а private sender/content — только после
//! найденного online-адресата и его region GameServer. Ответы `0x7FA01`
//! сохраняют status `0/1/2`, исходные owner type/ID и прежний порядок строк;
//! faction delivery делегирован точному `CFaction::talk`. Typed write-log FIFO
//! и Tiberius заменяют только SQL-строку/ADO, не меняя `bUseLogSys`,
//! `bFactionChat`/`bPrivateChat`, sender lookup и координатную семантику.

use crate::dbaccess::worlddb::rsplayer::TiberiusRsPlayer;
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::worldserver::appworld::misc::{add_copy_num, get_copy_num};
use crate::worldserver::appworld::goods::cgoods::{CGoods, GoodsCodecError};
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, create_goods,
};
use crate::worldserver::appworld::incrementlog::incrementlog::{
    CIncrementLog, IncrementLogPageBlock,
};
use crate::worldserver::appworld::message::writelogmessage::{
    WorldChatLogWrite, WorldWriteLogCommand,
};
use crate::worldserver::appworld::organizingsystem::faction::FactionTalkDelivery;
use crate::worldserver::appworld::organizingsystem::organizingctrl::{
    COrganizingCtrl, FreePlayerLookup,
};
use crate::worldserver::appworld::player::PlayerCodecError;
use crate::worldserver::appworld::shape::ShapeTileCoordinateBlock;
use crate::worldserver::worldserver::game::{
    CGame, WorldHonorEliminatorRegistration, WorldPlayerNameChangeReport,
    WorldPlayerNameLookupError, WorldGoodsLink, WorldGoodsLinkPayload,
};
use crate::worldserver::worldserver::honorranks::{
    CHonorRanks, HonorRankPushBlock, HonorRanksKilledPlayerReport,
};
use crate::worldserver::worldserver::worldserver::AddLogTextDisposition;

const HONOR_ELIMINATE_RESET: i32 = 0x0005_FD0C;
const HONOR_ELIMINATE_UPDATE: i32 = 0x0005_FD0D;
const HONOR_ELIMINATE_ACKNOWLEDGEMENT: i32 = 0x0007_FA16;
const WORLD_CHAT_REQUEST: i32 = 0x0005_FD01;
const PRIVATE_CHAT_RESPONSE: i32 = 0x0007_FA01;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldPrivateChatDelivery {
    pub(crate) status: i8,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldOtherChatLogOutcome {
    Disabled,
    MissingSender {
        sender_player_id: u32,
        operator_log: AddLogTextDisposition,
    },
    CoordinateBlocked {
        sender_player_id: u32,
        source: ShapeTileCoordinateBlock,
    },
    Queued {
        record: WorldChatLogWrite,
        queue_length_after: usize,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldOtherChatOutcome {
    Unsupported {
        chat_type: i32,
        owner_type: i32,
        owner_id: i32,
        header_complete: [bool; 3],
    },
    FactionUnavailable {
        owner_type: i32,
        owner_id: i32,
        header_complete: [bool; 3],
        lookup: FreePlayerLookup,
    },
    FactionMissing {
        owner_type: i32,
        owner_id: i32,
        faction_id: i32,
        header_complete: [bool; 3],
    },
    FactionDelivered {
        owner_type: i32,
        owner_id: i32,
        faction_id: i32,
        header_complete: [bool; 3],
        sender_name: Vec<u8>,
        content: Vec<u8>,
        deliveries: Vec<FactionTalkDelivery>,
        log: WorldOtherChatLogOutcome,
    },
    PrivateUnavailable {
        owner_type: i32,
        owner_id: i32,
        header_complete: [bool; 3],
        target_name: Vec<u8>,
        target_player_id: u32,
        response: WorldPrivateChatDelivery,
    },
    PrivateDelivered {
        owner_type: i32,
        owner_id: i32,
        header_complete: [bool; 3],
        target_name: Vec<u8>,
        target_player_id: u32,
        target_map_id: i32,
        sender_name: Vec<u8>,
        content: Vec<u8>,
        recipient: WorldPrivateChatDelivery,
        acknowledgement: WorldPrivateChatDelivery,
        log: WorldOtherChatLogOutcome,
    },
}

/// Наблюдаемый итог одной уже действующей ветки `OnOtherMessage`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldHonorEliminateReset {
    pub(crate) rank_mask: u32,
    pub(crate) payload_complete: bool,
    pub(crate) legacy_result: bool,
}

/// Safe-границы двух `std::string` операций publish-ветки.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGoodsLinkTextRewriteBlock {
    MissingChangeMarker,
    MissingTagEnd,
    MissingClosingTag,
}

/// Локальная граница очередного publish-entry; уже добавленные предыдущие
/// ссылки и текущая ссылка перед text-ошибкой не откатываются.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGoodsLinkPublishBlock {
    GoodsDecode {
        entry_index: usize,
        source: GoodsCodecError,
    },
    TextRewrite {
        entry_index: usize,
        source: WorldGoodsLinkTextRewriteBlock,
    },
}

/// Успешно сформированный response одной goods-link ветки.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGoodsLinkResponse {
    pub(crate) response_type: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Publish сохраняет prefix-side-effects: AddGoodsLink предшествует rewrite.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGoodsLinkPublishOutcome {
    pub(crate) link_type: i32,
    pub(crate) first_parameter: i32,
    pub(crate) source_player_id: i32,
    pub(crate) requested_count: i32,
    pub(crate) added_indexes: Vec<u32>,
    pub(crate) result: Result<WorldGoodsLinkResponse, WorldGoodsLinkPublishBlock>,
}

/// Lookup различает найденную ссылку и реально сериализованный товар: для
/// constructor-placeholder-а index `0` и неизвестного type оригинал посылает
/// found=`1`, но не добавляет goods bytes.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldGoodsLinkLookupOutcome {
    pub(crate) requester_player_id: i32,
    pub(crate) link_index: u32,
    pub(crate) found: bool,
    pub(crate) serialized_goods: bool,
    pub(crate) result: Result<WorldGoodsLinkResponse, GoodsCodecError>,
}

/// Наблюдаемый исход duplicate-ledger и rank-update ветки `0x5FD0D`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldHonorEliminateUpdate {
    MissingOnlinePlayer {
        player_id: u32,
        eliminator_id: u32,
        identity_payload_complete: [bool; 2],
    },
    Duplicate {
        player_id: u32,
        eliminator_id: u32,
        identity_payload_complete: [bool; 2],
    },
    RankUpdateBlocked {
        player_id: u32,
        eliminator_id: u32,
        identity_payload_complete: [bool; 2],
        eliminate_counts: [u32; 4],
        counts_payload_complete: [bool; 4],
        source: HonorRankPushBlock,
    },
    Updated {
        player_id: u32,
        eliminator_id: u32,
        identity_payload_complete: [bool; 2],
        eliminate_counts: [u32; 4],
        counts_payload_complete: [bool; 4],
        ranks: Option<HonorRanksKilledPlayerReport>,
        acknowledgement_type: i32,
        acknowledgement: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldIncrementLogPageOutcome {
    pub(crate) player_id: u32,
    pub(crate) page: i32,
    pub(crate) payload_complete: [bool; 2],
    pub(crate) online_player_found: bool,
    pub(crate) serialization: Option<Result<bool, IncrementLogPageBlock>>,
    pub(crate) response_type: i32,
    pub(crate) wire: Option<Vec<u8>>,
    pub(crate) delivery: Option<Result<i32, SendMessageError>>,
}

/// Один обработанный результат полного other-owner-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldOtherMessageOutcome {
 /// Default полного `switch`: message остаётся без side effects.
    NoOp {
        request_type: i32,
    },
    Chat(WorldOtherChatOutcome),
    Transport {
        request_type: i32,
        response_type: i32,
        target_map_id: Option<i32>,
        target_payload_complete: Option<bool>,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    DiscardedLong {
        request_type: i32,
        value: i32,
        payload_complete: bool,
    },
    CopyNumber {
        requester_player_id: i32,
        request_context: i32,
        reserve_flag: i32,
        payload_complete: [bool; 3],
        returned_copy_number: i32,
        copy_number_after: i32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    LeiTingUpdate {
        player_id: u32,
        player_id_complete: bool,
        cursor_before_decode: usize,
        cursor_after_decode: usize,
        decode: Result<bool, PlayerCodecError>,
    },
    PlayerNameChange {
        player_id: u32,
        player_id_complete: bool,
        requested_name: Vec<u8>,
        change: Result<WorldPlayerNameChangeReport, WorldPlayerNameLookupError>,
        response_type: i32,
        wire: Option<Vec<u8>>,
        delivery: Option<Result<i32, SendMessageError>>,
    },
    GoodsLinkPublish(WorldGoodsLinkPublishOutcome),
    GoodsLinkLookup(WorldGoodsLinkLookupOutcome),
    IncrementLogPage(WorldIncrementLogPageOutcome),
    HonorEliminateReset(WorldHonorEliminateReset),
    HonorEliminateUpdate(WorldHonorEliminateUpdate),
}

/// Узкая диспетчеризация уже выбранного other-owner-а.
pub(crate) enum WorldOtherMessageDispatch {
    Handled(WorldOtherMessageOutcome),
    Pending(CMessage),
}

/// Исполняет все transport/cursor/honor ветви other-owner-а.
pub(crate) async fn on_other_message(
    game: &mut CGame,
    organizing: &COrganizingCtrl,
    honor_ranks: &mut CHonorRanks,
    increment_log: &CIncrementLog,
    globe_setup: &GlobeSetupSnapshot,
    goods_registry: &GoodsBasePropertiesRegistry,
    random: &mut dyn FnMut(i32) -> i32,
    rs_player: &mut TiberiusRsPlayer,
    player_database: Option<&mut WorldTdsClient>,
    faction_chat_log_enabled: bool,
    private_chat_log_enabled: bool,
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    mut message: CMessage,
) -> WorldOtherMessageDispatch {
    match message.message_type() {
        WORLD_CHAT_REQUEST => WorldOtherMessageDispatch::Handled(handle_chat_message(
            game,
            organizing,
            faction_chat_log_enabled,
            private_chat_log_enabled,
            add_log_text,
            &mut message,
        )),
        0x0005_FD02 => {
            let decoded_target_map_id = message.base_mut().get_long();
            let target_map_id = decoded_target_map_id.unwrap_or(0);
            message.set_message_type(0x0007_FA05);
            let wire = message.as_wire_bytes().to_vec();
            let delivery = game.send_msg_to_game_server(target_map_id, &message);
            WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::Transport {
                request_type: 0x0005_FD02,
                response_type: 0x0007_FA05,
                target_map_id: Some(target_map_id),
                target_payload_complete: Some(decoded_target_map_id.is_some()),
                wire,
                delivery,
            })
        }
        0x0005_FD03 => WorldOtherMessageDispatch::Handled(
            handle_goods_link_publish(game, &mut message),
        ),
        0x0005_FD04 => WorldOtherMessageDispatch::Handled(handle_goods_link_lookup(
            game,
            goods_registry,
            random,
            &mut message,
        )),
        request_type @ (0x0005_FD06 | 0x0005_FD07 | 0x0005_FD08 | 0x0005_FD09) => {
            let response_type = match request_type {
                0x0005_FD06 => 0x0007_FA03,
                0x0005_FD07 => 0x0007_FA0F,
                0x0005_FD08 => 0x0007_FA10,
                0x0005_FD09 => 0x0007_FA11,
                _ => unreachable!("match pattern ограничивает exact transport set"),
            };
            message.set_message_type(response_type);
            let wire = message.as_wire_bytes().to_vec();
            let delivery = message.send_all(game.current_game_server_sender().as_ref());
            WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::Transport {
                request_type,
                response_type,
                target_map_id: None,
                target_payload_complete: None,
                wire,
                delivery,
            })
        }
        0x0005_FD0E => {
            let decoded_value = message.base_mut().get_long();
            WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::DiscardedLong {
                request_type: 0x0005_FD0E,
                value: decoded_value.unwrap_or(0),
                payload_complete: decoded_value.is_some(),
            })
        }
        0x0005_FD0A => {
            let decoded_player_id = message.base_mut().get_long();
            let decoded_page = message.base_mut().get_long();
            let player_id = decoded_player_id.unwrap_or(0) as u32;
            let page = decoded_page.unwrap_or(0);
            let online_player_found = game.online_player_by_id(player_id).is_some();
            let mut serialization = None;
            let mut wire = None;
            let mut delivery = None;

            if online_player_found {
                let mut response = CMessage::new(0x0007_FA12);
                response.base_mut().add_ulong(player_id);
                response.base_mut().add_long(page);
                let mut page_bytes = Vec::new();
                let result = increment_log.add_page_to_byte_array(
                    &mut page_bytes,
                    page,
                    player_id as i32,
                );
                if matches!(result, Ok(true)) {
                    response.base_mut().add(&page_bytes);
                    response.base_mut().update();
                    wire = Some(response.as_wire_bytes().to_vec());
                    delivery = Some(response.send_to_socket(
                        game.current_game_server_sender().as_ref(),
                        message.socket_id(),
                    ));
                }
                serialization = Some(result);
            }

            WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::IncrementLogPage(
                WorldIncrementLogPageOutcome {
                    player_id,
                    page,
                    payload_complete: [
                        decoded_player_id.is_some(),
                        decoded_page.is_some(),
                    ],
                    online_player_found,
                    serialization,
                    response_type: 0x0007_FA12,
                    wire,
                    delivery,
                },
            ))
        }
        0x0005_FD0B => {
            let decoded_requester = message.base_mut().get_long();
            let decoded_context = message.base_mut().get_long();
            let decoded_reserve = message.base_mut().get_long();
            let requester_player_id = decoded_requester.unwrap_or(0);
            let request_context = decoded_context.unwrap_or(0);
            let reserve_flag = decoded_reserve.unwrap_or(0);
            let returned_copy_number = get_copy_num();

            let mut response = CMessage::new(0x0007_FA15);
            response.base_mut().add_long(requester_player_id);
            response.base_mut().add_long(request_context);
            response.base_mut().add_long(returned_copy_number);
            let copy_number_after = if reserve_flag != 0 {
                add_copy_num()
            } else {
                returned_copy_number
            };
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_to_socket(
                game.current_game_server_sender().as_ref(),
                message.socket_id(),
            );
            WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::CopyNumber {
                requester_player_id,
                request_context,
                reserve_flag,
                payload_complete: [
                    decoded_requester.is_some(),
                    decoded_context.is_some(),
                    decoded_reserve.is_some(),
                ],
                returned_copy_number,
                copy_number_after,
                response_type: 0x0007_FA15,
                wire,
                delivery,
            })
        }
        0x0005_FD05 => {
            let decoded_player_id = message.base_mut().get_long();
            let player_id = decoded_player_id.unwrap_or(0) as u32;
            let requested_name = message
                .base_mut()
                .get_str_bytes(0x20)
                .expect("literal 0x20 исключает zero-capacity GetStr");
            let change = game
                .change_map_player_name(
                    player_id,
                    Some(&requested_name),
                    globe_setup,
                    rs_player,
                    player_database,
                )
                .await;

            let (wire, delivery) = if let Ok(report) = &change {
                let mut response = CMessage::new(0x0007_FA0E);
                response.base_mut().add_ulong(player_id);
                response.base_mut().add_char(report.legacy_result as i8);
                add_c_string(&mut response, &requested_name);
                let wire = response.as_wire_bytes().to_vec();
                let delivery = response.send_to_socket(
                    game.current_game_server_sender().as_ref(),
                    message.socket_id(),
                );
                (Some(wire), Some(delivery))
            } else {
                (None, None)
            };
            WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::PlayerNameChange {
                player_id,
                player_id_complete: decoded_player_id.is_some(),
                requested_name,
                change,
                response_type: 0x0007_FA0E,
                wire,
                delivery,
            })
        }
        0x0005_FD10 => {
            let decoded_player_id = message.base_mut().get_long();
            let player_id = decoded_player_id.unwrap_or(0) as u32;
            let cursor_before_decode = message.base_mut().cursor();
            let decode = {
                let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                game.decode_online_player_lei_ting(player_id, source, cursor)
            };
            let cursor_after_decode = message.base_mut().cursor();
            WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::LeiTingUpdate {
                player_id,
                player_id_complete: decoded_player_id.is_some(),
                cursor_before_decode,
                cursor_after_decode,
                decode,
            })
        }
        HONOR_ELIMINATE_RESET => {
            let decoded = message.base_mut().get_long();
            let rank_mask = decoded.unwrap_or(0) as u32;
            WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::HonorEliminateReset(
                WorldHonorEliminateReset {
                    rank_mask,
                    payload_complete: decoded.is_some(),
                    legacy_result: game.reset_honor_eliminate_info(rank_mask),
                },
            ))
        }
        HONOR_ELIMINATE_UPDATE => {
            let decoded_player = message.base_mut().get_long();
            let decoded_eliminator = message.base_mut().get_long();
            let player_id = decoded_player.unwrap_or(0) as u32;
            let eliminator_id = decoded_eliminator.unwrap_or(0) as u32;
            let identity_payload_complete =
                [decoded_player.is_some(), decoded_eliminator.is_some()];

            let early = match game.register_honor_eliminator(player_id, eliminator_id) {
                WorldHonorEliminatorRegistration::MissingOnlinePlayer => {
                    Some(WorldHonorEliminateUpdate::MissingOnlinePlayer {
                        player_id,
                        eliminator_id,
                        identity_payload_complete,
                    })
                }
                WorldHonorEliminatorRegistration::Duplicate => {
                    Some(WorldHonorEliminateUpdate::Duplicate {
                        player_id,
                        eliminator_id,
                        identity_payload_complete,
                    })
                }
                WorldHonorEliminatorRegistration::Accepted => None,
            };
            if let Some(outcome) = early {
                return WorldOtherMessageDispatch::Handled(
                    WorldOtherMessageOutcome::HonorEliminateUpdate(outcome),
                );
            }

            let decoded_counts = [
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
            ];
            let counts_payload_complete = decoded_counts.map(|value| value.is_some());
            let eliminate_counts =
                decoded_counts.map(|value| value.unwrap_or(0).wrapping_add(1) as u32);
            let player = game
                .online_player_by_id(player_id)
                .expect("accepted honor pair сохраняет прежнего online player");
            let ranks = match honor_ranks.killed_one_player(player, eliminate_counts) {
                Ok(ranks) => ranks,
                Err(source) => {
                    return WorldOtherMessageDispatch::Handled(
                        WorldOtherMessageOutcome::HonorEliminateUpdate(
                            WorldHonorEliminateUpdate::RankUpdateBlocked {
                                player_id,
                                eliminator_id,
                                identity_payload_complete,
                                eliminate_counts,
                                counts_payload_complete,
                                source,
                            },
                        ),
                    );
                }
            };

            let mut acknowledgement = CMessage::new(HONOR_ELIMINATE_ACKNOWLEDGEMENT);
            acknowledgement.base_mut().add_ulong(player_id);
            acknowledgement.base_mut().add_char(1);
            let socket_id = message.socket_id();
            let sender = game.current_game_server_sender();
            let delivery = acknowledgement.send_to_socket(sender.as_ref(), socket_id);
            WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::HonorEliminateUpdate(
                WorldHonorEliminateUpdate::Updated {
                    player_id,
                    eliminator_id,
                    identity_payload_complete,
                    eliminate_counts,
                    counts_payload_complete,
                    ranks,
                    acknowledgement_type: HONOR_ELIMINATE_ACKNOWLEDGEMENT,
                    acknowledgement: delivery,
                },
            ))
        }
        request_type => WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::NoOp {
            request_type,
        }),
    }
}

fn handle_chat_message(
    game: &CGame,
    organizing: &COrganizingCtrl,
    faction_chat_log_enabled: bool,
    private_chat_log_enabled: bool,
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    message: &mut CMessage,
) -> WorldOtherMessageOutcome {
    let decoded_chat_type = message.base_mut().get_long();
    let decoded_owner_type = message.base_mut().get_long();
    let decoded_owner_id = message.base_mut().get_long();
    let chat_type = decoded_chat_type.unwrap_or(0);
    let owner_type = decoded_owner_type.unwrap_or(0);
    let owner_id = decoded_owner_id.unwrap_or(0);
    let header_complete = [
        decoded_chat_type.is_some(),
        decoded_owner_type.is_some(),
        decoded_owner_id.is_some(),
    ];

    if chat_type == 2 {
        let lookup = organizing.is_free_player(owner_id);
        let faction_id = match lookup {
            FreePlayerLookup::Faction(faction_id) => faction_id,
            FreePlayerLookup::NoFaction | FreePlayerLookup::BlockedNullFaction { .. } => {
                return WorldOtherMessageOutcome::Chat(
                    WorldOtherChatOutcome::FactionUnavailable {
                        owner_type,
                        owner_id,
                        header_complete,
                        lookup,
                    },
                );
            }
        };
        let Some(faction) = organizing.faction_by_id(faction_id) else {
            return WorldOtherMessageOutcome::Chat(WorldOtherChatOutcome::FactionMissing {
                owner_type,
                owner_id,
                faction_id,
                header_complete,
            });
        };

        let sender_name = message
            .base_mut()
            .get_str_bytes(0x100)
            .expect("literal 0x100 исключает zero-capacity GetStr");
        let content = message
            .base_mut()
            .get_str_bytes(0x400)
            .expect("literal 0x400 исключает zero-capacity GetStr");
        let deliveries = faction.talk(game, owner_id, &sender_name, &content);
        let log = finish_chat_log(
            game,
            faction_chat_log_enabled,
            &sender_name,
            &content,
            0,
            b"<Faction>",
            2,
            add_log_text,
        );
        return WorldOtherMessageOutcome::Chat(WorldOtherChatOutcome::FactionDelivered {
            owner_type,
            owner_id,
            faction_id,
            header_complete,
            sender_name,
            content,
            deliveries,
            log,
        });
    }

    if chat_type == 4 {
        let target_name = message
            .base_mut()
            .get_str_bytes(0x100)
            .expect("literal 0x100 исключает zero-capacity GetStr");
        let target_player_id = game.online_player_id_by_name(&target_name);
        let target_region_id = (target_player_id != 0)
            .then(|| game.online_player_by_id(target_player_id))
            .flatten()
            .map(|player| player.get_region_id());
        let target_map_id = target_region_id
            .and_then(|region_id| game.get_region_game_server(region_id))
            .map(|game_server| game_server.index as i32);
        let Some(target_map_id) = target_map_id else {
            let response = send_private_chat_message(
                game,
                message.socket_id(),
                None,
                0,
                owner_type,
                owner_id,
                &[],
                &[],
                &[],
            );
            return WorldOtherMessageOutcome::Chat(WorldOtherChatOutcome::PrivateUnavailable {
                owner_type,
                owner_id,
                header_complete,
                target_name,
                target_player_id,
                response,
            });
        };

        let sender_name = message
            .base_mut()
            .get_str_bytes(0x100)
            .expect("literal 0x100 исключает zero-capacity GetStr");
        let content = message
            .base_mut()
            .get_str_bytes(0x400)
            .expect("literal 0x400 исключает zero-capacity GetStr");
        let recipient = send_private_chat_message(
            game,
            message.socket_id(),
            Some(target_map_id),
            1,
            owner_type,
            owner_id,
            &target_name,
            &sender_name,
            &content,
        );
        let acknowledgement = send_private_chat_message(
            game,
            message.socket_id(),
            None,
            2,
            owner_type,
            owner_id,
            &target_name,
            &sender_name,
            &content,
        );
        let receiver_id = game.map_player_id_by_name(&target_name) as i32;
        let log = finish_chat_log(
            game,
            private_chat_log_enabled,
            &sender_name,
            &content,
            receiver_id,
            &target_name,
            5,
            add_log_text,
        );
        return WorldOtherMessageOutcome::Chat(WorldOtherChatOutcome::PrivateDelivered {
            owner_type,
            owner_id,
            header_complete,
            target_name,
            target_player_id,
            target_map_id,
            sender_name,
            content,
            recipient,
            acknowledgement,
            log,
        });
    }

    WorldOtherMessageOutcome::Chat(WorldOtherChatOutcome::Unsupported {
        chat_type,
        owner_type,
        owner_id,
        header_complete,
    })
}

#[allow(clippy::too_many_arguments)]
fn send_private_chat_message(
    game: &CGame,
    source_socket_id: i32,
    target_map_id: Option<i32>,
    status: i8,
    owner_type: i32,
    owner_id: i32,
    target_name: &[u8],
    sender_name: &[u8],
    content: &[u8],
) -> WorldPrivateChatDelivery {
    let mut response = CMessage::new(PRIVATE_CHAT_RESPONSE);
    response.base_mut().add_char(status);
    response.base_mut().add_long(owner_type);
    response.base_mut().add_long(owner_id);
    if status != 0 {
        add_c_string(&mut response, target_name);
        add_c_string(&mut response, sender_name);
        add_c_string(&mut response, content);
    }
    let wire = response.as_wire_bytes().to_vec();
    let delivery = if let Some(target_map_id) = target_map_id {
        game.send_msg_to_game_server(target_map_id, &response)
    } else {
        response.send_to_socket(game.current_game_server_sender().as_ref(), source_socket_id)
    };
    WorldPrivateChatDelivery {
        status,
        wire,
        delivery,
    }
}

#[allow(clippy::too_many_arguments)]
fn finish_chat_log(
    game: &CGame,
    enabled: bool,
    sender_name: &[u8],
    content: &[u8],
    receiver_id: i32,
    receiver_name: &[u8],
    log_type: u8,
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
) -> WorldOtherChatLogOutcome {
    let sender_player_id = game.map_player_id_by_name(sender_name);
    let Some(sender) = game.map_player(sender_player_id) else {
        let signed_sender_player_id = sender_player_id as i32;
        let mut text = if log_type == 2 {
            format!("CHAT_FACTION : {signed_sender_player_id} pPlayer").into_bytes()
        } else {
            format!("CHAT_PRIVATE : {signed_sender_player_id} pPlayer").into_bytes()
        };
        text.extend_from_slice(b"\xCE\xAANULL");
        if log_type == 2 {
            text.push(b'!');
        } else {
            text.extend_from_slice(format!(" => {receiver_id} !").as_bytes());
        }
        return WorldOtherChatLogOutcome::MissingSender {
            sender_player_id,
            operator_log: add_log_text(&text),
        };
    };
    if !enabled {
        return WorldOtherChatLogOutcome::Disabled;
    }

 // `_sprintf` вычислял Y до X; первая невозможная x87-конверсия
 // остаётся наблюдаемой typed safe-границей в том же порядке.
    let position_y = match sender.get_tile_y() {
        Ok(value) => value,
        Err(source) => {
            return WorldOtherChatLogOutcome::CoordinateBlocked {
                sender_player_id,
                source,
            };
        }
    };
    let position_x = match sender.get_tile_x() {
        Ok(value) => value,
        Err(source) => {
            return WorldOtherChatLogOutcome::CoordinateBlocked {
                sender_player_id,
                source,
            };
        }
    };
    let record = WorldChatLogWrite {
        sender_id: sender_player_id as i32,
        sender_name: sender_name.to_vec(),
        map_id: sender.get_region_id(),
        position_x,
        position_y,
        receiver_id,
        receiver_name: receiver_name.to_vec(),
        content: content.to_vec(),
        log_type,
    };
    let queue_length_after =
        game.push_write_log_command(WorldWriteLogCommand::ChatLog(record.clone()));
    WorldOtherChatLogOutcome::Queued {
        record,
        queue_length_after,
    }
}

fn handle_goods_link_publish(game: &mut CGame, message: &mut CMessage) -> WorldOtherMessageOutcome {
    let link_type = message.base_mut().get_long().unwrap_or(0);
    let first_parameter = message.base_mut().get_long().unwrap_or(0);
    let source_player_id = message.base_mut().get_long().unwrap_or(0);
    let optional_name = (link_type == 2).then(|| {
        message
            .base_mut()
            .get_str_bytes(0x100)
            .expect("literal 0x100 исключает zero-capacity GetStr")
    });
    let title = message
        .base_mut()
        .get_str_bytes(0x100)
        .expect("literal 0x100 исключает zero-capacity GetStr");
    let mut rewritten_text = message
        .base_mut()
        .get_str_bytes(0x400)
        .expect("literal 0x400 исключает zero-capacity GetStr");
    let requested_count = message.base_mut().get_long().unwrap_or(0);
    let mut added_indexes = Vec::new();
    let mut text_search_offset = 0;

    for entry_index in 0..requested_count.max(0) as usize {
        let wire_kind = message.base_mut().get_long().unwrap_or(0);
        let link = if wire_kind == 1 {
            let mut goods = Box::new(CGoods::with_constructor_base_and_type());
            let decoded = {
                let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                goods.decord_from_byte_array(source, cursor, true)
            };
            if let Err(source) = decoded {
                return WorldOtherMessageOutcome::GoodsLinkPublish(
                    WorldGoodsLinkPublishOutcome {
                        link_type,
                        first_parameter,
                        source_player_id,
                        requested_count,
                        added_indexes,
                        result: Err(WorldGoodsLinkPublishBlock::GoodsDecode {
                            entry_index,
                            source,
                        }),
                    },
                );
            }
            WorldGoodsLink::changed(goods)
        } else {
            let goods_type = message.base_mut().get_long().unwrap_or(0) as u32;
            let amount = message.base_mut().get_byte().unwrap_or(0);
            WorldGoodsLink::original(goods_type, amount)
        };

        let index = game.add_goods_link(link);
        added_indexes.push(index);
        if let Err(source) = rewrite_goods_link_text(
            &mut rewritten_text,
            index,
            entry_index,
            &mut text_search_offset,
        ) {
            return WorldOtherMessageOutcome::GoodsLinkPublish(WorldGoodsLinkPublishOutcome {
                link_type,
                first_parameter,
                source_player_id,
                requested_count,
                added_indexes,
                result: Err(source),
            });
        }
    }

    let mut response = CMessage::new(0x0007_FA06);
    response.base_mut().add_long(link_type);
    response.base_mut().add_long(first_parameter);
    response.base_mut().add_long(source_player_id);
    if let Some(optional_name) = optional_name.as_deref() {
        add_c_string(&mut response, optional_name);
    }
    add_c_string(&mut response, &title);
    add_c_string(&mut response, &rewritten_text);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send_to_socket(
        game.current_game_server_sender().as_ref(),
        message.socket_id(),
    );
    WorldOtherMessageOutcome::GoodsLinkPublish(WorldGoodsLinkPublishOutcome {
        link_type,
        first_parameter,
        source_player_id,
        requested_count,
        added_indexes,
        result: Ok(WorldGoodsLinkResponse {
            response_type: 0x0007_FA06,
            wire,
            delivery,
        }),
    })
}

fn rewrite_goods_link_text(
    text: &mut Vec<u8>,
    index: u32,
    entry_index: usize,
    search_offset: &mut usize,
) -> Result<(), WorldGoodsLinkPublishBlock> {
    let marker = find_bytes(text, b"change=", *search_offset).ok_or(
        WorldGoodsLinkPublishBlock::TextRewrite {
            entry_index,
            source: WorldGoodsLinkTextRewriteBlock::MissingChangeMarker,
        },
    )?;
    text.drain(marker..(marker + 9).min(text.len()));

    let index_position = marker + 3;
    let tag_end = text
        .get(index_position..)
        .ok_or(WorldGoodsLinkPublishBlock::TextRewrite {
            entry_index,
            source: WorldGoodsLinkTextRewriteBlock::MissingTagEnd,
        })?
        .iter()
        .position(|byte| *byte == b'>')
        .map(|position| index_position + position)
        .ok_or(WorldGoodsLinkPublishBlock::TextRewrite {
            entry_index,
            source: WorldGoodsLinkTextRewriteBlock::MissingTagEnd,
        })?;
    text.drain(index_position..tag_end);
    let digits = (index as i32).to_string().into_bytes();
    text.splice(index_position..index_position, digits);
    let closing = find_bytes(text, b"</goodslink>", index_position).ok_or(
        WorldGoodsLinkPublishBlock::TextRewrite {
            entry_index,
            source: WorldGoodsLinkTextRewriteBlock::MissingClosingTag,
        },
    )?;

    *search_offset = closing + b"</goodslink>".len();
    Ok(())
}

fn find_bytes(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    haystack
        .get(start..)?
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|position| start + position)
}

fn handle_goods_link_lookup(
    game: &mut CGame,
    registry: &GoodsBasePropertiesRegistry,
    random: &mut dyn FnMut(i32) -> i32,
    message: &mut CMessage,
) -> WorldOtherMessageOutcome {
    let requester_player_id = message.base_mut().get_long().unwrap_or(0);
    let link_index = message.base_mut().get_long().unwrap_or(0) as u32;
    let mut response = CMessage::new(0x0007_FA07);
    response.base_mut().add_long(requester_player_id);

    let found = game.find_goods_link(link_index);
    response.base_mut().add_long(i32::from(found.is_some()));
    let mut serialized_goods = false;
    let serialization = if let Some(link) = found {
        let mut encoded = Vec::new();
        let encoded_result = match link.payload() {
            WorldGoodsLinkPayload::Changed(goods) => {
                goods.add_to_byte_array(&mut encoded, true)
            }
            WorldGoodsLinkPayload::Original { goods_type, amount } => {
                if let Some(mut goods) = create_goods(registry, *goods_type, random) {
                    goods.set_amount(u32::from(*amount));
                    goods.add_to_byte_array(&mut encoded, true)
                } else {
                    Ok(true)
                }
            }
        };
        encoded_result.map(|_| {
            serialized_goods = !encoded.is_empty();
            response.base_mut().add(&encoded);
        })
    } else {
        Ok(())
    };

    if let Err(source) = serialization {
        return WorldOtherMessageOutcome::GoodsLinkLookup(WorldGoodsLinkLookupOutcome {
            requester_player_id,
            link_index,
            found: found.is_some(),
            serialized_goods,
            result: Err(source),
        });
    }

    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send_to_socket(
        game.current_game_server_sender().as_ref(),
        message.socket_id(),
    );
    WorldOtherMessageOutcome::GoodsLinkLookup(WorldGoodsLinkLookupOutcome {
        requester_player_id,
        link_index,
        found: found.is_some(),
        serialized_goods,
        result: Ok(WorldGoodsLinkResponse {
            response_type: 0x0007_FA07,
            wire,
            delivery,
        }),
    })
}

fn add_c_string(message: &mut CMessage, bytes: &[u8]) {
    let visible = bytes.iter().position(|byte| *byte == 0).unwrap_or(bytes.len());
    message.base_mut().add(&bytes[..visible]);
    message.base_mut().add_char(0);
}
