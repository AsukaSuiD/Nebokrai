//! Общая конфигурация заданий исторического Miracle.
//!
//! Статус World `CQuestSystem::AddToByteArray` RVA `0x000669E0`:
//! `IMPLEMENTED`; loaders, singleton lifecycle и Game runtime ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные owner-ы PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:218` и
//! `e:\svn\fengyun_russia_dev\server\setup\questsystem.h`.
//!
//! Exact World serializer и Game decoder подтверждают positional wire:
//! max-count, level difference, три C-string script-а, signed quest count и
//! ordered quest records. `BTreeMap<u16, _>` заменяет `std::map<ushort, _>` и
//! сохраняет unsigned key-order; ключ отдельно не передаётся. Внутри записи
//! после пяти `u32` идут пять C-string, затем именно region/x/y/effect и byte
//! display, хотя PDB layout размещал effect раньше координат. Typed поля и
//! owned bytes устраняют C++ layout/lifetime; NUL/count ошибки блокируются до
//! изменения destination. Точная parser-семантика двух ini остаётся отдельной.
//! `FilesInfo.ril` подтверждает resource-границу loader-а: `Data/Quest.ini`
//! берётся из `patch01.pak/data/quest.ini`, где после `MaxQuestNum` есть
//! `maxlvldiff 130`; `Data/QuestEx.ini` отсутствует в package-index и
//! открывается как loose resource. Одноимённый loose `quest.ini` без
//! `maxlvldiff` устарел и не задаёт контракт `0x00467AA0`.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct QuestEntry {
    pub(crate) id: u16,
    pub(crate) old: u32,
    pub(crate) quest_type: u32,
    pub(crate) level: u32,
    pub(crate) difficulty: u32,
    pub(crate) track: u32,
    pub(crate) short_description: Vec<u8>,
    pub(crate) name: Vec<u8>,
    pub(crate) description: Vec<u8>,
    pub(crate) abandon_script: Vec<u8>,
    pub(crate) complete_script: Vec<u8>,
    pub(crate) region_id: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
    pub(crate) effect_id: i32,
    pub(crate) display: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CQuestSystem {
    pub(crate) max_quest_count: i32,
    pub(crate) level_difference: u32,
    pub(crate) player_login_script: Vec<u8>,
    pub(crate) player_level_up_script: Vec<u8>,
    pub(crate) player_died_script: Vec<u8>,
    quests: BTreeMap<u16, QuestEntry>,
}

impl CQuestSystem {
    /// Exact начало `Load`: очищается только map заданий; scalar и script
    /// поля сохраняются, если первый resource `Data/Quest.ini` недоступен.
    pub(crate) fn clear_quests_for_load(&mut self) {
        self.quests.clear();
    }

    pub(crate) fn insert(&mut self, quest: QuestEntry) -> Option<QuestEntry> {
        self.quests.insert(quest.id, quest)
    }

    pub(crate) fn quests(&self) -> &BTreeMap<u16, QuestEntry> {
        &self.quests
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), QuestSystemSerializationBlock> {
        let count = i32::try_from(self.quests.len()).map_err(|_| {
            QuestSystemSerializationBlock::QuestCountOutOfRange {
                count: self.quests.len(),
            }
        })?;
        let mut payload = Vec::new();
        payload.extend_from_slice(&self.max_quest_count.to_le_bytes());
        payload.extend_from_slice(&self.level_difference.to_le_bytes());
        write_quest_string(
            &mut payload,
            None,
            QuestStringField::PlayerLoginScript,
            &self.player_login_script,
        )?;
        write_quest_string(
            &mut payload,
            None,
            QuestStringField::PlayerLevelUpScript,
            &self.player_level_up_script,
        )?;
        write_quest_string(
            &mut payload,
            None,
            QuestStringField::PlayerDiedScript,
            &self.player_died_script,
        )?;
        payload.extend_from_slice(&count.to_le_bytes());
        for quest in self.quests.values() {
            payload.extend_from_slice(&quest.id.to_le_bytes());
            for value in [
                quest.old,
                quest.quest_type,
                quest.level,
                quest.difficulty,
                quest.track,
            ] {
                payload.extend_from_slice(&value.to_le_bytes());
            }
            for (field, value) in [
                (
                    QuestStringField::ShortDescription,
                    quest.short_description.as_slice(),
                ),
                (QuestStringField::Name, quest.name.as_slice()),
                (QuestStringField::Description, quest.description.as_slice()),
                (
                    QuestStringField::AbandonScript,
                    quest.abandon_script.as_slice(),
                ),
                (
                    QuestStringField::CompleteScript,
                    quest.complete_script.as_slice(),
                ),
            ] {
                write_quest_string(&mut payload, Some(quest.id), field, value)?;
            }
            payload.extend_from_slice(&quest.region_id.to_le_bytes());
            payload.extend_from_slice(&quest.tile_x.to_le_bytes());
            payload.extend_from_slice(&quest.tile_y.to_le_bytes());
            payload.extend_from_slice(&quest.effect_id.to_le_bytes());
            payload.push(u8::from(quest.display));
        }
        destination.extend_from_slice(&payload);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QuestStringField {
    PlayerLoginScript,
    PlayerLevelUpScript,
    PlayerDiedScript,
    ShortDescription,
    Name,
    Description,
    AbandonScript,
    CompleteScript,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum QuestSystemSerializationBlock {
    QuestCountOutOfRange { count: usize },
    StringContainsNul {
        quest_id: Option<u16>,
        field: QuestStringField,
    },
}

impl fmt::Display for QuestSystemSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QuestCountOutOfRange { count } => write!(
                formatter,
                "QuestSystem содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::StringContainsNul { quest_id, field } => {
                let owner = quest_id
                    .map_or_else(|| "setup".to_owned(), |id| format!("quest {id}"));
                write!(
                    formatter,
                    "QuestSystem {owner}, поле {field:?} содержит внутренний NUL"
                )
            }
        }
    }
}

impl Error for QuestSystemSerializationBlock {}

fn write_quest_string(
    destination: &mut Vec<u8>,
    quest_id: Option<u16>,
    field: QuestStringField,
    value: &[u8],
) -> Result<(), QuestSystemSerializationBlock> {
    if value.contains(&0) {
        return Err(QuestSystemSerializationBlock::StringContainsNul {
            quest_id,
            field,
        });
    }
    destination.extend_from_slice(value);
    destination.push(0);
    Ok(())
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\questsystem.h

// ============================================================================
// FUNCTION: CQuestSystem::GetQuestDataById
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:285
// RVA: 0x000617F0
// ADDRESS: 004617f0
// PROTOTYPE: tagQuest * __thiscall GetQuestDataById(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CQuestSystem::GetCompleteScripById
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:293
// RVA: 0x00061820
// ADDRESS: 00461820
// PROTOTYPE: char * __thiscall GetCompleteScripById(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CQuestSystem::GetDisbandScripById
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:301
// RVA: 0x00061860
// ADDRESS: 00461860
// PROTOTYPE: char * __thiscall GetDisbandScripById(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagQuest::tagQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.h:26
// RVA: 0x00061A00
// ADDRESS: 00461a00
// PROTOTYPE: undefined __thiscall tagQuest(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CQuestSystem::~CQuestSystem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:26
// RVA: 0x00062330
// ADDRESS: 00462330
// PROTOTYPE: void __thiscall ~CQuestSystem(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CQuestSystem::CQuestSystem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:22
// RVA: 0x00062580
// ADDRESS: 00462580
// PROTOTYPE: undefined __thiscall CQuestSystem(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CQuestSystem::getInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:31
// RVA: 0x00062610
// ADDRESS: 00462610
// PROTOTYPE: CQuestSystem * __cdecl getInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CQuestSystem::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:47
// RVA: 0x00062680
// ADDRESS: 00462680
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetQuestSys
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:308
// RVA: 0x000626B0
// ADDRESS: 004626b0
// PROTOTYPE: CQuestSystem * __cdecl GetQuestSys(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CQuestSystem::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:251
// RVA: 0x000627A0
// ADDRESS: 004627a0
// PROTOTYPE: void __thiscall DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004a4c11
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp
// RVA: 0x000A4C11
// ADDRESS: 004a4c11
// PROTOTYPE: undefined Catch@004a4c11()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004a52d1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp
// RVA: 0x000A52D1
// ADDRESS: 004a52d1
// PROTOTYPE: undefined Catch@004a52d1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004a5492
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp
// RVA: 0x000A5492
// ADDRESS: 004a5492
// PROTOTYPE: undefined Catch@004a5492()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//










// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\questsystem.h

// ============================================================================
// FUNCTION: CQuestSystem::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:218
// RVA: 0x000669E0
// ADDRESS: 004669e0
// PROTOTYPE: void __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagQuest::tagQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.h:26
// RVA: 0x00066D00
// ADDRESS: 00466d00
// PROTOTYPE: undefined __thiscall tagQuest(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CQuestSystem::~CQuestSystem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:26
// RVA: 0x00067630
// ADDRESS: 00467630
// PROTOTYPE: void __thiscall ~CQuestSystem(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CQuestSystem::CQuestSystem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:22
// RVA: 0x00067880
// ADDRESS: 00467880
// PROTOTYPE: undefined __thiscall CQuestSystem(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CQuestSystem::getInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:31
// RVA: 0x00067910
// ADDRESS: 00467910
// PROTOTYPE: CQuestSystem * __cdecl getInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CQuestSystem::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:47
// RVA: 0x00067980
// ADDRESS: 00467980
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetQuestSys
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:308
// RVA: 0x000679B0
// ADDRESS: 004679b0
// PROTOTYPE: CQuestSystem * __cdecl GetQuestSys(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CQuestSystem::Load
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:56
// RVA: 0x00067AA0
// ADDRESS: 00467aa0
// PROTOTYPE: void __thiscall Load(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CQuestSystem::Initialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp:40
// RVA: 0x00068BE0
// ADDRESS: 00468be0
// PROTOTYPE: bool __thiscall Initialize(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004b27e1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp
// RVA: 0x000B27E1
// ADDRESS: 004b27e1
// PROTOTYPE: undefined Catch@004b27e1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004b2cd2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp
// RVA: 0x000B2CD2
// ADDRESS: 004b2cd2
// PROTOTYPE: undefined Catch@004b2cd2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004b3311
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp
// RVA: 0x000B3311
// ADDRESS: 004b3311
// PROTOTYPE: undefined Catch@004b3311()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004b3566
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp
// RVA: 0x000B3566
// ADDRESS: 004b3566
// PROTOTYPE: undefined Catch@004b3566()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004b3a22
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp
// RVA: 0x000B3A22
// ADDRESS: 004b3a22
// PROTOTYPE: undefined Catch@004b3a22()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004b3d71
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp
// RVA: 0x000B3D71
// ADDRESS: 004b3d71
// PROTOTYPE: undefined Catch@004b3d71()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004b4331
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp
// RVA: 0x000B4331
// ADDRESS: 004b4331
// PROTOTYPE: undefined Catch@004b4331()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@005332e0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp
// RVA: 0x001332E0
// ADDRESS: 005332e0
// PROTOTYPE: undefined Unwind@005332e0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// FUNCTION: Unwind@00533330
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\questsystem.cpp
// RVA: 0x00133330
// ADDRESS: 00533330
// PROTOTYPE: undefined Unwind@00533330()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//









// COMPONENT_VARIANT_END: WorldServer
