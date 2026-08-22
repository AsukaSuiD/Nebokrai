//! Базовые свойства игрока и progression setup исторического Miracle.
//!
//! Статус World `CPlayerList::AddToByteArray` RVA `0x0002C4D0`:
//! `IMPLEMENTED`; loaders, lookup-ы и Game decoder ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp:247`.
//!
//! Wire строго состоит из пяти секций: ordered player map с raw records по
//! `0x58` байт, vector level-exp по четыре байта и ordered upgrade map для
//! Fighter, Hunter, Taoist именно в таком порядке. Ключ player map не пишется;
//! ключ каждого upgrade map пишется как level перед шестью `u32`, одним `u16`
//! и NUL-terminated notification. Все count — signed Windows `long`.
//!
//! Exact bulk-copy `0x58` захватывал три padding-участка, а World loader не
//! назначал часть current-state полей до вставки записи. Поэтому старый wire
//! мог зависеть от неинициализированного stack state. У такого UB нет
//! стабильного значения для совместимости: Rust кодирует padding нулями, а
//! current YP/RP остаются обычными явно инициализированными полями. Полезные
//! offsets и порядок сохраняются без объявления Rust layout копией MSVC ABI.
//! `BTreeMap`/`Vec` заменяют process-global STL owners; legacy-строки остаются
//! byte arrays и завершаются на первом NUL.
//!
//! Create-role дополнительно достигает process-global `m_listOrginEquip` и
//! прямого `m_mapPlayerList::operator[]`. PDB задаёт `tagOrginEquip` как byte
//! occupation, `u16` equipment position и `std::string`; exact
//! `CGame::AddOrginGoodsToPlayer` `0x00405A80..0x00405B13` подтверждает offsets
//! `+0/+2/+8` внутри старого list-value и list-order обход. Typed record и
//! ordered `Vec` сохраняют значения без копирования MSVC string layout.
//! Отсутствующий ключ `sex + occupation*2` по-прежнему вставляет нулевую запись.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// Содержимое одного исходного 88-байтового player-property record.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerBaseProperties {
    pub(crate) occupation: u8,
    pub(crate) sex: u8,
    pub(crate) hot_hit: u32,
    pub(crate) remain_point: u16,
    pub(crate) yp: u16,
    pub(crate) hp: u32,
    pub(crate) mp: u32,
    pub(crate) rp: u16,
    pub(crate) base_maximum_hp: u32,
    pub(crate) base_maximum_mp: u32,
    pub(crate) base_maximum_yp: u16,
    pub(crate) base_maximum_rp: u16,
    pub(crate) base_strength: u32,
    pub(crate) base_dexterity: u32,
    pub(crate) base_constitution: u32,
    pub(crate) base_intelligence: u32,
    pub(crate) base_minimum_attack: u32,
    pub(crate) base_maximum_attack: u32,
    pub(crate) base_hit: u16,
    pub(crate) base_burden: u16,
    pub(crate) base_cch: u16,
    pub(crate) base_defence: u32,
    pub(crate) base_dodge: u16,
    pub(crate) base_attack_speed: u16,
    pub(crate) base_element_resistant: u32,
    pub(crate) base_hp_recover_speed: u16,
    pub(crate) base_mp_recover_speed: u16,
    pub(crate) constitution_to_maximum_hp: u16,
    pub(crate) intelligence_to_maximum_mp: u16,
}

/// Одно level-keyed приращение свойств и локализованное уведомление.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerPropertiesUpgrade {
    pub(crate) base_maximum_hp: u32,
    pub(crate) base_maximum_mp: u32,
    pub(crate) base_strength: u32,
    pub(crate) base_dexterity: u32,
    pub(crate) base_constitution: u32,
    pub(crate) base_intelligence: u32,
    pub(crate) base_burden: u16,
    pub(crate) notification: Vec<u8>,
}

/// Одна логическая запись точного `CPlayerList::tagOrginEquip`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerOriginEquipment {
    pub(crate) occupation: u8,
    pub(crate) place_position: u16,
    pub(crate) original_name: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerCreationPropertiesLookup {
    pub(crate) key: u32,
    pub(crate) inserted: bool,
    pub(crate) properties: PlayerBaseProperties,
}

pub(crate) type PlayerBasePropertiesMap = BTreeMap<u32, PlayerBaseProperties>;
pub(crate) type PlayerPropertiesUpgradeMap = BTreeMap<u32, PlayerPropertiesUpgrade>;

/// Value-owner пяти точных секций World/Game setup-а.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CPlayerList {
    player_properties: PlayerBasePropertiesMap,
    origin_equipment: Vec<PlayerOriginEquipment>,
    player_experience: Vec<u32>,
    fighter_upgrades: PlayerPropertiesUpgradeMap,
    hunter_upgrades: PlayerPropertiesUpgradeMap,
    taoist_upgrades: PlayerPropertiesUpgradeMap,
}

impl CPlayerList {
    /// Создаёт owner из уже проверенных loader-ом секций без global state.
    pub(crate) fn from_parts(
        player_properties: PlayerBasePropertiesMap,
        player_experience: Vec<u32>,
        fighter_upgrades: PlayerPropertiesUpgradeMap,
        hunter_upgrades: PlayerPropertiesUpgradeMap,
        taoist_upgrades: PlayerPropertiesUpgradeMap,
    ) -> Self {
        Self {
            player_properties,
            origin_equipment: Vec::new(),
            player_experience,
            fighter_upgrades,
            hunter_upgrades,
            taoist_upgrades,
        }
    }

    /// Заменяет отдельно загружаемый owner `playerOrginEquip.ini`.
    pub(crate) fn set_origin_equipment(&mut self, equipment: Vec<PlayerOriginEquipment>) {
        self.origin_equipment = equipment;
    }

    pub(crate) fn origin_equipment(&self) -> &[PlayerOriginEquipment] {
        &self.origin_equipment
    }

    /// Выполняет exact create-role `map::operator[]` lookup.
    pub(crate) fn creation_properties(
        &mut self,
        sex: u8,
        occupation: u8,
    ) -> PlayerCreationPropertiesLookup {
        let key = u32::from(sex).wrapping_add(u32::from(occupation).wrapping_mul(2));
        let inserted = !self.player_properties.contains_key(&key);
        let properties = *self.player_properties.entry(key).or_default();
        PlayerCreationPropertiesLookup {
            key,
            inserted,
            properties,
        }
    }

    /// Дописывает exact пять секций `CPlayerList::AddToByteArray`.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), PlayerListSerializeError> {
        append_count(
            destination,
            "player property map",
            self.player_properties.len(),
        )?;
        for properties in self.player_properties.values() {
            append_player_properties(destination, properties);
        }

        append_count(
            destination,
            "player experience vector",
            self.player_experience.len(),
        )?;
        for experience in &self.player_experience {
            destination.extend_from_slice(&experience.to_le_bytes());
        }

        append_upgrade_map(destination, "fighter upgrade map", &self.fighter_upgrades)?;
        append_upgrade_map(destination, "hunter upgrade map", &self.hunter_upgrades)?;
        append_upgrade_map(destination, "taoist upgrade map", &self.taoist_upgrades)?;
        Ok(())
    }
}

/// Безопасная граница размера старого signed `long` count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerListSerializeError {
    pub(crate) owner: &'static str,
    pub(crate) count: usize,
}

impl fmt::Display for PlayerListSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} содержит {} элементов вне signed 32-битного диапазона",
            self.owner, self.count
        )
    }
}

impl Error for PlayerListSerializeError {}

fn append_player_properties(destination: &mut Vec<u8>, value: &PlayerBaseProperties) {
    destination.extend_from_slice(&[value.occupation, value.sex, 0, 0]);
    destination.extend_from_slice(&value.hot_hit.to_le_bytes());
    destination.extend_from_slice(&value.remain_point.to_le_bytes());
    destination.extend_from_slice(&value.yp.to_le_bytes());
    destination.extend_from_slice(&value.hp.to_le_bytes());
    destination.extend_from_slice(&value.mp.to_le_bytes());
    destination.extend_from_slice(&value.rp.to_le_bytes());
    destination.extend_from_slice(&0_u16.to_le_bytes());
    destination.extend_from_slice(&value.base_maximum_hp.to_le_bytes());
    destination.extend_from_slice(&value.base_maximum_mp.to_le_bytes());
    destination.extend_from_slice(&value.base_maximum_yp.to_le_bytes());
    destination.extend_from_slice(&value.base_maximum_rp.to_le_bytes());
    for scalar in [
        value.base_strength,
        value.base_dexterity,
        value.base_constitution,
        value.base_intelligence,
        value.base_minimum_attack,
        value.base_maximum_attack,
    ] {
        destination.extend_from_slice(&scalar.to_le_bytes());
    }
    destination.extend_from_slice(&value.base_hit.to_le_bytes());
    destination.extend_from_slice(&value.base_burden.to_le_bytes());
    destination.extend_from_slice(&value.base_cch.to_le_bytes());
    destination.extend_from_slice(&0_u16.to_le_bytes());
    destination.extend_from_slice(&value.base_defence.to_le_bytes());
    destination.extend_from_slice(&value.base_dodge.to_le_bytes());
    destination.extend_from_slice(&value.base_attack_speed.to_le_bytes());
    destination.extend_from_slice(&value.base_element_resistant.to_le_bytes());
    destination.extend_from_slice(&value.base_hp_recover_speed.to_le_bytes());
    destination.extend_from_slice(&value.base_mp_recover_speed.to_le_bytes());
    destination.extend_from_slice(&value.constitution_to_maximum_hp.to_le_bytes());
    destination.extend_from_slice(&value.intelligence_to_maximum_mp.to_le_bytes());
}

fn append_upgrade_map(
    destination: &mut Vec<u8>,
    owner: &'static str,
    upgrades: &PlayerPropertiesUpgradeMap,
) -> Result<(), PlayerListSerializeError> {
    append_count(destination, owner, upgrades.len())?;
    for (&level, properties) in upgrades {
        destination.extend_from_slice(&level.to_le_bytes());
        for scalar in [
            properties.base_maximum_hp,
            properties.base_maximum_mp,
            properties.base_strength,
            properties.base_dexterity,
            properties.base_constitution,
            properties.base_intelligence,
        ] {
            destination.extend_from_slice(&scalar.to_le_bytes());
        }
        destination.extend_from_slice(&properties.base_burden.to_le_bytes());
        append_legacy_string(destination, &properties.notification);
    }
    Ok(())
}

fn append_count(
    destination: &mut Vec<u8>,
    owner: &'static str,
    count: usize,
) -> Result<(), PlayerListSerializeError> {
    let count = i32::try_from(count).map_err(|_| PlayerListSerializeError { owner, count })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn append_legacy_string(destination: &mut Vec<u8>, value: &[u8]) {
    destination.extend_from_slice(value.split(|byte| *byte == 0).next().unwrap_or_default());
    destination.push(0);
}

// Остальной сырой C++ ниже является комментарием, а не Rust-реализацией.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\playerlist.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp

// ============================================================================
// FUNCTION: CPlayerList::GetLevelNum
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.h:104
// RVA: 0x000300B0
// ADDRESS: 004300b0
// PROTOTYPE: ulong __cdecl GetLevelNum(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerList::GetLelExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp:436
// RVA: 0x000C6240
// ADDRESS: 004c6240
// PROTOTYPE: ulong __cdecl GetLelExp(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerList::GetPropertiesUpgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp:396
// RVA: 0x000C6560
// ADDRESS: 004c6560
// PROTOTYPE: int __cdecl GetPropertiesUpgrade(eOccupation param_1, ulong param_2, tagPropertiesUpgrade * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerList::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp:318
// RVA: 0x000C76B0
// ADDRESS: 004c76b0
// PROTOTYPE: bool __cdecl DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerList::GetProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.h:101
// RVA: 0x000FAA80
// ADDRESS: 004faa80
// PROTOTYPE: tagPlayerProperty * __cdecl GetProperty(uchar param_1, uchar param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::Box::Odds::~Odds
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x001C58F0
// ADDRESS: 005c58f0
// PROTOTYPE: void __thiscall ~Odds(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c5bf3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x001C5BF3
// ADDRESS: 005c5bf3
// PROTOTYPE: undefined Catch@005c5bf3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c5db4
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x001C5DB4
// ADDRESS: 005c5db4
// PROTOTYPE: undefined Catch@005c5db4()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c6184
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x001C6184
// ADDRESS: 005c6184
// PROTOTYPE: undefined Catch@005c6184()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c6236
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x001C6236
// ADDRESS: 005c6236
// PROTOTYPE: undefined Catch@005c6236()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c63da
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x001C63DA
// ADDRESS: 005c63da
// PROTOTYPE: undefined Catch@005c63da()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c65bc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x001C65BC
// ADDRESS: 005c65bc
// PROTOTYPE: undefined Catch@005c65bc()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c666f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x001C666F
// ADDRESS: 005c666f
// PROTOTYPE: undefined Catch@005c666f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::Box::~Box
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x001C6700
// ADDRESS: 005c6700
// PROTOTYPE: void __thiscall ~Box(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c6a46
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x001C6A46
// ADDRESS: 005c6a46
// PROTOTYPE: undefined Catch@005c6a46()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp

// ============================================================================
// FUNCTION: CPlayerList::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp:247
// RVA: 0x0002C4D0
// ADDRESS: 0042c4d0
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerList::GetPropertiesUpgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp:396
// RVA: 0x0002CAE0
// ADDRESS: 0042cae0
// PROTOTYPE: int __cdecl GetPropertiesUpgrade(eOccupation param_1, ulong param_2, tagPropertiesUpgrade * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerList::LoadPlayerList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp:37
// RVA: 0x0002DD20
// ADDRESS: 0042dd20
// PROTOTYPE: int __cdecl LoadPlayerList(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerList::LoadPlayerExpList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp:212
// RVA: 0x0002E290
// ADDRESS: 0042e290
// PROTOTYPE: bool __cdecl LoadPlayerExpList(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerList::LoadPlayerProperitiesUpgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp:146
// RVA: 0x0002E710
// ADDRESS: 0042e710
// PROTOTYPE: int __cdecl LoadPlayerProperitiesUpgrade(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::BoxConf::OddsConf::~OddsConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x00046430
// ADDRESS: 00446430
// PROTOTYPE: void __thiscall ~OddsConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00446733
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x00046733
// ADDRESS: 00446733
// PROTOTYPE: undefined Catch@00446733()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004468f4
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x000468F4
// ADDRESS: 004468f4
// PROTOTYPE: undefined Catch@004468f4()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00446cc4
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x00046CC4
// ADDRESS: 00446cc4
// PROTOTYPE: undefined Catch@00446cc4()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00446d76
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x00046D76
// ADDRESS: 00446d76
// PROTOTYPE: undefined Catch@00446d76()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044705c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x0004705C
// ADDRESS: 0044705c
// PROTOTYPE: undefined Catch@0044705c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044710f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x0004710F
// ADDRESS: 0044710f
// PROTOTYPE: undefined Catch@0044710f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044722a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x0004722A
// ADDRESS: 0044722a
// PROTOTYPE: undefined Catch@0044722a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044740c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x0004740C
// ADDRESS: 0044740c
// PROTOTYPE: undefined Catch@0044740c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004474bf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x000474BF
// ADDRESS: 004474bf
// PROTOTYPE: undefined Catch@004474bf()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::BoxConf::~BoxConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x00047550
// ADDRESS: 00447550
// PROTOTYPE: void __thiscall ~BoxConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00447996
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x00047996
// ADDRESS: 00447996
// PROTOTYPE: undefined Catch@00447996()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052e3f0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x0012E3F0
// ADDRESS: 0052e3f0
// PROTOTYPE: undefined Unwind@0052e3f0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052e410
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x0012E410
// ADDRESS: 0052e410
// PROTOTYPE: undefined Unwind@0052e410()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052e480
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\playerlist.cpp
// RVA: 0x0012E480
// ADDRESS: 0052e480
// PROTOTYPE: undefined Unwind@0052e480()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
