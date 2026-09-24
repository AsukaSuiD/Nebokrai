//! Живые Begin/restart состояний восстановления от расходуемых предметов
//! в переходном Game. Данные, таймер, enum и 16-байтная запись перенесены
//! в Zone `effects/consumablerestore.rs` (там же адреса конструкторов,
//! общего кодека и AI).
//! Источник: gameserver.exe + GameServer.pdb, исходные владельцы
//! appserver/player.cpp и appserver/other states/restorehpstate.cpp,
//! restorempstate.cpp.
//!
//! CPlayer::RestoreHp0x00444C80/RestoreMp0x00444D50: clock1 проверяет
//! unsigned last+interval > now; после успеха clock2 меняет только свой last
//! до ctor. Object Begin HP0x004F8720/MP0x004F8A10 вызывает base0x005DBD70:
//! self/self читает независимый clock3, сохраняет стороны, затем создаёт
//! visual SetRun(1) и обнуляет count. Только после успеха caller добавляет
//! тот же экземпляр; нет instant-ветви для keep=0, замены предыдущего
//! Restore, Update или пакета. Подготовленный Begin ниже не регистрирует
//! payload; общий append сохраняет точные стороны и остаточный visual без
//! повторных часов/Begin. Отказ native allocator не эмулируется.
//!
//! NULL-user restart сохраняет timestamp/user, назначает holder sufferer,
//! создаёт visual SetRun(1), затем сбрасывает count без часов и пакетов.
//! SetRegion обеих vtable0x0065355C/0x006535BC +0x2C = 0x005E3B30
//! меняет только регион sufferer. AI работает с actual sufferer, а payload
//! остаётся у holder: HP принимает CMoveShape, MP требует CPlayer до death-gate.
//! Missing sufferer и MP non-player вызывают End до часов. End0x005EEBA0
//! только вызывает RemoveState(pointer) у actual sufferer, не обновляя
//! visual/state.ended.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, resolve_state_move_shape,
    resolve_state_move_shape_mut,
};
use crate::gameserver::gameserver::game::CGame;
use nebokrai_shared::values::CGuid;

pub(crate) use nebokrai_zone::effects::{
    CONSUMABLE_RESTORE_STATE_BYTES, ConsumableRestoreIntervals, ConsumableRestoreState,
    RestoreStateData,
};

pub(crate) fn begin_primary_consumable_restore_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    state: &mut ConsumableRestoreState,
    now: &mut dyn FnMut() -> u32,
) -> Option<(i32, ShapeIdentity)> {
    resolve_state_move_shape(game, region_id, holder)?;
    state.begin_primary_at(now());
    let shape = resolve_state_move_shape(game, region_id, holder)?.shape();
    Some((
        shape.get_region_id(),
        ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() },
    ))
}

pub(crate) fn restart_consumable_restore_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ConsumableRestoreState>(key)).is_none()
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) { return false }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    if let Some(state) = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<ConsumableRestoreState>(key))
    {
        state.reset_restore_count();
    }
    true
}
