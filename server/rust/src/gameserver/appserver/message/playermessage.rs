//! Диспетчер сообщений игрока GameServer.
//!
//! Источник — точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/message/playermessage.cpp`. Реализованы распределение
//! характеристик, возрождение, взаимодействие с NPC, использование предметов,
//! разрешения PvP, обмен, дружба, отображение снаряжения, время, титулы и
//! награды LeiTing. Имена, размеры полей, коды сообщений и порядок отправок
//! клиенту и WorldServer сохранены побайтово.
//!
//! Мутации игрока, контейнеров, сессий и состояния выполняются синхронно в
//! исходных точках. Отложенных эффектов у этого владельца нет, поэтому он не
//! записывает их в `GameEffectJournal`. Результаты отправок не влияют на
//! дальнейшее управление; причины отказов и диагностические значения
//! публикуются через `tracing`, а не возвращаются деревом отчётов.
//!
//! `CGame::run_script_file` получает фактический контекст игрока, NPC и региона.
//! `0x8FA04` сохраняет порядок проверок, проход свойств предмета, мутации
//! навыков и состояний, а затем расход и публикации `0xBF709/0xC0101/0xC0102`.
//! ReUseSkillItem после reuse gate вызывает общий DelSkill (0x004CF320),
//! включая current End(0), затем AddSkill (0x004D1C70); только успешная
//! регистрация получает SetItemPos и packet. Равный уровень не отменяет
//! обязательное пересоздание экземпляра в этой предметной ветви.
//! Ride-ветка UseItem0x00453C21..00453D12 читает текущие состояние/fight/progress:
//! первый Ride получает один End и отдельный Update; новый Mount вызывается
//! только после gates, а Update/consume остаются у caller-а. Раннего clock
//! в общем прологе нет: часы принадлежат Begin и конкретным timed-item веткам.
//! ReUseSkillItem0x00433610 ищет timestamp по QueryGoodsIDByName(goods.name);
//! trigger0 и отсутствующий ключ часов не читают. Существующий ключ требует
//! raw DWORD reuse, затем одного clock и wrapping now-last > reuse.
//! Процентный HP recovery сохраняет промежуточный `float`, процентный MP —
//! x87-произведение без такого сохранения; оба результата усекаются к нулю.
//! `0x8FA06/07/0B/0C` сохраняют границы частичных изменений сессии обмена.
//! `0x8FA15` завершает первый `CHBYState`, затем пересчитывает и публикует
//! свойства игрока. `0x8FA19` сохраняет порядок
//! `0xBF73E -> 0x5FD10 -> reward script`. Преобразование времени снаряжения
//! остаётся границей местного CRT; ветвь `0x8FA16` усекает `difftime` к нулю
//! перед знаковым делением на минуты. Четыре пустые ветви native switch явно
//! поглощаются как no-op.

use crate::gameserver::appserver::cs2ccontainerobjectamountchange::CS2CContainerObjectAmountChange;
use crate::gameserver::appserver::cs2ccontainerobjectmove::{
    CS2CContainerObjectMove, ContainerObjectMoveOperation,
};
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_CHANGEBODY_TYPE, GAP_EQUIP_STATE, GAP_MOUNT_LEVEL, GAP_MOUNT_PLAYER_ROLE_LIMIT,
    GAP_MOUNT_TYPE, GAP_SKILL_ID, GAP_SKILL_LEVEL, GAP_UNLIMITED_ACCESS, GOODS_TYPE_CONSUMABLE,
};
use crate::gameserver::appserver::player::{
    CiQingPacketConsumption, PlayerFriendAddOutcome, PlayerProgress,
};
use crate::gameserver::appserver::script::function::ScriptFunctionRuntime;
use crate::gameserver::appserver::script::script::ScriptExecutionContext;
use crate::gameserver::appserver::skills::fightdefense::truncate_original;
use crate::gameserver::gameserver::game::{
    CGame, GameContainerMessageRuntime, PlayerReliveContext, colored_player_notice_message,
    format_legacy_text_fields, game_wall_time_seconds,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::public::date::TagTime;
use nebokrai_shared::values::CGuid;

const ALLOCATE_STAT_POINT: u32 = 0x0008_fa01;
const REQUEST_RELIVE: u32 = 0x0008_fa02;
const INTERACT_WITH_NPC: u32 = 0x0008_fa03;
const USE_PACKET_ITEM: u32 = 0x0008_fa04;
const SET_PK_PERMISSION: u32 = 0x0008_fa05;
const REQUEST_TRADE: u32 = 0x0008_fa06;
const ANSWER_TRADE: u32 = 0x0008_fa07;
const LEGACY_NO_OP_08: u32 = 0x0008_fa08;
const LEGACY_NO_OP_09: u32 = 0x0008_fa09;
const LEGACY_NO_OP_0A: u32 = 0x0008_fa0a;
const TOGGLE_TRADE_READY: u32 = 0x0008_fa0b;
const ABORT_TRADE: u32 = 0x0008_fa0c;
const RUN_HELP_SCRIPT: u32 = 0x0008_fa10;
const REQUEST_FRIEND: u32 = 0x0008_fa0d;
const ANSWER_FRIEND: u32 = 0x0008_fa0e;
const DELETE_FRIEND: u32 = 0x0008_fa0f;
const SET_DISPLAY_HEAD_PIECE: u32 = 0x0008_fa11;
const QUERY_QUEST_TIME: u32 = 0x0008_fa12;
const ACKNOWLEDGE_HEARTBEAT: u32 = 0x0008_fa13;
const LEGACY_NO_OP_14: u32 = 0x0008_fa14;
const END_CHANGE_BODY_STATE: u32 = 0x0008_fa15;
const REFRESH_EXPIRED_EQUIPMENT_STATE: u32 = 0x0008_fa16;
const QUERY_HONOR_IDENTITY: u32 = 0x0008_fa17;
const REQUEST_CHANGE_APPELLATION: u32 = 0x0008_fa18;
const QUERY_LOCAL_TIME: u32 = 0x0008_fa1a;
const CLAIM_LEI_TING_REWARD: u32 = 0x0008_fa19;

pub(crate) const PLAYER_ITEM_BLOCKING_SKILL_IDS: [u32; 7] = [103, 115, 124, 221, 118, 402, 504];

const LEI_TING_REWARD_SCRIPTS: [&[u8]; 9] = [
    b"scripts/goods/leilifengxing_lingqu_20.script",
    b"scripts/goods/leilifengxing_lingqu_60.script",
    b"scripts/goods/leilifengxing_lingqu_80.script",
    b"scripts/goods/leilifengxing_lingqu_100.script",
    b"scripts/goods/leilifengxing_lingqu_4.script",
    b"scripts/goods/leilifengxing_lingqu_10.script",
    b"scripts/goods/leilifengxing_lingqu_16.script",
    b"scripts/goods/leilifengxing_lingqu_22.script",
    b"scripts/goods/leilifengxing_lingqu_28.script",
];

pub(crate) trait GamePlayerMessageRuntime:
    PlayerReliveContext + GameContainerMessageRuntime + ScriptFunctionRuntime
{}

impl<T> GamePlayerMessageRuntime for T where
    T: PlayerReliveContext + GameContainerMessageRuntime + ScriptFunctionRuntime + ?Sized
{}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerItemUseFacts {
    pub(crate) blocking_skill_state: bool,
    pub(crate) fight_state_count: i32,
    pub(crate) forbid_return_level: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerItemSkillWire {
    pub(crate) value_84: u32,
    pub(crate) value_70: u16,
    pub(crate) value_74: u16,
    pub(crate) value_78: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerPackedLocalTime {
    pub(crate) year_since_1900: i32,
    pub(crate) zero_based_month: i32,
    pub(crate) day: i32,
    pub(crate) hour: i32,
    pub(crate) minute: i32,
    pub(crate) second: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerMessageError {
    MissingField(&'static str),
}

fn add_c_string(message: &mut CMessage, value: &[u8]) {
    let value = value.split(|byte| *byte == 0).next().unwrap_or_default();
    message.base_mut().add(value);
    message.base_mut().add_byte(0);
}

fn send_trade_notice(game: &CGame, player_id: i32, string_id: &[u8]) -> i32 {
    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(string_id))
        .send_to_player(game.net_server(), player_id)
}

fn send_item_notice(
    game: &CGame,
    player_id: i32,
    string_id: &[u8],
    arguments: &[&[u8]],
    secondary_color: u32,
) -> i32 {
    let text = format_legacy_text_fields(game.get_string_by_id(string_id), arguments, 0xff);
    colored_player_notice_message(0xffff_ffff, secondary_color, &text)
        .send_to_player(game.net_server(), player_id)
}

fn send_item_skill_wire(
    game: &CGame,
    player_id: i32,
    message_type: i32,
    prefix: Option<u8>,
    skill_id: u32,
    skill_level: i32,
    wire: PlayerItemSkillWire,
) -> i32 {
    let mut message = CMessage::new(message_type);
    if let Some(prefix) = prefix {
        message.add_byte(prefix);
        message.add_ulong(skill_id);
        message.add_ulong(skill_level as u32);
    } else {
        add_c_string(
            &mut message,
            game.skill_factory()
                .query_skill_name(skill_id as i32)
                .unwrap_or_default(),
        );
        message.base_mut().add_short(skill_level as i16);
    }
    message.add_ulong(wire.value_84);
    message.base_mut().add_short(wire.value_70 as i16);
    message.base_mut().add_short(wire.value_74 as i16);
    message.base_mut().add_short(wire.value_78 as i16);
    message.send_to_player(game.net_server(), player_id)
}

fn item_skill_wire(game: &CGame, skill_id: u32, skill_level: i32) -> Option<PlayerItemSkillWire> {
    const USER_MP_LOSE: u32 = 2;
    const TARGET_MIN_DISTANCE: u32 = 5_002;
    const TARGET_MAX_DISTANCE: u32 = 5_003;
    const REUSE_DELAY: u32 = 10_005;
    let properties = game.skill_base_properties(skill_id, skill_level)?;
    let minimum = properties.query_property(TARGET_MIN_DISTANCE);
    let maximum = properties.query_property(TARGET_MAX_DISTANCE);
    Some(PlayerItemSkillWire {
        value_84: properties.query_property(REUSE_DELAY),
        value_70: if minimum as i32 > 0 { minimum as u16 } else { 1 },
        value_74: if maximum as i32 > 0 { maximum as u16 } else { 1 },
        value_78: properties.query_property(USER_MP_LOSE) as u16,
    })
}

fn publish_packet_item_consumption(
    game: &mut CGame,
    player_id: i32,
    slot: u8,
    goods_id: CGuid,
    identity_type: i32,
    base_index: u32,
) -> Option<CiQingPacketConsumption> {
    let mut used = CMessage::new(0x000b_f709);
    used.add_byte(b'3');
    used.add_long(player_id);
    used.add_ulong(base_index);
    let _ = game.send_player_shape_around(player_id, None, &used);
    let consumption = game
        .find_player_mut(player_id)?
        .remove_packet_goods_by_id(goods_id, 1)?;
    if consumption.remaining_amount == 0 {
        let mut deleted = CS2CContainerObjectMove::default();
        deleted.set_operation(ContainerObjectMoveOperation::DeleteObject);
        deleted.set_source_container(400, player_id, u32::from(slot));
        deleted.set_source_container_extend_id(1);
        deleted.set_source_object(identity_type, goods_id, 0);
        let _ = deleted.send_to_player(game, player_id);
    } else {
        let mut changed = CS2CContainerObjectAmountChange::default();
        changed.set_source_container(400, player_id, u32::from(slot));
        changed.set_source_container_extend_id(1);
        changed.set_object(identity_type, goods_id);
        changed.set_object_amount(consumption.remaining_amount);
        let _ = changed.send_to_player(game, player_id);
    }
    Some(consumption)
}

fn publish_friend_add(
    game: &CGame,
    owner_id: i32,
    friend_id: i32,
) -> Result<i32, SendMessageError> {
    let mut message = CMessage::new(0x0006_0501);
    message.add_long(owner_id);
    message.add_long(friend_id);
    message.send(game, false)
}

fn publish_friend_delete(
    game: &CGame,
    owner_id: i32,
    friend_id: i32,
    friend_name: &[u8],
) -> Result<i32, SendMessageError> {
    let mut message = CMessage::new(0x0006_0502);
    message.add_long(owner_id);
    message.add_long(friend_id);
    add_c_string(&mut message, friend_name);
    message.send(game, false)
}

fn publish_lei_ting_update(game: &CGame, player_id: i32) {
    let payload = game
        .find_player(player_id)
        .expect("LeiTing player сохранён после flag mutation")
        .encode_lei_ting();
    let mut client = CMessage::new(0x000b_f73e);
    client.base_mut().add(&payload);
    let _ = client.send_to_player(game.net_server(), player_id);
    let mut world = CMessage::new(0x0005_fd10);
    world.add_long(player_id);
    world.base_mut().add(&payload);
    let _ = world.send(game, false);
}

fn decode_equipment_state_local_time(packed: i32) -> PlayerPackedLocalTime {
    PlayerPackedLocalTime {
        year_since_1900: packed >> 23,
        zero_based_month: (packed >> 19) & 0x0f,
        day: (packed >> 14) & 0x1f,
        hour: (packed >> 9) & 0x1f,
        minute: (packed >> 3) & 0x3f,
        second: 0,
    }
}

/// Exact local `mktime`/`time`/`difftime` owner packed equipment timestamp.
/// `tm_isdst=-1` сохраняет CRT-выбор DST, `-1` остаётся invalid boundary.
pub(crate) fn equipment_state_elapsed_seconds(packed: i32) -> Option<f64> {
    let local = decode_equipment_state_local_time(packed);
    let mut native = libc::tm {
        tm_sec: local.second,
        tm_min: local.minute,
        tm_hour: local.hour,
        tm_mday: local.day,
        tm_mon: local.zero_based_month,
        tm_year: local.year_since_1900,
        tm_isdst: -1,
        ..unsafe { std::mem::zeroed() }
    };
    let expiry = unsafe { libc::mktime(&raw mut native) };
    if expiry == -1 {
        return None;
    }
    let now = unsafe { libc::time(std::ptr::null_mut()) };
    (now != -1).then(|| unsafe { libc::difftime(now, expiry) })
}

fn apply_friend_add(
    game: &mut CGame,
    owner_id: i32,
    friend_name: &[u8],
    online_friend_id: Option<i32>,
) {
    let Some(outcome) = game
        .find_player_mut(owner_id)
        .map(|player| player.add_friend_state(friend_name))
    else {
        return;
    };
    match outcome {
        PlayerFriendAddOutcome::Added => {
            if let Some(friend_id) = online_friend_id {
                let _ = publish_friend_add(game, owner_id, friend_id);
            }
        }
        PlayerFriendAddOutcome::AlreadyPresent => {}
        PlayerFriendAddOutcome::LimitReached => {
            let _ = colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0155"))
                .send_to_player(game.net_server(), owner_id);
        }
    }
}

fn trace_player_message_outcome(message_type: u32, player_id: Option<i32>, outcome: &'static str) {
    tracing::trace!(
        message_type,
        player_id,
        outcome,
        "обработано сообщение игрока"
    );
}

pub(crate) fn dispatch_game_player_message<Runtime: GamePlayerMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<(), GamePlayerMessageError>> {
    let message_type = message.message_type() as u32;
    if !matches!(
        message_type,
        ALLOCATE_STAT_POINT
            | REQUEST_RELIVE
            | INTERACT_WITH_NPC
            | USE_PACKET_ITEM
            | SET_PK_PERMISSION
            | REQUEST_TRADE
            | ANSWER_TRADE
            | LEGACY_NO_OP_08
            | LEGACY_NO_OP_09
            | LEGACY_NO_OP_0A
            | TOGGLE_TRADE_READY
            | ABORT_TRADE
            | RUN_HELP_SCRIPT
            | REQUEST_FRIEND
            | ANSWER_FRIEND
            | DELETE_FRIEND
            | SET_DISPLAY_HEAD_PIECE
            | QUERY_QUEST_TIME
            | ACKNOWLEDGE_HEARTBEAT
            | LEGACY_NO_OP_14
            | END_CHANGE_BODY_STATE
            | REFRESH_EXPIRED_EQUIPMENT_STATE
            | QUERY_HONOR_IDENTITY
            | REQUEST_CHANGE_APPELLATION
            | QUERY_LOCAL_TIME
            | CLAIM_LEI_TING_REWARD
    ) {
        return None;
    }
    message.resolve_player_context(game);
    let player_id = message.player_id();
    let Some(player_id) = player_id else {
        trace_player_message_outcome(message_type, None, "нет контекста игрока");
        return Some(Ok(()));
    };
    if game
        .find_player(player_id)
        .is_some_and(|player| player.in_changing_server() || player.in_changing_region())
    {
        trace_player_message_outcome(message_type, Some(player_id), "игрок меняет локацию");
        return Some(Ok(()));
    }

    match message_type {
        ALLOCATE_STAT_POINT => {
            let Some(state) = game
                .find_player(player_id)
                .map(|player| player.stat_allocation_state())
            else {
                return Some(Ok(()));
            };
            if state.remain_point == 0 {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "нет очков характеристик",
                );
                return Some(Ok(()));
            }
            let Some(selector) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField("stat selector")));
            };
            let (constitution_hp, intelligence_mp) = if matches!(selector, 2 | 3) {
                let properties = game
                    .player_list_mut()
                    .creation_properties(state.sex, state.occupation)
                    .properties;
                (
                    properties.constitution_to_maximum_hp,
                    properties.intelligence_to_maximum_mp,
                )
            } else {
                (0, 0)
            };
            let _ = game
                .find_player_mut(player_id)
                .expect("stat-allocation player сохранён после context lookup")
                .allocate_stat_point(selector as u8, constitution_hp, intelligence_mp);
            let properties = game
                .recompute_player_properties_for_update(player_id)
                .expect("stat-allocation player сохранён после mutation");
            let _applied = game.apply_recomputed_player_properties(player_id, properties);
            let (wire, base_maximum_hp, base_maximum_mp) = {
                let player = game
                    .find_player(player_id)
                    .expect("stat-allocation player сохранён до property response");
                let state = player.stat_allocation_state();
                (
                    *player.combat_property_wire(),
                    state.base_maximum_hp,
                    state.base_maximum_mp,
                )
            };
            let mut response = CMessage::new(0x000b_f702);
            response.base_mut().add(&wire);
            response.base_mut().add_ulong(base_maximum_hp);
            response.base_mut().add_ulong(base_maximum_mp);
            let _ = response.send_to_player(game.net_server(), player_id);
            trace_player_message_outcome(
                message_type,
                Some(player_id),
                "очки характеристик распределены",
            );
        }
        REQUEST_RELIVE => {
            game.relive_player(player_id, 0, || runtime.now_milliseconds());
            trace_player_message_outcome(message_type, Some(player_id), "игрок возрождён");
        }
        INTERACT_WITH_NPC => {
            let Some(region_id) = message
                .region_id()
                .filter(|region_id| game.find_region(*region_id).is_some())
            else {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "регион взаимодействия с NPC отсутствует",
                );
                return Some(Ok(()));
            };
            let Some(player) = game.find_player(player_id) else {
                return Some(Ok(()));
            };
            if player.is_dead() || player.current_progress() != PlayerProgress::None {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "взаимодействие с NPC заблокировано",
                );
                return Some(Ok(()));
            }
            let Some(npc_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField("NPC id")));
            };
            tracing::trace!(
                message_type,
                player_id,
                npc_id,
                "проверяется взаимодействие с NPC"
            );
            let target = game.find_region(region_id).and_then(|region| {
                let player = game.find_player(player_id)?;
                let npc = region.base().find_npc_by_id(npc_id)?;
                if !player
                    .shape()
                    .is_in_around(npc.move_shape().shape(), region.base())
                {
                    return None;
                }
                let distance = npc.shape_view()?.distance(player.shape_view()?);
                Some((distance, npc.script_file().to_vec()))
            });
            let Some((distance, script_file)) = target else {
                trace_player_message_outcome(message_type, Some(player_id), "цель NPC отсутствует");
                return Some(Ok(()));
            };
            tracing::trace!(
                message_type,
                player_id,
                npc_id,
                distance,
                "найден NPC для взаимодействия"
            );
            if distance >= 9 {
                let notification =
                    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0057"));
                let _ = notification.send_to_player(game.net_server(), player_id);
                trace_player_message_outcome(message_type, Some(player_id), "NPC слишком далеко");
                return Some(Ok(()));
            }
            if script_file.first() == Some(&b'0') {
                trace_player_message_outcome(message_type, Some(player_id), "скрипт NPC подавлен");
                return Some(Ok(()));
            }
            tracing::trace!(
                message_type,
                player_id,
                npc_id,
                script_data_present = game.script_file_data(&script_file).is_some(),
                "передан скрипт NPC"
            );
            let _ = game.run_script_file(
                &script_file,
                ScriptExecutionContext {
                    player_id: Some(player_id),
                    npc_id: Some(npc_id),
                    region_id: Some(region_id),
                    ..ScriptExecutionContext::default()
                },
                runtime,
            );
            trace_player_message_outcome(message_type, Some(player_id), "скрипт NPC запущен");
        }
        USE_PACKET_ITEM => {
            let Some(player) = game.find_player(player_id) else {
                return Some(Ok(()));
            };
            if player.is_dead()
                || matches!(
                    player.current_progress(),
                    PlayerProgress::Trading
                        | PlayerProgress::Shopping
                        | PlayerProgress::OpenStall
                        | PlayerProgress::Increment
                        | PlayerProgress::Synthesis
                )
            {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "использование предмета заблокировано",
                );
                return Some(Ok(()));
            }
            let facts = game
                .find_player(player_id)
                .map(|player| PlayerItemUseFacts {
                    blocking_skill_state: PLAYER_ITEM_BLOCKING_SKILL_IDS
                        .iter()
                        .copied()
                        .any(|state_id| player.has_state_by_skill_id(state_id)),
                    fight_state_count: player.fight_state_count(),
                    forbid_return_level: game.globe_setup().forbid_return_level(),
                })
                .unwrap_or_default();
            if facts.blocking_skill_state {
                let _ = send_item_notice(game, player_id, b"GS0146", &[], 0);
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "использование предмета заблокировано",
                );
                return Some(Ok(()));
            }
            if game
                .find_player(player_id)
                .is_some_and(|player| player.script_move_state_count(110000) != 0)
            {
                let _ = game.end_script_auto_protect_state(player_id);
            }
            let Some(slot) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "packet item slot",
                )));
            };
            let slot = slot as u8;
            tracing::trace!(
                message_type,
                player_id,
                slot,
                "проверяется предмет в рюкзаке"
            );
            let Some(goods) = game
                .find_player(player_id)
                .and_then(|player| player.packet().get_goods(u32::from(slot)))
                .cloned()
            else {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "использование предмета отклонено",
                );
                return Some(Ok(()));
            };
            let goods_id = goods.identity().ex_id;
            let identity_type = goods.identity().object_type;
            let base_index = goods.base_properties_index();
            tracing::trace!(
                message_type,
                player_id,
                ?goods_id,
                "найден предмет в рюкзаке"
            );
            if game.change_body_item_conflicts(player_id, base_index) {
                let _ = send_item_notice(game, player_id, b"GSN0337", &[goods.name()], 0);
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "использование предмета отклонено",
                );
                return Some(Ok(()));
            }
            let Some(region_id) = game
                .find_player(player_id)
                .and_then(|player| player.server_region_id())
            else {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "использование предмета отклонено",
                );
                return Some(Ok(()));
            };
            let Some(base_properties) =
                game.goods_factory().query_goods_base_properties(base_index)
            else {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "использование предмета отклонено",
                );
                return Some(Ok(()));
            };
            if base_properties.goods_type() != GOODS_TYPE_CONSUMABLE {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "использование предмета отклонено",
                );
                return Some(Ok(()));
            }
            let original_name = base_properties.original_name().to_vec();
            let Some(region) = game.find_region(region_id) else {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "использование предмета отклонено",
                );
                return Some(Ok(()));
            };
            let region_country = region.base().country;
            let forbidden = region.base().find_forbid_good(&original_name);
            let can_use = if forbidden {
                8
            } else {
                game.find_player(player_id)
                    .expect("item-use player сохранён")
                    .can_use_item(&goods, game.goods_factory())
            };
            tracing::trace!(
                message_type,
                player_id,
                can_use,
                "получен результат проверки предмета"
            );
            if can_use != 9 {
                let mut failure = CMessage::new(0x000b_f709);
                failure.add_byte(b'4');
                failure.add_byte(can_use as u8);
                let _ = failure.send_to_player(game.net_server(), player_id);
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "использование предмета отклонено",
                );
                return Some(Ok(()));
            }
            if goods.amount() == 0 {
                let _ = game
                    .find_player_mut(player_id)
                    .and_then(|player| player.remove_packet_goods_by_id(goods_id, 1));
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "использование предмета отклонено",
                );
                return Some(Ok(()));
            }
            if game.find_player(player_id).is_some_and(|player| player.contend_state())
                && game.cancel_player_contend(player_id)
            {
                let _ = send_item_notice(game, player_id, b"GS0147", &[], 0xffff_0000);
            }

            let mut consume = true;
            let mut return_after_use = false;
            let change_body_type =
                goods.addon_property_value(game.goods_factory(), GAP_CHANGEBODY_TYPE, 1);
            if change_body_type != 0 && game.script_change_body_check(player_id, false) == 0 {
                let _ = send_item_notice(game, player_id, b"GSN1063", &[], 0);
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "использование предмета отклонено",
                );
                return Some(Ok(()));
            }

            let mount_type = goods.addon_property_value(game.goods_factory(), GAP_MOUNT_TYPE, 1);
            if mount_type != 0 {
                consume = false;
                if game.find_player(player_id).is_some_and(|player| player.is_rider()) {
                    let _ended = game.end_player_ride(player_id);
                    let properties = game
                        .recompute_player_properties_for_update(player_id)
                        .expect("mount player сохранён");
                    game.apply_player_state_properties(player_id, properties);
                } else if game.find_player(player_id).is_some_and(|player| {
                    player.current_progress() != PlayerProgress::OpenStall
                        && player.fight_state_count() == 0
                        && player.appearance_and_mode().2 == 0
                }) {
                    let applied = game.begin_player_ride(
                        player_id,
                        mount_type as u32,
                        goods.addon_property_value(game.goods_factory(), GAP_MOUNT_LEVEL, 1) as u32,
                        goods.addon_property_value(
                            game.goods_factory(),
                            GAP_MOUNT_PLAYER_ROLE_LIMIT,
                            1,
                        ) as u32,
                        Some(&original_name),
                        &mut || runtime.now_milliseconds(),
                    );
                    if applied {
                        let properties = game
                            .recompute_player_properties_for_update(player_id)
                            .expect("mounted player сохранён");
                        game.apply_player_state_properties(player_id, properties);
                        consume = goods.addon_property_value(
                            game.goods_factory(),
                            GAP_UNLIMITED_ACCESS,
                            1,
                        ) != 1;
                    }
                }
            } else {
                let goods_factory = game.goods_factory().clone();
                for property in goods.enabled_addon_properties(&goods_factory) {
                    tracing::trace!(
                        message_type,
                        player_id,
                        property,
                        "применяется свойство предмета"
                    );
                    let value =
                        |value_id| goods.addon_property_value(&goods_factory, property, value_id);
                    match property {
                        0x21 | 0x43 => {
                            consume = game.begin_player_consumable_health_restore(
                                player_id,
                                value(1) as u32,
                                (value(2) as u32).wrapping_mul(400),
                                400,
                                || runtime.now_milliseconds(),
                            );
                        }
                        0x22 => {
                            let maximum = game
                                .find_player(player_id)
                                .expect("restore-hp player сохранён")
                                .maximum_health();
                            let percent = value(1) as f32 * 0.01_f32;
                            let amount = (f64::from(maximum) * f64::from(percent)).trunc()
                                as i32 as u32;
                            consume = game.begin_player_consumable_health_restore(
                                player_id,
                                amount,
                                0,
                                400,
                                || runtime.now_milliseconds(),
                            );
                        }
                        0x23 => {
                            let delay_ms = if value(2) == 0 {
                                0
                            } else {
                                (value(2) as u32).wrapping_sub(1).wrapping_mul(400)
                            };
                            consume = game.begin_player_consumable_mana_restore(
                                player_id,
                                value(1) as u32,
                                delay_ms,
                                400,
                                || runtime.now_milliseconds(),
                            );
                        }
                        0x24 => {
                            let maximum = game
                                .find_player(player_id)
                                .expect("restore-mp player сохранён")
                                .maximum_mana();
                            let amount = (f64::from(maximum)
                                * f64::from(value(1))
                                * f64::from(0.01_f32))
                            .trunc() as i32 as u32;
                            consume = game.begin_player_consumable_mana_restore(
                                player_id,
                                amount,
                                0,
                                400,
                                || runtime.now_milliseconds(),
                            );
                        }
                        0x27 if 0 <= value(2) => {
                            let skill_id = value(1) as u32;
                            let requested_level = value(2);
                            let current_level = game
                                .find_player(player_id)
                                .expect("skill-book player сохранён")
                                .item_skill_level(skill_id, game.skill_factory());
                            if requested_level <= current_level {
                                consume = false;
                                let _ = send_item_notice(game, player_id, b"GS0148", &[], 0);
                            } else if requested_level.wrapping_sub(current_level) != 1 {
                                consume = false;
                                let _ = send_item_notice(game, player_id, b"GS0149", &[], 0);
                            } else {
                                let skill_factory = game.skill_factory().clone();
                                let added = game
                                    .find_player_mut(player_id)
                                    .expect("skill-book player сохранён для add")
                                    .learn_item_skill(skill_id, requested_level, &skill_factory);
                                if added {
                                    if let Some(wire) =
                                        item_skill_wire(game, skill_id, requested_level)
                                    {
                                        let _ = send_item_skill_wire(
                                            game,
                                            player_id,
                                            0x000b_f71d,
                                            None,
                                            skill_id,
                                            requested_level,
                                            wire,
                                        );
                                    }
                                }
                            }
                        }
                        0x2d => {
                            if region_country != 0
                                && region_country
                                    != game
                                        .find_player(player_id)
                                        .expect("return player")
                                        .country()
                            {
                                consume = false;
                                let _ = send_item_notice(game, player_id, b"GS0150", &[], 0);
                            } else if game.find_player(player_id).is_some_and(|player| {
                                player.current_progress() == PlayerProgress::Synthesis
                            }) {
                                // EXE указывает только неразрешённый literal pointer;
                                // consume-guard подтверждён, текст не выдумывается.
                                consume = false;
                            } else if i32::from(
                                game.find_player(player_id).expect("return player").level(),
                            ) < facts.forbid_return_level
                                || facts.fight_state_count < 1
                            {
                                return_after_use = true;
                            } else {
                                consume = false;
                                let _ = send_item_notice(game, player_id, b"GS0151", &[], 0);
                            }
                        }
                        0x2e => {
                            if region_country
                                != game
                                    .find_player(player_id)
                                    .expect("random recall player")
                                    .country()
                            {
                                consume = false;
                                let _ = send_item_notice(game, player_id, b"GS0152", &[], 0);
                            } else if game.find_player(player_id).is_some_and(|player| {
                                player.current_progress() == PlayerProgress::Synthesis
                            }) {
                                consume = false;
                                let _ = send_item_notice(game, player_id, b"GS1041", &[], 0);
                            } else if i32::from(
                                game.find_player(player_id)
                                    .expect("random recall player")
                                    .level(),
                            ) >= facts.forbid_return_level
                                && 0 < facts.fight_state_count
                            {
                                consume = false;
                                let _ = send_item_notice(game, player_id, b"GS0153", &[], 0);
                            } else {
                                let _ = game.recall_player_inside_region(player_id);
                            }
                        }
                        0x2f => {
                            let path = format!("scripts/goods/{}.script", value(1)).into_bytes();
                            let ran = game
                                .run_script_file(
                                    &path,
                                    ScriptExecutionContext {
                                        player_id: Some(player_id),
                                        region_id: Some(region_id),
                                        used_item_id: Some(goods_id),
                                        ..ScriptExecutionContext::default()
                                    },
                                    runtime,
                                )
                                .is_some();
                            if !ran {
                                consume = false;
                            } else if goods.addon_property_value(
                                game.goods_factory(),
                                GAP_UNLIMITED_ACCESS,
                                1,
                            ) == 1
                            {
                                consume = false;
                            }
                        }
                        0x4a..=0x4d => {
                            let resulting = game
                                .find_player_mut(player_id)
                                .expect("expendable-effect player сохранён")
                                .apply_expendable_item_effect(
                                    property,
                                    value,
                                    || runtime.now_milliseconds(),
                                );
                            let mut update = CMessage::new(0x000b_f70a);
                            if property == 0x4b {
                                update.base_mut().add_short(resulting as i16);
                            } else {
                                update.add_long(resulting);
                            }
                            let _ = update.send_to_player(game.net_server(), player_id);
                        }
                        0x85 => {
                            consume = false;
                            const GAP_TRIGGER_SKILL: i32 = 133;
                            const GAP_REUSE_TIME: i32 = 134;
                            let skill_id =
                                goods.addon_property_value(game.goods_factory(), GAP_SKILL_ID, 1)
                                    as u32;
                            let skill_level = goods.addon_property_value(
                                game.goods_factory(),
                                GAP_SKILL_LEVEL,
                                1,
                            );
                            let item_index = game.goods_factory()
                                .query_goods_id_by_name(Some(goods.name()));
                            let reusable = goods.addon_property_value(
                                game.goods_factory(), GAP_TRIGGER_SKILL, 1,
                            ) != 0
                                && game.find_player(player_id).is_some_and(|player| {
                                    player.last_skill_item_use_ms(item_index).is_none_or(|last_used| {
                                        let reuse_time = goods.addon_property_value(
                                            game.goods_factory(), GAP_REUSE_TIME, 1,
                                        ) as u32;
                                        runtime.now_milliseconds().wrapping_sub(last_used) > reuse_time
                                    })
                                });
                            if reusable {
                                let holder = game
                                    .find_player(player_id)
                                    .expect("reuse-item player сохранён для skill replacement")
                                    .shape()
                                    .identity();
                                let _ = game.delete_move_shape_skill(region_id, holder, skill_id);
                                let replaced = game.add_move_shape_skill(
                                    region_id, holder, skill_id, skill_level,
                                );
                                if replaced {
                                    let skill_factory = game.skill_factory().clone();
                                    let _ = game
                                        .find_player_mut(player_id)
                                        .expect("reuse-item player сохранён для позиции")
                                        .set_item_skill_position(skill_id, i32::from(slot), &skill_factory);
                                }
                                if replaced
                                    && let Some(wire) = item_skill_wire(game, skill_id, skill_level)
                                {
                                    let _ = send_item_skill_wire(
                                        game,
                                        player_id,
                                        0x000b_fe07,
                                        Some(0x55),
                                        skill_id,
                                        skill_level,
                                        wire,
                                    );
                                }
                            } else {
                                let _ = send_item_notice(
                                    game,
                                    player_id,
                                    b"GS1178",
                                    &[goods.name()],
                                    0,
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
            if consume {
                let _ = publish_packet_item_consumption(
                    game,
                    player_id,
                    slot,
                    goods_id,
                    identity_type,
                    base_index,
                );
            }
            if return_after_use {
                let _ = game.recall_player_to_return_point(player_id);
            }
            trace_player_message_outcome(message_type, Some(player_id), "предмет использован");
        }
        SET_PK_PERMISSION => {
            if game.find_player(player_id).is_none() {
                return Some(Ok(()));
            }
            let Some(selector) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "PK permission selector",
                )));
            };
            let Some(value) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "PK permission value",
                )));
            };
            let _ = game
                .find_player_mut(player_id)
                .expect("PK-permission player сохранён после context lookup")
                .set_pk_permission(selector, value != 0);
            trace_player_message_outcome(message_type, Some(player_id), "разрешение PvP изменено");
        }
        RUN_HELP_SCRIPT => {
            let _ = game.run_script_file(
                b"scripts/help/help.script",
                ScriptExecutionContext {
                    player_id: Some(player_id),
                    ..ScriptExecutionContext::default()
                },
                runtime,
            );
            trace_player_message_outcome(message_type, Some(player_id), "скрипт игрока запущен");
        }
        REQUEST_TRADE => {
            let Some(target_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "trade target player id",
                )));
            };
            tracing::trace!(
                message_type,
                player_id,
                target_id,
                "проверяется предложение обмена"
            );
            let Some(requester) = game.find_player(player_id) else {
                return Some(Ok(()));
            };
            let notice = if requester.is_dead() {
                Some(b"GS0065".as_slice())
            } else if target_id == player_id {
                Some(b"GS0058".as_slice())
            } else if requester.current_progress() != PlayerProgress::None {
                Some(b"GS0059".as_slice())
            } else {
                match game.find_player(target_id) {
                    None => Some(b"GS0064".as_slice()),
                    Some(target) if target.is_dead() => Some(b"GS0063".as_slice()),
                    Some(target) if target.current_progress() != PlayerProgress::None => {
                        Some(b"GS0062".as_slice())
                    }
                    Some(_)
                        if game
                            .player_trade_distance(player_id, target_id)
                            .unwrap_or(9)
                            >= 9 =>
                    {
                        Some(b"GS0061".as_slice())
                    }
                    Some(_) => None,
                }
            };
            if let Some(string_id) = notice {
                let _ = send_trade_notice(game, player_id, string_id);
                trace_player_message_outcome(message_type, Some(player_id), "обмен предложен");
                return Some(Ok(()));
            }
            let mut invitation = CMessage::new(0x000b_f70f);
            invitation.add_long(player_id);
            let _ = invitation.send_to_player(game.net_server(), target_id);
            let _ = send_trade_notice(game, player_id, b"GS0060");
            trace_player_message_outcome(message_type, Some(player_id), "обмен предложен");
        }
        ANSWER_TRADE => {
            let Some(inviter_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "trade inviter player id",
                )));
            };
            let Some(accepted) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField("trade answer")));
            };
            tracing::trace!(
                message_type,
                player_id,
                inviter_id,
                "проверяется ответ на обмен"
            );
            if inviter_id == player_id {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "ответ на обмен обработан",
                );
                return Some(Ok(()));
            }
            let Some(answerer) = game.find_player(player_id) else {
                return Some(Ok(()));
            };
            if answerer.is_dead() {
                let _ = send_trade_notice(game, player_id, b"GS0065");
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "ответ на обмен обработан",
                );
                return Some(Ok(()));
            }
            let inviter_exists = game.find_player(inviter_id).is_some();
            if answerer.current_progress() != PlayerProgress::None {
                for string_id in [b"GS0068".as_slice(), b"GS0071".as_slice()]
                    .into_iter()
                    .take(if inviter_exists { 2 } else { 1 })
                {
                    let _ = send_trade_notice(game, player_id, string_id);
                }
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "ответ на обмен обработан",
                );
                return Some(Ok(()));
            }
            let Some(inviter) = game.find_player(inviter_id) else {
                let _ = send_trade_notice(game, player_id, b"GS0070");
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "ответ на обмен обработан",
                );
                return Some(Ok(()));
            };
            let notices: &[&[u8]] = if inviter.is_dead() {
                &[b"GS0069", b"GS0065"]
            } else if inviter.current_progress() != PlayerProgress::None {
                &[b"GS0067", b"GS0068"]
            } else if game
                .player_trade_distance(player_id, inviter_id)
                .unwrap_or(9)
                >= 9
            {
                &[b"GS0061", b"GS0061"]
            } else if accepted == 0 {
                &[b"GS0066"]
            } else {
                &[]
            };
            if !notices.is_empty() {
                for string_id in notices {
                    let _ = send_trade_notice(game, player_id, string_id);
                }
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "ответ на обмен обработан",
                );
                return Some(Ok(()));
            }
            game.find_player_mut(player_id)
                .expect("trade answerer проверен")
                .set_current_progress_snapshot(PlayerProgress::Trading);
            game.find_player_mut(inviter_id)
                .expect("trade inviter проверен")
                .set_current_progress_snapshot(PlayerProgress::Trading);
            let session = game.create_player_trade_session(inviter_id, player_id);
            if let Some((session_id, inviter_plug_id, answerer_plug_id)) = session {
                let mut opened = CMessage::new(0x000b_f710);
                opened.add_long(session_id);
                opened.add_long(inviter_id);
                opened.add_long(inviter_plug_id);
                opened.add_long(player_id);
                opened.add_long(answerer_plug_id);
                for owner_id in [inviter_id, player_id] {
                    let _ = opened.send_to_player(game.net_server(), owner_id);
                }
            }
            trace_player_message_outcome(message_type, Some(player_id), "ответ на обмен обработан");
        }
        LEGACY_NO_OP_08 | LEGACY_NO_OP_09 | LEGACY_NO_OP_0A | LEGACY_NO_OP_14 => {
            trace_player_message_outcome(
                message_type,
                Some(player_id),
                "пустая legacy-ветвь",
            );
        }
        TOGGLE_TRADE_READY => {
            let Some(session_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "trade session id",
                )));
            };
            let Some(plug_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField("trade plug id")));
            };
            let _ = game.toggle_player_trade_ready(player_id, session_id, plug_id, runtime);
            trace_player_message_outcome(
                message_type,
                Some(player_id),
                "состояние обмена изменено",
            );
        }
        ABORT_TRADE => {
            let Some(session_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "trade session id",
                )));
            };
            let Some(plug_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField("trade plug id")));
            };
            let _ = game.abort_player_trade(player_id, session_id, plug_id);
            trace_player_message_outcome(message_type, Some(player_id), "обмен прерван");
        }
        REQUEST_FRIEND => {
            let Some(target_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "target player id",
                )));
            };
            tracing::trace!(
                message_type,
                player_id,
                target_id,
                "проверяется запрос дружбы"
            );
            let Some(requester_name) = game
                .find_player(player_id)
                .map(|player| player.player_name().to_vec())
            else {
                return Some(Ok(()));
            };
            if game.find_player(target_id).is_none() {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "целевой игрок отсутствует",
                );
                return Some(Ok(()));
            }
            let mut response = CMessage::new(0x000b_f719);
            add_c_string(&mut response, &requester_name);
            let _ = response.send_to_player(game.net_server(), target_id);
            trace_player_message_outcome(message_type, Some(player_id), "запрос дружбы отправлен");
        }
        ANSWER_FRIEND => {
            let friend_name = message.base_mut().get_str_bytes(0x32).unwrap_or_default();
            let Some(accepted) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField("friend answer")));
            };
            let target_id = game
                .find_player_by_name(&friend_name)
                .map(|player| player.player_id());
            tracing::trace!(
                message_type,
                player_id,
                target_id,
                "проверяется ответ на дружбу"
            );
            if accepted == 1 {
                apply_friend_add(game, player_id, &friend_name, target_id);
            }
            let Some(target_id) = target_id else {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "целевой игрок отсутствует",
                );
                return Some(Ok(()));
            };
            let requester_name = game
                .find_player(player_id)
                .expect("friend answer requester сохранён после context lookup")
                .player_name()
                .to_vec();
            let target_name = game
                .find_player(target_id)
                .expect("friend answer target сохранён после name lookup")
                .player_name()
                .to_vec();
            if accepted == 1 {
                apply_friend_add(game, target_id, &requester_name, Some(player_id));
            }
            let mut to_target = CMessage::new(0x000b_f71a);
            add_c_string(&mut to_target, &requester_name);
            to_target.base_mut().add_byte(accepted as u8);
            let _ = to_target.send_to_player(game.net_server(), target_id);
            let mut to_requester = CMessage::new(0x000b_f71a);
            add_c_string(&mut to_requester, &target_name);
            to_requester.base_mut().add_byte(accepted as u8);
            let _ = to_requester.send_to_player(game.net_server(), player_id);
            trace_player_message_outcome(
                message_type,
                Some(player_id),
                "ответ на дружбу обработан",
            );
        }
        DELETE_FRIEND => {
            let friend_name = message.base_mut().get_str_bytes(0x32).unwrap_or_default();
            let online_friend_id = game
                .find_player_by_name(&friend_name)
                .map(|player| player.player_id());
            tracing::trace!(
                message_type,
                player_id,
                online_friend_id,
                "проверяется удаление друга"
            );
            let exists = game
                .find_player(player_id)
                .is_some_and(|player| player.has_friend(&friend_name));
            if !exists {
                trace_player_message_outcome(message_type, Some(player_id), "друг отсутствует");
                return Some(Ok(()));
            }
            let _ =
                publish_friend_delete(game, player_id, online_friend_id.unwrap_or(0), &friend_name);
            game.find_player_mut(player_id)
                .expect("friend owner сохранён после existence lookup")
                .delete_friend_state(&friend_name);
            let mut response = CMessage::new(0x000b_f71b);
            add_c_string(&mut response, &friend_name);
            let _ = response.send_to_player(game.net_server(), player_id);
            trace_player_message_outcome(message_type, Some(player_id), "друг удалён");
        }
        SET_DISPLAY_HEAD_PIECE => {
            let Some(display) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "display head piece",
                )));
            };
            let display = display != 0;
            game.find_player_mut(player_id)
                .expect("display-head player сохранён после context lookup")
                .set_display_head_piece(display);
            let mut response = CMessage::new(0x000b_f722);
            response.add_long(player_id);
            response.base_mut().add_byte(u8::from(display));
            let _ = game.send_player_shape_around(player_id, Some(player_id), &response);
            trace_player_message_outcome(
                message_type,
                Some(player_id),
                "отображение головного предмета изменено",
            );
        }
        QUERY_QUEST_TIME => {
            let remaining = game
                .find_player(player_id)
                .expect("quest-time player сохранён после context lookup")
                .client_quest_time_remaining(game_wall_time_seconds() as i32);
            let mut response = CMessage::new(0x000b_f72b);
            response.add_long(remaining);
            let _ = response.send_to_player(game.net_server(), player_id);
            trace_player_message_outcome(message_type, Some(player_id), "время задания отправлено");
        }
        ACKNOWLEDGE_HEARTBEAT => {
            game.find_player_mut(player_id)
                .expect("heartbeat player сохранён после context lookup")
                .acknowledge_heartbeat();
            trace_player_message_outcome(message_type, Some(player_id), "heartbeat подтверждён");
        }
        END_CHANGE_BODY_STATE => {
            let ended = game.end_first_player_change_body_state(player_id);
            tracing::trace!(
                message_type,
                player_id,
                ?ended,
                "завершено клиентское состояние преображения"
            );
        }
        REFRESH_EXPIRED_EQUIPMENT_STATE => {
            let Some(goods_id) = message.base_mut().get_guid() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "expired equipment guid",
                )));
            };
            tracing::trace!(
                message_type,
                player_id,
                ?goods_id,
                "проверяется состояние снаряжения"
            );
            let Some((state, packed_time)) = game.find_player(player_id).and_then(|player| {
                player.get_goods_by_id(goods_id).map(|goods| {
                    (
                        goods.addon_property_value(game.goods_factory(), GAP_EQUIP_STATE, 1),
                        goods.addon_property_value(game.goods_factory(), GAP_EQUIP_STATE, 2),
                    )
                })
            }) else {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "истёкшее снаряжение отсутствует",
                );
                return Some(Ok(()));
            };
            if state != 2 || packed_time == 0 {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "состояние снаряжения не требует обновления",
                );
                return Some(Ok(()));
            }
            let Some(source) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "expired equipment source",
                )));
            };
            tracing::trace!(
                message_type,
                player_id,
                source,
                "получен источник снаряжения"
            );
            let Some(elapsed_seconds) = equipment_state_elapsed_seconds(packed_time) else {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "время снаряжения некорректно",
                );
                return Some(Ok(()));
            };
            let elapsed_seconds = truncate_original(elapsed_seconds);
            tracing::trace!(
                message_type,
                player_id,
                elapsed_seconds,
                "вычислен срок состояния снаряжения"
            );
            if elapsed_seconds / 60 <= 0x275f {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "снаряжение ещё действует",
                );
                return Some(Ok(()));
            }
            let (identity, goods, mutated) = {
                let goods = game
                    .find_player_mut(player_id)
                    .expect("expired-equipment player сохранён после context lookup")
                    .get_goods_by_id_mut(goods_id)
                    .expect("expired equipment сохранён после addon validation");
                let mutated = goods.set_addon_property_modifier_core(GAP_EQUIP_STATE, 1, 3);
                (
                    goods.identity(),
                    goods.clone(),
                    mutated,
                )
            };
            let payload = game.encode_goods_for_old_client(&goods);
            tracing::trace!(
                message_type,
                player_id,
                mutated,
                "изменено состояние снаряжения"
            );
            let mut response = CMessage::new(0x000b_f928);
            response.add_long(player_id);
            response.base_mut().add_guid(identity.ex_id);
            response.base_mut().add_ulong(payload.len() as u32);
            response.base_mut().add(&payload);
            let _ = game.send_player_shape_around(player_id, None, &response);
            trace_player_message_outcome(
                message_type,
                Some(player_id),
                "состояние снаряжения опубликовано",
            );
        }
        QUERY_HONOR_IDENTITY => {
            let country_identity = game.player_country_identity(player_id);
            let honor = game
                .find_player(player_id)
                .expect("honor player сохранён после context lookup")
                .honor_snapshot();
            let mut response = CMessage::new(0x000b_f737);
            response.base_mut().add_ulong(honor.rank_of_nobility_id);
            response.base_mut().add_ulong(u32::from(country_identity));
            response.base_mut().add_ulong(honor.appellation_id);
            response.base_mut().add_ulong(honor.days_eliminate);
            response.base_mut().add_ulong(honor.weeks_eliminate);
            response.base_mut().add_ulong(honor.months_eliminate);
            response.base_mut().add_ulong(honor.total_eliminate);
            let _ = response.send_to_player(game.net_server(), player_id);
            trace_player_message_outcome(
                message_type,
                Some(player_id),
                "почётная идентичность отправлена",
            );
        }
        REQUEST_CHANGE_APPELLATION => {
            let Some(appellation_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField("appellation id")));
            };
            let Some(_legacy_ignored) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "appellation request tail",
                )));
            };
            let _ = game.request_player_change_appellation(
                player_id,
                appellation_id as u32,
                runtime,
            );
            trace_player_message_outcome(message_type, Some(player_id), "смена титула запрошена");
        }
        QUERY_LOCAL_TIME => {
            let system_time = TagTime::local_now().fields();
            let mut response = CMessage::new(0x000b_f73f);
            for field in system_time {
                response.base_mut().add_short(field as i16);
            }
            let _ = response.send_to_player(game.net_server(), player_id);
            trace_player_message_outcome(
                message_type,
                Some(player_id),
                "локальное время отправлено",
            );
        }
        CLAIM_LEI_TING_REWARD => {
            let Some(reward) = message.base_mut().get_word() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "LeiTing reward index",
                )));
            };
            tracing::trace!(
                message_type,
                player_id,
                reward,
                "проверяется награда LeiTing"
            );
            let Some(script) = LEI_TING_REWARD_SCRIPTS.get(usize::from(reward)) else {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "награда LeiTing некорректна",
                );
                return Some(Ok(()));
            };
            if !game
                .find_player(player_id)
                .expect("LeiTing player сохранён после context lookup")
                .packet()
                .check_space(3)
            {
                let notice =
                    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"E19681"));
                let _ = notice.send_to_player(game.net_server(), player_id);
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "рюкзак для награды LeiTing заполнен",
                );
                return Some(Ok(()));
            }
            if !game
                .find_player_mut(player_id)
                .expect("LeiTing player сохранён после packet-space lookup")
                .change_fy_energy_flag(reward)
            {
                trace_player_message_outcome(
                    message_type,
                    Some(player_id),
                    "награда LeiTing недоступна",
                );
                return Some(Ok(()));
            }
            publish_lei_ting_update(game, player_id);
            let _ = game.run_script_file(
                script,
                ScriptExecutionContext {
                    player_id: Some(player_id),
                    ..ScriptExecutionContext::default()
                },
                runtime,
            );
            trace_player_message_outcome(message_type, Some(player_id), "награда LeiTing получена");
        }
        _ => unreachable!("player opcode отфильтрован до decode"),
    }
    Some(Ok(()))
}
