//! Конфигурация Gods Battle исторического Miracle.
//!
//! Статус World `CGodsBattleConf::AddByteToArray` RVA `0x0007E990`:
//! `IMPLEMENTED`; loaders, persistence и runtime accessors ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\godsbattleconf.cpp:138`.
//!
//! Game decoder `CGodsBattleMgr::DecordFromByteArray` RVA `0x000AB260`
//! подтверждает wire: NPC `i32 + C-string + C-string`, ordered base-money
//! records по 12 bytes, затем vector records `16/12/12` bytes, два `u32` XYD
//! и 28-байтные die-back records. Все counts — signed Windows `long`.
//! `BTreeMap<u32, _>` заменяет `std::map` без изменения unsigned key-order;
//! его ключ отдельно не передаётся. Owned byte strings сохраняют исходную
//! кодировку, а typed fields исключают C++ padding и raw-memory lifetime.
//! Неизменяемая ссылка даёт serializer-у согласованный state; синхронизация
//! общего runtime owner-а не встраивается в сам wire-value.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleFactionNpcName {
    pub(crate) faction: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) monsters: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleBaseMoney {
    pub(crate) money_level: u32,
    pub(crate) add: u32,
    pub(crate) subtract: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleReviseMoney {
    pub(crate) level_gap_revise: u32,
    pub(crate) money_level_gap_revise: u32,
    pub(crate) revise_min: u32,
    pub(crate) revise_max: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleSzlLevel {
    pub(crate) level: u32,
    pub(crate) min_szl: u32,
    pub(crate) max_szl: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleFactionRule {
    pub(crate) faction: u32,
    pub(crate) country_a: u32,
    pub(crate) country_b: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleDieBackPosition {
    pub(crate) region: i32,
    pub(crate) destination_region: i32,
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
    pub(crate) faction: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CGodsBattleConf {
    npc_names: Vec<GodsBattleFactionNpcName>,
    base_money: BTreeMap<u32, GodsBattleBaseMoney>,
    revise_money: Vec<GodsBattleReviseMoney>,
    szl_levels: Vec<GodsBattleSzlLevel>,
    faction_rules: Vec<GodsBattleFactionRule>,
    xyd: [u32; 3],
    die_back_positions: Vec<GodsBattleDieBackPosition>,
}

impl CGodsBattleConf {
    pub(crate) fn push_npc(&mut self, npc: GodsBattleFactionNpcName) {
        self.npc_names.push(npc);
    }

    pub(crate) fn insert_base_money(
        &mut self,
        key: u32,
        value: GodsBattleBaseMoney,
    ) -> Option<GodsBattleBaseMoney> {
        self.base_money.insert(key, value)
    }

    pub(crate) fn push_revise_money(&mut self, value: GodsBattleReviseMoney) {
        self.revise_money.push(value);
    }

    pub(crate) fn push_szl_level(&mut self, value: GodsBattleSzlLevel) {
        self.szl_levels.push(value);
    }

    pub(crate) fn push_faction_rule(&mut self, value: GodsBattleFactionRule) {
        self.faction_rules.push(value);
    }

    pub(crate) fn set_xyd_from_db(&mut self, faction_one: u32, faction_two: u32) {
        self.xyd[1] = faction_one;
        self.xyd[2] = faction_two;
    }

    pub(crate) fn push_die_back_position(&mut self, value: GodsBattleDieBackPosition) {
        self.die_back_positions.push(value);
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), GodsBattleSerializeError> {
        write_gods_battle_count(
            destination,
            GodsBattleCollection::NpcNames,
            self.npc_names.len(),
        )?;
        for (record_index, npc) in self.npc_names.iter().enumerate() {
            destination.extend_from_slice(&npc.faction.to_le_bytes());
            write_gods_battle_string(
                destination,
                record_index,
                GodsBattleStringField::NpcName,
                &npc.name,
            )?;
            write_gods_battle_string(
                destination,
                record_index,
                GodsBattleStringField::Monsters,
                &npc.monsters,
            )?;
        }

        write_gods_battle_count(
            destination,
            GodsBattleCollection::BaseMoney,
            self.base_money.len(),
        )?;
        for money in self.base_money.values() {
            destination.extend_from_slice(&money.money_level.to_le_bytes());
            destination.extend_from_slice(&money.add.to_le_bytes());
            destination.extend_from_slice(&money.subtract.to_le_bytes());
        }

        write_gods_battle_count(
            destination,
            GodsBattleCollection::ReviseMoney,
            self.revise_money.len(),
        )?;
        for revise in &self.revise_money {
            destination.extend_from_slice(&revise.level_gap_revise.to_le_bytes());
            destination.extend_from_slice(&revise.money_level_gap_revise.to_le_bytes());
            destination.extend_from_slice(&revise.revise_min.to_le_bytes());
            destination.extend_from_slice(&revise.revise_max.to_le_bytes());
        }

        write_gods_battle_count(
            destination,
            GodsBattleCollection::SzlLevels,
            self.szl_levels.len(),
        )?;
        for level in &self.szl_levels {
            destination.extend_from_slice(&level.level.to_le_bytes());
            destination.extend_from_slice(&level.min_szl.to_le_bytes());
            destination.extend_from_slice(&level.max_szl.to_le_bytes());
        }

        write_gods_battle_count(
            destination,
            GodsBattleCollection::FactionRules,
            self.faction_rules.len(),
        )?;
        for rule in &self.faction_rules {
            destination.extend_from_slice(&rule.faction.to_le_bytes());
            destination.extend_from_slice(&rule.country_a.to_le_bytes());
            destination.extend_from_slice(&rule.country_b.to_le_bytes());
        }

        destination.extend_from_slice(&self.xyd[1].to_le_bytes());
        destination.extend_from_slice(&self.xyd[2].to_le_bytes());

        write_gods_battle_count(
            destination,
            GodsBattleCollection::DieBackPositions,
            self.die_back_positions.len(),
        )?;
        for position in &self.die_back_positions {
            destination.extend_from_slice(&position.region.to_le_bytes());
            destination.extend_from_slice(&position.destination_region.to_le_bytes());
            destination.extend_from_slice(&position.left.to_le_bytes());
            destination.extend_from_slice(&position.top.to_le_bytes());
            destination.extend_from_slice(&position.right.to_le_bytes());
            destination.extend_from_slice(&position.bottom.to_le_bytes());
            destination.extend_from_slice(&position.faction.to_le_bytes());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleCollection {
    NpcNames,
    BaseMoney,
    ReviseMoney,
    SzlLevels,
    FactionRules,
    DieBackPositions,
}

impl fmt::Display for GodsBattleCollection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NpcNames => "NPC-записей",
            Self::BaseMoney => "base-money записей",
            Self::ReviseMoney => "revise-money записей",
            Self::SzlLevels => "SZL-level записей",
            Self::FactionRules => "правил фракций",
            Self::DieBackPositions => "точек возврата после смерти",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleStringField {
    NpcName,
    Monsters,
}

impl fmt::Display for GodsBattleStringField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NpcName => "имя NPC",
            Self::Monsters => "список монстров",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleSerializeError {
    CountOutOfRange {
        collection: GodsBattleCollection,
        count: usize,
    },
    StringContainsNul {
        record_index: usize,
        field: GodsBattleStringField,
    },
}

impl fmt::Display for GodsBattleSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange { collection, count } => write!(
                formatter,
                "GodsBattle содержит {count} {collection} вне signed 32-битного диапазона"
            ),
            Self::StringContainsNul {
                record_index,
                field,
            } => write!(
                formatter,
                "GodsBattle NPC #{record_index}: {field} содержит внутренний NUL"
            ),
        }
    }
}

impl Error for GodsBattleSerializeError {}

fn write_gods_battle_count(
    destination: &mut Vec<u8>,
    collection: GodsBattleCollection,
    count: usize,
) -> Result<(), GodsBattleSerializeError> {
    let count_i32 = i32::try_from(count)
        .map_err(|_| GodsBattleSerializeError::CountOutOfRange { collection, count })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

fn write_gods_battle_string(
    destination: &mut Vec<u8>,
    record_index: usize,
    field: GodsBattleStringField,
    value: &[u8],
) -> Result<(), GodsBattleSerializeError> {
    if value.contains(&0) {
        return Err(GodsBattleSerializeError::StringContainsNul {
            record_index,
            field,
        });
    }
    destination.extend_from_slice(value);
    destination.push(0);
    Ok(())
}

// Сырой C++ ниже сохранён как локальная документация loaders, persistence и
// runtime accessors, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\godsbattleconf.cpp

// ============================================================================
// FUNCTION: CGodsBattleConf::SetXYDFrmDB
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\godsbattleconf.cpp:199
// RVA: 0x0007E460
// ADDRESS: 0047e460
// PROTOTYPE: void __thiscall SetXYDFrmDB(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleConf::SetFactionXYD
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\godsbattleconf.cpp:234
// RVA: 0x0007E480
// ADDRESS: 0047e480
// PROTOTYPE: void __thiscall SetFactionXYD(int param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleConf::SaveNpcFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\godsbattleconf.cpp:321
// RVA: 0x0007E610
// ADDRESS: 0047e610
// PROTOTYPE: void __thiscall SaveNpcFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleConf::AddByteToArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\godsbattleconf.cpp:138
// RVA: 0x0007E990
// ADDRESS: 0047e990
// PROTOTYPE: void __thiscall AddByteToArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Реализация находится выше; декомпиляция сохранена как byte-order provenance.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleConf::SetNpcFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\godsbattleconf.cpp:247
// RVA: 0x0007EC50
// ADDRESS: 0047ec50
// PROTOTYPE: void __thiscall SetNpcFaction(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleConf::CGodsBattleConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\godsbattleconf.cpp:227
// RVA: 0x00080FA0
// ADDRESS: 00480fa0
// PROTOTYPE: undefined __thiscall CGodsBattleConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleConf::LoadFile
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\godsbattleconf.cpp:10
// RVA: 0x000810D0
// ADDRESS: 004810d0
// PROTOTYPE: int __thiscall LoadFile(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleConf::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\godsbattleconf.cpp:206
// RVA: 0x00081A20
// ADDRESS: 00481a20
// PROTOTYPE: CGodsBattleConf * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::GetPos
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\godsbattleconf.cpp:217
// RVA: 0x000DEA50
// ADDRESS: 004dea50
// PROTOTYPE: long __thiscall GetPos(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleConf::GetBFactionXYD
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\godsbattleconf.cpp:222
// RVA: 0x000DEBA0
// ADDRESS: 004deba0
// PROTOTYPE: ulong __thiscall GetBFactionXYD(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
