//! Каноническое периодическое состояние потери крови `CBloodLossState`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bloodlossstate.cpp`. Состояние хранит снимок владельца,
//! выполняет два отдельных чтения часов, затем ровно два обращения к
//! `legacy MSVCRT RNG`: равномерный урон на включительном диапазоне и проверку
//! критического удара.
//! Встроенная `tagAttackInformation` сохраняет конструкторские skill-id
//! `0x7fffffff` и уровень `1`: очистка между тиками уровень не перезаписывает.
//! `CalculateAttackPower` `0x005E3E30..0x005E3F82` держит сумму броска,
//! минимального урона и `float`-модификатора в x87 до `__ftol2`, а critical
//! умножает уже целый урон на `float` rate и делает `FISTP` с режимом
//! усечения. Первый результат берётся как младший DWORD усечённого `i64`,
//! тогда как critical сохраняет отдельную overflow-семантику `FISTP dword`;
//! оба пути моделируются через `f64` без промежуточного округления Rust `f32`.
//! Этот же владелец извлекает каноническое состояние на такте ИИ, возвращает
//! его до применения удара и передаёт рассчитанную атаку координатору `CGame`.
//! DB-запись буквально сохраняет десять DWORD `MasterInfo`, остаток срока,
//! частоту, биты двух `float` и границы атаки. Координатные перегрузки `Begin`
//! остаются ниже как RAW без параллельного изменяемого представления.
//! Клиентский `GetRemainedTime` разделяет общее тело по `0x00606320` с
//! периодическими poison/burn-состояниями и читает wrapping clock дважды.

use super::bloodloss::BLOOD_LOSS_SKILL_ID;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::appserver::skills::fightdefense::truncate_original;
use crate::gameserver::appserver::skills::thunder::truncate_original_i64_low;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;
const DEFAULT_PERIODIC_SKILL_ID: u32 = i32::MAX as u32;
pub(crate) const BLOOD_LOSS_STATE_BYTES: usize = 68;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BloodLossStateTick {
    Pending,
    Attack(AttackInformation),
    Ended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BloodLossState {
    master: MasterInfo,
    started_at_ms: u32,
    keep_time_ms: u32,
    frequency_ms: u32,
    damage_factor_bits: u32,
    damage_modifier_bits: u32,
    minimum_attack: u16,
    maximum_attack: u16,
    attack_count: u32,
}

impl BloodLossState {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют состоянию EXE")]
    pub(crate) const fn new(
        master: MasterInfo,
        started_at_ms: u32,
        keep_time_ms: u32,
        frequency_ms: u32,
        damage_factor: f32,
        damage_modifier: f32,
        minimum_attack: u16,
        maximum_attack: u16,
    ) -> Self {
        Self {
            master,
            started_at_ms,
            keep_time_ms,
            frequency_ms,
            damage_factor_bits: damage_factor.to_bits(),
            damage_modifier_bits: damage_modifier.to_bits(),
            minimum_attack,
            maximum_attack,
            attack_count: 0,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        BLOOD_LOSS_SKILL_ID
    }

    pub(crate) const fn master(self) -> MasterInfo {
        self.master
    }

    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let elapsed = now_ms.wrapping_sub(self.started_at_ms);
        if elapsed >= self.keep_time_ms {
            0
        } else {
            self.keep_time_ms.wrapping_sub(elapsed) as i32
        }
    }

    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds)
    }

    pub(crate) fn decode(
        payload: &[u8],
        offset: usize,
        now_ms: u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != BLOOD_LOSS_SKILL_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        let master = MasterInfo {
            master_type: reader.read_i32()?,
            master_id: reader.read_i32()?,
            master_guild_id: reader.read_i32()?,
            master_team_id: reader.read_i32()?,
            master_union_id: reader.read_i32()?,
            master_country_id: reader.read_i32()?,
            permitted_to_kill_player: reader.read_i32()?,
            permitted_to_kill_teammate: reader.read_i32()?,
            permitted_to_kill_guild_member: reader.read_i32()?,
            permitted_to_kill_criminal: reader.read_i32()?,
        };
        Ok(Self::new(
            master,
            now_ms,
            reader.read_u32()?,
            reader.read_u32()?,
            f32::from_bits(reader.read_u32()?),
            f32::from_bits(reader.read_u32()?),
            reader.read_u16()?,
            reader.read_u16()?,
        ))
    }

    pub(crate) fn encoded(self, now_ms: u32) -> [u8; BLOOD_LOSS_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(BLOOD_LOSS_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(BLOOD_LOSS_SKILL_ID);
        for value in [
            self.master.master_type,
            self.master.master_id,
            self.master.master_guild_id,
            self.master.master_team_id,
            self.master.master_union_id,
            self.master.master_country_id,
            self.master.permitted_to_kill_player,
            self.master.permitted_to_kill_teammate,
            self.master.permitted_to_kill_guild_member,
            self.master.permitted_to_kill_criminal,
        ] {
            writer.write_i32(value);
        }
        writer.write_u32(self.client_time(now_ms) as u32);
        writer.write_u32(self.frequency_ms);
        writer.write_u32(self.damage_factor_bits);
        writer.write_u32(self.damage_modifier_bits);
        writer.write_u16(self.minimum_attack);
        writer.write_u16(self.maximum_attack);
        bytes
            .try_into()
            .expect("размер состояния потери крови фиксирован")
    }

    pub(crate) fn encoded_for_install(self) -> [u8; BLOOD_LOSS_STATE_BYTES] {
        self.encoded(self.started_at_ms)
    }

    pub(crate) fn activate_loaded(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub(crate) fn tick(
        &mut self,
        lifetime_now_ms: u32,
        frequency_now_ms: u32,
        target_dead: bool,
        critical_chance: u16,
        critical_rate: f32,
        random: &mut dyn FnMut(i32) -> i32,
    ) -> BloodLossStateTick {
        if lifetime_now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms || target_dead {
            return BloodLossStateTick::Ended;
        }
        let delay = self.frequency_ms.wrapping_mul(self.attack_count);
        if frequency_now_ms.wrapping_sub(self.started_at_ms) <= delay {
            return BloodLossStateTick::Pending;
        }
        self.attack_count = self.attack_count.wrapping_add(1);
        BloodLossStateTick::Attack(self.attack(critical_chance, critical_rate, random))
    }

    fn attack(
        self,
        critical_chance: u16,
        critical_rate: f32,
        random: &mut dyn FnMut(i32) -> i32,
    ) -> AttackInformation {
        let minimum = i32::from(self.minimum_attack);
        let maximum = i32::from(self.maximum_attack);
        let span = minimum.abs_diff(maximum).wrapping_add(1) as i32;
        let rolled = minimum.wrapping_add(random(span));
        let mut damage = truncate_original_i64_low(
            f64::from(rolled) + f64::from(f32::from_bits(self.damage_modifier_bits)),
        );
        if damage < 0 {
            damage = 0;
        }
        let critical = random(100) < i32::from(critical_chance);
        if critical {
            damage = truncate_original(f64::from(damage) * f64::from(critical_rate));
        }
        AttackInformation {
            skill_id: DEFAULT_PERIODIC_SKILL_ID,
            skill_level: 1,
            attacker_type: self.master.master_type,
            attacker_id: self.master.master_id,
            attacker_team_id: self.master.master_team_id,
            attacker_faction_id: self.master.master_guild_id,
            attacker_union_id: self.master.master_union_id,
            hit_modifier: 0,
            damage_factor: f32::from_bits(self.damage_factor_bits),
            damage_modifier: 0,
            critical,
            blast_attack: false,
            full_miss: 0,
            damages: vec![AttackPower {
                kind: AttackPowerType::Physical,
                hp_damage: damage,
                mp_damage: 0,
            }],
        }
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической around-доставки")]
pub(crate) fn send_blood_loss_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: BloodLossState,
    begin: bool,
    now_ms: u32,
) {
    let mut message = CMessage::new(if begin {
        STATE_BEGIN_MESSAGE
    } else {
        STATE_END_MESSAGE
    });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_ulong(state.client_state_time(|| now_ms));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn update_player_blood_loss_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    runtime: &mut Runtime,
) -> bool {
    let Some(mut state) = game
        .find_player_mut(player_id)
        .and_then(CPlayer::take_blood_loss_state_for_ai)
    else {
        return false;
    };
    let target = game.find_player(player_id).and_then(|player| {
        Some((
            player.shape().identity(),
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
            player.server_region_id()?,
            player.is_dead(),
            player.combat_properties().cch,
        ))
    });
    let Some((identity, x, y, region_id, dead, critical_chance)) = target else {
        return false;
    };
    let lifetime_now_ms = runtime.now_milliseconds();
    let frequency_now_ms = runtime.now_milliseconds();
    let critical_rate = game.globe_setup().critical_rate();
    let tick = state.tick(
        lifetime_now_ms,
        frequency_now_ms,
        dead,
        critical_chance,
        critical_rate,
        &mut |maximum| game.skill_random_below(maximum),
    );
    match tick {
        BloodLossStateTick::Pending => {
            if let Some(player) = game.find_player_mut(player_id) {
                let _ = player.replace_blood_loss_state(state);
            }
        }
        BloodLossStateTick::Attack(attack) => {
            let master = state.master();
            if let Some(player) = game.find_player_mut(player_id) {
                let _ = player.replace_blood_loss_state(state);
            }
            game.apply_owned_skill_attack_to_player(
                master,
                player_id,
                region_id,
                attack,
                runtime,
            );
        }
        BloodLossStateTick::Ended => {
            if let Some(player) = game.find_player_mut(player_id) {
                player.finish_blood_loss_state(state);
            }
            send_blood_loss_state_visual(
                game,
                region_id,
                identity,
                x,
                y,
                state,
                false,
                lifetime_now_ms,
            );
            let _ = game.publish_player_states(player_id);
        }
    }
    true
}

pub(crate) fn update_monster_blood_loss_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool {
    let Some(mut owner) = game.take_region_owner(region_id) else {
        return false;
    };
    let state_and_target = owner
        .base_mut()
        .find_monster_by_id_mut(monster_id)
        .and_then(|monster| {
            let state = monster.move_shape_mut().take_blood_loss_state_for_ai()?;
            let shape = monster.move_shape().shape();
            Some((
                state,
                shape.identity(),
                shape.get_tile_x().ok()?,
                shape.get_tile_y().ok()?,
                monster.hit_points() == 0,
            ))
        });
    game.restore_region_owner(owner);
    let Some((mut state, identity, x, y, dead)) = state_and_target else {
        return false;
    };
    let lifetime_now_ms = runtime.now_milliseconds();
    let frequency_now_ms = runtime.now_milliseconds();
    let critical_rate = game.globe_setup().critical_rate();
    let tick = state.tick(
        lifetime_now_ms,
        frequency_now_ms,
        dead,
        0,
        critical_rate,
        &mut |maximum| game.skill_random_below(maximum),
    );
    match tick {
        BloodLossStateTick::Pending => {
            if let Some(mut owner) = game.take_region_owner(region_id) {
                if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                    let _ = monster.move_shape_mut().replace_blood_loss_state(state);
                }
                game.restore_region_owner(owner);
            }
        }
        BloodLossStateTick::Attack(attack) => {
            let master = state.master();
            if let Some(mut owner) = game.take_region_owner(region_id) {
                if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                    let _ = monster.move_shape_mut().replace_blood_loss_state(state);
                }
                game.restore_region_owner(owner);
            }
            game.apply_owned_skill_attack_to_monster(
                master,
                monster_id,
                region_id,
                attack,
                runtime,
            );
        }
        BloodLossStateTick::Ended => {
            if let Some(mut owner) = game.take_region_owner(region_id) {
                if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                    monster.move_shape_mut().finish_blood_loss_state(state);
                }
                game.restore_region_owner(owner);
            }
            send_blood_loss_state_visual(
                game,
                region_id,
                identity,
                x,
                y,
                state,
                false,
                lifetime_now_ms,
            );
        }
    }
    true
}

// Остаются недостигнутыми только координатные перегрузки `Begin`.
// ============================================================================
// FUNCTION: CBloodLossState::CBloodLossState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bloodlossstate.cpp:34
// RVA: 0x001E38E0
// ADDRESS: 005e38e0
// PROTOTYPE: undefined __thiscall CBloodLossState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBloodLossState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bloodlossstate.cpp:75
// RVA: 0x001E39E0
// ADDRESS: 005e39e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBloodLossState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bloodlossstate.cpp:89
// RVA: 0x001E3A80
// ADDRESS: 005e3a80
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
