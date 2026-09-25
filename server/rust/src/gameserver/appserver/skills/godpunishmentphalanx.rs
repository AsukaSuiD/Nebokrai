//! Одноклеточная область GodPunishment — шов к Zone.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/godpunishmentphalanx.cpp.
//! Снимок конструктора (уровень `+0xC8`, min/max/element по сдвинутому
//! профилю `+0xBC/+0xC0/+0xC4`, без CScope), общий полёт, элементный контакт
//! и клиентский снимок перенесены буквально в
//! `nebokrai_zone::skills::projectile` (основание и статусы см. там); здесь
//! остаётся только адаптер снимка атаки: live-разрешение и доставка контакта
//! принадлежат `CGame`. Замена игнорирует уровень и завершает форму при
//! совпадении живой клетки.

use super::elementprojectileattack::ElementProjectileAttack;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::CShape;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CGodPunishmentPhalanx(nebokrai_zone::skills::CGodPunishmentPhalanx);

impl CGodPunishmentPhalanx {
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32, element_modifier: i32,
    ) -> Self {
        Self(nebokrai_zone::skills::CGodPunishmentPhalanx::new(
            id, master, started_at_ms, lifetime_ms, skill_level,
            minimum_attack, maximum_attack, element_modifier,
        ))
    }

    pub(crate) const fn shape(&self) -> &CShape { self.0.shape() }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { self.0.shape_mut() }
    pub(crate) const fn master(&self) -> MasterInfo { self.0.master() }
    pub(crate) const fn attack_snapshot(&self) -> ElementProjectileAttack {
        ElementProjectileAttack::from_rule(self.0.attack_snapshot())
    }

    pub(crate) fn expired_at(&self, now: u32) -> bool { self.0.expired_at(now) }

    pub(crate) fn replacement_matches(&self, _level: i32, x: i32, y: i32) -> bool {
        self.0.replacement_matches(_level, x, y)
    }

    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        self.0.encode_client_snapshot(now)
    }
}

// Серверный `DecordFromByteArray` GodPunishment (pub `1:001ff7c0`, RVA
// `0x2007C0`, VA 006007C0) перенесён в `nebokrai_zone::skills::
// BaseProjectileFlight::decode_server_snapshot` (основание см. там),
// достижимого caller-а у оригинала нет. До появления реального входящего
// owner-а сериализованный остаток не подменяет конструктор снимка боевых
// свойств.
