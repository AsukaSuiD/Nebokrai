//! Настройки журналирования исторического Miracle.
//!
//! Контракт World `CLogSystem::AddToByteArray`:
//!; text loader, accessors и Game decoder не входят в этот owner и остаются
//! Точная пара:
//! Исходный владелец PDB:
//!
//! Wire состоит из точного 64-байтного `tagLogSystem`, signed количества и
//! ordered `long`-ключей предметов. GameServer читает структуру целиком и
//! использует её поля по ABI offsets, поэтому fixed byte snapshot здесь
//! сохраняет подтверждённый контракт без выдуманной раскладки. Windows
//! `long` моделируется `i32`, а `BTreeSet` стандартной библиотеки заменяет
//! `std::set` и сохраняет его signed-порядок и уникальность. Static storage
//! оригинала было нулевым до loader-а; `Default` воспроизводит это без
//! C++ singleton/lifetime plumbing. Typed overlay полей будет уместен вместе
//! с действуетием loader-а; неизвестные offsets сейчас не именуются.

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

// Game decoder side effects, а не как Rust-реализация.
