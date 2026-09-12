//! Прицельный региональный снаряд базовой стрельбы Archery.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/archeryphalanx.cpp.
//! Два независимых чтения часов сравнивают unsigned start+life и start+delay.
//! После задержки цель заново ищется в фактическом регионе формы по type/id
//! с GUID_INVALID. Attack проверяет смерть, фиксирует PK и доставляет сырой
//! OnBeenAttacked без допуска, DaubPoison и RP. End только отмечает удаление,
//! после контакта, без немедленного сообщения выхода.
//! Calculate ищет игрока по attacker ID независимо от сохранённого типа.
//! Отсутствие игрока или таблицы оставляет исходную пустую атаку. Живые
//! weapon modifier, hit, MIN/MAX, ELEMENT/SOUL и CCH читаются в исходном
//! порядке; физический RNG получает max(MAX-MIN,0), без +1.
//! Неиспользуемые MIN/MAX/ELEMENT конструктора и выделение CScope не
//! дублируются: область не участвует ни в выборе цели, ни в расчёте.
//! Клиентский снимок содержит skill/level, master type/id и остаток времени.
//! Серверный decoder ниже не имеет достигнутого caller-а.

use super::archery::ARCHERY_SKILL_ID;
use super::weaponattack::{PlayerWeaponRoll, fill_ordinary_weapon_damage};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::summonshape::{
    SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArcheryAttack {
    master: MasterInfo,
    skill_level: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CArcheryPhalanx {
    shape: CShape,
    started_at_ms: u32,
    lifetime_ms: u32,
    attack_delay_ms: u32,
    target: ShapeIdentity,
    attack: ArcheryAttack,
}

impl CArcheryPhalanx {
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, attack_delay_ms: u32, target: ShapeIdentity,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape, started_at_ms, lifetime_ms, attack_delay_ms,
            target: ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..target },
            attack: ArcheryAttack { master, skill_level },
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master }
    pub(crate) const fn target(&self) -> ShapeIdentity { self.target }
    pub(crate) const fn attack_snapshot(&self) -> ArcheryAttack { self.attack }

    pub(crate) fn expired_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms
    }

    pub(crate) fn attack_due_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.attack_delay_ms) < now_ms
    }

    pub(crate) fn end(&mut self) {
        self.shape.set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub(crate) fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, ARCHERY_SKILL_ID as i32, self.attack.skill_level,
            self.attack.master.master_type, self.attack.master.master_id,
            self.started_at_ms, self.lifetime_ms, now_milliseconds,
        )
    }
}

impl ArcheryAttack {
    fn attack_master(self) -> MasterInfo {
        if self.master.master_type == 400 { return self.master; }
        MasterInfo {
            master_type: self.master.master_type, master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }
}

fn calculate_archery_attack(
    game: &mut CGame, snapshot: ArcheryAttack, target: (i32, ShapeIdentity),
    attack: &mut AttackInformation,
) {
    let Some(player) = game.find_player(attack.attacker_id) else { return; };
    let Some(properties) = game.skill_base_properties(ARCHERY_SKILL_ID, snapshot.skill_level)
    else { return; };
    let source = (player.shape().get_region_id(), player.shape().identity());
    attack.skill_id = ARCHERY_SKILL_ID;
    attack.skill_level = snapshot.skill_level as u8;
    attack.damage_modifier = 0;
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
    attack.damage_factor = player.weapon_modifier(
        game.goods_factory(), i32::from(target_level), divisor, minimum_factor,
    );
    attack.hit_modifier = properties.query_property(20_001) as i32;
    fill_ordinary_weapon_damage(game, source, PlayerWeaponRoll::Archery, attack);
}

pub(crate) fn apply_archery_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, snapshot: ArcheryAttack, target: (i32, ShapeIdentity),
    runtime: &mut Runtime,
) {
    if game.move_shape_health(target.0, target.1).is_none_or(|hp| hp == 0) { return; }
    let master = snapshot.attack_master();
    let mut attack = AttackInformation::for_master(master);
    calculate_archery_attack(game, snapshot, target, &mut attack);
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}

// Неподключённый серверный декодер снимка. Клиентский encoder не заменяет
// его runtime: после чтения префикса native начинает отсчёт заново.
// FUNCTION: CArcheryPhalanx::DecordFromByteArray
// SOURCE: appserver/skills/archeryphalanx.cpp:281
// RVA: 0x001EB070
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar *source, long *offset, bool include_ex_data)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
