//! Отображаемое оставшееся время наложенного состояния.
//!
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/healstate.cpp` и `healstate.h`,
//! `CHealState::GetRemainedTime` (VA `0x005F2CD0`). При истёкшем сроке
//! второе чтение `timeGetTime` не выполняется.

/// Срок и остаток считаются в арифметике DWORD; для положительного остатка
/// исходный getter читает часы повторно после проверки срока.
pub fn timed_client_state_time(
    started_at_ms: u32,
    keep_time_ms: u32,
    mut now_milliseconds: impl FnMut() -> u32,
) -> u32 {
    let deadline = started_at_ms.wrapping_add(keep_time_ms);
    if deadline <= now_milliseconds() {
        0
    } else {
        deadline.wrapping_sub(now_milliseconds())
    }
}

/// Расширенный вариант `CNotDisappearAfterDead::GetRemainedTime` VA
/// `0x005D6320` и `CExtendedState`: нулевой срок возвращает 0 без чтения
/// часов; иначе одно чтение проверяет границу, второе — вычитание остатка.
pub fn guarded_client_state_time(
    started_at_ms: u32,
    keep_time_ms: u32,
    now_milliseconds: impl FnMut() -> u32,
) -> u32 {
    if keep_time_ms == 0 {
        return 0;
    }
    timed_client_state_time(started_at_ms, keep_time_ms, now_milliseconds)
}

/// Вариант `CHBYState::GetRemainedTime`, общий с `CExState` Original:
/// после истечения ненулевого срока возвращает 1, после бессрочного — 0,
/// живой срок читает часы повторно для вычитания.
pub fn change_body_client_state_time(
    started_at_ms: u32,
    keep_time_ms: u32,
    mut now_milliseconds: impl FnMut() -> u32,
) -> u32 {
    let deadline = started_at_ms.wrapping_add(keep_time_ms);
    if keep_time_ms != 0 && deadline <= now_milliseconds() {
        return 1;
    }
    if deadline <= now_milliseconds() {
        return 0;
    }
    deadline.wrapping_sub(now_milliseconds())
}
