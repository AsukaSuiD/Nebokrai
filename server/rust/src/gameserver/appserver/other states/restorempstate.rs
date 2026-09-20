//! Состояние восстановления маны `CRestoreMpState`.
//!
//! Источник: gameserver.exe + GameServer.pdb, исходный владелец
//! appserver/other states/restorempstate.cpp. ID=100001,
//! vtable0x006535BC, AI0x004F8AA0. Общий RestoreStateData заменяет лишь
//! одинаковые layout, счётчик, таймер и codec; HP/MP остаются разными типами
//! и отдельными экземплярами общей арены. Игровые различия выполняет CGame.
//! AI сначала делает RTTI CPlayer для actual sufferer. NULL/non-player
//! вызывают End без часов даже при нулевом HP. Только игрок проходит
//! death-gate, периодическую прибавку MP и два последовательных clock.
//! MP складывается с gain через DWORD wrapping и ограничивается максимумом;
//! запись и OnChangeStates происходят после увеличения счётчика.
//! Живая цель получает один шаг при strict frequency*count+started < now;
//! count увеличивается до чтения/записи ресурса и публикации. Второй clock
//! проверяет strict keep+started < now; смерть приостанавливает оба действия.
//! NULL sufferer ведёт в End0x005EEBA0: только RemoveState(pointer) у
//! фактической цели, без visual/state.ended и удаления ключа из чужой арены.
//!
//! Ctor0x004F87B0 копирует keep/frequency/gain, count и timestamp оставляет
//! нулевыми без часов; default0x004F8840 также обнуляет параметры.
//! Object Begin0x004F8A10: base self/self читает отдельный clock, затем
//! visual SetRun(1) и count=0; NULL-user restart сохраняет timestamp/user.
//! Begin не отправляет пакет и не вызывает Update; caller добавляет только
//! после него. Destructor вызывает базовый destructor: ресурсы арены безопасно
//! освобождаются Rust Drop без дополнительных End или пакетов.
//! SetRegion vtable+0x2C=0x005E3B30 меняет только sufferer-region.
//! Client-time+0x30=0x005F2CD0, additional+0x38=0x00601200 (ноль).
//! Serialize0x005F65F0 пишет четыре DWORD ID/remaining/frequency/gain
//! и не изменяет live keep. Unserialize0x005EEC70 читает свой clock перед
//! тремя полями. Visual Update0x004F86F0 выполняет только базовый хвост.
//! Подробный общий порядок зафиксирован в restorestate.rs. Координатный и
//! typed-target Begin0x004F88C0/0x004F8950 ещё не перенесены и сохранены ниже.

pub(crate) const RESTORE_MP_STATE_ID: i32 = 100_001;
pub(crate) const RESTORE_MP_STATE_BYTES: usize =
    super::restorestate::CONSUMABLE_RESTORE_STATE_BYTES;

pub(crate) type RestoreMpState = super::restorestate::RestoreStateData<RESTORE_MP_STATE_ID>;

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\restorempstate.cpp

// ============================================================================
// FUNCTION: CRestoreMpState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\restorempstate.cpp:60
// RVA: 0x000F88C0
// ADDRESS: 004f88c0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRestoreMpState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\restorempstate.cpp:69
// RVA: 0x000F8950
// ADDRESS: 004f8950
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================

// COMPONENT_VARIANT_END: GameServer
