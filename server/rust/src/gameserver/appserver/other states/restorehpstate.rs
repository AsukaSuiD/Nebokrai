//! Состояние восстановления здоровья `CRestoreHpState`.
//!
//! Источник: gameserver.exe + GameServer.pdb, исходный владелец
//! appserver/other states/restorehpstate.cpp. ID=100000,
//! vtable0x0065355C, AI0x004F8650. Общий RestoreStateData заменяет лишь
//! одинаковые layout, счётчик, таймер и codec; HP/MP остаются разными типами
//! и отдельными экземплярами общей арены. Игровые различия выполняет CGame.
//! AI не содержит CPlayer RTTI: GetSufferer разрешает живой CMoveShape;
//! virtual health/max/set (+0xD0/+0xD8/+0xD4), затем OnChangeStates +0x164.
//! NPC vtable0x0065DA1C возвращает HP=0 и останавливается на death-gate.
//! Monster0x00652AE4, Build0x0065E704 и CityGate0x0065E8C4 публикуют
//! базовый OnChangeStates0x004CD3E0 (BFE02), без player team-route.
//! Живая цель получает один шаг при strict frequency*count+started < now;
//! count увеличивается до чтения/записи ресурса и публикации. Второй clock
//! проверяет strict keep+started < now; смерть приостанавливает оба действия.
//! NULL sufferer ведёт в End0x005EEBA0: только RemoveState(pointer) у
//! фактической цели, без visual/state.ended и удаления ключа из чужой арены.
//!
//! Ctor0x004F8410 копирует keep/frequency/gain, count и timestamp оставляет
//! нулевыми без часов; default0x004F84A0 также обнуляет параметры.
//! Object Begin0x004F8720: base self/self читает отдельный clock, затем
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
//! typed-target Begin0x004F8520/0x004F85B0 ещё не перенесены и сохранены ниже.

pub(crate) const RESTORE_HP_STATE_ID: i32 = 100_000;
pub(crate) const RESTORE_HP_STATE_BYTES: usize =
    super::restorestate::CONSUMABLE_RESTORE_STATE_BYTES;

pub(crate) type RestoreHpState = super::restorestate::RestoreStateData<RESTORE_HP_STATE_ID>;

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\restorehpstate.cpp

// ============================================================================
// FUNCTION: CRestoreHpState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\restorehpstate.cpp:60
// RVA: 0x000F8520
// ADDRESS: 004f8520
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRestoreHpState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\restorehpstate.cpp:69
// RVA: 0x000F85B0
// ADDRESS: 004f85b0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================

// COMPONENT_VARIANT_END: GameServer
