//! Increment-shop initial configuration исторического Miracle.
//!
//! Статус World `AddToByteArray` RVA `0x0003BA60` и безопасной замены
//! `Release` RVA `0x0003BB30`: `IMPLEMENTED`; `LoadItems`, singleton plumbing и
//! Game decoder ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.cpp:27,43`.
//!
//! Owner хранит `multimap<unsigned char, Item*>`: wire начинается с signed
//! count, затем идёт unsigned page-key, первые `0x18` байт Item, description и
//! localized goods key как две C-строки; после всех записей следует C-строка
//! affiche. Равные page-key сохраняют insertion order, поэтому Rust использует
//! `BTreeMap<u8, Vec<Item>>`.
//!
//! EXE копировал весь 24-байтный prefix, хотя loader заполнял только category,
//! четыре `u32` и icon: два байта после category и три после icon оставались
//! heap-мусором. Точный Game decoder пропускает эти reserved-позиции. Rust
//! сохраняет размер/layout, но пишет нули, устраняя внутреннюю утечку без
//! изменения принимаемых полей. Safe owner не допускает null Item slot-ов и
//! освобождает весь registry через `Drop`, а не воспроизводит ошибочный ручной
//! lifetime.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

const ITEM_WIRE_LENGTH: usize = 0x18;

/// Значимые поля первых `0x18` байт legacy Item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IncrementShopItem {
    pub(crate) category: u16,
    pub(crate) overlapped_amount: u32,
    pub(crate) goods_id: u32,
    pub(crate) yuan_bao_price: u32,
    pub(crate) deduction_goods_id: u32,
    pub(crate) icon_id: u8,
    pub(crate) description: Vec<u8>,
    pub(crate) key: Vec<u8>,
}

/// Safe owner исходного singleton state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CIncrementShopList {
    items: BTreeMap<u8, Vec<IncrementShopItem>>,
    affiche: Vec<u8>,
}

impl CIncrementShopList {
    /// Эквивалент multimap insertion в конец диапазона равного page-key.
    pub(crate) fn insert(&mut self, page: u8, item: IncrementShopItem) {
        self.items.entry(page).or_default().push(item);
    }

    pub(crate) fn set_affiche(&mut self, affiche: &[u8]) {
        self.affiche.clear();
        self.affiche.extend_from_slice(truncate_at_nul(affiche));
    }

    /// Сохраняет исходный порядок очистки, а ownership освобождает все Item.
    pub(crate) fn release(&mut self) {
        self.affiche.clear();
        self.items.clear();
    }

    /// Дописывает exact `count + items + affiche` wire.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), IncrementShopSerializeError> {
        let item_count = self
            .items
            .values()
            .try_fold(0_usize, |count, items| count.checked_add(items.len()))
            .ok_or(IncrementShopSerializeError::ItemCountOverflow)?;
        let item_count = i32::try_from(item_count)
            .map_err(|_| IncrementShopSerializeError::ItemCountOverflow)?;
        destination.extend_from_slice(&item_count.to_le_bytes());

        for (&page, items) in &self.items {
            for item in items {
                ensure_c_string(&item.description, IncrementShopStringField::Description)?;
                ensure_c_string(&item.key, IncrementShopStringField::Key)?;
                destination.push(page);

                let prefix_start = destination.len();
                destination.extend_from_slice(&item.category.to_le_bytes());
                destination.extend_from_slice(&[0; 2]);
                destination.extend_from_slice(&item.overlapped_amount.to_le_bytes());
                destination.extend_from_slice(&item.goods_id.to_le_bytes());
                destination.extend_from_slice(&item.yuan_bao_price.to_le_bytes());
                destination.extend_from_slice(&item.deduction_goods_id.to_le_bytes());
                destination.push(item.icon_id);
                destination.extend_from_slice(&[0; 3]);
                debug_assert_eq!(destination.len() - prefix_start, ITEM_WIRE_LENGTH);

                destination.extend_from_slice(&item.description);
                destination.push(0);
                destination.extend_from_slice(&item.key);
                destination.push(0);
            }
        }

        ensure_c_string(&self.affiche, IncrementShopStringField::Affiche)?;
        destination.extend_from_slice(&self.affiche);
        destination.push(0);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IncrementShopStringField {
    Description,
    Key,
    Affiche,
}

impl fmt::Display for IncrementShopStringField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Description => "описание increment-shop товара",
            Self::Key => "поисковый ключ increment-shop товара",
            Self::Affiche => "increment-shop affiche",
        })
    }
}

/// Safe boundary для state, не представимого legacy C-string/count wire.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IncrementShopSerializeError {
    ItemCountOverflow,
    StringContainsNul { field: IncrementShopStringField },
}

impl fmt::Display for IncrementShopSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ItemCountOverflow => formatter
                .write_str("increment-shop item count не помещается в signed 32-битный диапазон"),
            Self::StringContainsNul { field } => {
                write!(formatter, "{field} содержит внутренний NUL")
            }
        }
    }
}

impl Error for IncrementShopSerializeError {}

fn ensure_c_string(
    value: &[u8],
    field: IncrementShopStringField,
) -> Result<(), IncrementShopSerializeError> {
    if value.contains(&0) {
        return Err(IncrementShopSerializeError::StringContainsNul { field });
    }
    Ok(())
}

fn truncate_at_nul(value: &[u8]) -> &[u8] {
    &value[..value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len())]
}

// Сырой C++ ниже сохранён как локальная документация loaders, singleton и
// Game decoder-а, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.cpp

// ============================================================================
// FUNCTION: CIncrementShopList::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.h:36
// RVA: 0x00002030
// ADDRESS: 00402030
// PROTOTYPE: CIncrementShopList * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CIncrementShopList::GetItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.cpp:238
// RVA: 0x00021CE0
// ADDRESS: 00421ce0
// PROTOTYPE: Item * __thiscall GetItem(ulong param_1, char param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CIncrementShopList::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.cpp:27
// RVA: 0x00021DB0
// ADDRESS: 00421db0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CIncrementShopList::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.cpp:199
// RVA: 0x00022490
// ADDRESS: 00422490
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CIncrementShopList::~CIncrementShopList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.cpp:22
// RVA: 0x00022640
// ADDRESS: 00422640
// PROTOTYPE: void __thiscall ~CIncrementShopList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CIncrementShopList::CIncrementShopList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.cpp:18
// RVA: 0x000226E0
// ADDRESS: 004226e0
// PROTOTYPE: undefined __thiscall CIncrementShopList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.cpp

// ============================================================================
// FUNCTION: CIncrementShopList::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.h:36
// RVA: 0x000011E0
// ADDRESS: 004011e0
// PROTOTYPE: CIncrementShopList * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CIncrementShopList::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.cpp:43
// RVA: 0x0003BA60
// ADDRESS: 0043ba60
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CIncrementShopList::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.cpp:27
// RVA: 0x0003BB30
// ADDRESS: 0043bb30
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CIncrementShopList::LoadItems
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.cpp:65
// RVA: 0x0003C230
// ADDRESS: 0043c230
// PROTOTYPE: int __thiscall LoadItems(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CIncrementShopList::~CIncrementShopList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.cpp:22
// RVA: 0x0003C910
// ADDRESS: 0043c910
// PROTOTYPE: void __thiscall ~CIncrementShopList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CIncrementShopList::CIncrementShopList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\incrementshoplist.cpp:18
// RVA: 0x0003C9B0
// ADDRESS: 0043c9b0
// PROTOTYPE: undefined __thiscall CIncrementShopList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
