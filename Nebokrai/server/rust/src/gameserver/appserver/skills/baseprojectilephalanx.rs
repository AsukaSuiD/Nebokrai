//! Общие данные полёта региональных снарядов Archery и BaseMagic.
//! Источник: gameserver.exe/GameServer.pdb, archeryphalanx.cpp и
//! basemagicphalanx.cpp. Цель хранится как type/id с GUID_INVALID;
//! срок жизни и задержка сравниваются как абсолютные unsigned суммы.
//! Их общий End только отмечает удаление. Клиентский encoder пишет
//! master type/id, а не сохранённую цель. Неиспользуемая CScope заменена
//! отсутствием выделения: она не участвует в поиске цели или формулах.

use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use crate::public::guid::CGuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BaseProjectileFlight {
    shape: CShape,
    started_at_ms: u32,
    lifetime_ms: u32,
    attack_delay_ms: u32,
    target: ShapeIdentity,
}

impl BaseProjectileFlight {
    pub(super) fn new(
        id: i32, started_at_ms: u32, lifetime_ms: u32,
        attack_delay_ms: u32, target: ShapeIdentity,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape, started_at_ms, lifetime_ms, attack_delay_ms,
            target: ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..target },
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn target(&self) -> ShapeIdentity { self.target }

    pub(crate) fn expired_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms
    }

    pub(crate) fn attack_due_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.attack_delay_ms) < now_ms
    }

    pub(crate) fn end(&mut self) {
        self.shape.set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub(super) fn encode_client_snapshot(
        &self, skill_id: u32, skill_level: i32, master: MasterInfo,
        now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, skill_id as i32, skill_level,
            master.master_type, master.master_id,
            self.started_at_ms, self.lifetime_ms, now_milliseconds,
        )
    }
}
