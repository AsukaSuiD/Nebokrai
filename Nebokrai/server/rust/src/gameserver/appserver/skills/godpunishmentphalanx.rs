//! Одноклеточная область GodPunishment.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/godpunishmentphalanx.cpp.
//! Конструктор сохраняет MIN/MAX/ELEMENT; живые свойства игрока участвуют
//! в общем элементном контакте без усиления душами, RP и переноса яда.
//! AI проверяет абсолютный unsigned срок и обходит один снимок клетки.
//! Каждый допущенный target вызывает Attack, затем полный Summon End,
//! включая отказ Attack по смерти; End не останавливает снимок.
//! Замена игнорирует уровень и завершает форму при совпадении живой клетки.
//! Wire содержит master type/id, а не координаты. Достижимых серверных
//! caller-ов decoder не установлено; это не контракт сохранения формы в БД.

use super::elementprojectileattack::ElementProjectileAttack;
use super::godpunishment::GOD_PUNISHMENT_SKILL_ID;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use crate::public::guid::CGuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CGodPunishmentPhalanx {
    shape: CShape,
    attack: ElementProjectileAttack,
    started_at_ms: u32,
    lifetime_ms: u32,
}

impl CGodPunishmentPhalanx {
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32, element_modifier: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape,
            attack: ElementProjectileAttack::new(
                master, GOD_PUNISHMENT_SKILL_ID, skill_level,
                minimum_attack, maximum_attack, element_modifier, None,
            ),
            started_at_ms, lifetime_ms,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master() }
    pub(crate) const fn attack_snapshot(&self) -> ElementProjectileAttack { self.attack }

    pub(crate) fn expired_at(&self, now: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now
    }

    pub(crate) fn replacement_matches(&self, _level: i32, x: i32, y: i32) -> bool {
        self.shape.get_tile_x().unwrap_or(i32::MIN) == x
            && self.shape.get_tile_y().unwrap_or(i32::MIN) == y
    }

    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, GOD_PUNISHMENT_SKILL_ID as i32, self.attack.skill_level(),
            self.master().master_type, self.master().master_id,
            self.started_at_ms, self.lifetime_ms, now,
        )
    }
}

// Не подключён CGodPunishmentPhalanx::DecordFromByteArray, VA 006007C0:
// пять DWORD восстанавливают skill/level/master/remaining, затем clock и
// CShape::DecordFromByteArray. До появления реального входящего owner-а
// сериализованный остаток не подменяет конструктор снимка боевых свойств.
