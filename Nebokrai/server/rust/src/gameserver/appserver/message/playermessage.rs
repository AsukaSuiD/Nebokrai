//! Player-message dispatcher GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/playermessage.cpp`. Материализован friend lifecycle
//! `0x8FA0D..0x8FA0F`: byte-exact names, 40-entry ordered state, reciprocal
//! accept mutation, addressed `0xBF719/71A/71B` и WorldServer
//! `0x60501/0x60502`. Public identity `0x8FA11/17/18` замыкает headpiece
//! state/around publication, honor-country-appellation snapshot и attempt ID
//! до server-trusted change-appellation script boundary. Client timing
//! `0x8FA12/13/1A` замыкает quest countdown, heartbeat acknowledgement и exact
//! 16-byte Windows `SYSTEMTIME`; wall-clock берётся из общего process owner-а,
//! а local Windows calendar/CRT conversion остаётся runtime-границей.
//! Player item-use `0x8FA04` замыкает outer progress/death guard, packet slot,
//! region forbidden goods и `CanUseItem`, mount/change-body ветви, полный
//! consumable-addon loop, skill/player combat mutations и recall/state runtime
//! boundaries. ChangeBody restriction/check читают live state и конфигурацию
//! прямо из `CGame`; goods-script входит в `CGame::run_script_file` с GUID
//! исходного packet item и выполняет state side effects через тот же owner.
//! Terminal расход публикует `0xBF709/0xC0101/0xC0102`.
//! Общий outer guard сохраняет исходный запрет player-message во время смены
//! сервера или региона; `0x8FA02` вызывает полный достигнутый
//! `CPlayer::OnRelive(0)` через владельца возрождения в `CGame` со всеми
//! изменениями состояния, региона и сетевыми эффектами.
//! LeiTing claim `0x8FA19` сохраняет packet-space gate, exact thresholds,
//! `BF73E -> 5FD10 -> reward script` ordering; `0x8FA10` использует тот же
//! server-trusted script runtime для help script.
//! Player trade `0x8FA06/07/0B/0C` замыкает invitation/answer guards,
//! normal session с двумя trader plug-ами, ready toggle, синхронный commit,
//! Billing-pending YuanBao tail и terminal End/Abort публикации.
//! Stat allocation `0x8FA01` сохраняет no-point/no-read guard, legacy STR gate,
//! occupation/sex HP/MP increments, virtual property recompute и exact
//! `0xBF702 = m_Property[0x9c] + base max HP/MP` response.
//! NPC interaction `0x8FA03` замыкает player/region/death/progress guards,
//! around-area и figure-aware distance, `GS0057` и контекстный RunScript
//! request; reached `CGame::run_script_file` строит concrete player/NPC/region
//! context и исполняет поддержанные `CScript::RunFunction` selector-ы.
//! PvP permissions `0x8FA05` декодируют оба signed char до selector switch и
//! меняют один из пяти live player flags без дополнительной публикации.
//! Equipment-state refresh `0x8FA16` сохраняет packed local-time decode,
//! strict grace-minute comparison, addon mutation и around `0xBF928`.
//! Остальные opcode ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).

use crate::gameserver::appserver::cs2ccontainerobjectamountchange::CS2CContainerObjectAmountChange;
use crate::gameserver::appserver::cs2ccontainerobjectmove::{
    CS2CContainerObjectMove, ContainerObjectMoveOperation,
};
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_CHANGEBODY_TYPE, GAP_EQUIP_STATE, GAP_MOUNT_LEVEL, GAP_MOUNT_PLAYER_ROLE_LIMIT,
    GAP_MOUNT_TYPE, GAP_SKILL_ID, GAP_SKILL_LEVEL, GAP_UNLIMITED_ACCESS, GOODS_TYPE_CONSUMABLE,
};
use crate::gameserver::appserver::player::{
    CiQingPacketConsumption, PlayerFriendAddOutcome, PlayerPkPermissionMutation, PlayerProgress,
    PlayerStatAllocationMutation,
};
use crate::gameserver::appserver::script::function::ScriptFunctionRuntime;
use crate::gameserver::appserver::script::script::ScriptExecutionContext;
use crate::gameserver::appserver::shape::ShapeCoordinateBlock;
use crate::gameserver::gameserver::game::{
    CGame, GameContainerMessageRuntime, PlayerReliveContext, PlayerReliveReport,
    PlayerTradeAbortReport, PlayerTradeReadyReport, colored_player_notice_message,
    format_legacy_text_fields, game_wall_time_seconds,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::public::date::TagTime;
use crate::public::guid::CGuid;

const ALLOCATE_STAT_POINT: u32 = 0x0008_fa01;
const REQUEST_RELIVE: u32 = 0x0008_fa02;
const INTERACT_WITH_NPC: u32 = 0x0008_fa03;
const USE_PACKET_ITEM: u32 = 0x0008_fa04;
const SET_PK_PERMISSION: u32 = 0x0008_fa05;
const REQUEST_TRADE: u32 = 0x0008_fa06;
const ANSWER_TRADE: u32 = 0x0008_fa07;
const TOGGLE_TRADE_READY: u32 = 0x0008_fa0b;
const ABORT_TRADE: u32 = 0x0008_fa0c;
const RUN_HELP_SCRIPT: u32 = 0x0008_fa10;
const REQUEST_FRIEND: u32 = 0x0008_fa0d;
const ANSWER_FRIEND: u32 = 0x0008_fa0e;
const DELETE_FRIEND: u32 = 0x0008_fa0f;
const SET_DISPLAY_HEAD_PIECE: u32 = 0x0008_fa11;
const QUERY_QUEST_TIME: u32 = 0x0008_fa12;
const ACKNOWLEDGE_HEARTBEAT: u32 = 0x0008_fa13;
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
{
    /// Snapshot отсутствующего `CState`/timer/setup owner-а до item mutation;
    /// blocking state проверяет ordered `PLAYER_ITEM_BLOCKING_SKILL_IDS`.
    fn player_item_use_facts(&mut self, game: &CGame, player_id: i32) -> PlayerItemUseFacts;

    /// Исполняет только ещё не owned concrete state/skill/relocation
    /// owner; container/player scalars и wire хвост остаются у dispatcher-а.
    fn apply_player_item_runtime_effect(
        &mut self,
        game: &mut CGame,
        player_id: i32,
        effect: PlayerItemRuntimeEffect,
    ) -> PlayerItemRuntimeResult;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerItemUseFacts {
    pub(crate) blocking_skill_state: bool,
    pub(crate) state_110000_exists: bool,
    pub(crate) fight_state_count: i32,
    pub(crate) mount_state_exists: bool,
    pub(crate) contend_use_forbidden: bool,
    pub(crate) forbid_return_level: i32,
    pub(crate) tick_ms: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerItemRuntimeEffect {
    EndState(u32),
    RestoreHp {
        amount: u32,
        delay_ms: u32,
        step_ms: u32,
    },
    RestoreMp {
        amount: u32,
        delay_ms: u32,
        step_ms: u32,
    },
    RecallToReturnPoint,
    RecallInsideRegion,
    SkillWire {
        skill_id: u32,
    },
    CheckReuseSkillItem {
        goods_id: CGuid,
        skill_id: u32,
    },
    PrepareReuseSkillItem {
        slot: u8,
        goods_id: CGuid,
        skill_id: u32,
        skill_level: i32,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerItemSkillWire {
    pub(crate) value_84: u32,
    pub(crate) value_70: u16,
    pub(crate) value_74: u16,
    pub(crate) value_78: u16,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerItemRuntimeResult {
    pub(crate) applied: bool,
    pub(crate) skill_wire: Option<PlayerItemSkillWire>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerMessageOutcome {
    MissingContext,
    ChangingLocation,
    TargetMissing,
    StatPointUnavailable,
    StatPointAllocated,
    NpcInteractionRegionMissing,
    NpcInteractionBlocked,
    NpcInteractionTargetMissing,
    NpcInteractionTooFar,
    NpcInteractionScriptSuppressed,
    NpcInteractionScriptRequested,
    ItemUseBlocked,
    ItemUseRejected,
    ItemUsed,
    PkPermissionSet,
    Relived,
    PlayerScriptRun,
    TradeRequested,
    TradeAnswered,
    TradeStateChanged,
    TradeAborted,
    FriendRequested,
    FriendAnswered,
    FriendMissing,
    FriendDeleted,
    DisplayHeadPieceChanged,
    QuestTimeSent,
    HeartbeatAcknowledged,
    HonorIdentitySent,
    AppellationChangeRequested,
    ExpiredEquipmentMissing,
    ExpiredEquipmentStateIgnored,
    ExpiredEquipmentTimeInvalid,
    ExpiredEquipmentStillActive,
    ExpiredEquipmentPublished,
    LocalTimeSent,
    LeiTingInvalidReward,
    LeiTingPacketFull,
    LeiTingRewardUnavailable,
    LeiTingRewardClaimed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerMessageDelivery {
    Player(i32),
    Around(Option<Result<i32, ShapeCoordinateBlock>>),
    World(Result<i32, SendMessageError>),
}

#[must_use = "player-message report сохраняет friend state и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GamePlayerMessageReport {
    pub(crate) message_type: u32,
    pub(crate) player_id: Option<i32>,
    pub(crate) target_player_id: Option<i32>,
    pub(crate) npc_id: Option<i32>,
    pub(crate) npc_distance: Option<i32>,
    pub(crate) npc_script_file: Vec<u8>,
    pub(crate) npc_script_data_present: Option<bool>,
    pub(crate) item_slot: Option<u8>,
    pub(crate) item_goods_id: Option<CGuid>,
    pub(crate) item_can_use_result: Option<i32>,
    pub(crate) item_consumption: Option<CiQingPacketConsumption>,
    pub(crate) item_effects: Vec<i32>,
    pub(crate) pk_permission: Option<PlayerPkPermissionMutation>,
    pub(crate) friend_name: Vec<u8>,
    pub(crate) lei_ting_reward: Option<u16>,
    pub(crate) equipment_goods_id: Option<CGuid>,
    pub(crate) equipment_source: Option<i8>,
    pub(crate) equipment_source_position: Option<i32>,
    pub(crate) equipment_elapsed_seconds: Option<i32>,
    pub(crate) equipment_state_mutated: Option<bool>,
    pub(crate) outcome: GamePlayerMessageOutcome,
    pub(crate) stat_allocation: Option<PlayerStatAllocationMutation>,
    pub(crate) relive: Option<PlayerReliveReport>,
    pub(crate) trade_session: Option<(i32, i32, i32)>,
    pub(crate) trade_ready: Option<PlayerTradeReadyReport>,
    pub(crate) trade_abort: Option<PlayerTradeAbortReport>,
    pub(crate) deliveries: Vec<GamePlayerMessageDelivery>,
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
        message.base_mut().add(&(skill_level as i16).to_le_bytes());
    }
    message.add_ulong(wire.value_84);
    message.base_mut().add(&wire.value_70.to_le_bytes());
    message.base_mut().add(&wire.value_74.to_le_bytes());
    message.base_mut().add(&wire.value_78.to_le_bytes());
    message.send_to_player(game.net_server(), player_id)
}

fn publish_packet_item_consumption(
    game: &mut CGame,
    player_id: i32,
    slot: u8,
    goods_id: CGuid,
    identity_type: i32,
    base_index: u32,
    deliveries: &mut Vec<GamePlayerMessageDelivery>,
) -> Option<CiQingPacketConsumption> {
    let mut used = CMessage::new(0x000b_f709);
    used.add_byte(b'3');
    used.add_long(player_id);
    used.add_ulong(base_index);
    deliveries.push(GamePlayerMessageDelivery::Around(
        game.send_player_shape_around(player_id, None, &used),
    ));
    let consumption = game
        .find_player_mut(player_id)?
        .remove_packet_goods_by_id(goods_id, 1)?;
    if consumption.remaining_amount == 0 {
        let mut deleted = CS2CContainerObjectMove::default();
        deleted.set_operation(ContainerObjectMoveOperation::DeleteObject);
        deleted.set_source_container(400, player_id, u32::from(slot));
        deleted.set_source_container_extend_id(1);
        deleted.set_source_object(identity_type, goods_id, 0);
        deliveries.push(GamePlayerMessageDelivery::Player(
            deleted.send_to_player(game, player_id),
        ));
    } else {
        let mut changed = CS2CContainerObjectAmountChange::default();
        changed.set_source_container(400, player_id, u32::from(slot));
        changed.set_source_container_extend_id(1);
        changed.set_object(identity_type, goods_id);
        changed.set_object_amount(consumption.remaining_amount);
        deliveries.push(GamePlayerMessageDelivery::Player(
            changed.send_to_player(game, player_id),
        ));
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

fn publish_lei_ting_update(
    game: &CGame,
    player_id: i32,
    deliveries: &mut Vec<GamePlayerMessageDelivery>,
) {
    let payload = game
        .find_player(player_id)
        .expect("LeiTing player сохранён после flag mutation")
        .encode_lei_ting();
    let mut client = CMessage::new(0x000b_f73e);
    client.base_mut().add(&payload);
    deliveries.push(GamePlayerMessageDelivery::Player(
        client.send_to_player(game.net_server(), player_id),
    ));
    let mut world = CMessage::new(0x0005_fd10);
    world.add_long(player_id);
    world.base_mut().add(&payload);
    deliveries.push(GamePlayerMessageDelivery::World(world.send(game, false)));
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
    deliveries: &mut Vec<GamePlayerMessageDelivery>,
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
                deliveries.push(GamePlayerMessageDelivery::World(publish_friend_add(
                    game, owner_id, friend_id,
                )));
            }
        }
        PlayerFriendAddOutcome::AlreadyPresent => {}
        PlayerFriendAddOutcome::LimitReached => {
            let delivery =
                colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0155"))
                    .send_to_player(game.net_server(), owner_id);
            deliveries.push(GamePlayerMessageDelivery::Player(delivery));
        }
    }
}

pub(crate) fn dispatch_game_player_message<Runtime: GamePlayerMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<GamePlayerMessageReport, GamePlayerMessageError>> {
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
            | TOGGLE_TRADE_READY
            | ABORT_TRADE
            | RUN_HELP_SCRIPT
            | REQUEST_FRIEND
            | ANSWER_FRIEND
            | DELETE_FRIEND
            | SET_DISPLAY_HEAD_PIECE
            | QUERY_QUEST_TIME
            | ACKNOWLEDGE_HEARTBEAT
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
    let mut report = GamePlayerMessageReport {
        message_type,
        player_id,
        target_player_id: None,
        npc_id: None,
        npc_distance: None,
        npc_script_file: Vec::new(),
        npc_script_data_present: None,
        item_slot: None,
        item_goods_id: None,
        item_can_use_result: None,
        item_consumption: None,
        item_effects: Vec::new(),
        pk_permission: None,
        friend_name: Vec::new(),
        lei_ting_reward: None,
        equipment_goods_id: None,
        equipment_source: None,
        equipment_source_position: None,
        equipment_elapsed_seconds: None,
        equipment_state_mutated: None,
        outcome: GamePlayerMessageOutcome::MissingContext,
        stat_allocation: None,
        relive: None,
        trade_session: None,
        trade_ready: None,
        trade_abort: None,
        deliveries: Vec::new(),
    };
    let Some(player_id) = player_id else {
        return Some(Ok(report));
    };
    if game
        .find_player(player_id)
        .is_some_and(|player| player.in_changing_server() || player.in_changing_region())
    {
        report.outcome = GamePlayerMessageOutcome::ChangingLocation;
        return Some(Ok(report));
    }

    match message_type {
        ALLOCATE_STAT_POINT => {
            let Some(state) = game
                .find_player(player_id)
                .map(|player| player.stat_allocation_state())
            else {
                return Some(Ok(report));
            };
            if state.remain_point == 0 {
                report.outcome = GamePlayerMessageOutcome::StatPointUnavailable;
                return Some(Ok(report));
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
            report.stat_allocation = game
                .find_player_mut(player_id)
                .expect("stat-allocation player сохранён после context lookup")
                .allocate_stat_point(selector as u8, constitution_hp, intelligence_mp);
            let properties = runtime.recompute_enhancement_player_properties(
                game.find_player(player_id)
                    .expect("stat-allocation player сохранён после mutation"),
            );
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
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                response.send_to_player(game.net_server(), player_id),
            ));
            report.outcome = GamePlayerMessageOutcome::StatPointAllocated;
        }
        REQUEST_RELIVE => {
            report.relive = Some(game.relive_player(player_id, 0, runtime));
            report.outcome = GamePlayerMessageOutcome::Relived;
        }
        INTERACT_WITH_NPC => {
            let Some(region_id) = message
                .region_id()
                .filter(|region_id| game.find_region(*region_id).is_some())
            else {
                report.outcome = GamePlayerMessageOutcome::NpcInteractionRegionMissing;
                return Some(Ok(report));
            };
            let Some(player) = game.find_player(player_id) else {
                return Some(Ok(report));
            };
            if player.is_dead() || player.current_progress() != PlayerProgress::None {
                report.outcome = GamePlayerMessageOutcome::NpcInteractionBlocked;
                return Some(Ok(report));
            }
            let Some(npc_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField("NPC id")));
            };
            report.npc_id = Some(npc_id);
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
                report.outcome = GamePlayerMessageOutcome::NpcInteractionTargetMissing;
                return Some(Ok(report));
            };
            report.npc_distance = Some(distance);
            report.npc_script_file.clone_from(&script_file);
            if distance >= 9 {
                let notification =
                    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0057"));
                report.deliveries.push(GamePlayerMessageDelivery::Player(
                    notification.send_to_player(game.net_server(), player_id),
                ));
                report.outcome = GamePlayerMessageOutcome::NpcInteractionTooFar;
                return Some(Ok(report));
            }
            if script_file.first() == Some(&b'0') {
                report.outcome = GamePlayerMessageOutcome::NpcInteractionScriptSuppressed;
                return Some(Ok(report));
            }
            report.npc_script_data_present = Some(game.script_file_data(&script_file).is_some());
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
            report.outcome = GamePlayerMessageOutcome::NpcInteractionScriptRequested;
        }
        USE_PACKET_ITEM => {
            let Some(player) = game.find_player(player_id) else {
                return Some(Ok(report));
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
                report.outcome = GamePlayerMessageOutcome::ItemUseBlocked;
                return Some(Ok(report));
            }
            let mut facts = runtime.player_item_use_facts(game, player_id);
            if let Some(player) = game.find_player(player_id) {
                facts.mount_state_exists = player.is_rider();
                facts.fight_state_count = player.fight_state_count();
                facts.state_110000_exists |= player.script_move_state_count(110000) != 0;
            }
            if facts.blocking_skill_state {
                report
                    .deliveries
                    .push(GamePlayerMessageDelivery::Player(send_item_notice(
                        game,
                        player_id,
                        b"GS0146",
                        &[],
                        0,
                    )));
                report.outcome = GamePlayerMessageOutcome::ItemUseBlocked;
                return Some(Ok(report));
            }
            if facts.state_110000_exists {
                if !game.end_script_auto_protect_state(player_id) {
                    let _ended = runtime.apply_player_item_runtime_effect(
                        game,
                        player_id,
                        PlayerItemRuntimeEffect::EndState(110000),
                    );
                }
            }
            let Some(slot) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "packet item slot",
                )));
            };
            let slot = slot as u8;
            report.item_slot = Some(slot);
            let Some(goods) = game
                .find_player(player_id)
                .and_then(|player| player.packet().get_goods(u32::from(slot)))
                .cloned()
            else {
                report.outcome = GamePlayerMessageOutcome::ItemUseRejected;
                return Some(Ok(report));
            };
            let goods_id = goods.identity().ex_id;
            let identity_type = goods.identity().object_type;
            let base_index = goods.base_properties_index();
            report.item_goods_id = Some(goods_id);
            if game.change_body_item_conflicts(player_id, base_index) {
                report
                    .deliveries
                    .push(GamePlayerMessageDelivery::Player(send_item_notice(
                        game,
                        player_id,
                        b"GSN0337",
                        &[goods.name()],
                        0,
                    )));
                report.outcome = GamePlayerMessageOutcome::ItemUseRejected;
                return Some(Ok(report));
            }
            let Some(region_id) = game
                .find_player(player_id)
                .and_then(|player| player.server_region_id())
            else {
                report.outcome = GamePlayerMessageOutcome::ItemUseRejected;
                return Some(Ok(report));
            };
            let Some(base_properties) =
                game.goods_factory().query_goods_base_properties(base_index)
            else {
                report.outcome = GamePlayerMessageOutcome::ItemUseRejected;
                return Some(Ok(report));
            };
            if base_properties.goods_type() != GOODS_TYPE_CONSUMABLE {
                report.outcome = GamePlayerMessageOutcome::ItemUseRejected;
                return Some(Ok(report));
            }
            let original_name = base_properties.original_name().to_vec();
            let Some(region) = game.find_region(region_id) else {
                report.outcome = GamePlayerMessageOutcome::ItemUseRejected;
                return Some(Ok(report));
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
            report.item_can_use_result = Some(can_use);
            if can_use != 9 {
                let mut failure = CMessage::new(0x000b_f709);
                failure.add_byte(b'4');
                failure.add_byte(can_use as u8);
                report.deliveries.push(GamePlayerMessageDelivery::Player(
                    failure.send_to_player(game.net_server(), player_id),
                ));
                report.outcome = GamePlayerMessageOutcome::ItemUseRejected;
                return Some(Ok(report));
            }
            if goods.amount() == 0 {
                report.item_consumption = game
                    .find_player_mut(player_id)
                    .and_then(|player| player.remove_packet_goods_by_id(goods_id, 1));
                report.outcome = GamePlayerMessageOutcome::ItemUseRejected;
                return Some(Ok(report));
            }
            if game
                .find_player(player_id)
                .is_some_and(|player| player.contend_state())
                && facts.contend_use_forbidden
            {
                report
                    .deliveries
                    .push(GamePlayerMessageDelivery::Player(send_item_notice(
                        game,
                        player_id,
                        b"GS0147",
                        &[],
                        0xffff_0000,
                    )));
            }

            let mut consume = true;
            let mut return_after_use = false;
            let change_body_type =
                goods.addon_property_value(game.goods_factory(), GAP_CHANGEBODY_TYPE, 1);
            if change_body_type != 0 && game.script_change_body_check(player_id, false) == 0 {
                report
                    .deliveries
                    .push(GamePlayerMessageDelivery::Player(send_item_notice(
                        game,
                        player_id,
                        b"GSN1063",
                        &[],
                        0,
                    )));
                report.outcome = GamePlayerMessageOutcome::ItemUseRejected;
                return Some(Ok(report));
            }

            let mount_type = goods.addon_property_value(game.goods_factory(), GAP_MOUNT_TYPE, 1);
            if mount_type != 0 {
                consume = false;
                if facts.mount_state_exists {
                    let _ended = game.end_player_ride(player_id);
                    let properties = runtime.recompute_enhancement_player_properties(
                        game.find_player(player_id).expect("mount player сохранён"),
                    );
                    game.apply_player_state_properties(player_id, properties);
                } else if game.find_player(player_id).is_some_and(|player| {
                    player.current_progress() != PlayerProgress::OpenStall
                        && facts.fight_state_count == 0
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
                        &original_name,
                    );
                    if applied {
                        let properties = runtime.recompute_enhancement_player_properties(
                            game.find_player(player_id)
                                .expect("mounted player сохранён"),
                        );
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
                    report.item_effects.push(property);
                    let value =
                        |value_id| goods.addon_property_value(&goods_factory, property, value_id);
                    match property {
                        0x21 | 0x43 => {
                            let result = runtime.apply_player_item_runtime_effect(
                                game,
                                player_id,
                                PlayerItemRuntimeEffect::RestoreHp {
                                    amount: value(1) as u32,
                                    delay_ms: (value(2) as u32).wrapping_mul(400),
                                    step_ms: 400,
                                },
                            );
                            consume = result.applied;
                        }
                        0x22 => {
                            let maximum = game
                                .find_player(player_id)
                                .expect("restore-hp player сохранён")
                                .maximum_health();
                            let amount =
                                (f64::from(maximum) * f64::from(value(1)) * 0.01).round() as u32;
                            consume = runtime
                                .apply_player_item_runtime_effect(
                                    game,
                                    player_id,
                                    PlayerItemRuntimeEffect::RestoreHp {
                                        amount,
                                        delay_ms: 0,
                                        step_ms: 400,
                                    },
                                )
                                .applied;
                        }
                        0x23 => {
                            let delay_ms = if value(2) == 0 {
                                0
                            } else {
                                (value(2) as u32).wrapping_sub(1).wrapping_mul(400)
                            };
                            consume = runtime
                                .apply_player_item_runtime_effect(
                                    game,
                                    player_id,
                                    PlayerItemRuntimeEffect::RestoreMp {
                                        amount: value(1) as u32,
                                        delay_ms,
                                        step_ms: 400,
                                    },
                                )
                                .applied;
                        }
                        0x24 => {
                            let maximum = game
                                .find_player(player_id)
                                .expect("restore-mp player сохранён")
                                .maximum_mana();
                            let amount =
                                (f64::from(maximum) * f64::from(value(1)) * 0.01).round() as u32;
                            consume = runtime
                                .apply_player_item_runtime_effect(
                                    game,
                                    player_id,
                                    PlayerItemRuntimeEffect::RestoreMp {
                                        amount,
                                        delay_ms: 0,
                                        step_ms: 400,
                                    },
                                )
                                .applied;
                        }
                        0x27 if 0 <= value(2) => {
                            let skill_id = value(1) as u32;
                            let requested_level = value(2);
                            let current_level = game
                                .find_player(player_id)
                                .expect("skill-book player сохранён")
                                .item_skill_level(skill_id);
                            if requested_level <= current_level {
                                consume = false;
                                report.deliveries.push(GamePlayerMessageDelivery::Player(
                                    send_item_notice(game, player_id, b"GS0148", &[], 0),
                                ));
                            } else if requested_level.wrapping_sub(current_level) != 1 {
                                consume = false;
                                report.deliveries.push(GamePlayerMessageDelivery::Player(
                                    send_item_notice(game, player_id, b"GS0149", &[], 0),
                                ));
                            } else {
                                let skill_factory = game.skill_factory().clone();
                                let added = game
                                    .find_player_mut(player_id)
                                    .expect("skill-book player сохранён для add")
                                    .learn_item_skill(skill_id, requested_level, &skill_factory);
                                if added {
                                    let result = runtime.apply_player_item_runtime_effect(
                                        game,
                                        player_id,
                                        PlayerItemRuntimeEffect::SkillWire { skill_id },
                                    );
                                    if let Some(wire) = result.skill_wire {
                                        report.deliveries.push(GamePlayerMessageDelivery::Player(
                                            send_item_skill_wire(
                                                game,
                                                player_id,
                                                0x000b_f71d,
                                                None,
                                                skill_id,
                                                requested_level,
                                                wire,
                                            ),
                                        ));
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
                                report.deliveries.push(GamePlayerMessageDelivery::Player(
                                    send_item_notice(game, player_id, b"GS0150", &[], 0),
                                ));
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
                                report.deliveries.push(GamePlayerMessageDelivery::Player(
                                    send_item_notice(game, player_id, b"GS0151", &[], 0),
                                ));
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
                                report.deliveries.push(GamePlayerMessageDelivery::Player(
                                    send_item_notice(game, player_id, b"GS0152", &[], 0),
                                ));
                            } else if game.find_player(player_id).is_some_and(|player| {
                                player.current_progress() == PlayerProgress::Synthesis
                            }) {
                                consume = false;
                                report.deliveries.push(GamePlayerMessageDelivery::Player(
                                    send_item_notice(game, player_id, b"GS1041", &[], 0),
                                ));
                            } else if i32::from(
                                game.find_player(player_id)
                                    .expect("random recall player")
                                    .level(),
                            ) >= facts.forbid_return_level
                                && 0 < facts.fight_state_count
                            {
                                consume = false;
                                report.deliveries.push(GamePlayerMessageDelivery::Player(
                                    send_item_notice(game, player_id, b"GS0153", &[], 0),
                                ));
                            } else {
                                let _recalled = runtime.apply_player_item_runtime_effect(
                                    game,
                                    player_id,
                                    PlayerItemRuntimeEffect::RecallInsideRegion,
                                );
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
                                    value(1),
                                    facts.tick_ms,
                                    value(2) as u32,
                                );
                            let mut update = CMessage::new(0x000b_f70a);
                            if property == 0x4b {
                                update.base_mut().add(&(resulting as i16).to_le_bytes());
                            } else {
                                update.add_long(resulting);
                            }
                            report.deliveries.push(GamePlayerMessageDelivery::Player(
                                update.send_to_player(game.net_server(), player_id),
                            ));
                        }
                        0x85 => {
                            consume = false;
                            let skill_id =
                                goods.addon_property_value(game.goods_factory(), GAP_SKILL_ID, 1)
                                    as u32;
                            let skill_level = goods.addon_property_value(
                                game.goods_factory(),
                                GAP_SKILL_LEVEL,
                                1,
                            );
                            let reusable = runtime.apply_player_item_runtime_effect(
                                game,
                                player_id,
                                PlayerItemRuntimeEffect::CheckReuseSkillItem { goods_id, skill_id },
                            );
                            if reusable.applied {
                                let skill_factory = game.skill_factory().clone();
                                let replaced = game
                                    .find_player_mut(player_id)
                                    .expect("reuse-item player сохранён для skill replacement")
                                    .replace_item_skill(skill_id, skill_level, &skill_factory);
                                let result = replaced.then(|| {
                                    runtime.apply_player_item_runtime_effect(
                                        game,
                                        player_id,
                                        PlayerItemRuntimeEffect::PrepareReuseSkillItem {
                                            slot,
                                            goods_id,
                                            skill_id,
                                            skill_level,
                                        },
                                    )
                                });
                                if let Some(result) = result {
                                    if let Some(wire) = result.skill_wire {
                                        report.deliveries.push(GamePlayerMessageDelivery::Player(
                                            send_item_skill_wire(
                                                game,
                                                player_id,
                                                0x000b_fe07,
                                                Some(0x55),
                                                skill_id,
                                                skill_level,
                                                wire,
                                            ),
                                        ));
                                    }
                                }
                            } else {
                                report.deliveries.push(GamePlayerMessageDelivery::Player(
                                    send_item_notice(
                                        game,
                                        player_id,
                                        b"GS1178",
                                        &[goods.name()],
                                        0,
                                    ),
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
            if consume {
                report.item_consumption = publish_packet_item_consumption(
                    game,
                    player_id,
                    slot,
                    goods_id,
                    identity_type,
                    base_index,
                    &mut report.deliveries,
                );
            }
            if return_after_use {
                let _returned = runtime.apply_player_item_runtime_effect(
                    game,
                    player_id,
                    PlayerItemRuntimeEffect::RecallToReturnPoint,
                );
            }
            report.outcome = GamePlayerMessageOutcome::ItemUsed;
        }
        SET_PK_PERMISSION => {
            if game.find_player(player_id).is_none() {
                return Some(Ok(report));
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
            report.pk_permission = Some(
                game.find_player_mut(player_id)
                    .expect("PK-permission player сохранён после context lookup")
                    .set_pk_permission(selector, value != 0),
            );
            report.outcome = GamePlayerMessageOutcome::PkPermissionSet;
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
            report.outcome = GamePlayerMessageOutcome::PlayerScriptRun;
        }
        REQUEST_TRADE => {
            let Some(target_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "trade target player id",
                )));
            };
            report.target_player_id = Some(target_id);
            let Some(requester) = game.find_player(player_id) else {
                return Some(Ok(report));
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
                report
                    .deliveries
                    .push(GamePlayerMessageDelivery::Player(send_trade_notice(
                        game, player_id, string_id,
                    )));
                report.outcome = GamePlayerMessageOutcome::TradeRequested;
                return Some(Ok(report));
            }
            let mut invitation = CMessage::new(0x000b_f70f);
            invitation.add_long(player_id);
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                invitation.send_to_player(game.net_server(), target_id),
            ));
            report
                .deliveries
                .push(GamePlayerMessageDelivery::Player(send_trade_notice(
                    game, player_id, b"GS0060",
                )));
            report.outcome = GamePlayerMessageOutcome::TradeRequested;
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
            report.target_player_id = Some(inviter_id);
            if inviter_id == player_id {
                report.outcome = GamePlayerMessageOutcome::TradeAnswered;
                return Some(Ok(report));
            }
            let Some(answerer) = game.find_player(player_id) else {
                return Some(Ok(report));
            };
            if answerer.is_dead() {
                report
                    .deliveries
                    .push(GamePlayerMessageDelivery::Player(send_trade_notice(
                        game, player_id, b"GS0065",
                    )));
                report.outcome = GamePlayerMessageOutcome::TradeAnswered;
                return Some(Ok(report));
            }
            let inviter_exists = game.find_player(inviter_id).is_some();
            if answerer.current_progress() != PlayerProgress::None {
                for string_id in [b"GS0068".as_slice(), b"GS0071".as_slice()]
                    .into_iter()
                    .take(if inviter_exists { 2 } else { 1 })
                {
                    report
                        .deliveries
                        .push(GamePlayerMessageDelivery::Player(send_trade_notice(
                            game, player_id, string_id,
                        )));
                }
                report.outcome = GamePlayerMessageOutcome::TradeAnswered;
                return Some(Ok(report));
            }
            let Some(inviter) = game.find_player(inviter_id) else {
                report
                    .deliveries
                    .push(GamePlayerMessageDelivery::Player(send_trade_notice(
                        game, player_id, b"GS0070",
                    )));
                report.outcome = GamePlayerMessageOutcome::TradeAnswered;
                return Some(Ok(report));
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
                    report
                        .deliveries
                        .push(GamePlayerMessageDelivery::Player(send_trade_notice(
                            game, player_id, string_id,
                        )));
                }
                report.outcome = GamePlayerMessageOutcome::TradeAnswered;
                return Some(Ok(report));
            }
            game.find_player_mut(player_id)
                .expect("trade answerer проверен")
                .set_current_progress_snapshot(PlayerProgress::Trading);
            game.find_player_mut(inviter_id)
                .expect("trade inviter проверен")
                .set_current_progress_snapshot(PlayerProgress::Trading);
            let session = game.create_player_trade_session(inviter_id, player_id);
            report.trade_session = session;
            if let Some((session_id, inviter_plug_id, answerer_plug_id)) = session {
                let mut opened = CMessage::new(0x000b_f710);
                opened.add_long(session_id);
                opened.add_long(inviter_id);
                opened.add_long(inviter_plug_id);
                opened.add_long(player_id);
                opened.add_long(answerer_plug_id);
                for owner_id in [inviter_id, player_id] {
                    report.deliveries.push(GamePlayerMessageDelivery::Player(
                        opened.send_to_player(game.net_server(), owner_id),
                    ));
                }
            }
            report.outcome = GamePlayerMessageOutcome::TradeAnswered;
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
            report.trade_ready =
                Some(game.toggle_player_trade_ready(player_id, session_id, plug_id, runtime));
            report.outcome = GamePlayerMessageOutcome::TradeStateChanged;
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
            report.trade_abort = Some(game.abort_player_trade(player_id, session_id, plug_id));
            report.outcome = GamePlayerMessageOutcome::TradeAborted;
        }
        REQUEST_FRIEND => {
            let Some(target_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "target player id",
                )));
            };
            report.target_player_id = Some(target_id);
            let Some(requester_name) = game
                .find_player(player_id)
                .map(|player| player.player_name().to_vec())
            else {
                return Some(Ok(report));
            };
            if game.find_player(target_id).is_none() {
                report.outcome = GamePlayerMessageOutcome::TargetMissing;
                return Some(Ok(report));
            }
            let mut response = CMessage::new(0x000b_f719);
            add_c_string(&mut response, &requester_name);
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                response.send_to_player(game.net_server(), target_id),
            ));
            report.outcome = GamePlayerMessageOutcome::FriendRequested;
        }
        ANSWER_FRIEND => {
            let friend_name = message.base_mut().get_str_bytes(0x32).unwrap_or_default();
            let Some(accepted) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField("friend answer")));
            };
            report.friend_name.clone_from(&friend_name);
            let target_id = game
                .find_player_by_name(&friend_name)
                .map(|player| player.player_id());
            report.target_player_id = target_id;
            if accepted == 1 {
                apply_friend_add(
                    game,
                    player_id,
                    &friend_name,
                    target_id,
                    &mut report.deliveries,
                );
            }
            let Some(target_id) = target_id else {
                report.outcome = GamePlayerMessageOutcome::TargetMissing;
                return Some(Ok(report));
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
                apply_friend_add(
                    game,
                    target_id,
                    &requester_name,
                    Some(player_id),
                    &mut report.deliveries,
                );
            }
            let mut to_target = CMessage::new(0x000b_f71a);
            add_c_string(&mut to_target, &requester_name);
            to_target.base_mut().add_byte(accepted as u8);
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                to_target.send_to_player(game.net_server(), target_id),
            ));
            let mut to_requester = CMessage::new(0x000b_f71a);
            add_c_string(&mut to_requester, &target_name);
            to_requester.base_mut().add_byte(accepted as u8);
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                to_requester.send_to_player(game.net_server(), player_id),
            ));
            report.outcome = GamePlayerMessageOutcome::FriendAnswered;
        }
        DELETE_FRIEND => {
            let friend_name = message.base_mut().get_str_bytes(0x32).unwrap_or_default();
            report.friend_name.clone_from(&friend_name);
            let online_friend_id = game
                .find_player_by_name(&friend_name)
                .map(|player| player.player_id());
            report.target_player_id = online_friend_id;
            let exists = game
                .find_player(player_id)
                .is_some_and(|player| player.has_friend(&friend_name));
            if !exists {
                report.outcome = GamePlayerMessageOutcome::FriendMissing;
                return Some(Ok(report));
            }
            report
                .deliveries
                .push(GamePlayerMessageDelivery::World(publish_friend_delete(
                    game,
                    player_id,
                    online_friend_id.unwrap_or(0),
                    &friend_name,
                )));
            game.find_player_mut(player_id)
                .expect("friend owner сохранён после existence lookup")
                .delete_friend_state(&friend_name);
            let mut response = CMessage::new(0x000b_f71b);
            add_c_string(&mut response, &friend_name);
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                response.send_to_player(game.net_server(), player_id),
            ));
            report.outcome = GamePlayerMessageOutcome::FriendDeleted;
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
            report.deliveries.push(GamePlayerMessageDelivery::Around(
                game.send_player_shape_around(player_id, Some(player_id), &response),
            ));
            report.outcome = GamePlayerMessageOutcome::DisplayHeadPieceChanged;
        }
        QUERY_QUEST_TIME => {
            let remaining = game
                .find_player(player_id)
                .expect("quest-time player сохранён после context lookup")
                .quest_time_remaining(game_wall_time_seconds() as i32);
            let mut response = CMessage::new(0x000b_f72b);
            response.add_long(remaining);
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                response.send_to_player(game.net_server(), player_id),
            ));
            report.outcome = GamePlayerMessageOutcome::QuestTimeSent;
        }
        ACKNOWLEDGE_HEARTBEAT => {
            game.find_player_mut(player_id)
                .expect("heartbeat player сохранён после context lookup")
                .acknowledge_heartbeat();
            report.outcome = GamePlayerMessageOutcome::HeartbeatAcknowledged;
        }
        REFRESH_EXPIRED_EQUIPMENT_STATE => {
            let Some(goods_id) = message.base_mut().get_guid() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "expired equipment guid",
                )));
            };
            report.equipment_goods_id = Some(goods_id);
            let Some((state, packed_time)) = game.find_player(player_id).and_then(|player| {
                player.get_goods_by_id(goods_id).map(|goods| {
                    (
                        goods.addon_property_value(game.goods_factory(), GAP_EQUIP_STATE, 1),
                        goods.addon_property_value(game.goods_factory(), GAP_EQUIP_STATE, 2),
                    )
                })
            }) else {
                report.outcome = GamePlayerMessageOutcome::ExpiredEquipmentMissing;
                return Some(Ok(report));
            };
            if state != 2 || packed_time == 0 {
                report.outcome = GamePlayerMessageOutcome::ExpiredEquipmentStateIgnored;
                return Some(Ok(report));
            }
            let Some(source) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "expired equipment source",
                )));
            };
            report.equipment_source = Some(source);
            report.equipment_source_position = game.find_player(player_id).map(|player| {
                if source != 0 {
                    return -1;
                }
                player
                    .packet()
                    .query_goods_position(goods_id)
                    .map(|position| position as i32)
                    .unwrap_or_else(|| {
                        player
                            .equipment()
                            .query_goods_position_by_id(goods_id)
                            .map(|position| position.position())
                            .unwrap_or(u32::MAX)
                            .wrapping_add(2) as i32
                    })
            });
            let Some(elapsed_seconds) = equipment_state_elapsed_seconds(packed_time) else {
                report.outcome = GamePlayerMessageOutcome::ExpiredEquipmentTimeInvalid;
                return Some(Ok(report));
            };
            let elapsed_seconds = elapsed_seconds.round() as i32;
            report.equipment_elapsed_seconds = Some(elapsed_seconds);
            if elapsed_seconds / 60 <= 0x275f {
                report.outcome = GamePlayerMessageOutcome::ExpiredEquipmentStillActive;
                return Some(Ok(report));
            }
            let (identity, payload, mutated) = {
                let goods = game
                    .find_player_mut(player_id)
                    .expect("expired-equipment player сохранён после context lookup")
                    .get_goods_by_id_mut(goods_id)
                    .expect("expired equipment сохранён после addon validation");
                let mutated = goods.set_addon_property_modifier_core(GAP_EQUIP_STATE, 1, 3);
                (
                    goods.identity(),
                    runtime.encode_goods_for_old_client(goods),
                    mutated,
                )
            };
            report.equipment_state_mutated = Some(mutated);
            let mut response = CMessage::new(0x000b_f928);
            response.add_long(player_id);
            response.base_mut().add_guid(identity.ex_id);
            response.base_mut().add_ulong(payload.len() as u32);
            response.base_mut().add(&payload);
            report.deliveries.push(GamePlayerMessageDelivery::Around(
                game.send_player_shape_around(player_id, None, &response),
            ));
            report.outcome = GamePlayerMessageOutcome::ExpiredEquipmentPublished;
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
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                response.send_to_player(game.net_server(), player_id),
            ));
            report.outcome = GamePlayerMessageOutcome::HonorIdentitySent;
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
            game.find_player_mut(player_id)
                .expect("appellation player сохранён после context lookup")
                .request_change_appellation_state(appellation_id as u32);
            let _ = game.run_script_file(
                b"scripts/circle/honorrank/changeappellation.script",
                ScriptExecutionContext {
                    player_id: Some(player_id),
                    ..ScriptExecutionContext::default()
                },
                runtime,
            );
            report.outcome = GamePlayerMessageOutcome::AppellationChangeRequested;
        }
        QUERY_LOCAL_TIME => {
            let system_time = TagTime::local_now().fields();
            let mut response = CMessage::new(0x000b_f73f);
            for field in system_time {
                response.base_mut().add(&field.to_le_bytes());
            }
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                response.send_to_player(game.net_server(), player_id),
            ));
            report.outcome = GamePlayerMessageOutcome::LocalTimeSent;
        }
        CLAIM_LEI_TING_REWARD => {
            let Some(reward) = message.base_mut().get_word() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "LeiTing reward index",
                )));
            };
            report.lei_ting_reward = Some(reward);
            let Some(script) = LEI_TING_REWARD_SCRIPTS.get(usize::from(reward)) else {
                report.outcome = GamePlayerMessageOutcome::LeiTingInvalidReward;
                return Some(Ok(report));
            };
            if !game
                .find_player(player_id)
                .expect("LeiTing player сохранён после context lookup")
                .packet()
                .check_space(3)
            {
                let notice =
                    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"E19681"));
                report.deliveries.push(GamePlayerMessageDelivery::Player(
                    notice.send_to_player(game.net_server(), player_id),
                ));
                report.outcome = GamePlayerMessageOutcome::LeiTingPacketFull;
                return Some(Ok(report));
            }
            if !game
                .find_player_mut(player_id)
                .expect("LeiTing player сохранён после packet-space lookup")
                .change_fy_energy_flag(reward)
            {
                report.outcome = GamePlayerMessageOutcome::LeiTingRewardUnavailable;
                return Some(Ok(report));
            }
            publish_lei_ting_update(game, player_id, &mut report.deliveries);
            let _ = game.run_script_file(
                script,
                ScriptExecutionContext {
                    player_id: Some(player_id),
                    ..ScriptExecutionContext::default()
                },
                runtime,
            );
            report.outcome = GamePlayerMessageOutcome::LeiTingRewardClaimed;
        }
        _ => unreachable!("player opcode отфильтрован до decode"),
    }
    Some(Ok(report))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\playermessage.cpp

// ============================================================================
// FUNCTION: CPlayer::OnMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\playermessage.cpp:26
// RVA: 0x000FAAB0
// ADDRESS: 004faab0
// PROTOTYPE: void __thiscall OnMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
