//! Входные сообщения GameServer для предметов и связанных с ними сеансов.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `server/gameserver/appserver/message/goodsmessage.cpp`. Обработчик сохраняет
//! точные коды сообщений, двоичные поля и подтверждённый порядок изменений
//! игрока, контейнеров, предметов, региона, World и клиентских ответов.
//! Диагностические итоги публикуются через `tracing` в месте возникновения и
//! не возвращаются вызывающей стороне отдельными деревьями отчётов.
//!
//! Особо существенны необратимые границы операций: расход предметов и денег
//! остаётся на прежнем месте относительно бросков RNG, аудита и отправки
//! пакетов; завершение сеанса выполняет `end_session`, затем освобождение
//! состояния игрока и только потом `exit_plug`; обработка `0x8FC29` сохраняет
//! последовательность отсоединения навыков боевой феи, запуска сценария,
//! обратного присоединения и подтверждения World. Для `0x8FC2F/0x8FC30`
//! сохранён ранний выход после первого непустого предпросмотра CiQing, а
//! терминальные сеансы экипировки по-прежнему снимают слушателей контейнера и
//! удаляются на штатной стадии основного цикла.
//!
//! Все подтверждённые ветви этого владельца подключены к исполняемым маршрутам
//! Rust; полностью замещённое тело исходного `switch` удалено.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\goodsmessage.cpp

use crate::gameserver::appserver::container::cfairycontainer::FairySyncreticProperty;
use crate::gameserver::appserver::script::function::ScriptFunctionRuntime;
use crate::gameserver::appserver::script::script::ScriptExecutionContext;
use crate::gameserver::appserver::session::cequipmentdakong::EquipmentDaKongOperation;
use crate::gameserver::gameserver::game::{
    BattleFairyDeathContext, CGame, CiQingComposeContext, CiQingOtherPersonTarget,
    EquipmentComposeContext, EquipmentDaKongContext, EquipmentUpgradeContext, FairyContext,
    GameContainerMessageRuntime, SynthesisContext,
};
use crate::nets::netserver::message::CMessage;

const CHECK_BATTLE_FAIRY_COMBINE: u32 = 0x0008_fc26;
const ASSIGN_HOTKEY: u32 = 0x0008_fc08;
const REMOVE_HOTKEY: u32 = 0x0008_fc09;
const CHANGE_HOTKEY: u32 = 0x0008_fc0a;
const QUERY_PLAYER_EQUIPMENT: u32 = 0x0008_fc0b;
const UPGRADE_EQUIPMENT: u32 = 0x0008_fc0f;
const CLOSE_EQUIPMENT_UPGRADE: u32 = 0x0008_fc10;
const HANDLE_CONTAINER_SCRIPT_ACTION: u32 = 0x0008_fc11;
const RUN_PRECIOUS_BOX_ITEM_SCRIPT: u32 = 0x0008_fc12;
const UPDATE_FAIRY_HATCH: u32 = 0x0008_fc13;
const IMPLANT_FAIRY_EXPERIENCE: u32 = 0x0008_fc14;
const SYNCRETIZE_FAIRY: u32 = 0x0008_fc15;
const QUERY_FAIRY_SETUP: u32 = 0x0008_fc16;
const OPEN_SYNTHESIS: u32 = 0x0008_fc17;
const QUERY_SYNTHESIS_LIST: u32 = 0x0008_fc18;
const QUERY_SYNTHESIS_FORMULA: u32 = 0x0008_fc19;
const COMPOSE_SYNTHESIS: u32 = 0x0008_fc1a;
const CLOSE_SYNTHESIS: u32 = 0x0008_fc1b;
const OPEN_GOODS_DESTROY: u32 = 0x0008_fc1c;
const CONFIRM_GOODS_DESTROY: u32 = 0x0008_fc1d;
const CLOSE_EQUIPMENT_DA_KONG: u32 = 0x0008_fc1e;
const EQUIPMENT_DA_KONG: u32 = 0x0008_fc1f;
const EQUIPMENT_ENCHASE_GEM: u32 = 0x0008_fc20;
const EQUIPMENT_CHANGE_ROLE_COLOR: u32 = 0x0008_fc21;
const EQUIPMENT_QUERY_DA_KONG_RESULT: u32 = 0x0008_fc22;
const EQUIPMENT_DESTROY_GEM: u32 = 0x0008_fc23;
const COMBINE_BATTLE_FAIRY: u32 = 0x0008_fc27;
const UPGRADE_BATTLE_FAIRY: u32 = 0x0008_fc28;
const RESET_BATTLE_FAIRY_SKILLS: u32 = 0x0008_fc29;
const ALLOCATE_BATTLE_FAIRY_POTENTIAL: u32 = 0x0008_fc2a;
const RESET_BATTLE_FAIRY_POTENTIAL: u32 = 0x0008_fc2b;
const SUMMON_BATTLE_FAIRY: u32 = 0x0008_fc2c;
const RECALL_BATTLE_FAIRY: u32 = 0x0008_fc2d;
const REFRESH_BATTLE_FAIRY_PROPERTY: u32 = 0x0008_fc2e;
const QUERY_CI_QING_GOODS: u32 = 0x0008_fc2f;
const QUERY_CI_QING_SETUP: u32 = 0x0008_fc30;
const MAKE_CI_QING_NODE: u32 = 0x0008_fc31;
const COMPOSE_CI_QING_NODE: u32 = 0x0008_fc32;
const DELETE_CI_QING_GOODS: u32 = 0x0008_fc33;
const MOUNT_CI_QING_FROM_HAND: u32 = 0x0008_fc34;
const QUERY_CI_QING_OTHER_PERSON: u32 = 0x0008_fc35;
const COMPOSE_EQUIPMENT: u32 = 0x0008_fc24;
const END_GOODS_SESSION: u32 = 0x0008_fc25;

pub(crate) trait GameGoodsMessageRuntime:
    ScriptFunctionRuntime
    + BattleFairyDeathContext
    + CiQingComposeContext
    + EquipmentComposeContext
    + EquipmentDaKongContext
    + EquipmentUpgradeContext
    + GameContainerMessageRuntime
    + FairyContext
    + SynthesisContext
{
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameGoodsMessageError {
    MissingField(&'static str),
}

pub(crate) fn dispatch_game_goods_message<Runtime: GameGoodsMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<(), GameGoodsMessageError>> {
    let message_type = message.message_type() as u32;
    if !matches!(
        message_type,
        ASSIGN_HOTKEY
            | REMOVE_HOTKEY
            | CHANGE_HOTKEY
            | QUERY_PLAYER_EQUIPMENT
            | UPGRADE_EQUIPMENT
            | CLOSE_EQUIPMENT_UPGRADE
            | HANDLE_CONTAINER_SCRIPT_ACTION
            | RUN_PRECIOUS_BOX_ITEM_SCRIPT
            | UPDATE_FAIRY_HATCH
            | IMPLANT_FAIRY_EXPERIENCE
            | SYNCRETIZE_FAIRY
            | QUERY_FAIRY_SETUP
            | OPEN_SYNTHESIS
            | QUERY_SYNTHESIS_LIST
            | QUERY_SYNTHESIS_FORMULA
            | COMPOSE_SYNTHESIS
            | CLOSE_SYNTHESIS
            | OPEN_GOODS_DESTROY
            | CONFIRM_GOODS_DESTROY
            | CLOSE_EQUIPMENT_DA_KONG
            | EQUIPMENT_DA_KONG
            | EQUIPMENT_ENCHASE_GEM
            | EQUIPMENT_CHANGE_ROLE_COLOR
            | EQUIPMENT_QUERY_DA_KONG_RESULT
            | EQUIPMENT_DESTROY_GEM
            | COMPOSE_EQUIPMENT
            | END_GOODS_SESSION
            | CHECK_BATTLE_FAIRY_COMBINE
            | COMBINE_BATTLE_FAIRY
            | UPGRADE_BATTLE_FAIRY
            | RESET_BATTLE_FAIRY_SKILLS
            | ALLOCATE_BATTLE_FAIRY_POTENTIAL
            | RESET_BATTLE_FAIRY_POTENTIAL
            | SUMMON_BATTLE_FAIRY
            | RECALL_BATTLE_FAIRY
            | REFRESH_BATTLE_FAIRY_PROPERTY
            | QUERY_CI_QING_GOODS
            | QUERY_CI_QING_SETUP
            | MAKE_CI_QING_NODE
            | COMPOSE_CI_QING_NODE
            | DELETE_CI_QING_GOODS
            | MOUNT_CI_QING_FROM_HAND
            | QUERY_CI_QING_OTHER_PERSON
    ) {
        return None;
    }

    message.resolve_player_context(game);
    let player_id = message.player_id();
    let region_id = message.region_id();
    let Some(player_id) = player_id else {
        return Some(Ok(()));
    };
    let read_long = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long()
            .ok_or(GameGoodsMessageError::MissingField(field))
    };
    match message_type {
        ASSIGN_HOTKEY => {
            let slot = match message.base_mut().get_char() {
                Some(value) => value as u8,
                None => return Some(Err(GameGoodsMessageError::MissingField("hotkey slot"))),
            };
            let value = match read_long(message, "hotkey value") {
                Ok(value) => value as u32,
                Err(error) => return Some(Err(error)),
            };
            let _ = game
                .assign_hotkey(player_id, slot, value)
                .expect("resolved message player остаётся live во время hotkey assignment");
        }
        REMOVE_HOTKEY => {
            let slot = match message.base_mut().get_char() {
                Some(value) => value as u8,
                None => return Some(Err(GameGoodsMessageError::MissingField("hotkey slot"))),
            };
            let _ = game
                .remove_hotkey(player_id, slot)
                .expect("resolved message player остаётся live во время hotkey removal");
        }
        CHANGE_HOTKEY => {
            let slot = match message.base_mut().get_char() {
                Some(value) => value as u8,
                None => return Some(Err(GameGoodsMessageError::MissingField("hotkey slot"))),
            };
            let value = match read_long(message, "hotkey value") {
                Ok(value) => value as u32,
                Err(error) => return Some(Err(error)),
            };
            let _ = game
                .change_hotkey(player_id, slot, value)
                .expect("resolved message player остаётся live во время hotkey change");
        }
        QUERY_PLAYER_EQUIPMENT => {
            let target_id = match read_long(message, "equipment target player ID") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let _ = game.query_player_equipment(player_id, target_id, runtime);
        }
        UPGRADE_EQUIPMENT => {
            let session_id = match read_long(message, "equipment upgrade session ID") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let requested_plug_id = match read_long(message, "equipment upgrade plug ID") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let _ = game.upgrade_equipment(
                player_id,
                session_id,
                requested_plug_id,
                runtime,
            );
        }
        CLOSE_EQUIPMENT_UPGRADE => {
            let session_id = match read_long(message, "equipment upgrade close session ID") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let _ = game.close_equipment_upgrade(player_id, session_id);
        }
        HANDLE_CONTAINER_SCRIPT_ACTION => {
            let action = match message.base_mut().get_char() {
                Some(value) => value,
                None => {
                    return Some(Err(GameGoodsMessageError::MissingField(
                        "container script action",
                    )));
                }
            };
            let _ = game
                .handle_container_script_action(player_id, region_id, action, runtime)
                .expect("resolved player остаётся live во время container script action");
        }
        RUN_PRECIOUS_BOX_ITEM_SCRIPT => {
            let _ = game
                .run_precious_box_item_script(player_id, region_id, runtime)
                .expect("resolved player остаётся live во время precious-box script action");
        }
        UPDATE_FAIRY_HATCH => {
            if !game
                .find_player(player_id)
                .is_some_and(|player| player.fairy_container_enabled())
            {
                tracing::trace!(player_id, "контейнер феи недоступен");
            } else {
                let slot = match read_long(message, "fairy hatch slot") {
                    Ok(value) => value as u32,
                    Err(error) => return Some(Err(error)),
                };
                let action = match message.base_mut().get_char() {
                    Some(value) => value,
                    None => {
                        return Some(Err(GameGoodsMessageError::MissingField(
                            "fairy hatch action",
                        )));
                    }
                };
                game.update_fairy_hatch_state(player_id, slot, action, runtime);
            }
        }
        IMPLANT_FAIRY_EXPERIENCE => {
            let Some(player) = game.find_player(player_id) else {
                unreachable!("resolved player checked before goods dispatch")
            };
            if !player.fairy_container_enabled() {
                tracing::trace!(player_id, "контейнер феи недоступен");
            } else {
                let needs_vigour = player
                    .fairy_container()
                    .base()
                    .get_goods(0)
                    .and_then(|goods| goods.fairy_properties())
                    .is_some_and(|fairy| {
                        !((fairy.fairy_state == 0
                            && game.globe_setup().fairy_egg_max_level() <= fairy.level)
                            || fairy.ripe_max_level <= fairy.level)
                    });
                let requested_vigour = if needs_vigour {
                    match read_long(message, "fairy implantation vigour") {
                        Ok(value) => value as u32,
                        Err(error) => return Some(Err(error)),
                    }
                } else {
                    0
                };
                game.implant_fairy_experience(player_id, requested_vigour, runtime);
            }
        }
        SYNCRETIZE_FAIRY => {
            if !game
                .find_player(player_id)
                .is_some_and(|player| player.fairy_container_enabled())
            {
                tracing::trace!(player_id, "контейнер феи недоступен");
            } else {
                let property = match read_long(message, "fairy syncretic property") {
                    Ok(0) => FairySyncreticProperty::FairyAttribute,
                    Ok(_) => FairySyncreticProperty::GrowingRate,
                    Err(error) => return Some(Err(error)),
                };
                game.syncretize_fairy(player_id, property, runtime)
                    .expect("игрок с доступным контейнером феи остаётся зарегистрирован при обработке сообщения");
            }
        }
        QUERY_FAIRY_SETUP => {
            game.query_fairy_setup(player_id);
        }
        OPEN_SYNTHESIS => {
            let _ = game
                .open_synthesis(player_id)
                .expect("resolved message player остаётся в CGame");
        }
        QUERY_SYNTHESIS_LIST => {
            let mode = match message.base_mut().get_char() {
                Some(value) => value,
                None => {
                    return Some(Err(GameGoodsMessageError::MissingField(
                        "synthesis list mode",
                    )));
                }
            };
            let synthesis_type = match message.base_mut().get_word() {
                Some(value) => value,
                None => return Some(Err(GameGoodsMessageError::MissingField("synthesis type"))),
            };
            let mut include_count = true;
            let forms = match mode {
                0 => game.synthesis().forms(synthesis_type),
                2 => {
                    let keyword = match message.base_mut().get_str_bytes(0x100) {
                        Some(value) => value
                            .into_iter()
                            .filter(|byte| *byte != b' ')
                            .collect::<Vec<_>>(),
                        None => {
                            return Some(Err(GameGoodsMessageError::MissingField(
                                "synthesis keyword",
                            )));
                        }
                    };
                    if keyword.is_empty() {
                        include_count = false;
                        Some(Vec::new())
                    } else {
                        Some(game.synthesis().search_forms(synthesis_type, &keyword))
                    }
                }
                _ => None,
            };
            if let Some(forms) = forms {
                let count = forms.len() as u32;
                let mut response = CMessage::new(0x0b_f923);
                if include_count {
                    response.add_ulong(count);
                    for (name, index) in forms {
                        response.base_mut().add(name);
                        response.base_mut().add_byte(0);
                        response.add_ulong(index);
                    }
                }
                let _ = response.send_to_player(game.net_server(), player_id);
            }
        }
        QUERY_SYNTHESIS_FORMULA => {
            let synthesis_index = match read_long(message, "synthesis formula index") {
                Ok(value) => value as u32,
                Err(error) => return Some(Err(error)),
            };
            let _ = game
                .synthesis()
                .recipe(synthesis_index)
                .filter(|recipe| {
                    recipe.coins != -1
                        && recipe.prestige != -1
                        && recipe.probability != u16::MAX
                        && !recipe.formulas.is_empty()
                })
                .map(|recipe| {
                    let mut response = CMessage::new(0x0b_f924);
                    response.add_long(recipe.coins);
                    response.add_long(recipe.prestige);
                    response
                        .base_mut()
                        .add_byte(u8::from(recipe.probability == 100));
                    response.add_ulong(recipe.formulas.len() as u32 + 1);
                    response.add_ulong(recipe.goods_index);
                    response.add_ulong(1);
                    for formula in &recipe.formulas {
                        response.add_ulong(formula.goods_index);
                        response.add_ulong(formula.amount);
                    }
                    response.send_to_player(game.net_server(), player_id)
                });
        }
        COMPOSE_SYNTHESIS => {
            let synthesis_index = match read_long(message, "synthesis compose index") {
                Ok(value) => value as u32,
                Err(error) => return Some(Err(error)),
            };
            let amount = match read_long(message, "synthesis compose amount") {
                Ok(value) => value as u32,
                Err(error) => return Some(Err(error)),
            };
            let Some(()) = game.compose_synthesis(player_id, synthesis_index, amount, runtime)
            else {
                return Some(Ok(()));
            };
        }
        CLOSE_SYNTHESIS => {
            let _ = game.close_synthesis(player_id);
        }
        OPEN_GOODS_DESTROY => {
            let container_extend_id = match read_long(message, "goods destroy container extend ID")
            {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let (goods_id, requested_amount) = if container_extend_id == 0 {
                (None, 0)
            } else {
                let goods_id = match message.base_mut().get_guid() {
                    Some(value) => value,
                    None => {
                        return Some(Err(GameGoodsMessageError::MissingField(
                            "goods destroy goods GUID",
                        )));
                    }
                };
                let requested_amount = match read_long(message, "goods destroy amount") {
                    Ok(value) => value as u32,
                    Err(error) => return Some(Err(error)),
                };
                (Some(goods_id), requested_amount)
            };
            game.open_goods_destroy(
                player_id,
                container_extend_id,
                goods_id,
                requested_amount,
                runtime,
            );
        }
        CONFIRM_GOODS_DESTROY => {
            game.confirm_goods_destroy(player_id);
        }
        CLOSE_EQUIPMENT_DA_KONG => {
            let session_id = match read_long(message, "equipment DaKong close session ID") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let requested_plug_id = match read_long(message, "equipment DaKong close plug ID") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let _ = game.close_equipment_da_kong(
                player_id,
                session_id,
                requested_plug_id,
                runtime,
            );
        }
        EQUIPMENT_DA_KONG
        | EQUIPMENT_ENCHASE_GEM
        | EQUIPMENT_CHANGE_ROLE_COLOR
        | EQUIPMENT_QUERY_DA_KONG_RESULT
        | EQUIPMENT_DESTROY_GEM => {
            let session_id = match read_long(message, "equipment DaKong session ID") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let requested_plug_id = match read_long(message, "equipment DaKong plug ID") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let operation = match message_type {
                EQUIPMENT_DA_KONG => {
                    let color_index = match read_long(message, "DaKong color index") {
                        Ok(value) => value,
                        Err(error) => return Some(Err(error)),
                    };
                    EquipmentDaKongOperation::DaKong { color_index }
                }
                EQUIPMENT_ENCHASE_GEM => {
                    let parameter = match read_long(message, "enchase gem parameter") {
                        Ok(value) => value,
                        Err(error) => return Some(Err(error)),
                    };
                    EquipmentDaKongOperation::EnchaseGem { parameter }
                }
                EQUIPMENT_CHANGE_ROLE_COLOR => {
                    let socket = match read_long(message, "DaKong color socket") {
                        Ok(value) => value,
                        Err(error) => return Some(Err(error)),
                    };
                    EquipmentDaKongOperation::ChangeRoleColor { socket }
                }
                EQUIPMENT_QUERY_DA_KONG_RESULT => EquipmentDaKongOperation::QueryResult,
                EQUIPMENT_DESTROY_GEM => {
                    let socket = match read_long(message, "destroy gem socket") {
                        Ok(value) => value as u32,
                        Err(error) => return Some(Err(error)),
                    };
                    EquipmentDaKongOperation::DestroyGem { socket }
                }
                _ => unreachable!("DaKong opcode отфильтрован outer match"),
            };
            let _ = game.process_equipment_da_kong(
                player_id,
                session_id,
                requested_plug_id,
                operation,
                runtime,
            );
        }
        COMPOSE_EQUIPMENT => {
            let session_id = match read_long(message, "equipment compose session ID") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let requested_plug_id = match read_long(message, "equipment compose plug ID") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let _ = game.compose_equipment(
                player_id,
                session_id,
                requested_plug_id,
                runtime,
            );
        }
        END_GOODS_SESSION => {
            let session_id = match read_long(message, "goods session ID") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let requested_plug_id = match read_long(message, "goods session plug ID") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            if game.session_factory().query_session(session_id).is_some() {
                let actual_plug_id = game
                    .session_factory()
                    .query_session_plug_by_owner(session_id, 400, player_id)
                    .map(|plug| plug.id());
                if let Some(actual_plug_id) = actual_plug_id {
                    if actual_plug_id == requested_plug_id {
                        let _ = game.session_factory_mut().end_session(session_id);
                        let _ = game
                            .find_player_mut(player_id)
                            .map(|player| player.release_goods_session_state());
                        let _ = game
                            .session_factory_mut()
                            .exit_plug(session_id, actual_plug_id);
                    }
                }
            }
        }
        CHECK_BATTLE_FAIRY_COMBINE => {
            game.check_battle_fairy_combine(player_id);
        }
        COMBINE_BATTLE_FAIRY => {
            game.combine_battle_fairy(player_id, runtime)
                .expect("resolved message player остаётся в CGame во время synchronous dispatch");
        }
        UPGRADE_BATTLE_FAIRY => {
            let _ = game
                .upgrade_battle_fairy_equipment(player_id, runtime)
                .expect("resolved message player остаётся в CGame во время synchronous dispatch");
        }
        RESET_BATTLE_FAIRY_SKILLS => {
            let _ignored_value = match read_long(message, "skill reset ignored value") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let script_index = match read_long(message, "skill reset script index") {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let script_path =
                format!("scripts/skills/restskills_0{script_index}.script").into_bytes();
            let Some(_detached_skill_ids) = game.detach_battle_fairy_script_skills(player_id) else {
                return Some(Ok(()));
            };
            let _ = game.run_script_file(
                &script_path,
                ScriptExecutionContext {
                    player_id: Some(player_id),
                    npc_id: None,
                    region_id,
                    ..ScriptExecutionContext::default()
                },
                runtime,
            );
            game.attach_battle_fairy_script_skills(player_id);
            let _ = CMessage::new(0x0b_f931).send(game, player_id != 0);
        }
        ALLOCATE_BATTLE_FAIRY_POTENTIAL => {
            let count = match read_long(message, "allocation count") {
                Ok(count) => count,
                Err(error) => return Some(Err(error)),
            };
            if let Err(error) = read_long(message, "allocation reserved value") {
                return Some(Err(error));
            }
            let mut allocations = Vec::new();
            for _ in 0..count.max(0) {
                let property = match read_long(message, "allocation property") {
                    Ok(property) => property,
                    Err(error) => return Some(Err(error)),
                };
                let points = match read_long(message, "allocation points") {
                    Ok(points) => points,
                    Err(error) => return Some(Err(error)),
                };
                allocations.push((property, points));
            }
            let _ = game
                .allocate_battle_fairy_potential(player_id, &allocations, runtime)
                .expect("resolved message player остаётся в CGame во время synchronous dispatch");
        }
        RESET_BATTLE_FAIRY_POTENTIAL => {
            let _ = game
                .reset_battle_fairy_potential(player_id, runtime)
                .expect("resolved message player остаётся в CGame во время synchronous dispatch");
        }
        SUMMON_BATTLE_FAIRY | RECALL_BATTLE_FAIRY => {
            let mode = if message_type == SUMMON_BATTLE_FAIRY {
                1
            } else {
                -1
            };
            game.summon_battle_fairy(player_id, mode).expect(
                "resolved message player остаётся в CGame во время synchronous dispatch",
            );
        }
        REFRESH_BATTLE_FAIRY_PROPERTY => {
            let _ignored_guid = match message.base_mut().get_guid() {
                Some(guid) => guid,
                None => return Some(Err(GameGoodsMessageError::MissingField("ignored GUID"))),
            };
            let goods_guid = match message.base_mut().get_guid() {
                Some(guid) => guid,
                None => return Some(Err(GameGoodsMessageError::MissingField("goods GUID"))),
            };
            let goods_exists = game
                .find_player(player_id)
                .is_some_and(|player| player.get_goods_by_id(goods_guid).is_some());
            if goods_exists {
                let _ = game
                    .update_player_properties(player_id, runtime)
                    .expect("0x8FC2E player сохранён после four-container lookup");
            }
        }
        QUERY_CI_QING_GOODS => {
            game.query_ci_qing_goods(player_id, runtime)
                .expect("resolved message player остаётся в CGame во время synchronous dispatch");
        }
        QUERY_CI_QING_SETUP => {
            game.query_ci_qing_setup(player_id);
        }
        MAKE_CI_QING_NODE => {
            if !game.ci_qing_message_enabled(player_id) {
                tracing::trace!(player_id, "CiQing недоступен");
            } else {
                let base_index = match read_long(message, "CiQing make base index") {
                    Ok(value) => value as u32,
                    Err(error) => return Some(Err(error)),
                };
                let amount = match read_long(message, "CiQing make amount") {
                    Ok(value) => value as u32,
                    Err(error) => return Some(Err(error)),
                };
                game.make_ci_qing_node(player_id, base_index, amount, runtime)
                    .expect(
                        "resolved message player остаётся в CGame во время synchronous dispatch",
                    );
            }
        }
        COMPOSE_CI_QING_NODE => {
            if !game.ci_qing_message_enabled(player_id) {
                tracing::trace!(player_id, "CiQing недоступен");
            } else {
                game.compose_ci_qing_node(player_id, runtime).expect(
                    "resolved message player остаётся в CGame во время synchronous dispatch",
                );
            }
        }
        DELETE_CI_QING_GOODS => {
            if !game.ci_qing_message_enabled(player_id) {
                tracing::trace!(player_id, "CiQing недоступен");
            } else {
                let position = match read_long(message, "CiQing delete position") {
                    Ok(value) => value as u32,
                    Err(error) => return Some(Err(error)),
                };
                if position >= 8 {
                    tracing::trace!(player_id, position, "позиция CiQing вне диапазона");
                } else {
                    let _ = game
                        .delete_goods_from_ci_qing(player_id, position, runtime)
                        .expect(
                            "resolved message player остаётся в CGame во время synchronous dispatch",
                        );
                }
            }
        }
        MOUNT_CI_QING_FROM_HAND => {
            let amount = match read_long(message, "CiQing mount amount") {
                Ok(value) => value as u32,
                Err(error) => return Some(Err(error)),
            };
            if !game.ci_qing_message_enabled(player_id) {
                tracing::trace!(player_id, "CiQing недоступен");
            } else {
                let _ = game
                    .mount_ci_qing_from_hand(player_id, amount, runtime)
                    .expect(
                        "resolved message player остаётся в CGame во время synchronous dispatch",
                    );
            }
        }
        QUERY_CI_QING_OTHER_PERSON => {
            let mode = match message.base_mut().get_char() {
                Some(mode) => mode,
                None => {
                    return Some(Err(GameGoodsMessageError::MissingField(
                        "CiQing target mode",
                    )));
                }
            };
            let target = match mode {
                0 => match read_long(message, "CiQing target player ID") {
                    Ok(player_id) => CiQingOtherPersonTarget::Id(player_id),
                    Err(error) => return Some(Err(error)),
                },
                1 => match message.base_mut().get_str_bytes(0x32) {
                    Some(name) => CiQingOtherPersonTarget::Name(name),
                    None => {
                        return Some(Err(GameGoodsMessageError::MissingField(
                            "CiQing target name",
                        )));
                    }
                },
                mode => CiQingOtherPersonTarget::UnsupportedMode(mode),
            };
            game.query_ci_qing_other_person(player_id, target, runtime);
        }
        _ => unreachable!("opcode отфильтрован перед dispatch"),
    }
    Some(Ok(()))
}

// IMPLEMENTED: оставшиеся cases `0x8FC0F/0x8FC10` материализованы выше;
// полностью замещённое RAW-тело `OnGoodsMessage` удалено.

// COMPONENT_VARIANT_END: GameServer
