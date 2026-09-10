//! Базовая атака боевой феи GameServer (`SKILL_BATTLEFAIRY_BASE_ATTACK`).
//! Успешный Begin возвращает Begun; проверка задержки первого AI выполняется
//! при повторном входе в том же Run по исходному раннему отсчёту.
//! BFBaseAttack::Begin (0x00517020) оставляет +0x50 равным нулю.
//! Первый AI проверяет цель, отправляет start через effect(+8, 0)
//! (0x00517802..0x0051780E), затем переводит стадию в Check и проверяет
//! задержку. Смерть цели в этой первой фазе даёт один 4,10 и ZHGS0050;
//! повторная проверка перед выстрелом сохраняет отдельный отказ ниже.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/battlefairybasemagic.cpp`. Отдельный FIFO боевой феи
//! проходит общий `SkillExecutionKernel`, но не блокирует движение игрока.
//! Начало, выстрел и обязательное завершение используют визуальный тип `700`;
//! ошибки используют отдельный префикс `4`. Общий owned Effect публикуется
//! через `battlefairyskill`, а проверки цели, cooldown, стадии и создание
//! `CBattleFairyBaseMagicPhalanx` принадлежат этому owner-у; cooldown
//! использует exact `CSkill::IsRestored`. Общий
//! `CState::GetSufferer` сохраняет player/NPC/monster/
//! build/gate; NPC отклоняется как мёртвый, а постройки проходят собственный
//! region-owned defence. `CGame` только предоставляет владельцев, регион и
//! доставку.
//! AI (0x00517610) при отсутствии цели вызывает End(0); смерть перед выстрелом
//! выдаёт 4,10, ZHGS0050, ещё один 4,10 и End(0). После попытки Summon
//! (+0x94, 0x005179d9) всегда следует End(1), независимо от создания снаряда.
//! Задержка — unsigned now >= wrapping(start + delay), cmp/jb 0x00517836.
//! Уже первый AI использует ранний отсчёт CState::Begin из отдельного
//! контекста феи, а не локальное время после OnBeginSkill и проверок.
//! Отказ до успешного Begin заканчивается action 3, затем внешним 4,2
//! планировщика; повторный AI-отказ такого внешнего ответа не добавляет.
//! В Rust этот внешний 4,2 отправляет только координатор после общего
//! End(0), без дополнительной публикации в concrete Begin.
//! Begin после общего начала создаёт Effect размером 0xC (vtable
//! 0x00656988), сохраняет его в skill+0x34 и вызывает BeginVisualEffect(1)
//! до конкретной проверки (0x0051704A..0x0051708A). Wire-контракт
//! Effect::Update (0x005170E0) и его базовый хвост (0x005175B3) сохранены
//! общей границей `battlefairyskill`; mode 1 берёт живую цель либо
//! координаты skill+0x24/+0x28 и добавляет время из skill+0x54.
//! Время выстрела записывается в skill+0x54 до Effect(mode 1): AI
//! 0x00517940..0x0051799A временно вызывает SetTileXY с целочисленным
//! CPlayer::m_ptWarSoul (+0xCA8/+0xCAC), вычисляет RealDistance(target)
//! (0x0045B780) * summoned_speed и восстанавливает tile-позицию игрока.
//! SetWarSoulXY (0x0042DF50) подтверждает POINT-поля; дробные visual X/Y
//! лежат отдельно (+0xCB0/+0xCB4). SetTileXY (0x0045B170) добавляет 0.5f,
//! а RealDistance использует truncated координаты и figure обеих сторон,
//! не ближайшую точку постройки. Rust подставляет координаты только в
//! Copy-проекцию ShapeView и сохраняет время в concrete execution; живой
//! player и его пространственная регистрация при этом не изменяются.

use super::baseattack::real_distance;
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_ELEMENT_MODIFIER, SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_SUMMONED_LIFETIME,
    SKILL_USAGE_SUMMONED_SPEED, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::battlefairybasemagicphalanx::CBattleFairyBaseMagicPhalanx;
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

const PLAYER_TYPE: i32 = 400;

fn add_legacy_c_string(
    message: &mut crate::nets::basemessage::CBaseMessage,
    value: &[u8],
) {
    let length = value.iter().position(|byte| *byte == 0).unwrap_or(value.len());
    message.add(&value[..length]);
    message.add_byte(0);
}

pub(crate) const BATTLE_FAIRY_BASE_MAGIC_SKILL_ID: u32 = 0x224;
pub(crate) const DENIED_STATE_A: u32 = 0x192;
pub(crate) const DENIED_STATE_B: u32 = 0x67;
pub(crate) const DENIED_STATE_C: u32 = 0xd2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyBaseMagicExecutionState {
    kernel: SkillExecutionKernel<BattleFairySkillDispatch>,
    target: ShapeIdentity,
    attack_time: u32,
}

impl BattleFairyBaseMagicExecutionState {
    pub(crate) const fn begin(
        dispatch: BattleFairySkillDispatch,
        target: ShapeIdentity,
        started_at_ms: u32,
    ) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            target,
            attack_time: 0,
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<BattleFairySkillDispatch> {
        &self.kernel
    }

    pub(crate) fn kernel_mut(
        &mut self,
    ) -> &mut SkillExecutionKernel<BattleFairySkillDispatch> {
        &mut self.kernel
    }

    pub(crate) const fn attack_time(&self) -> u32 {
        self.attack_time
    }

    pub(crate) const fn target(self) -> ShapeIdentity {
        self.target
    }
}

pub(crate) fn execute_battle_fairy_base_magic<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let rejected = || QueuedSkillExecutionOutcome {
        state: QueuedSkillExecutionState::Rejected,
        first_contact: false,
    };
    let pending = || QueuedSkillExecutionOutcome {
        state: QueuedSkillExecutionState::Pending,
        first_contact: false,
    };
    let Some(player) = game.find_player(player_id) else {
        return rejected();
    };
    let Some(region_id) = player.server_region_id() else {
        return rejected();
    };
    let skill_level = match dispatch {
        BattleFairySkillDispatch::SelfTarget { skill_level, .. }
        | BattleFairySkillDispatch::Point { skill_level, .. }
        | BattleFairySkillDispatch::Object { skill_level, .. } => skill_level,
    };
    let Some(properties) = game.skill_base_properties(
        BATTLE_FAIRY_BASE_MAGIC_SKILL_ID,
        skill_level,
    ) else {
        return rejected();
    };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let summoned_speed = properties.query_property(SKILL_USAGE_SUMMONED_SPEED);
    let summoned_lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let started_at_ms = runtime.now_milliseconds();
    let target = match dispatch {
        BattleFairySkillDispatch::Object { target, .. } => target,
        _ => {
            game.update_player_skill_visual(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, 10);
            game.send_skill_system_info(player_id, b"ZHGS0045");
            return rejected();
        }
    };

    if game.battle_fairy_base_magic(player_id).is_some()
        && game.base_magic_target_view(region_id, target).is_none()
    {
        game.update_player_skill_visual(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, 10);
        return rejected();
    }

    if game.battle_fairy_base_magic(player_id).is_none() {
        if target.object_type == PLAYER_TYPE && target.id == player_id {
            game.update_player_skill_visual(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, 10);
            game.send_skill_system_info(player_id, b"ZHGS0045");
            return rejected();
        }
        if game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_A)
            || game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_B)
        {
            let mut message = CMessage::new(0x000b_f807);
            message.add_ulong(0xffff_ffff);
            add_legacy_c_string(message.base_mut(), game.get_string_by_id(b"ZHGS0046"));
            let _ = message.send_to_player(game.net_server(), player_id);
            return rejected();
        }
        if game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_C) {
            let mut message = CMessage::new(0x000b_f807);
            message.add_ulong(0xffff_ffff);
            add_legacy_c_string(message.base_mut(), game.get_string_by_id(b"ZHGS0047"));
            let _ = message.send_to_player(game.net_server(), player_id);
            return rejected();
        }
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            game.battle_fairy_skill_last_used_ms(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID),
            reuse_delay_ms,
            cooldown_now_ms,
        ) {
            game.update_player_skill_visual(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, 0x0d);
            game.send_skill_system_info(player_id, b"ZHGS0048");
            return rejected();
        }
        let Some(_target_view) = game.base_magic_target_view(region_id, target) else {
            game.update_player_skill_visual(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, 10);
            game.send_skill_system_info(player_id, b"ZHGS0050");
            return rejected();
        };
        let (source_x, source_y) = match (
            player.shape().get_tile_x(),
            player.shape().get_tile_y(),
        ) {
            (Ok(x), Ok(y)) => (x, y),
            _ => return rejected(),
        };
        let Some((target_x, target_y)) =
            game.base_magic_target_point(region_id, source_x, source_y, target)
        else {
            game.update_player_skill_visual(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, 10);
            game.send_skill_system_info(player_id, b"ZHGS0050");
            return rejected();
        };
        let path = game.base_magic_path(
            region_id,
            source_x,
            source_y,
            target_x,
            target_y,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            game.update_player_skill_visual(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, 0x0b);
            game.send_skill_system_info(player_id, b"ZHGS0049");
            return rejected();
        }
        let target_dead = game.base_magic_target_dead(region_id, target);
        if target_dead {
            game.update_player_skill_visual(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, 10);
            game.send_skill_system_info(player_id, b"ZHGS0050");
            return rejected();
        }
        let execution = BattleFairyBaseMagicExecutionState::begin(
            dispatch,
            target,
            started_at_ms,
        );
        game.begin_battle_fairy_base_magic(player_id, execution);
        return QueuedSkillExecutionOutcome {
            state: QueuedSkillExecutionState::Begun,
            ..pending()
        };
    } else if game.battle_fairy_base_magic(player_id)
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return rejected();
    }

    if game.battle_fairy_base_magic(player_id)
        .is_some_and(|state| state.kernel().stage() == SkillStage::Begin)
    {
        if game.base_magic_target_dead(region_id, target) {
            game.update_player_skill_visual(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, 10);
            game.send_skill_system_info(player_id, b"ZHGS0050");
            return rejected();
        }
        game.update_player_skill_visual(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, 0);
        if let Some(kernel) = game.battle_fairy_execution_mut(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    if runtime.now_milliseconds() < game.battle_fairy_base_magic(player_id)
            .map_or(started_at_ms, |state| state.kernel().started_at_ms())
            .wrapping_add(delay_ms)
    {
        return pending();
    }

    let Some(target_view) = game.base_magic_target_view(region_id, target) else {
        game.update_player_skill_visual(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, 10);
        return rejected();
    };
    let target_dead = game.base_magic_target_dead(region_id, target);
    if target_dead {
        game.update_player_skill_visual(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, 10);
        game.send_skill_system_info(player_id, b"ZHGS0050");
        game.update_player_skill_visual(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, 10);
        return rejected();
    }
    let Some(player) = game.find_player(player_id) else {
        return rejected();
    };
    let Some(source_view) = player.shape_view() else {
        return rejected();
    };
    let war_soul_point = player.war_soul_point();
    let mut fire_source_view = source_view;
    fire_source_view.pos_x_bits = (war_soul_point.x as f32 + 0.5).to_bits();
    fire_source_view.pos_y_bits = (war_soul_point.y as f32 + 0.5).to_bits();
    let Some((target_x, target_y)) = game.base_magic_target_point(
        region_id,
        source_view.tile_x,
        source_view.tile_y,
        target,
    ) else {
        return rejected();
    };
    let attack_time = fire_source_view
        .real_distance(Some(target_view))
        .wrapping_mul(summoned_speed as i32);
    if let Some(super::kernel::BattleFairyExecution::BaseMagic(state)) =
        game.battle_fairy_execution_state_mut(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID)
    {
        state.attack_time = attack_time as u32;
    }
    game.update_player_skill_visual(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, 1);

    let forced_distance = real_distance(
        source_view.tile_x,
        source_view.tile_y,
        target_x,
        target_y,
    ) as u32;
    let path = game.base_magic_path(
        region_id,
        source_view.tile_x,
        source_view.tile_y,
        target_x,
        target_y,
        Some(forced_distance),
    );
    let has_war_soul = game
        .find_player(player_id)
        .and_then(|player| player.war_soul_goods(game.goods_factory()))
        .is_some();
    let mut summoned = false;
    if has_war_soul && path.iter().all(|cell| cell.2 != 2) {
        let player = game
            .find_player(player_id)
            .expect("владелец боевой феи сохранён");
        let permissions = player.pk_permissions();
        let master = MasterInfo {
            master_type: PLAYER_TYPE,
            master_id: player_id,
            master_guild_id: player.faction_id(),
            master_team_id: player.team_id(),
            master_union_id: player.union_id(),
            master_country_id: i32::from(player.country()),
            permitted_to_kill_player: i32::from(permissions.player),
            permitted_to_kill_teammate: i32::from(permissions.teammate),
            permitted_to_kill_guild_member: i32::from(permissions.guild_member),
            permitted_to_kill_criminal: i32::from(permissions.criminal),
        };
        let summon_id = game.allocate_summon_shape_id();
        let summon_started_at_ms = runtime.now_milliseconds();
        let mut phalanx = CBattleFairyBaseMagicPhalanx::new(
            summon_id,
            master,
            summon_started_at_ms,
            summoned_lifetime,
            skill_level,
            minimum_attack,
            maximum_attack,
            element_modifier,
            attack_time as u32,
            target,
        );
        phalanx.shape_mut().set_region_id(region_id);
        let (tile_x, tile_y) = path
            .first()
            .map(|cell| (cell.0, cell.1))
            .unwrap_or((source_view.tile_x, source_view.tile_y));
        let result = game.add_battle_fairy_base_magic_phalanx(
            region_id,
            phalanx,
            tile_x,
            tile_y,
            summon_started_at_ms,
            runtime,
        );
        summoned = result.is_some_and(|result| result.is_ok());
        tracing::trace!(region_id, player_id, summon_id, ?result, "создан снаряд базовой атаки боевой феи");
    }

    if let Some(state) = game.battle_fairy_execution_mut(player_id, BATTLE_FAIRY_BASE_MAGIC_SKILL_ID) {
        let _ = state
            .advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state
            .advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state
            .advance(SkillStage::Attack, SkillStage::Apply);
    }
    QueuedSkillExecutionOutcome {
        state: if summoned {
            QueuedSkillExecutionState::Completed
        } else {
            QueuedSkillExecutionState::RejectedAfterUse
        },
        first_contact: false,
    }
}
