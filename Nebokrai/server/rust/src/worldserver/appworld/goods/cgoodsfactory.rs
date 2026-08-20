//! Фабрика товаров исторического `WorldServer`.
//!
//! Статус `QueryGoodsBaseProperties` RVA `0x00055DB0`,
//! `UnserializeGoods` RVA `0x00055E20` и `QueryGoodsIDByOriginalName` RVA
//! `0x000566F0` — `IMPLEMENTED`; остальной корпус ниже остаётся
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:38,88,696`.
//!
//! Старые static `std::map<unsigned long, CGoodsBaseProperties*>` и
//! `std::map<std::string, unsigned long>` заменены caller-owned `BTreeMap`:
//! unsigned key-order и возможность null mapped-value первой карты сохранены,
//! а глобальный mutable pointer-lifetime не вводится до восстановления
//! `Load/Release` фабрики. Original-name остаётся последовательностью legacy-
//! байтов, а не обязанной быть UTF-8 строкой; `CStr` сохраняет доказанную
//! границу нуль-терминированного `char const*`. `nullptr` результата base-
//! lookup выражен `Option`; успешный factory-result остаётся heap-owned
//! `Box<CGoods>`.
//!
//! `UnserializeGoods` создаёт default `CGoods`, вызывает его decoder, затем
//! оставляет объект только при non-null lookup его base-properties index.
//! Exact инструкции `0x00455E9A..0x00455EA6` подтверждают третий аргумент
//! virtual slot `+0xAC`: `push 1`, cursor, source, то есть `include_child=true`.
//! После ответа reverse прекращён. Вызванный затем `CShape::GetDir` не меняет
//! состояние и не влияет на решение; Rust не сохраняет этот пустой getter-call.
//! Нулевой source pointer исходно давал `nullptr`, а Rust API принимает
//! только non-null slice. Ошибки безопасного `CGoods` decoder-а остаются
//! typed-ошибками вместо старого безразмерного чтения.
//!
//! Для original-name lookup exact `0x004566F0..0x00456803` подтверждает:
//! null-вход возвращает `0`, отсутствующий key возвращает `0`, найденный узел
//! возвращает mapped `u32` по `+0x28`. Два временных `std::string`, tree node
//! и security-cookie являются библиотечной/компиляторной формой; Rust
//! выполняет тот же точный поиск непосредственно в `BTreeMap`.

use std::collections::BTreeMap;
use std::ffi::CStr;

use super::cgoods::{CGoods, GoodsCodecError};
use super::cgoodsbaseproperties::CGoodsBaseProperties;

/// Достигнутая lookup-форма static base-properties map.
pub(crate) type GoodsBasePropertiesRegistry = BTreeMap<u32, Option<CGoodsBaseProperties>>;

/// Достигнутый индекс exact legacy original-name в unsigned goods id.
pub(crate) type GoodsOriginalNameIndex = BTreeMap<Vec<u8>, u32>;

/// Возвращает non-null base-properties для точного unsigned index.
pub(crate) fn query_goods_base_properties(
    registry: &GoodsBasePropertiesRegistry,
    index: u32,
) -> Option<&CGoodsBaseProperties> {
    registry.get(&index).and_then(Option::as_ref)
}

/// Возвращает goods id по точному legacy original-name или исходный `0`.
pub(crate) fn query_goods_id_by_original_name(
    index: &GoodsOriginalNameIndex,
    original_name: Option<&CStr>,
) -> u32 {
    original_name
        .and_then(|name| index.get(name.to_bytes()).copied())
        .unwrap_or(0)
}

/// Декодирует heap-owned товар и отбрасывает неизвестный base-properties index.
pub(crate) fn unserialize_goods(
    source: &[u8],
    cursor: &mut usize,
    registry: &GoodsBasePropertiesRegistry,
) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
    let mut goods = Box::new(CGoods::with_constructor_base_and_type());
    let _ = goods.unserialize(source, cursor, true)?;
    let index = goods
        .get_base_properties_index()
        .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
    if query_goods_base_properties(registry, index).is_none() {
        return Ok(None);
    }
    Ok(Some(goods))
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp

// ============================================================================
// FUNCTION: CGoodsFactory::GarbageCollect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:681
// RVA: 0x00055C20
// ADDRESS: 00455c20
// PROTOTYPE: int __cdecl GarbageCollect(CGoods * * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::QueryGoodsBaseProperties
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:38
// RVA: 0x00055DB0
// ADDRESS: 00455db0
// PROTOTYPE: CGoodsBaseProperties * __cdecl QueryGoodsBaseProperties(ulong param_1)
//
// IMPLEMENTED выше; отсутствие key и null mapped-value дают один `None`.

// ============================================================================
// FUNCTION: CGoodsFactory::QueryGoodsName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:76
// RVA: 0x00055DE0
// ADDRESS: 00455de0
// PROTOTYPE: char * __cdecl QueryGoodsName(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::UnserializeGoods
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:696
// RVA: 0x00055E20
// ADDRESS: 00455e20
// PROTOTYPE: CGoods * __cdecl UnserializeGoods(uchar * param_1, long * param_2)
//
// IMPLEMENTED выше; exact `push 1` закрывает include-child call-site.

// ============================================================================
// FUNCTION: CGoodsFactory::Upgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:859
// RVA: 0x00055F20
// ADDRESS: 00455f20
// PROTOTYPE: int __cdecl Upgrade(CGoods * param_1, GOODS_ADDON_PROPERTIES param_2, GOODS_ADDON_PROPERTIES param_3, int param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:122
// RVA: 0x00056130
// ADDRESS: 00456130
// PROTOTYPE: int __cdecl Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::UpgradeEquipment
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:731
// RVA: 0x000561C0
// ADDRESS: 004561c0
// PROTOTYPE: int __cdecl UpgradeEquipment(CGoods * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::QueryGoodsIDByOriginalName
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:88
// RVA: 0x000566F0
// ADDRESS: 004566f0
// PROTOTYPE: ulong __cdecl QueryGoodsIDByOriginalName(char * param_1)
//
// IMPLEMENTED выше; exact ASM закрывает ошибочно потерянный Ghidra return.

// ============================================================================
// FUNCTION: CGoodsFactory::GetGoldCoinIndex
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:362
// RVA: 0x00056810
// ADDRESS: 00456810
// PROTOTYPE: ulong __cdecl GetGoldCoinIndex(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::GetYuanBaoIndex
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:370
// RVA: 0x000568B0
// ADDRESS: 004568b0
// PROTOTYPE: ulong __cdecl GetYuanBaoIndex(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::GetJiFenIndex
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:379
// RVA: 0x00056950
// ADDRESS: 00456950
// PROTOTYPE: ulong __cdecl GetJiFenIndex(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::QueryGoodsBasePropertiesByOriginalName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:50
// RVA: 0x00057390
// ADDRESS: 00457390
// PROTOTYPE: CGoodsBaseProperties * __cdecl QueryGoodsBasePropertiesByOriginalName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:312
// RVA: 0x00058380
// ADDRESS: 00458380
// PROTOTYPE: void __cdecl Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::CreateGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:165
// RVA: 0x00059460
// ADDRESS: 00459460
// PROTOTYPE: CGoods * __cdecl CreateGoods(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::CreateGoodsNoProbability
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:243
// RVA: 0x000597C0
// ADDRESS: 004597c0
// PROTOTYPE: CGoods * __cdecl CreateGoodsNoProbability(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::Load
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:396
// RVA: 0x00059EE0
// ADDRESS: 00459ee0
// PROTOTYPE: int __cdecl Load(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004dc3a5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp
// RVA: 0x000DC3A5
// ADDRESS: 004dc3a5
// PROTOTYPE: undefined Catch@004dc3a5()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004dc486
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp
// RVA: 0x000DC486
// ADDRESS: 004dc486
// PROTOTYPE: undefined Catch@004dc486()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004dc56c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp
// RVA: 0x000DC56C
// ADDRESS: 004dc56c
// PROTOTYPE: undefined Catch@004dc56c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00535870
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp
// RVA: 0x00135870
// ADDRESS: 00535870
// PROTOTYPE: undefined Unwind@00535870()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0053587b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp
// RVA: 0x0013587B
// ADDRESS: 0053587b
// PROTOTYPE: undefined Unwind@0053587b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
