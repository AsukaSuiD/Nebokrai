//! Владелец обычных extended-state `CExState/CExStateNew` GameServer.
//! Данные, записи 40/52 байта и кодек перенесены в Zone
//! `effects/extended.rs` (там же адреса конструкторов, vtable, кодека и AI).
//! Источник: gameserver.exe + GameServer.pdb, исходные owner-ы
//! `other states/exstate.cpp`, `exstatenew.cpp` и caller-ы
//! `CMoveShape::Add/Del/GetExState*`.
//! Ниже сохранены исходные перегрузки и операции, не закрытые Begin/End.
//! AddEx/AddExNew (0x004D1E40/0x004D20D0) сначала сохраняют параметры фабрики,
//! затем обходят все живые позиции того же ID с совпавшим WORD type или level:
//! direct End, свежий остаток той же позиции и его destructor, без уплотнения.
//! Только после этого Begin(this,this) (0x005D9780/0x005D9C40) проверяет sufferer,
//! читает один базовый clock, создаёт loop1 visual и делает Update(0) до append.
//! Успех завершает отдельный UpdateProperty; самостоятельного OnChangeStates нет.
//! Объектный Begin меняет только базовый timestamp, не перезапуская item clock.
//! Vtable 0x0065E33C/0x0065E39C имеют End +0x1C = 0x005FD420:
//! optional visual Update(1), свежий GetSufferer и RemoveState, без base End
//! и без записи state.ended. Visual 0x005D9830/0x005D9CF0 проверяет свой ended
//! и фактического sufferer; base visual tail выполняется и при missing sufferer.
//! ExNew.use_item0x005D9E50 использует фактического Sufferer и возвращает
//! реальное списанное количество. GS0128 отправляется через SendSystemInfo
//! 0x0042CD70: BF807(FFFFFFFF,CString), без второго цвета BF806.
//! Общий адаптер CGame заменяет небезопасный sprintf в buffer256 ограничением
//! 255 байт, NULL goods-name — пустой строкой, неверный non-player cast — нулём.
//! Это безопасные границы для native UB, а не native-контракт этих случаев.

use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;

pub(crate) use nebokrai_zone::effects::{
    EX_STATE_ID, EX_STATE_NEW_ID, ExtendedState, ExtendedStateKind,
};

/// Тонкая оболочка переходного Game: фабрика навыков остаётся у старого
/// владельца, а данные и записи — в Zone `effects/extended.rs`.
pub(crate) fn extended_state_from_factory(
    kind: ExtendedStateKind,
    level: u32,
    factory: &CSkillFactory,
) -> Option<ExtendedState> {
    let properties = factory.query_skill_base_properties(kind.state_id(), level as i32)?;
    ExtendedState::from_properties(kind, level, |usage| properties.query_property(usage))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp

// ============================================================================
// FUNCTION: CExState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:113
// RVA: 0x001D9350
// ADDRESS: 005d9350
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:131
// RVA: 0x001D9410
// ADDRESS: 005d9410
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
