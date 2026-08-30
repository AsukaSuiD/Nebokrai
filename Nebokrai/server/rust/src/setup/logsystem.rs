//! Настройки `CLogSystem` из WorldServer/GameServer.
//! Контракт подтверждён точными `worldserver.exe + worldserver.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный владелец `setup/logsystem.cpp`.
//!
//! Протокол содержит сырую 64-байтную структуру `tagLogSystem`, знаковое число
//! и упорядоченные идентификаторы предметов. Парный декодер использует смещения
//! ABI, поэтому снимок остаётся массивом фиксированной длины с нулевым
//! статическим значением по умолчанию. `BTreeSet<i32>` сохраняет знаковый
//! порядок и уникальность. Байты улучшения `7/17/18` доступны вызывающему коду
//! аудита боевых фей; подтверждённый байт `56` немедленно передаётся в
//! `CDaKongXiangQian::SetLogKey`, остальные неподтверждённые смещения не
//! именуются. Покупка и продажа у NPC используют первые два подтверждённых
//! байта. Подъём и выбрасывание предметов в регионе используют байты `2/5`,
//! причём подъём сохраняет точный фильтр набора предметов.
//! Обмен камней и изготовление украшений используют соседние байты `3/4` и
//! публикуют соответственно `0x60204/0x60205`.
//! Помещение в склад и извлечение из него используют байты `9/10`, банк —
//! `11/12`. Уничтожение предметов, объединение снаряжения и жизненный цикл
//! повозки используют подтверждённые байты `55/57/58`; рост, инкубация,
//! имплантация и слияние фей — хвост `60..63` того же снимка. Аудит повышения
//! уровня игрока использует позиционный байт `24`, названный `bLevelLog`.
//! `CPlayer::ChangeRegion` читает соседние байты `51..53`, названные
//! `bChMap0/1/2`, для трёх путей аудита `0x6020C`.
//! GM-команды используют подтверждённый positional byte `50`, `bGMCommand`,
//! для межсерверного аудита `0x6020B`.

use crate::gameserver::appserver::legacycodec::LegacyReader;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

pub(crate) const LOG_SETTINGS_LENGTH: usize = 0x40;
const DA_KONG_LOG_OFFSET: usize = 56;

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
    pub(crate) fn goods_trade_log_enabled(&self) -> bool {
        self.setting(0)
    }

    pub(crate) fn goods_sell_to_npc_log_enabled(&self) -> bool {
        self.setting(1)
    }

    pub(crate) fn goods_get_from_region_log_enabled(&self) -> bool {
        self.setting(2)
    }

    pub(crate) fn goods_gem_exchange_log_enabled(&self) -> bool {
        self.setting(3)
    }

    pub(crate) fn goods_jewelry_made_log_enabled(&self) -> bool {
        self.setting(4)
    }

    pub(crate) fn goods_drop_to_region_log_enabled(&self) -> bool {
        self.setting(5)
    }

    pub(crate) fn goods_bank_set_log_enabled(&self) -> bool {
        self.setting(11)
    }

    pub(crate) fn goods_depot_set_log_enabled(&self) -> bool {
        self.setting(9)
    }

    pub(crate) fn goods_depot_get_log_enabled(&self) -> bool {
        self.setting(10)
    }

    pub(crate) fn goods_bank_get_log_enabled(&self) -> bool {
        self.setting(12)
    }

    pub(crate) fn is_log_item(&self, goods_id: i32) -> bool {
        self.items.contains(&goods_id)
    }

    pub(crate) fn da_kong_log_enabled(&self) -> bool {
        self.setting(DA_KONG_LOG_OFFSET)
    }

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

    pub(crate) fn increment_log_enabled(&self) -> bool {
        self.setting(54)
    }

    pub(crate) fn player_level_log_enabled(&self) -> bool {
        self.setting(24)
    }

    /// Positional `bTeamJion` (опечатка исходного имени сохранена в PDB).
    pub(crate) fn team_join_log_enabled(&self) -> bool {
        self.setting(28)
    }

    /// Positional `bPlayerKiller +30`: первый удар/skill по допустимой
    /// player-цели пишет `0x6020A` в World только при включённом аудите.
    pub(crate) fn player_killer_log_enabled(&self) -> bool {
        self.setting(30)
    }

    pub(crate) fn goods_lost_by_dead_enabled(&self) -> bool {
        self.setting(8)
    }

    pub(crate) fn experience_decrease_enabled(&self) -> bool {
        self.setting(22)
    }

    pub(crate) fn player_died_enabled(&self) -> bool {
        self.setting(27)
    }

    pub(crate) fn player_killed_enabled(&self) -> bool {
        self.setting(31)
    }

    pub(crate) fn goods_lost_by_upgrade_enabled(&self) -> bool {
        self.setting(7)
    }

    pub(crate) fn goods_upgrade_success_enabled(&self) -> bool {
        self.setting(17)
    }

    pub(crate) fn goods_upgrade_failure_enabled(&self) -> bool {
        self.setting(18)
    }

    pub(crate) fn equipment_compose_enabled(&self) -> bool {
        self.setting(57)
    }

    pub(crate) fn carriage_enabled(&self) -> bool {
        self.setting(58)
    }

    pub(crate) fn goods_destroy_enabled(&self) -> bool {
        self.setting(55)
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

    pub(crate) fn normal_chat_enabled(&self) -> bool {
        self.setting(44)
    }

    pub(crate) fn region_chat_enabled(&self) -> bool {
        self.setting(45)
    }

    pub(crate) fn private_chat_enabled(&self) -> bool {
        self.setting(49)
    }

    pub(crate) fn gm_command_enabled(&self) -> bool {
        self.setting(50)
    }

    pub(crate) fn change_region_log_enabled(&self, kind: u8) -> bool {
        matches!(kind, 0..=2) && self.setting(51 + usize::from(kind))
    }

    pub(crate) fn fairy_grow_enabled(&self) -> bool {
        self.setting(60)
    }

    pub(crate) fn fairy_incubate_enabled(&self) -> bool {
        self.setting(61)
    }

    pub(crate) fn fairy_implantation_enabled(&self) -> bool {
        self.setting(62)
    }

    pub(crate) fn fairy_syncretize_enabled(&self) -> bool {
        self.setting(63)
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

    /// Заменяет settings только после полного 64-byte read, затем
    /// очищает items и сохраняет полностью decoded prefix при обрыве.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), LogSystemDecodeError> {
        self.settings = LegacyReader::read_bytes_from(source, cursor, LOG_SETTINGS_LENGTH)
            .map_err(|block| LogSystemDecodeError {
                offset: block.offset,
                needed: block.needed,
                available: block.available,
            })?
            .try_into()
            .expect("LegacyReader вернул ровно 64 байта настроек");
        self.items.clear();
        let count = read_wire_i32(source, cursor)?;
        for _ in 0..count.max(0) {
            self.items.insert(read_wire_i32(source, cursor)?);
        }
        tracing::trace!(
            items = self.items.len(),
            da_kong_log = self.da_kong_log_enabled(),
            "настройки журналирования декодированы"
        );
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
pub(crate) struct LogSystemDecodeError {
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

impl fmt::Display for LogSystemDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "LogSystem snapshot обрывается на {}: нужно {}, доступно {}",
            self.offset, self.needed, self.available
        )
    }
}

impl Error for LogSystemDecodeError {}

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
                write!(
                    formatter,
                    "LogSystem, строка {line}: после '*' отсутствует имя предмета"
                )
            }
        }
    }
}

impl Error for LogSystemLoadError {}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, LogSystemDecodeError> {
    LegacyReader::read_i32_from(source, cursor).map_err(|block| LogSystemDecodeError {
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    })
}
