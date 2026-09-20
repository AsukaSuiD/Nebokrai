//! Региональные стрелы CHeartLessArrowPhalanx2/3, различающиеся ID E5/E6.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/
//! heartlessarrowphalanx2.cpp и heartlessarrowphalanx3.cpp.
//!
//! Срок жизни проверяется абсолютным unsigned start+life. Один снимок
//! текущей клетки обходится целиком: после каждой допущенной попытки Attack
//! вызывается общий End, но флаг удаления не обрывает остальные контакты.
//! Attack проверяет смерть, фиксирует PK, переносит DaubPoison, затем читает
//! живого игрока по ID владельца и доставляет сырой OnBeenAttacked без RP.
//! MIN сохраняется до физического RNG; ELEMENT/SOUL читаются позднее, CCH
//! и знаковый процент урона принадлежат конструктору формы. Общий оружейный
//! расчёт сохраняет x87 factor и усечение критического урона.
//! Клиентский снимок содержит ID/уровень навыка, type/id владельца и остаток
//! времени перед CShape. Серверный decoder снимка не имеет достигнутого
//! caller-а; это не контракт сохранения региональной формы в БД.

use super::heartlessarrow::apply_daub_poison;
use super::weaponattack::fill_captured_weapon_damage;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::summonshape::{
    SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HeartlessArrowAttack {
    master: MasterInfo,
    skill_id: u32,
    skill_level: i32,
    damage_factor: i32,
    critical_chance: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CHeartlessArrowPhalanx {
    shape: CShape,
    started_at_ms: u32,
    lifetime_ms: u32,
    attack: HeartlessArrowAttack,
}

impl CHeartlessArrowPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля повторяют состояние исходной призванной формы")]
    pub(crate) fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_id: u32,
        skill_level: i32,
        damage_factor: i32,
        critical_chance: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE,
            id,
            ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape,
            started_at_ms,
            lifetime_ms,
            attack: HeartlessArrowAttack {
                master, skill_id, skill_level, damage_factor, critical_chance,
            },
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master }
    pub(crate) const fn attack_snapshot(&self) -> HeartlessArrowAttack { self.attack }

    pub(crate) fn encode_client_snapshot(
        &self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape,
            self.attack.skill_id as i32,
            self.attack.skill_level,
            self.attack.master.master_type,
            self.attack.master.master_id,
            self.started_at_ms,
            self.lifetime_ms,
            now_milliseconds,
        )
    }

    pub(crate) fn expired_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms
    }
}

impl HeartlessArrowAttack {
    fn attack_master(self) -> MasterInfo {
        if self.master.master_type == 400 { return self.master; }
        MasterInfo {
            master_type: self.master.master_type, master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }
}

fn calculate_heartless_arrow_attack(
    game: &mut CGame, snapshot: HeartlessArrowAttack, attack: &mut AttackInformation,
) {
    let source = game.find_player(attack.attacker_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    attack.skill_id = snapshot.skill_id;
    attack.skill_level = snapshot.skill_level as u8;
    attack.damage_modifier = 0;
    attack.hit_modifier = 0;
    attack.damage_factor = (f64::from(snapshot.damage_factor) * f64::from(0.01_f32)) as f32;
    if let Some(source) = source {
        fill_captured_weapon_damage(game, source, snapshot.critical_chance, attack);
    } else {
        // Native допускает NULL до SOUL, затем разыменовывает его. Сохраняем
        // достигнутые physical/element и RNG(1), не выдумывая SOUL или CCH.
        let physical = game.skill_random_below(1).max(0);
        attack.damages.push(AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 });
        attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: 0, mp_damage: 0 });
    }
}

pub(crate) fn apply_heartless_arrow_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, snapshot: HeartlessArrowAttack, region: i32,
    target: ShapeIdentity, runtime: &mut Runtime,
) {
    if game.move_shape_health(region, target).is_none_or(|hp| hp == 0) { return; }
    let master = snapshot.attack_master();
    let mut attack = AttackInformation::for_master(master);
    apply_daub_poison(game, attack.attacker_id, region, target, &mut || runtime.now_milliseconds());
    calculate_heartless_arrow_attack(game, snapshot, &mut attack);
    game.apply_owned_skill_contact(master, target, region, attack, runtime);
}
