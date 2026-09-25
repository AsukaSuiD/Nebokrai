//! Одноклеточная область GodPunishment.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/godpunishmentphalanx.cpp.
//! Снимок конструктора (уровень `+0xC8`, min/max/element по сдвинутому
//! профилю `+0xBC/+0xC0/+0xC4`, без CScope), часы и клиентский префикс
//! разделяют общий полёт и элементный контакт —
//! `nebokrai_zone::skills::projectile` (основание и статусы см. там);
//! обёртка компонует их, сохраняя прежние имена и интерфейс для владельцев
//! game/. Конструктор сохраняет MIN/MAX/ELEMENT; живые свойства игрока
//! участвуют в общем элементном контакте без усиления душами, RP и
//! переноса яда. Литерал навыка 0x13A кладётся в `+0xB8`. Замена игнорирует
//! уровень и завершает форму при совпадении живой клетки. Wire содержит
//! master type/id, а не координаты.

use super::baseprojectilephalanx::BaseProjectileFlight;
use super::elementprojectileattack::ElementProjectileAttack;
use super::godpunishment::GOD_PUNISHMENT_SKILL_ID;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::CShape;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CGodPunishmentPhalanx {
    flight: BaseProjectileFlight,
    attack: ElementProjectileAttack,
}

impl CGodPunishmentPhalanx {
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32, element_modifier: i32,
    ) -> Self {
        Self {
            flight: BaseProjectileFlight::new_untargeted(id, started_at_ms, lifetime_ms),
            attack: ElementProjectileAttack::new(
                master, GOD_PUNISHMENT_SKILL_ID, skill_level,
                minimum_attack, maximum_attack, element_modifier, None,
            ),
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { self.flight.shape() }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { self.flight.shape_mut() }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master() }
    pub(crate) const fn attack_snapshot(&self) -> ElementProjectileAttack { self.attack }

    pub(crate) fn expired_at(&self, now: u32) -> bool { self.flight.expired_at(now) }

    pub(crate) fn replacement_matches(&self, _level: i32, x: i32, y: i32) -> bool {
        self.flight.shape().get_tile_x().unwrap_or(i32::MIN) == x
            && self.flight.shape().get_tile_y().unwrap_or(i32::MIN) == y
    }

    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        self.flight.encode_client_snapshot(
            GOD_PUNISHMENT_SKILL_ID, self.attack.skill_level(), self.attack.master(), now,
        )
    }
}

// Серверный `DecordFromByteArray` GodPunishment (pub `1:001ff7c0`, RVA
// `0x2007C0`, VA 006007C0) повторяет общую форму прицельных фаланг с
// уровнем в слоте профиля `+0xC8`; общий decoder перенесён в
// `nebokrai_zone::skills::BaseProjectileFlight::decode_server_snapshot`
// (основание см. там), достижимого caller-а у оригинала нет. До появления
// реального входящего owner-а сериализованный остаток не подменяет
// конструктор снимка боевых свойств.
