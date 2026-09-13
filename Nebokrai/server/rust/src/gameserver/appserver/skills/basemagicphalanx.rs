//! Прицельный региональный снаряд базовой магии BaseMagic.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/basemagicphalanx.cpp.
//! Полёт, два абсолютных unsigned срока, поиск цели и тихий End общие с
//! Archery и FireBolt. Элементный контакт выполняется без допуска, яда и RP;
//! его сохранённые MIN/MAX/ELEMENT не получают усиления собранными душами.
//! Calculate и критический хвост принадлежат elementprojectileattack.
//! Клиентский снимок содержит master type/id, не цель. Неподключённый
//! общий серверный decoder сохранён у archeryphalanx.

use super::basemagic::BASE_MAGIC_SKILL_ID;
use super::baseprojectilephalanx::BaseProjectileFlight;
use super::elementprojectileattack::ElementProjectileAttack;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CBaseMagicPhalanx {
    flight: BaseProjectileFlight,
    attack: ElementProjectileAttack,
}

impl CBaseMagicPhalanx {
    #[allow(clippy::too_many_arguments, reason = "снимок конструктора BaseMagicPhalanx")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32,
        element_modifier: i32, attack_delay_ms: u32, target: ShapeIdentity,
    ) -> Self {
        Self {
            flight: BaseProjectileFlight::new(id, started_at_ms, lifetime_ms, attack_delay_ms, target),
            attack: ElementProjectileAttack::new(
                master, BASE_MAGIC_SKILL_ID, skill_level, minimum_attack,
                maximum_attack, element_modifier, None,
            ),
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { self.flight.shape() }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { self.flight.shape_mut() }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master() }
    pub(crate) const fn flight(&self) -> &BaseProjectileFlight { &self.flight }
    pub(crate) const fn flight_mut(&mut self) -> &mut BaseProjectileFlight { &mut self.flight }
    pub(crate) const fn attack_snapshot(&self) -> ElementProjectileAttack { self.attack }

    pub(crate) fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        self.flight.encode_client_snapshot(
            BASE_MAGIC_SKILL_ID, self.attack.skill_level(), self.attack.master(), now_milliseconds,
        )
    }
}
