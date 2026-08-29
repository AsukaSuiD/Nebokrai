//! Каноническое состояние усиления `CPromotionState` (`0x142`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/promotionstate.cpp`. Состояние остаётся в упорядоченной
//! ветви `CFightDefense::PreDefense`: для стихийной части удара множитель
//! применяется в точном месте исходного `m_vStates`. Второй коэффициент
//! принадлежит лечению. Начальный пакет состояния имеет исходный формат
//! `0xBFE03`; отдельного пакета завершения этот владелец не создаёт.

use super::promotion::PROMOTION_SKILL_ID;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const PROMOTION_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PromotionState {
    started_at_ms: u32,
    keep_time_ms: u32,
    magic_attack_factor: u16,
    heal_recover_factor: u16,
}

impl PromotionState {
    pub(crate) const fn new(
        started_at_ms: u32,
        keep_time_ms: u32,
        magic_attack_factor: u16,
        heal_recover_factor: u16,
    ) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
            magic_attack_factor,
            heal_recover_factor,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        PROMOTION_SKILL_ID
    }

    pub(crate) const fn magic_attack_factor(self) -> u16 {
        self.magic_attack_factor
    }

    pub(crate) const fn heal_recover_factor(self) -> u16 {
        self.heal_recover_factor
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let deadline = self.started_at_ms.wrapping_add(self.keep_time_ms);
        if deadline <= now_ms {
            0
        } else {
            deadline.wrapping_sub(now_ms) as i32
        }
    }
}

pub(crate) fn send_promotion_state_begin(
    game: &mut CGame,
    region_id: i32,
    target: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: PromotionState,
    now_ms: u32,
) {
    let mut message = CMessage::new(PROMOTION_STATE_BEGIN_MESSAGE);
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(state.skill_id() as i32);
    message.add_long(state.client_time(now_ms));
    message.add_long(0);
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn expire_monster_promotion_state(
    region: &mut CServerRegion,
    monster_id: i32,
    now_ms: u32,
) -> bool {
    let ended = region
        .find_monster_by_id_mut(monster_id)
        .and_then(|monster| {
            monster
                .move_shape_mut()
                .take_expired_promotion_state(now_ms)
        })
        .is_some();
    if ended {
        tracing::trace!(
            region_id = region.id,
            monster_id,
            "состояние усиления монстра завершено"
        );
    }
    ended
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально.
// Не подключённые загрузка и сохранение общего списка `CState` остаются у
// будущего владельца фабрики состояний; текущая цепочка выполнения их не вызывает.

// ============================================================================
// FUNCTION: CPromotionState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\promotionstate.cpp:125
// RVA: 0x001F2E90
// ADDRESS: 005f2e90
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.

// ============================================================================
// FUNCTION: CPromotionState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\promotionstate.cpp:137
// RVA: 0x001F2FB0
// ADDRESS: 005f2fb0
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1,long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
