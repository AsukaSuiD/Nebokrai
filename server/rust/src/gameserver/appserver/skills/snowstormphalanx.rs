//! Адаптер снежной бури к CShape, региону и элементальной атаке.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/snowstormphalanx.cpp/.h.
//! Окна клеток и часы находятся в zone/skills/snowstorm.rs.
//! Для уровней 1–3 размеры маски равны 5×5. Initialize после SetTile
//! расходует X/Y RNG для каждой цели каждого окна в текущем Rust-пути.
//! AI сохраняет last-attack до обхода; счётчик окна увеличивает после callbacks.
//! Клетки не дедуплицируются, нулевая пара завершает текущее окно. Пакет
//! содержит skill/level/master type/id, срок, частоту и весь массив клеток.
//! Нулевая частота, число целей >25 и невозможный размер массива отклоняются
//! явно: исходные деление на ноль и выход за массив не воспроизводятся.

use nebokrai_shared::protocol::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_shared::values::CGuid;
use nebokrai_zone::skills::{SNOW_STORM_SKILL_ID, SnowStormPhalanx};

pub(crate) use nebokrai_zone::skills::{SNOW_STORM_SCOPE_AREA, SnowStormAttack, SnowStormParametersError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CSnowStormPhalanx {
    shape: CShape,
    rule: SnowStormPhalanx,
}

impl CSnowStormPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля конструктора исходной области")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, frequency_ms: u32, minimum_attack: i32,
        maximum_attack: i32, element_modifier: i32, target_count: u32,
    ) -> Result<Self, SnowStormParametersError> {
        let attack = SnowStormAttack { master, skill_level, minimum_attack, maximum_attack, element_modifier };
        let rule = SnowStormPhalanx::new(attack, started_at_ms, lifetime_ms, frequency_ms, target_count)?;
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Ok(Self { shape, rule })
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.rule.master() }
    pub(crate) const fn attack_snapshot(&self) -> SnowStormAttack { self.rule.attack_snapshot() }

    pub(crate) fn initialize(&mut self, random_below: &mut dyn FnMut(i32) -> i32) {
        let center_x = self.shape.get_tile_x().unwrap_or(i32::MIN);
        let center_y = self.shape.get_tile_y().unwrap_or(i32::MIN);
        self.rule.initialize(center_x, center_y, random_below);
    }

    pub(crate) const fn expired_at(&self, now: u32) -> bool {
        self.rule.expired_at(now)
    }

    pub(crate) const fn attack_due_at(&self, now: u32) -> bool {
        self.rule.attack_due_at(now)
    }

    pub(crate) fn mark_attack_at(&mut self, now: u32) { self.rule.mark_attack_at(now); }
    pub(crate) fn advance_attack_window(&mut self) { self.rule.advance_attack_window(); }

    pub(crate) fn current_cell(&self, index: u32) -> Option<(i32, i32)> {
        self.rule.current_cell(index)
    }

    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_u32(SNOW_STORM_SKILL_ID);
        writer.write_i32(self.rule.skill_level());
        writer.write_i32(self.rule.master().master_type);
        writer.write_i32(self.rule.master().master_id);
        writer.write_u32(timed_client_state_time(self.rule.started_at_ms(), self.rule.lifetime_ms(), now));
        writer.write_u32(self.rule.lifetime_ms());
        writer.write_u32(self.rule.frequency_ms());
        writer.write_u32(self.rule.cells().len() as u32);
        for &(x, y) in self.rule.cells() { writer.write_i32(x); writer.write_i32(y); }
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}

pub(crate) fn apply_snow_storm_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, snapshot: SnowStormAttack, region: i32,
    target: ShapeIdentity, runtime: &mut Runtime,
) {
    if game.move_shape_health(region, target).is_none_or(|hp| hp == 0) { return; }
    let master = snapshot.attack_master();
    let mut attack = AttackInformation::for_master(master);
    attack.skill_id = SNOW_STORM_SKILL_ID;
    attack.skill_level = snapshot.skill_level as u8;
    attack.damage_modifier = 0;
    attack.damage_factor = 1.0;
    attack.hit_modifier = 100;
    let damage = snapshot.element_damage(|width| game.skill_random_below(width));
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 });
    game.apply_owned_skill_contact(master, target, region, attack, runtime);
}
