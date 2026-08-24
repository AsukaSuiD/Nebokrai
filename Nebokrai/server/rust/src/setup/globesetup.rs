//! Общий глобальный setup `CGlobeSetup`, подтверждённый WorldServer и
//! GameServer EXE/PDB.
//!
//! Основной wire — raw 0x1114-байтный `tagSetup`, затем полный
//! `CRegionRouter`. Парный Game decoder копирует тот же snapshot, затем очищает
//! и восстанавливает router; фиксированный byte-array сохраняет ABI-формат,
//! static storage и padding обнулены.
//!
//! Typed loaders/accessors накладываются только на подтверждённые offsets:
//! create-role limit остаётся signed `i16`, country names/identities и special
//! string — fixed C-строки, auction/JJC/DbMisc поля читаются из общего snapshot.
//! BattleFairy и CiQing feature gates, а также полный ordinary-fairy setup
//! `+0x85C..+0x8A8` читаются из подтверждённых byte offsets.
//! Public-talk projection читает оба fixed goods-name, stack-count/money и
//! country/world interval прямо из тех же `+0x768..+0x848` полей. Абсолютные
//! reads `0xEF4528/68/6C/70/B0/B4` и `0xEF4604/08` в `OnOtherMessage`
//! подтверждают общий base `0xEF3DC0` и эти offsets по точному EXE.
//! `dwPkCountPerKill +0x4F4` обслуживает GameServer kill-confirmation path;
//! raw snapshot остаётся единым wire owner-ом без дублирующей config-модели.
//! `GetBaseMaxRp` сохраняет пороги только occupation 0, а auction formulas —
//! исходные `fSxfJinMax/fSxfJinMin/fAuctionFactorC`. Nation contender damage
//! читает подтверждённый `fDecTimeParam +0x568`, а death penalty — signed
//! `lDiedStateTime +0x56C` из того же snapshot.

use crate::setup::regionrouter::{
    RegionRouter, RegionRouterDecodeError, RegionRouterDecodeReport, RegionRouterSerializeError,
};

use std::error::Error;
use std::fmt;

pub(crate) const GLOBE_SETUP_BLOB_LENGTH: usize = 0x1114;
const COUNTRY_NAME_OFFSET: usize = 0x906;
const COUNTRY_NAME_SLOT_LENGTH: usize = 0x40;
const COUNTRY_NAME_COUNT: usize = 5;
const COUNTRY_IDENTITY_OFFSET: usize = 0xA46;
const COUNTRY_IDENTITY_COUNT: usize = 8;
const SPECIAL_STRING_OFFSET: usize = 0x520;
const SPECIAL_STRING_LENGTH: usize = 0x40;
const DELETION_DAYS_OFFSET: usize = 0x51C;
const TOTAL_JING_LI_DAN_COUNT_OFFSET: usize = 0x1110;
const MAXIMUM_CHARACTERS_OFFSET: usize = 0;
const BASE_RP_LEVEL_1_OFFSET: usize = 0x3F0;
const BASE_RP_LEVEL_2_OFFSET: usize = 0x3F2;
const BASE_MAX_RP_LEVEL_1_OFFSET: usize = 0x3F4;
const BASE_MAX_RP_LEVEL_2_OFFSET: usize = 0x3F6;
const PLAYER_SPEED_OFFSET: usize = 0x7F8;
const MONSTER_NUMBER_SCALE_OFFSET: usize = 0x508;
const SAVE_POINT_TIME_OFFSET: usize = 0x510;
const AUCTION_ENABLED_OFFSET: usize = 0xC87;
const AUCTION_FEE_MAXIMUM_OFFSET: usize = 0xC98;
const AUCTION_FEE_MINIMUM_OFFSET: usize = 0xCA0;
const AUCTION_FACTOR_C_OFFSET: usize = 0xCB0;
const AUCTION_OPEN_VALUE_OFFSETS: [usize; 12] = [
    0xC8C, 0xC90, 0xC94, 0xC98, 0xC9C, 0xCA0, 0xCA4, 0xCB0, 0xCB4, 0xCB8, 0xCBC, 0xCC0,
];
const JJC_ENABLED_OFFSET: usize = 0xCD4;
const JJC_REGION_MIN_OFFSET: usize = 0xCD8;
const JJC_REGION_MAX_OFFSET: usize = 0xCDC;
const JJC_MAX_REGIONS_IN_USE_OFFSET: usize = 0xCE0;
const JJC_PK_TIMEOUT_OFFSET: usize = 0xCE8;
const JJC_RANK_INTERVAL_OFFSET: usize = 0xCF0;
const TRANSFER_MONEY_INTERVAL_OFFSET: usize = 0x110C;
const GOODS_AI_OFFSET: usize = 0xC4C;
const PACK_ADD_OFFSET: usize = 0xC4E;
const DA_KONG_KEY_OFFSET: usize = 0xC85;
const AREA_WIDTH_OFFSET: usize = 0x514;
const AREA_HEIGHT_OFFSET: usize = 0x518;
const CONTEND_DAMAGE_TIME_FACTOR_OFFSET: usize = 0x568;
const DIED_STATE_TIME_OFFSET: usize = 0x56C;
const MAX_FETCH_POWER_OFFSET: usize = 0x900;
const BATTLE_FAIRY_ENABLED_OFFSET: usize = 0x904;
const CI_QING_ENABLED_OFFSET: usize = 0xD00;
const FAIRY_EGG_MAX_LEVEL_OFFSET: usize = 0x85C;
const FAIRY_HATCH_TIME_OFFSET: usize = 0x860;
const FAIRY_VIGOUR_CRYSTAL_SCALE_OFFSET: usize = 0x864;
const FAIRY_EXP_VIGOUR_SCALE_OFFSET: usize = 0x868;
const FAIRY_UPGRADE_RATE_OFFSET: usize = 0x870;
const FAIRY_SYNCRETIC_SUCCESS_RATE_OFFSET: usize = 0x874;
const FAIRY_SYNCRETIC_RATE_N_OFFSET: usize = 0x878;
const FAIRY_SYNCRETIC_RATE_Y_OFFSET: usize = 0x87C;
const FAIRY_SYNCRETIC_RATE_A_OFFSET: usize = 0x880;
const FAIRY_SYNCRETIC_NEEDED_GOODS_OFFSET: usize = 0x8A0;
const FAIRY_SYNCRETIC_NEEDED_EXP_OFFSET: usize = 0x8A4;
const FAIRY_SYNCRETIC_NEEDED_MONEY_OFFSET: usize = 0x8A8;
const PK_COUNT_PER_KILL_OFFSET: usize = 0x4F4;
const TALK_WORLD_GOODS_NAME_OFFSET: usize = 0x768;
const TALK_WORLD_GOODS_AMOUNT_OFFSET: usize = 0x7A8;
const TALK_WORLD_MONEY_OFFSET: usize = 0x7AC;
const TALK_COUNTRY_GOODS_NAME_OFFSET: usize = 0x7B0;
const TALK_COUNTRY_GOODS_AMOUNT_OFFSET: usize = 0x7F0;
const TALK_COUNTRY_MONEY_OFFSET: usize = 0x7F4;
const COUNTRY_TALK_INTERVAL_OFFSET: usize = 0x844;
const WORLD_TALK_INTERVAL_OFFSET: usize = 0x848;
const NORMAL_TALK_INTERVAL_OFFSET: usize = 0x83C;
const AREA_TALK_INTERVAL_OFFSET: usize = 0x840;
const PRIVATE_TALK_INTERVAL_OFFSET: usize = 0x84C;
const UNION_TALK_INTERVAL_OFFSET: usize = 0x858;
const REGION_CHAT_LEVEL_LIMIT_OFFSET: usize = 0x560;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GlobeSetupSnapshot {
    bytes: [u8; GLOBE_SETUP_BLOB_LENGTH],
    country_name_ids: [Vec<u8>; COUNTRY_NAME_COUNT],
    country_identity_ids: [Vec<u8>; COUNTRY_IDENTITY_COUNT],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlobePlayerPropertyCoefficients {
    pub(crate) str_to_max_attack: [f32; 3],
    pub(crate) str_to_burden: [f32; 3],
    pub(crate) dex_to_min_attack: [f32; 3],
    pub(crate) dex_to_stiff: [f32; 3],
    pub(crate) con_to_max_hp: [f32; 3],
    pub(crate) con_to_defense: [f32; 3],
    pub(crate) int_to_element: [f32; 3],
    pub(crate) int_to_max_mp: [f32; 3],
    pub(crate) int_to_resistant: [f32; 3],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GlobeSetupDecodeReport {
    pub(crate) router: RegionRouterDecodeReport,
    pub(crate) area_width: i32,
    pub(crate) area_height: i32,
    pub(crate) da_kong_key: bool,
    pub(crate) goods_ai_enabled: bool,
    pub(crate) auction_enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GlobeSetupDecodeError {
    Snapshot {
        offset: usize,
        needed: usize,
        available: usize,
    },
    RegionRouter(RegionRouterDecodeError),
}

impl fmt::Display for GlobeSetupDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "GlobeSetup snapshot обрывается на {offset}: нужно {needed}, доступно {available}"
            ),
            Self::RegionRouter(error) => error.fmt(formatter),
        }
    }
}

impl Error for GlobeSetupDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Snapshot { .. } => None,
            Self::RegionRouter(error) => Some(error),
        }
    }
}

impl Default for GlobeSetupSnapshot {
    fn default() -> Self {
        Self {
            bytes: [0; GLOBE_SETUP_BLOB_LENGTH],
            country_name_ids: std::array::from_fn(|_| Vec::new()),
            country_identity_ids: std::array::from_fn(|_| Vec::new()),
        }
    }
}

impl GlobeSetupSnapshot {
    pub(crate) fn from_bytes(bytes: [u8; GLOBE_SETUP_BLOB_LENGTH]) -> Self {
        Self {
            bytes,
            country_name_ids: std::array::from_fn(|_| Vec::new()),
            country_identity_ids: std::array::from_fn(|_| Vec::new()),
        }
    }

    pub(crate) fn bytes_mut(&mut self) -> &mut [u8; GLOBE_SETUP_BLOB_LENGTH] {
        &mut self.bytes
    }

    /// Восстанавливает Game-side wire projection. Полный blob публикуется до
    /// начала router decode; при обрыве router сохраняет уже прочитанный prefix.
    /// Последующие DaKong/message/auction effects выполняет `CGame` caller.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        router: &mut RegionRouter,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<GlobeSetupDecodeReport, GlobeSetupDecodeError> {
        let offset = *cursor;
        let available = source.len().saturating_sub(offset);
        let Some(bytes) = source.get(offset..offset.saturating_add(GLOBE_SETUP_BLOB_LENGTH)) else {
            return Err(GlobeSetupDecodeError::Snapshot {
                offset,
                needed: GLOBE_SETUP_BLOB_LENGTH,
                available,
            });
        };
        self.bytes.copy_from_slice(bytes);
        *cursor += GLOBE_SETUP_BLOB_LENGTH;

        let router = router
            .decord_from_byte_array(source, cursor)
            .map_err(GlobeSetupDecodeError::RegionRouter)?;
        Ok(GlobeSetupDecodeReport {
            router,
            area_width: self.area_width(),
            area_height: self.area_height(),
            da_kong_key: self.da_kong_key(),
            goods_ai_enabled: self.goods_ai_enabled(),
            auction_enabled: self.auction_enabled(),
        })
    }

    /// Читает positional `setup/globesetup.ini` в точные PDB-offsets.
    /// Активный snapshot меняется только после полного успешного разбора.
    pub(crate) fn load_globe_setup(
        &mut self,
        source: &[u8],
    ) -> Result<GlobeSetupLoadReport, GlobeSetupLoadError> {
        let records = read_records(source);
        let specs = globe_record_specs();
        if records.len() != 549 {
            return Err(GlobeSetupLoadError::RecordCount {
                expected_minimum: 549,
                expected_maximum: 549,
                actual: records.len(),
            });
        }
        debug_assert_eq!(specs.len(), 549);
        debug_assert_eq!(specs.iter().map(Vec::len).sum::<usize>(), 571);

        let mut candidate = Self::default();
        apply_records(&records, &specs, &mut candidate)?;
        *self = candidate;
        Ok(GlobeSetupLoadReport {
            records: records.len(),
        })
    }

    /// Накладывает `setup/gamesetup.ini` поверх уже загруженного Globe blob.
    /// Поставочный RU-файл содержит 47 записей; EXE допускает 48-ю запись
    /// `lTransferMoneyTime`, оставляя ноль при её отсутствии.
    pub(crate) fn load_game_setup(
        &mut self,
        source: &[u8],
    ) -> Result<GlobeSetupLoadReport, GlobeSetupLoadError> {
        let records = read_records(source);
        let specs = game_record_specs();
        if !(47..=48).contains(&records.len()) {
            return Err(GlobeSetupLoadError::RecordCount {
                expected_minimum: 47,
                expected_maximum: 48,
                actual: records.len(),
            });
        }
        debug_assert_eq!(specs.len(), 48);

        let mut candidate = self.clone();
        apply_records(&records, &specs, &mut candidate)?;
        *self = candidate;
        Ok(GlobeSetupLoadReport {
            records: records.len(),
        })
    }

    pub(crate) fn load_auction_goods(
        &mut self,
        source: &[u8],
    ) -> Result<GlobeSetupLoadReport, GlobeSetupLoadError> {
        const OFFSET: usize = 3336;
        const CAPACITY: usize = 256;
        let records = read_records(source);
        if records.len() > CAPACITY {
            return Err(GlobeSetupLoadError::RecordCount {
                expected_minimum: 0,
                expected_maximum: CAPACITY,
                actual: records.len(),
            });
        }
        let mut candidate = self.clone();
        candidate.bytes[OFFSET..OFFSET + CAPACITY * 4].fill(0);
        for (index, record) in records.iter().enumerate() {
            if record.label != b"#" || record.values.len() != 1 {
                return Err(GlobeSetupLoadError::Record {
                    line: record.line,
                    label: record.label.to_vec(),
                    reason: "ожидалась запись '# <signed 32-bit goods id>'",
                });
            }
            let value = parse_number::<i32>(&record.values[0]).ok_or_else(|| {
                GlobeSetupLoadError::Record {
                    line: record.line,
                    label: record.label.to_vec(),
                    reason: "goods id не является signed 32-bit числом",
                }
            })?;
            candidate.bytes[OFFSET + index * 4..OFFSET + index * 4 + 4]
                .copy_from_slice(&value.to_le_bytes());
        }
        *self = candidate;
        Ok(GlobeSetupLoadReport {
            records: records.len(),
        })
    }

    pub(crate) fn resolve_country_text(
        &mut self,
        mut resolve: impl FnMut(&[u8]) -> Option<Vec<u8>>,
    ) -> Result<(), GlobeSetupLoadError> {
        let mut candidate = self.clone();
        for index in 0..COUNTRY_NAME_COUNT {
            let value = resolve(&candidate.country_name_ids[index]).unwrap_or_default();
            write_fixed_string(
                &mut candidate.bytes,
                COUNTRY_NAME_OFFSET + index * COUNTRY_NAME_SLOT_LENGTH,
                &value,
            )
            .map_err(|reason| GlobeSetupLoadError::Record {
                line: 0,
                label: candidate.country_name_ids[index].clone(),
                reason,
            })?;
        }
        for index in 0..COUNTRY_IDENTITY_COUNT {
            let value = resolve(&candidate.country_identity_ids[index]).unwrap_or_default();
            write_fixed_string(
                &mut candidate.bytes,
                COUNTRY_IDENTITY_OFFSET + index * COUNTRY_NAME_SLOT_LENGTH,
                &value,
            )
            .map_err(|reason| GlobeSetupLoadError::Record {
                line: 0,
                label: candidate.country_identity_ids[index].clone(),
                reason,
            })?;
        }
        *self = candidate;
        Ok(())
    }

    pub(crate) fn maximum_characters(&self) -> i16 {
        i16::from_le_bytes(
            self.bytes[MAXIMUM_CHARACTERS_OFFSET..MAXIMUM_CHARACTERS_OFFSET + 2]
                .try_into()
                .expect("PDB-offset находится внутри globe snapshot"),
        )
    }

    pub(crate) fn player_speed(&self) -> f32 {
        self.read_f32(PLAYER_SPEED_OFFSET)
    }

    pub(crate) fn area_width(&self) -> i32 {
        self.read_i32(AREA_WIDTH_OFFSET)
    }

    pub(crate) fn area_height(&self) -> i32 {
        self.read_i32(AREA_HEIGHT_OFFSET)
    }

    pub(crate) fn contend_damage_time_factor(&self) -> f32 {
        self.read_f32(CONTEND_DAMAGE_TIME_FACTOR_OFFSET)
    }

    pub(crate) fn died_state_time_seconds(&self) -> i32 {
        self.read_i32(DIED_STATE_TIME_OFFSET)
    }

    /// `lMaxFetchPower +0x900`: `CPlayer::SetFetchPower` сравнивает его
    /// после unsigned cast, поэтому отрицательное значение не нормализуется.
    pub(crate) fn maximum_fetch_power(&self) -> i32 {
        self.read_i32(MAX_FETCH_POWER_OFFSET)
    }

    /// `bBattleFairy +0x904` — общий gate для combine и связанных skill
    /// сообщений; это не persisted `CPlayer::bBattleFairyEnabled`.
    pub(crate) const fn battle_fairy_enabled(&self) -> bool {
        self.bytes[BATTLE_FAIRY_ENABLED_OFFSET] != 0
    }

    /// `bCiQing +0xD00` — общий gate клиентских операций татуировок.
    pub(crate) const fn ci_qing_enabled(&self) -> bool {
        self.bytes[CI_QING_ENABLED_OFFSET] != 0
    }

    pub(crate) fn fairy_egg_max_level(&self) -> u32 {
        self.read_u32(FAIRY_EGG_MAX_LEVEL_OFFSET)
    }

    pub(crate) fn fairy_hatch_time(&self) -> u32 {
        self.read_u32(FAIRY_HATCH_TIME_OFFSET)
    }

    pub(crate) fn fairy_vigour_crystal_scale(&self) -> f32 {
        self.read_f32(FAIRY_VIGOUR_CRYSTAL_SCALE_OFFSET)
    }

    pub(crate) fn fairy_exp_vigour_scale(&self) -> f32 {
        self.read_f32(FAIRY_EXP_VIGOUR_SCALE_OFFSET)
    }

    pub(crate) fn fairy_upgrade_rate(&self) -> f32 {
        self.read_f32(FAIRY_UPGRADE_RATE_OFFSET)
    }

    pub(crate) fn fairy_syncretic_success_rate(&self) -> f32 {
        self.read_f32(FAIRY_SYNCRETIC_SUCCESS_RATE_OFFSET)
    }

    pub(crate) fn fairy_syncretic_rate(&self, index: usize) -> f32 {
        self.read_f32(FAIRY_SYNCRETIC_RATE_A_OFFSET + index * 4)
    }

    pub(crate) fn fairy_syncretic_rate_n(&self) -> f32 {
        self.read_f32(FAIRY_SYNCRETIC_RATE_N_OFFSET)
    }

    pub(crate) fn fairy_syncretic_rate_y(&self) -> f32 {
        self.read_f32(FAIRY_SYNCRETIC_RATE_Y_OFFSET)
    }

    pub(crate) fn fairy_syncretic_needed_goods(&self) -> u32 {
        self.read_u32(FAIRY_SYNCRETIC_NEEDED_GOODS_OFFSET)
    }

    pub(crate) fn fairy_syncretic_needed_experience(&self) -> u32 {
        self.read_u32(FAIRY_SYNCRETIC_NEEDED_EXP_OFFSET)
    }

    pub(crate) fn fairy_syncretic_needed_money(&self) -> u32 {
        self.read_u32(FAIRY_SYNCRETIC_NEEDED_MONEY_OFFSET)
    }

    pub(crate) const fn da_kong_key(&self) -> bool {
        self.bytes[DA_KONG_KEY_OFFSET] != 0
    }

    pub(crate) const fn goods_ai_enabled(&self) -> bool {
        self.bytes[GOODS_AI_OFFSET] != 0
    }

    /// `bPackAdd +0xC4E` соседствует с подтверждёнными `bGoodsAi/bToAdd`;
    /// container/equipment paths читают его до capacity и package effects.
    pub(crate) const fn pack_add_enabled(&self) -> bool {
        self.bytes[PACK_ADD_OFFSET] != 0
    }

    pub(crate) fn monster_number_scale(&self) -> f32 {
        self.read_f32(MONSTER_NUMBER_SCALE_OFFSET)
    }

    /// Возвращает точное поле `dwSavePointTime` по PDB-смещению `+0x510`.
    /// `CGame::MainLoop` читает это значение при interval-gate сохранения.
    pub(crate) fn save_point_time_ms(&self) -> u32 {
        self.read_u32(SAVE_POINT_TIME_OFFSET)
    }

    pub(crate) fn gold_coin_limit(&self) -> u32 {
        self.read_u32(0x4fc)
    }

    /// Exact `dwPkCountPerKill` по подтверждённому ABI offset `+0x4F4`.
    pub(crate) fn pk_count_per_kill(&self) -> u32 {
        self.read_u32(PK_COUNT_PER_KILL_OFFSET)
    }

    pub(crate) fn increment_log_days(&self) -> u32 {
        self.read_u32(0x80c)
    }

    pub(crate) fn use_appellation_function(&self) -> bool {
        self.bytes[0xcd1] != 0
    }

    /// Возвращает process-настройки, которые `CJJcSystem::Run` читает из
    /// загруженного `gamesetup.ini`. Неиспользуемые этим owner-ом соседние
    /// `DefaultJJcLevel`, queue interval и buff ID в проекцию не входят.
    pub(crate) fn jjc_run_config(&self) -> crate::worldserver::appworld::jjcsystem::JjcRunConfig {
        crate::worldserver::appworld::jjcsystem::JjcRunConfig {
            use_jjc: self.read_i32(JJC_ENABLED_OFFSET),
            rank_interval_seconds: self.read_i32(JJC_RANK_INTERVAL_OFFSET),
            pk_timeout_seconds: self.read_i32(JJC_PK_TIMEOUT_OFFSET),
            region_id_min: self.read_i32(JJC_REGION_MIN_OFFSET),
            region_id_max: self.read_i32(JJC_REGION_MAX_OFFSET),
            max_regions_in_use: self.read_i32(JJC_MAX_REGIONS_IN_USE_OFFSET),
        }
    }

    /// Возвращает `lTransferMoneyTime` для reconnect-gate `CDbMisc`.
    /// Поставочный файл может не содержать последнюю запись; zero-filled
    /// snapshot тогда сохраняет исходный нулевой интервал.
    pub(crate) fn transfer_money_interval_ms(&self) -> i32 {
        self.read_i32(TRANSFER_MONEY_INTERVAL_OFFSET)
    }

    pub(crate) fn player_property_coefficients(&self) -> GlobePlayerPropertyCoefficients {
        let triplet = |offset| std::array::from_fn(|index| self.read_f32(offset + index * 4));
        GlobePlayerPropertyCoefficients {
            str_to_max_attack: triplet(4),
            str_to_burden: triplet(16),
            dex_to_min_attack: triplet(28),
            dex_to_stiff: triplet(40),
            con_to_max_hp: triplet(52),
            con_to_defense: triplet(64),
            int_to_element: triplet(76),
            int_to_max_mp: triplet(88),
            int_to_resistant: triplet(100),
        }
    }

    pub(crate) const fn auction_enabled(&self) -> bool {
        self.bytes[AUCTION_ENABLED_OFFSET] != 0
    }

    pub(crate) fn auction_fee_maximum(&self) -> f32 {
        self.read_f32(AUCTION_FEE_MAXIMUM_OFFSET)
    }

    pub(crate) fn auction_fee_minimum(&self) -> f32 {
        self.read_f32(AUCTION_FEE_MINIMUM_OFFSET)
    }

    pub(crate) fn auction_factor_c(&self) -> f32 {
        self.read_f32(AUCTION_FACTOR_C_OFFSET)
    }

    /// Exact `CPlayer::OpenAuction` projection: float-поля передаются клиенту
    /// после legacy truncation к `ulong` и в исходном ABI-порядке.
    pub(crate) fn auction_open_values(&self) -> [u32; 12] {
        AUCTION_OPEN_VALUE_OFFSETS.map(|offset| self.read_f32(offset) as u32)
    }

    pub(crate) fn base_max_rp(&self, occupation: u8, level: u8) -> u16 {
        if occupation != 0 {
            return 0;
        }
        let read_u16 = |offset| {
            u16::from_le_bytes(
                self.bytes[offset..offset + 2]
                    .try_into()
                    .expect("PDB-offset находится внутри globe snapshot"),
            )
        };
        let level = u16::from(level);
        let level_1 = read_u16(BASE_RP_LEVEL_1_OFFSET);
        let level_2 = read_u16(BASE_RP_LEVEL_2_OFFSET);
        if level < level_1 {
            if level < level_2 {
                return 0;
            }
        } else if level < level_2 {
            return read_u16(BASE_MAX_RP_LEVEL_1_OFFSET);
        }
        read_u16(BASE_MAX_RP_LEVEL_2_OFFSET)
    }

    pub(crate) fn country_name(&self, country_id: u8) -> Option<&[u8]> {
        let index = usize::from(country_id);
        if index >= COUNTRY_NAME_COUNT {
            return None;
        }
        let start = COUNTRY_NAME_OFFSET + index * COUNTRY_NAME_SLOT_LENGTH;
        let slot = &self.bytes[start..start + COUNTRY_NAME_SLOT_LENGTH];
        let visible_len = slot
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(slot.len());
        Some(&slot[..visible_len])
    }

    pub(crate) fn country_identity_name(&self, identity: u8) -> Option<&[u8]> {
        let index = usize::from(identity);
        if index >= COUNTRY_IDENTITY_COUNT {
            return None;
        }
        let start = COUNTRY_IDENTITY_OFFSET + index * COUNTRY_NAME_SLOT_LENGTH;
        let slot = &self.bytes[start..start + COUNTRY_NAME_SLOT_LENGTH];
        let visible_len = slot
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(slot.len());
        Some(&slot[..visible_len])
    }

    pub(crate) fn special_string(&self) -> &[u8] {
        let slot =
            &self.bytes[SPECIAL_STRING_OFFSET..SPECIAL_STRING_OFFSET + SPECIAL_STRING_LENGTH];
        let visible_len = slot
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(slot.len());
        &slot[..visible_len]
    }

    pub(crate) fn public_talk_goods_name(&self, country: bool) -> &[u8] {
        let start = if country {
            TALK_COUNTRY_GOODS_NAME_OFFSET
        } else {
            TALK_WORLD_GOODS_NAME_OFFSET
        };
        let slot = &self.bytes[start..start + 0x40];
        let visible_len = slot
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(slot.len());
        &slot[..visible_len]
    }

    pub(crate) fn public_talk_goods_amount(&self, country: bool) -> i32 {
        self.read_i32(if country {
            TALK_COUNTRY_GOODS_AMOUNT_OFFSET
        } else {
            TALK_WORLD_GOODS_AMOUNT_OFFSET
        })
    }

    pub(crate) fn public_talk_money(&self, country: bool) -> u32 {
        self.read_u32(if country {
            TALK_COUNTRY_MONEY_OFFSET
        } else {
            TALK_WORLD_MONEY_OFFSET
        })
    }

    pub(crate) fn public_talk_interval_ms(&self, country: bool) -> u32 {
        self.read_u32(if country {
            COUNTRY_TALK_INTERVAL_OFFSET
        } else {
            WORLD_TALK_INTERVAL_OFFSET
        })
    }

    pub(crate) fn normal_talk_interval_ms(&self) -> u32 {
        self.read_u32(NORMAL_TALK_INTERVAL_OFFSET)
    }

    pub(crate) fn area_talk_interval_ms(&self) -> u32 {
        self.read_u32(AREA_TALK_INTERVAL_OFFSET)
    }

    pub(crate) fn private_talk_interval_ms(&self) -> u32 {
        self.read_u32(PRIVATE_TALK_INTERVAL_OFFSET)
    }

    pub(crate) fn union_talk_interval_ms(&self) -> u32 {
        self.read_u32(UNION_TALK_INTERVAL_OFFSET)
    }

    pub(crate) fn region_chat_level_limit(&self) -> i32 {
        self.read_i32(REGION_CHAT_LEVEL_LIMIT_OFFSET)
    }

    pub(crate) fn deletion_days(&self) -> u32 {
        u32::from_le_bytes(
            self.bytes[DELETION_DAYS_OFFSET..DELETION_DAYS_OFFSET + 4]
                .try_into()
                .expect("PDB-offset находится внутри globe snapshot"),
        )
    }

    pub(crate) fn total_jing_li_dan_count(&self) -> u16 {
        u16::from_le_bytes(
            self.bytes[TOTAL_JING_LI_DAN_COUNT_OFFSET..TOTAL_JING_LI_DAN_COUNT_OFFSET + 2]
                .try_into()
                .expect("PDB-offset находится внутри globe snapshot"),
        )
    }

    pub(crate) fn reset_total_jing_li_dan_count(&mut self) {
        self.bytes[TOTAL_JING_LI_DAN_COUNT_OFFSET..TOTAL_JING_LI_DAN_COUNT_OFFSET + 2].fill(0);
    }

    fn read_f32(&self, offset: usize) -> f32 {
        f32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("PDB-offset находится внутри globe snapshot"),
        )
    }

    fn read_i32(&self, offset: usize) -> i32 {
        i32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("PDB-offset находится внутри globe snapshot"),
        )
    }

    fn read_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("PDB-offset находится внутри globe snapshot"),
        )
    }

    pub(crate) fn add_to_byte_array(
        &self,
        router: &RegionRouter,
        destination: &mut Vec<u8>,
    ) -> Result<(), RegionRouterSerializeError> {
        destination.extend_from_slice(&self.bytes);
        router.add_to_byte_array(destination)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GlobeSetupLoadReport {
    pub(crate) records: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GlobeSetupLoadError {
    RecordCount {
        expected_minimum: usize,
        expected_maximum: usize,
        actual: usize,
    },
    Record {
        line: usize,
        label: Vec<u8>,
        reason: &'static str,
    },
}

impl fmt::Display for GlobeSetupLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RecordCount {
                expected_minimum,
                expected_maximum,
                actual,
            } if expected_minimum == expected_maximum => write!(
                formatter,
                "ожидалось {expected_minimum} записей GlobeSetup, найдено {actual}"
            ),
            Self::RecordCount {
                expected_minimum,
                expected_maximum,
                actual,
            } => write!(
                formatter,
                "ожидалось от {expected_minimum} до {expected_maximum} записей GlobeSetup, найдено {actual}"
            ),
            Self::Record {
                line,
                label,
                reason,
            } => write!(
                formatter,
                "GlobeSetup, строка {line} ('{}'): {reason}",
                String::from_utf8_lossy(label)
            ),
        }
    }
}

impl Error for GlobeSetupLoadError {}

#[derive(Clone, Copy)]
enum GlobeFieldKind {
    Ignore,
    U8,
    Bool,
    U16,
    I32,
    U32,
    F32,
    Char64,
    CountryName,
    CountryIdentity,
}

#[derive(Clone, Copy)]
struct GlobeField {
    offset: usize,
    kind: GlobeFieldKind,
}

struct GlobeRecord<'a> {
    line: usize,
    label: &'a [u8],
    values: Vec<&'a [u8]>,
}

fn field(offset: usize, kind: GlobeFieldKind) -> GlobeField {
    GlobeField { offset, kind }
}

fn add_one(records: &mut Vec<Vec<GlobeField>>, offset: usize, kind: GlobeFieldKind) {
    records.push(vec![field(offset, kind)]);
}

fn add_array(
    records: &mut Vec<Vec<GlobeField>>,
    offset: usize,
    kind: GlobeFieldKind,
    count: usize,
    stride: usize,
) {
    for index in 0..count {
        add_one(records, offset + index * stride, kind);
    }
}

fn globe_record_specs() -> Vec<Vec<GlobeField>> {
    use GlobeFieldKind as K;
    let mut records = Vec::with_capacity(549);
    add_one(&mut records, 0, K::U8);
    for offset in [4, 16, 28, 40, 52, 64, 88, 76, 100] {
        add_array(&mut records, offset, K::F32, 3, 4);
    }
    add_one(&mut records, 112, K::I32);
    add_one(&mut records, 116, K::F32);
    add_one(&mut records, 120, K::F32);
    add_one(&mut records, 124, K::I32);
    add_one(&mut records, 128, K::I32);
    add_array(&mut records, 132, K::I32, 3, 4);
    add_array(&mut records, 144, K::I32, 3, 4);
    add_array(&mut records, 156, K::F32, 4, 4);

    // Два назначения offset 396 и отсутствующий 268 повторяют extraction-chain EXE.
    let drop_bases = [
        172, 204, 236, 396, 300, 332, 364, 396, 428, 460, 492, 524, 556, 588, 620, 652, 684, 716,
    ];
    for row in 0..2 {
        for column in 0..4 {
            for offset in drop_bases {
                add_one(&mut records, offset + (row * 4 + column) * 4, K::F32);
            }
        }
    }
    add_array(&mut records, 748, K::F32, 3, 4);
    add_array(&mut records, 760, K::I32, 15, 4);
    add_array(&mut records, 820, K::U16, 3, 2);
    add_array(&mut records, 828, K::F32, 4, 4);
    add_one(&mut records, 844, K::U32);
    add_one(&mut records, 848, K::U32);
    add_one(&mut records, 852, K::I32);
    add_one(&mut records, 856, K::I32);
    add_one(&mut records, 860, K::F32);
    add_one(&mut records, 864, K::I32);
    add_one(&mut records, 868, K::I32);
    add_one(&mut records, 872, K::I32);
    add_one(&mut records, 876, K::U32);
    for index in 0..3 {
        add_one(&mut records, 880 + index * 4, K::I32);
        add_array(&mut records, 892 + index * 16, K::I32, 4, 4);
        add_one(&mut records, 940 + index * 4, K::I32);
    }
    add_array(&mut records, 952, K::F32, 8, 4);
    add_one(&mut records, 984, K::F32);
    add_one(&mut records, 988, K::F32);
    add_one(&mut records, 992, K::I32);
    add_one(&mut records, 996, K::F32);
    add_one(&mut records, 1000, K::F32);
    add_one(&mut records, 1004, K::I32);
    add_array(&mut records, 1008, K::U16, 5, 2);
    for index in 0..6 {
        records.push(vec![
            field(1020 + index * 4, K::F32),
            field(1044 + index * 2, K::U16),
        ]);
    }
    add_one(&mut records, 1056, K::I32);
    add_one(&mut records, 1060, K::I32);
    add_one(&mut records, 1064, K::F32);
    add_one(&mut records, 1068, K::F32);
    for index in 0..12 {
        records.push(vec![
            field(1072 + index * 4, K::U32),
            field(1120 + index * 4, K::U32),
        ]);
    }
    add_array(&mut records, 1168, K::I32, 3, 4);
    add_array(&mut records, 1180, K::I32, 3, 4);
    add_array(&mut records, 1192, K::I32, 3, 4);
    add_array(&mut records, 0, K::Ignore, 3, 0);
    add_one(&mut records, 1216, K::U16);
    for index in 0..4 {
        records.push(vec![
            field(1220 + index * 4, K::F32),
            field(1236 + index * 2, K::U16),
        ]);
    }
    add_array(&mut records, 1244, K::U32, 11, 4);
    add_one(&mut records, 1288, K::F32);
    add_array(&mut records, 0, K::Ignore, 3, 0);
    add_one(&mut records, 1300, K::I32);
    add_one(&mut records, 1304, K::I32);
    add_array(&mut records, 0, K::Ignore, 2, 0);
    add_one(&mut records, 1376, K::I32);
    add_one(&mut records, 1380, K::U32);
    add_one(&mut records, 1384, K::F32);
    add_one(&mut records, 1388, K::I32);
    for offset in [
        1392, 1432, 1472, 1512, 1552, 1592, 1632, 1672, 1712, 1752, 1792,
    ] {
        add_array(&mut records, offset, K::F32, 10, 4);
    }
    add_one(&mut records, 1832, K::U32);
    add_one(&mut records, 1836, K::U32);
    add_one(&mut records, 1840, K::F32);
    add_one(&mut records, 1844, K::U32);
    add_one(&mut records, 1848, K::U32);
    add_array(&mut records, 1852, K::F32, 8, 4);
    add_array(&mut records, 1884, K::U32, 3, 4);
    add_one(&mut records, 1896, K::Char64);
    add_one(&mut records, 1960, K::I32);
    add_one(&mut records, 1964, K::U32);
    add_one(&mut records, 1968, K::Char64);
    add_one(&mut records, 2032, K::I32);
    add_one(&mut records, 2036, K::U32);
    add_one(&mut records, 2040, K::F32);
    add_one(&mut records, 2044, K::I32);
    add_one(&mut records, 2048, K::I32);
    add_array(&mut records, 2052, K::U32, 7, 4);
    add_one(&mut records, 2080, K::F32);
    add_one(&mut records, 2084, K::F32);
    add_one(&mut records, 2088, K::U32);
    add_one(&mut records, 2140, K::U32);
    add_one(&mut records, 2144, K::U32);
    add_array(&mut records, 2148, K::F32, 5, 4);
    add_array(&mut records, 2168, K::F32, 10, 4);
    add_array(&mut records, 2208, K::U32, 3, 4);
    add_array(&mut records, 2220, K::F32, 4, 4);
    add_array(&mut records, 3152, K::I32, 4, 4);
    add_array(&mut records, 3168, K::U32, 4, 4);
    for index in 0..COUNTRY_NAME_COUNT {
        add_one(&mut records, index, K::CountryName);
    }
    for index in 0..COUNTRY_IDENTITY_COUNT {
        add_one(&mut records, index, K::CountryIdentity);
    }
    add_array(&mut records, 2236, K::F32, 17, 4);
    add_one(&mut records, 2304, K::I32);
    add_one(&mut records, 3204, K::Bool);
    add_array(&mut records, 3208, K::F32, 15, 4);
    add_one(&mut records, 3268, K::I32);
    add_one(&mut records, 3276, K::I32);
    add_one(&mut records, 3272, K::I32);
    add_one(&mut records, 3332, K::I32);
    add_one(&mut records, 4361, K::Bool);
    records
}

fn game_record_specs() -> Vec<Vec<GlobeField>> {
    use GlobeFieldKind as K;
    let mut records = Vec::with_capacity(48);
    add_array(&mut records, 1204, K::F32, 3, 4);
    add_one(&mut records, 1292, K::Bool);
    add_one(&mut records, 1293, K::Bool);
    add_one(&mut records, 1296, K::U32);
    add_one(&mut records, 1308, K::U32);
    add_one(&mut records, 1312, K::Char64);
    add_array(&mut records, 2092, K::I32, 4, 4);
    add_array(&mut records, 2108, K::U32, 8, 4);
    add_one(&mut records, 2309, K::Bool);
    add_one(&mut records, 3148, K::Bool);
    add_one(&mut records, 3149, K::Bool);
    add_one(&mut records, 3150, K::Bool);
    add_array(&mut records, 3188, K::I32, 4, 4);
    add_one(&mut records, 2308, K::Bool);
    add_one(&mut records, 3205, K::Bool);
    add_one(&mut records, 3207, K::Bool);
    add_one(&mut records, 3281, K::Bool);
    add_one(&mut records, 3280, K::Bool);
    add_one(&mut records, 3206, K::Bool);
    add_one(&mut records, 3284, K::I32);
    add_array(&mut records, 3288, K::I32, 8, 4);
    add_one(&mut records, 3320, K::Bool);
    add_one(&mut records, 3324, K::I32);
    add_one(&mut records, 3328, K::Bool);
    add_one(&mut records, 4360, K::Bool);
    add_one(&mut records, 4364, K::I32);
    records
}

fn read_records(source: &[u8]) -> Vec<GlobeRecord<'_>> {
    source
        .split(|byte| *byte == b'\n')
        .enumerate()
        .filter_map(|(index, line)| {
            let mut tokens = line
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty());
            let label = tokens.next()?;
            if label.starts_with(b"//") {
                return None;
            }
            Some(GlobeRecord {
                line: index + 1,
                label,
                values: tokens.collect(),
            })
        })
        .collect()
}

fn apply_records(
    records: &[GlobeRecord<'_>],
    specs: &[Vec<GlobeField>],
    candidate: &mut GlobeSetupSnapshot,
) -> Result<(), GlobeSetupLoadError> {
    for (record, spec) in records.iter().zip(specs) {
        if record.values.len() != spec.len() {
            return Err(GlobeSetupLoadError::Record {
                line: record.line,
                label: record.label.to_vec(),
                reason: "число значений не совпадает с positional-контрактом EXE",
            });
        }
        for (&value, &field) in record.values.iter().zip(spec) {
            apply_field(candidate, field, value).map_err(|reason| GlobeSetupLoadError::Record {
                line: record.line,
                label: record.label.to_vec(),
                reason,
            })?;
        }
    }
    Ok(())
}

fn apply_field(
    candidate: &mut GlobeSetupSnapshot,
    field: GlobeField,
    value: &[u8],
) -> Result<(), &'static str> {
    use GlobeFieldKind as K;
    match field.kind {
        K::Ignore => Ok(()),
        K::CountryName => {
            candidate.country_name_ids[field.offset] = value.to_vec();
            Ok(())
        }
        K::CountryIdentity => {
            candidate.country_identity_ids[field.offset] = value.to_vec();
            Ok(())
        }
        K::Char64 => write_fixed_string(&mut candidate.bytes, field.offset, value),
        K::U8 => write_number::<u8>(
            &mut candidate.bytes,
            field.offset,
            value,
            "ожидалось unsigned 8-bit число",
        ),
        K::Bool => {
            if !matches!(value, b"0" | b"1") {
                return Err("ожидалось boolean 0 или 1");
            }
            write_number::<u8>(
                &mut candidate.bytes,
                field.offset,
                value,
                "ожидалось boolean 0 или 1",
            )
        }
        K::U16 => write_number::<u16>(
            &mut candidate.bytes,
            field.offset,
            value,
            "ожидалось unsigned 16-bit число",
        ),
        K::I32 => write_number::<i32>(
            &mut candidate.bytes,
            field.offset,
            value,
            "ожидалось signed 32-bit число",
        ),
        K::U32 => write_number::<u32>(
            &mut candidate.bytes,
            field.offset,
            value,
            "ожидалось unsigned 32-bit число",
        ),
        K::F32 => {
            let parsed = parse_number::<f32>(value)
                .filter(|number| number.is_finite())
                .ok_or("ожидалось конечное 32-bit floating-point число")?;
            candidate.bytes[field.offset..field.offset + 4].copy_from_slice(&parsed.to_le_bytes());
            Ok(())
        }
    }
}

trait GlobeNumber: Sized {
    const WIDTH: usize;
    fn parse(text: &str) -> Option<Self>;
    fn write(self, destination: &mut [u8]);
}

macro_rules! globe_number {
    ($type:ty) => {
        impl GlobeNumber for $type {
            const WIDTH: usize = std::mem::size_of::<Self>();
            fn parse(text: &str) -> Option<Self> {
                text.parse().ok()
            }
            fn write(self, destination: &mut [u8]) {
                destination.copy_from_slice(&self.to_le_bytes());
            }
        }
    };
}

globe_number!(u8);
globe_number!(u16);
globe_number!(i32);
globe_number!(u32);
globe_number!(f32);

fn parse_number<T: GlobeNumber>(value: &[u8]) -> Option<T> {
    T::parse(std::str::from_utf8(value).ok()?)
}

fn write_number<T: GlobeNumber>(
    destination: &mut [u8],
    offset: usize,
    value: &[u8],
    reason: &'static str,
) -> Result<(), &'static str> {
    let parsed = parse_number::<T>(value).ok_or(reason)?;
    parsed.write(&mut destination[offset..offset + T::WIDTH]);
    Ok(())
}

fn write_fixed_string(
    destination: &mut [u8],
    offset: usize,
    value: &[u8],
) -> Result<(), &'static str> {
    if value.len() >= COUNTRY_NAME_SLOT_LENGTH {
        return Err("строка не помещается в Windows char[64]");
    }
    destination[offset..offset + COUNTRY_NAME_SLOT_LENGTH].fill(0);
    destination[offset..offset + value.len()].copy_from_slice(value);
    Ok(())
}

// Game decoder side effects, а не как Rust-реализация.
