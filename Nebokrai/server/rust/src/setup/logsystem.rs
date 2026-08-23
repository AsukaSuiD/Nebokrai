//! Настройки журналирования исторического Miracle.
//!
//! Статус World `CLogSystem::AddToByteArray` RVA `0x00099B60`:
//! `IMPLEMENTED`; text loader, accessors и Game decoder ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp:164`.
//!
//! Wire состоит из точного 64-байтного `tagLogSystem`, signed количества и
//! ordered `long`-ключей предметов. GameServer читает структуру целиком и
//! использует её поля по ABI offsets, поэтому fixed byte snapshot здесь
//! сохраняет подтверждённый контракт без выдуманной раскладки. Windows
//! `long` моделируется `i32`, а `BTreeSet` стандартной библиотеки заменяет
//! `std::set` и сохраняет его signed-порядок и уникальность. Static storage
//! оригинала было нулевым до loader-а; `Default` воспроизводит это без
//! C++ singleton/lifetime plumbing. Typed overlay полей будет уместен вместе
//! с восстановлением loader-а; неизвестные offsets сейчас не именуются.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

pub(crate) const LOG_SETTINGS_LENGTH: usize = 0x40;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CLogSystem {
    settings: [u8; LOG_SETTINGS_LENGTH],
    items: BTreeSet<i32>,
}

impl Default for CLogSystem {
    fn default() -> Self {
        Self {
            settings: [0; LOG_SETTINGS_LENGTH],
            items: BTreeSet::new(),
        }
    }
}

impl CLogSystem {
    pub(crate) fn from_settings(settings: [u8; LOG_SETTINGS_LENGTH]) -> Self {
        Self {
            settings,
            items: BTreeSet::new(),
        }
    }

    pub(crate) fn settings_mut(&mut self) -> &mut [u8; LOG_SETTINGS_LENGTH] {
        &mut self.settings
    }

    pub(crate) fn insert_item(&mut self, goods_id: i32) -> bool {
        self.items.insert(goods_id)
    }

    pub(crate) fn clear_items(&mut self) {
        self.items.clear();
    }

    fn setting(&self, offset: usize) -> bool {
        self.settings[offset] != 0
    }

    pub(crate) fn delete_log_enabled(&self) -> bool {
        self.setting(26)
    }

    pub(crate) fn faction_create_enabled(&self) -> bool {
        self.setting(32)
    }

    pub(crate) fn faction_disband_enabled(&self) -> bool {
        self.setting(33)
    }

    pub(crate) fn faction_apply_enabled(&self) -> bool {
        self.setting(34)
    }

    pub(crate) fn faction_quit_enabled(&self) -> bool {
        self.setting(35)
    }

    pub(crate) fn faction_join_enabled(&self) -> bool {
        self.setting(36)
    }

    pub(crate) fn faction_fire_out_enabled(&self) -> bool {
        self.setting(37)
    }

    pub(crate) fn faction_title_enabled(&self) -> bool {
        self.setting(38)
    }

    pub(crate) fn faction_purview_add_enabled(&self) -> bool {
        self.setting(39)
    }

    pub(crate) fn faction_purview_revoke_enabled(&self) -> bool {
        self.setting(40)
    }

    pub(crate) fn faction_master_changed_enabled(&self) -> bool {
        self.setting(41)
    }

    pub(crate) fn faction_experience_enabled(&self) -> bool {
        self.setting(42)
    }

    pub(crate) fn faction_level_enabled(&self) -> bool {
        self.setting(43)
    }

    pub(crate) fn faction_chat_enabled(&self) -> bool {
        self.setting(46)
    }

    pub(crate) fn private_chat_enabled(&self) -> bool {
        self.setting(49)
    }

    /// Читает 64 positional boolean-а и последующий `* original-name` список.
    /// Goods lookup выполняется тем же factory-owner-ом, который обслуживает
    /// runtime и initial configuration.
    pub(crate) fn load_from_bytes(
        &mut self,
        source: &[u8],
        mut query_goods_id: impl FnMut(&[u8]) -> u32,
    ) -> Result<LogSystemLoadReport, LogSystemLoadError> {
        let mut settings = [0_u8; LOG_SETTINGS_LENGTH];
        let mut setting_count = 0;
        let mut items = BTreeSet::new();
        for (line_index, line) in source.split(|byte| *byte == b'\n').enumerate() {
            let tokens: Vec<&[u8]> = line
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty())
                .collect();
            if tokens.is_empty() || tokens[0].starts_with(b"//") {
                continue;
            }
            if tokens[0] == b"*" {
                let original_name = tokens.get(1).ok_or(LogSystemLoadError::MissingGoodsName {
                    line: line_index + 1,
                })?;
                items.insert(query_goods_id(original_name) as i32);
                continue;
            }
            if setting_count < LOG_SETTINGS_LENGTH
                && tokens.len() >= 2
                && matches!(tokens[1], b"0" | b"1")
            {
                settings[setting_count] = tokens[1][0] - b'0';
                setting_count += 1;
            }
        }
        if setting_count != LOG_SETTINGS_LENGTH {
            return Err(LogSystemLoadError::SettingCount {
                actual: setting_count,
            });
        }
        self.settings = settings;
        self.items = items;
        Ok(LogSystemLoadReport {
            settings: setting_count,
            items: self.items.len(),
        })
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), LogSystemSerializeError> {
        let count = self.items.len();
        let count_i32 = i32::try_from(count)
            .map_err(|_| LogSystemSerializeError::ItemCountOutOfRange { count })?;

        destination.extend_from_slice(&self.settings);
        destination.extend_from_slice(&count_i32.to_le_bytes());
        for &goods_id in &self.items {
            destination.extend_from_slice(&goods_id.to_le_bytes());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LogSystemSerializeError {
    ItemCountOutOfRange { count: usize },
}

impl fmt::Display for LogSystemSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ItemCountOutOfRange { count } => write!(
                formatter,
                "CLogSystem содержит {count} предметов вне signed 32-битного диапазона"
            ),
        }
    }
}

impl Error for LogSystemSerializeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LogSystemLoadReport {
    pub(crate) settings: usize,
    pub(crate) items: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LogSystemLoadError {
    SettingCount { actual: usize },
    MissingGoodsName { line: usize },
}

impl fmt::Display for LogSystemLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SettingCount { actual } => write!(
                formatter,
                "LogSystem содержит {actual} positional boolean-настроек вместо 64"
            ),
            Self::MissingGoodsName { line } => {
                write!(formatter, "LogSystem, строка {line}: после '*' отсутствует имя предмета")
            }
        }
    }
}

impl Error for LogSystemLoadError {}

// Сырой C++ ниже сохранён как локальная документация loader-а, accessors и
// Game decoder side effects, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp

// ============================================================================
// FUNCTION: CMonsterList::tagMonster::~tagMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006D3B0
// ADDRESS: 0046d3b0
// PROTOTYPE: void __thiscall ~tagMonster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::tagMonster::tagMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006D600
// ADDRESS: 0046d600
// PROTOTYPE: undefined __thiscall tagMonster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::tagDropGoods::tagDrop::tagDrop
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006D7E0
// ADDRESS: 0046d7e0
// PROTOTYPE: undefined __thiscall tagDrop(tagDrop * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0046de3c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006DE3C
// ADDRESS: 0046de3c
// PROTOTYPE: undefined Catch@0046de3c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0046e04d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006E04D
// ADDRESS: 0046e04d
// PROTOTYPE: undefined Catch@0046e04d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0046e105
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006E105
// ADDRESS: 0046e105
// PROTOTYPE: undefined Catch@0046e105()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0046e369
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006E369
// ADDRESS: 0046e369
// PROTOTYPE: undefined Catch@0046e369()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0046e40c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006E40C
// ADDRESS: 0046e40c
// PROTOTYPE: undefined Catch@0046e40c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0046e5fc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006E5FC
// ADDRESS: 0046e5fc
// PROTOTYPE: undefined Catch@0046e5fc()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0046e6b9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006E6B9
// ADDRESS: 0046e6b9
// PROTOTYPE: undefined Catch@0046e6b9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::tagDropGoods::~tagDropGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006E750
// ADDRESS: 0046e750
// PROTOTYPE: void __thiscall ~tagDropGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::tagMonster::tagMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006E9D0
// ADDRESS: 0046e9d0
// PROTOTYPE: undefined __thiscall tagMonster(tagMonster * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::tagDropGoods::tagDropGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006EBA0
// ADDRESS: 0046eba0
// PROTOTYPE: undefined __thiscall tagDropGoods(tagDropGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::tagMonster::operator=
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006ED30
// ADDRESS: 0046ed30
// PROTOTYPE: tagMonster * __thiscall operator=(tagMonster * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0046f0e7
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006F0E7
// ADDRESS: 0046f0e7
// PROTOTYPE: undefined Catch@0046f0e7()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0046f17e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0006F17E
// ADDRESS: 0046f17e
// PROTOTYPE: undefined Catch@0046f17e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLogSystem::is_log_item
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp:197
// RVA: 0x000A5A80
// ADDRESS: 004a5a80
// PROTOTYPE: bool __cdecl is_log_item(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLogSystem::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp:177
// RVA: 0x000A5AB0
// ADDRESS: 004a5ab0
// PROTOTYPE: bool __cdecl DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

















// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp

// ============================================================================
// FUNCTION: CLogSystem::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp:164
// RVA: 0x00099B60
// ADDRESS: 00499b60
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLogSystem::Load
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp:33
// RVA: 0x00099E90
// ADDRESS: 00499e90
// PROTOTYPE: bool __cdecl Load(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::tagMonster::~tagMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009C2D0
// ADDRESS: 0049c2d0
// PROTOTYPE: void __thiscall ~tagMonster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049c3da
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009C3DA
// ADDRESS: 0049c3da
// PROTOTYPE: undefined Catch@0049c3da()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0049c408
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009C408
// ADDRESS: 0049c408
// PROTOTYPE: undefined FUN_0049c408()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049c526
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009C526
// ADDRESS: 0049c526
// PROTOTYPE: undefined Catch@0049c526()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0049c54e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009C54E
// ADDRESS: 0049c54e
// PROTOTYPE: undefined FUN_0049c54e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::tagMonster::tagMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009C6E0
// ADDRESS: 0049c6e0
// PROTOTYPE: undefined __thiscall tagMonster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::tagDropGoods::tagDrop::tagDrop
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009CA30
// ADDRESS: 0049ca30
// PROTOTYPE: undefined __thiscall tagDrop(tagDrop * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049d08c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009D08C
// ADDRESS: 0049d08c
// PROTOTYPE: undefined Catch@0049d08c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049d29d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009D29D
// ADDRESS: 0049d29d
// PROTOTYPE: undefined Catch@0049d29d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049d355
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009D355
// ADDRESS: 0049d355
// PROTOTYPE: undefined Catch@0049d355()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049d5b9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009D5B9
// ADDRESS: 0049d5b9
// PROTOTYPE: undefined Catch@0049d5b9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049d65c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009D65C
// ADDRESS: 0049d65c
// PROTOTYPE: undefined Catch@0049d65c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049d84c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009D84C
// ADDRESS: 0049d84c
// PROTOTYPE: undefined Catch@0049d84c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049d909
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009D909
// ADDRESS: 0049d909
// PROTOTYPE: undefined Catch@0049d909()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::tagDropGoods::~tagDropGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009D9A0
// ADDRESS: 0049d9a0
// PROTOTYPE: void __thiscall ~tagDropGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::tagMonster::tagMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009DC20
// ADDRESS: 0049dc20
// PROTOTYPE: undefined __thiscall tagMonster(tagMonster * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::tagDropGoods::tagDropGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009DDF0
// ADDRESS: 0049ddf0
// PROTOTYPE: undefined __thiscall tagDropGoods(tagDropGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::tagMonster::operator=
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009DF80
// ADDRESS: 0049df80
// PROTOTYPE: tagMonster * __thiscall operator=(tagMonster * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049e337
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009E337
// ADDRESS: 0049e337
// PROTOTYPE: undefined Catch@0049e337()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049e3ce
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x0009E3CE
// ADDRESS: 0049e3ce
// PROTOTYPE: undefined Catch@0049e3ce()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// FUNCTION: Unwind@00532090
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x00132090
// ADDRESS: 00532090
// PROTOTYPE: undefined Unwind@00532090()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@005320b0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x001320B0
// ADDRESS: 005320b0
// PROTOTYPE: undefined Unwind@005320b0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// ============================================================================
// FUNCTION: Unwind@00532170
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x00132170
// ADDRESS: 00532170
// PROTOTYPE: undefined Unwind@00532170()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00532190
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\logsystem.cpp
// RVA: 0x00132190
// ADDRESS: 00532190
// PROTOTYPE: undefined Unwind@00532190()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//











// COMPONENT_VARIANT_END: WorldServer
