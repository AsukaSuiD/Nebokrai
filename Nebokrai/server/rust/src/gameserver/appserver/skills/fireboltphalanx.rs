//! Прицельный снаряд огненной стрелы FireBolt.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/fireboltphalanx.cpp.
//! Полёт совпадает с Archery/BaseMagic: два отдельных абсолютных unsigned
//! срока, свежий GetObject фактического региона, затем контакт и тихий End.
//! Удаление не предшествует Attack и не отправляет немедленный BF504.
//! Снимок душ усиливает элементный урон в общем elementprojectileattack;
//! Calculate не ищет живое SoulCollect, не запрашивает таблицу и не даёт RP.
//! Клиентский префикс содержит master type/id, а не цель; души в него
//! не входят. Неподключённый общий decoder сохранён у fireballphalanx.

use super::firebolt::FIRE_BOLT_SKILL_ID;
use super::baseprojectilephalanx::BaseProjectileFlight;
use super::elementprojectileattack::{ElementProjectileAttack, SoulProjectileAmplification};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CFireBoltPhalanx {
    flight: BaseProjectileFlight,
    attack: ElementProjectileAttack,
}

impl CFireBoltPhalanx {
    #[allow(clippy::too_many_arguments, reason = "снимок конструктора FireBoltPhalanx")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32,
        element_modifier: i32, target: ShapeIdentity, attack_delay_ms: u32,
        soul_count: i32, soul_variable: i32,
    ) -> Self {
        Self {
            flight: BaseProjectileFlight::new(id, started_at_ms, lifetime_ms, attack_delay_ms, target),
            attack: ElementProjectileAttack::new(
                master, FIRE_BOLT_SKILL_ID, skill_level, minimum_attack, maximum_attack,
                element_modifier, Some(SoulProjectileAmplification::new(soul_count, soul_variable)),
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
            FIRE_BOLT_SKILL_ID, self.attack.skill_level(), self.attack.master(), now_milliseconds,
        )
    }
}
