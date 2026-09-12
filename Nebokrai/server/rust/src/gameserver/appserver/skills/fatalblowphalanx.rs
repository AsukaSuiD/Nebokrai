//! Снаряд смертельного удара боевого духа `CFatalBlowPhalanx`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/fatalblowphalanx.cpp`. Снаряд хранит снимок владельца,
//! цель, уровень и коэффициент навыка. На первом AI-проходе после создания он
//! рассчитывает один физический удар и удаляется; до этого проверка срока жизни
//! имеет приоритет. При наличии боевого духа и таблицы свойств формула
//! выполняет один вызов `legacy MSVCRT RNG`; иначе сохраняется атака без
//! составляющих урона, но с уже записанными идентификатором и коэффициентом навыка.
//! Поиск цели, проверка `IsAttackAble`, защита и сетевые последствия остаются
//! у исполняющего владельца, которому требуется доступ к нескольким сущностям.
//! Атака боевого духа усекается через исходный `i64` с чтением младших 32 бит;
//! процентный damage factor сохраняется в `f32` только после x87-умножения.

use super::basemagic::{SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK};
use super::fatalblow::FATAL_BLOW_SKILL_ID;
use super::thunder::scaled_battle_fairy_sprite;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_BF_ATTACK;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::summonshape::{
    SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot,
};
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FatalBlowPhalanxTick {
    Ready(ShapeIdentity),
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CFatalBlowPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    damage_factor: i32,
    target: ShapeIdentity,
    minimum_attack: i32,
    maximum_attack: i32,
}

pub(crate) fn calculate_owned_fatal_blow_attack(
    game: &mut CGame,
    phalanx: &CFatalBlowPhalanx,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let master = phalanx.master();
    let player = game.find_player(master.master_id)?;
    let combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let mut attack = AttackInformation::for_master(master);
    attack.skill_id = FATAL_BLOW_SKILL_ID;
    attack.skill_level = phalanx.skill_level as u8;
    attack.damage_factor = (f64::from(phalanx.damage_factor) * f64::from(0.01_f32)) as f32;
    let goods = player.war_soul_goods(game.goods_factory());
    let properties = game.skill_base_properties(FATAL_BLOW_SKILL_ID, phalanx.skill_level);
    if let (Some(goods), Some(properties)) = (goods, properties) {
        let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
        let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
        let fairy_attack = scaled_battle_fairy_sprite(
            goods.addon_property_value(game.goods_factory(), GAP_BF_ATTACK, 1),
        );
        let width = maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1);
        let damage = fairy_attack
            .wrapping_add(game.skill_random_below(width))
            .wrapping_add(minimum)
            .max(0);
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Physical,
            hp_damage: damage,
            mp_damage: 0,
        });
    }
    Some((attack, combat, occupation, attacker_level))
}

impl CFatalBlowPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        damage_factor: i32,
        target: ShapeIdentity,
        minimum_attack: i32,
        maximum_attack: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE,
            id,
            ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape,
            master,
            started_at_ms,
            lifetime_ms,
            skill_level,
            damage_factor,
            target,
            minimum_attack,
            maximum_attack,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape {
        &self.shape
    }

    pub(crate) const fn shape_mut(&mut self) -> &mut CShape {
        &mut self.shape
    }

    pub(crate) const fn master(&self) -> MasterInfo {
        self.master
    }

    pub(crate) const fn skill_level(&self) -> i32 {
        self.skill_level
    }

    pub(crate) fn tick(&mut self, now_ms: u32) -> FatalBlowPhalanxTick {
        if now_ms.wrapping_sub(self.started_at_ms) > self.lifetime_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            FatalBlowPhalanxTick::Expired
        } else {
            FatalBlowPhalanxTick::Ready(self.target)
        }
    }

    pub(crate) fn finish(&mut self) {
        self.shape.set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub(crate) fn encode_client_snapshot(
        &self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape,
            FATAL_BLOW_SKILL_ID as i32,
            self.skill_level,
            self.target.object_type,
            self.target.id,
            self.started_at_ms,
            self.lifetime_ms,
            now_milliseconds,
        )
    }
}


// Восстановление снаряда из массива байтов пока не достигнуто и сохранено RAW.
// ============================================================================
// FUNCTION: CFatalBlowPhalanx::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fatalblowphalanx.cpp:181
// RVA: 0x001ED890
// ADDRESS: 005ed890
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
