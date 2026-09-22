//! Сохранённое состояние `CWangshengState` (`0x221`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/skills/wangshengstate.cpp`. Общий codec `CFuryState` сохраняет
//! ID, остаток времени и знаковую прибавку в 12 байтах. Состояние независимо
//! от active-skill owner-а `wangsheng.rs`: exact `CWangsheng::AI` лечит
//! напрямую и его не создаёт. Загруженная legacy-запись использует timed AI
//! `CPobingState`; её `OnUpdateProperties` ставит HP в максимум только когда
//! `current + gain >= max`, а при меньшем результате не меняет HP.
//! Достигнутый AI получает один поколенческий ключ общей арены;
//! порядок вызовов и границу прохода задаёт общий CMoveShape::UpdateAbnormality.
//! Любое удаление адресует тот же экземпляр, а не первый дубль.
//! AI/End разрешают общий CMoveShape по region/type/id; правила свойств
//! игрока не запрещают жизненный цикл региональных держателей.
//! Exact vtable 0x0066208C: timed AI 0x005E6E20 вызывает End(false)
//! из slot +0x48 (0x005E7310): visual → базовый End → GetUser → RemoveState.
//! Это не прямой slot +0x1C (0x005DBCE0), который сам visual не отправляет.
//! При фактическом удалении base End вызывает общий virtual UpdateProperty держателя.
//! Прямой End не проверяет срок и не вызывает timer End(false).
//! StartAllStates 0x004CE050 вызывает Begin(0, self); Begin 0x00606040
//! сразу возвращает при NULL User, не создавая visual. Для загруженной записи
//! User остаётся NULL: оба End лишь отмечают ended, не удаляя payload.

//! Begin(NULL, holder) (0x00606040) возвращает 0 на проверке User до базы:
//! restart не меняет время, ended или прежний visual-ресурс.

//! Unserialize 0x005FD660 сохраняет один собственный clock в timestamp;
//! decode получает его в now_ms для этой wire-записи, а restart не заменяет его.

//! OnUpdateProperties 0x00605FB0 сначала требует GetUser, затем обновляет
//! существующий visual (он самостоятельно читает GetSufferer), после type400
//! и RTTI CPlayer условно вызывает SetHealth(maximum). OnChangeStates и
//! отдельного пакета HP здесь нет. Временная сумма +0x3C сериализатором
//! не читается и остаётся локальным скаляром; loaded NULL user даёт return0.
//! Timer End0x005E7310 вызывает Update(1) у существующего visual до base End,
//! не проверяя User. Visual0x006060E0 требует !visual.ended и GetSufferer для
//! BFE04; base visual tail выполняется при любом имеющемся ресурсе. Затем
//! base End ищет фактического User. Прямой +0x1C остаётся без visual-фазы;
//! происхождение DB-записи не заменяет эти независимые lookup/gates.

use crate::gameserver::appserver::states::state::{
    resolve_applied_state_user, update_property_state_visual, StatePropertyTarget,
    update_applied_state_end_visual,
};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{end_base_applied_state, resolve_state_move_shape};

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::CGame;

pub(crate) const WANGSHENG_STATE_ID: u32 = 0x221;
pub(crate) const WANGSHENG_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WangshengState {
    started_at_ms: u32,
    keep_time_ms: u32,
    gain: i32,
}

impl WangshengState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32, gain: i32) -> Self {
        Self { started_at_ms, keep_time_ms, gain }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != WANGSHENG_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(now_ms, reader.read_u32()?, reader.read_i32()?))
    }

    pub(crate) const fn state_id(self) -> u32 { WANGSHENG_STATE_ID }



    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds)
    }

    pub(crate) const fn capped_health(self, current: u32, maximum: u32) -> Option<u32> {
        let actual = current.wrapping_add(self.gain as u32);
        if maximum <= actual { Some(maximum) } else { None }
    }

    pub(crate) fn encoded(
        self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> [u8; WANGSHENG_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(WANGSHENG_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(self.state_id());
        writer.write_u32(self.client_time(now_milliseconds));
        writer.write_i32(self.gain);
        bytes.try_into().expect("размер состояния восстановления фиксирован")
    }
}



pub(crate) fn update_wangsheng_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((_, target)) = resolve_applied_state_user(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<WangshengState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now),
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WangshengState>(key)).copied()
    else { return false; };
    if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            if let Some(health) = state.capped_health(player.health(), player.maximum_health()) {
                player.set_health(health);
            }
        }
    }
    true
}

pub(crate) fn restart_wangsheng_state(
    _game: &mut CGame,
    _region_id: i32,
    _holder: ShapeIdentity,
    _key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    false
}

pub(crate) fn update_wangsheng_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WangshengState>(key))
        .is_some_and(|state| state.expired(now_ms))
    {
        return false;
    }
    update_applied_state_end_visual(
        game, region_id, holder, key, StatePropertyTarget::Sufferer,
    );
    end_wangsheng_state(game, region_id, holder, key)
}

pub(crate) fn end_wangsheng_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WangshengState>(key)).is_none() {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, WANGSHENG_STATE_BYTES)
}
