//! Глобальный gameplay snapshot исторического Miracle.
//!
//! Статус World `CGlobeSetup::AddToByteArray` RVA `0x00033470` и
//! `GetBaseMaxRp` RVA `0x0002EFA0` и три auction accessors, прочитанные
//! `CGame::GetOptMoneyJin` RVA `0x00002250`: `IMPLEMENTED`; loaders, остальные accessors
//! и Game decoder side effects ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:836`.
//!
//! Наблюдаемый протокол здесь намеренно является raw ABI snapshot: EXE сначала
//! копирует ровно `0x1114` байт static `m_stSetup`, затем дописывает полный
//! `CRegionRouter` wire с legacy `sendSelf=true` (параметр router serializer не
//! читает). Game decoder забирает те же `0x1114` байт без field conversion.
//! Поэтому fixed byte array — точная модель wire, а не перенос C++ ownership;
//! typed loaders/accessors могут безопасно накладывать подтверждённые offsets
//! поверх него. Static storage оригинала было zero-initialized, что Rust
//! сохраняет через `Default` без утечки padding/heap-мусора.
//! `OnPlayerDeclareWar` exact `0x0046FE62..0x0046FE71` индексирует country
//! name как `m_stSetup + 0x906 + country * 0x40` только для `0..=4`; typed
//! accessor ниже накладывает эту подтверждённую границу на тот же raw snapshot.
//! Country `IsMinister` exact использует соседний `szCountryIdentity` по
//! `+0xA46`, восемь slots по `0x40`; второй accessor не копирует строки.
//! PDB type `CGlobeSetup::tagSetup` дополнительно подтверждает
//! `strSpeStr[0x40]` по `+0x520` и `wTotalJingLiDanCnt` по `+0x1110`;
//! соседний `dwDelDays` по `+0x51C` читает World player-list owner;
//! player rename и LeiTing owners читают их прямо из того же snapshot без
//! отдельного дублирующего state.
//! Create-role exact `0x004B17C9..0x004B17D3` сравнивает zero-extended DB
//! byte-count с signed word в самом начале `m_stSetup` через `CMP AX`/`JL`.
//! Поэтому misleading `btMaxCharactersNum` публикуется как `i16` по `+0`, а
//! не как Rust byte: отрицательная настройка остаётся немедленным отказом.
//! `GetBaseMaxRp` exact `0x0042EFA0..0x0042EFD8` читает static-адреса
//! `0x006BA070/72/74/76`; относительно `m_stSetup 0x006B9C80` это PDB-поля
//! `+0x3F0/+0x3F2/+0x3F4/+0x3F6`. Ветка сохраняет необычный общий случай
//! переставленных level-порогов, а occupation вне нуля сразу возвращает `0`.
//! `GetOptMoneyJin` exact `0x0040225E..0x004022D2` читает
//! `fSxfJinMax/fSxfJinMin/fAuctionFactorC` по `+0xC98/+0xCA0/+0xCB0`;
//! typed accessors ниже лишь накладывают эти PDB-offsets на тот же snapshot.
//! PDB/raw-owner называет соседний one-byte `bAuction`; сохранённая schema
//! предыдущего прохода помещает его по `+0xC87`, что согласуется с этими
//! auction-полями. Accessor ниже использует только `byte != 0`, как exact
//! условие `OnMSG_S2W_AUCTION::0x60808`, не выдавая Rust layout за MSVC ABI.

use crate::setup::regionrouter::{RegionRouter, RegionRouterSerializeError};

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
const AUCTION_ENABLED_OFFSET: usize = 0xC87;
const AUCTION_FEE_MAXIMUM_OFFSET: usize = 0xC98;
const AUCTION_FEE_MINIMUM_OFFSET: usize = 0xCA0;
const AUCTION_FACTOR_C_OFFSET: usize = 0xCB0;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GlobeSetupSnapshot {
    bytes: [u8; GLOBE_SETUP_BLOB_LENGTH],
    country_name_ids: [Vec<u8>; COUNTRY_NAME_COUNT],
    country_identity_ids: [Vec<u8>; COUNTRY_IDENTITY_COUNT],
}

/// Девять occupation-массивов, которые `CPlayer::LoadData` читает из Globe.
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

    /// Загружает поставочный `setup/AuctionList.ini` в `long[256]`.
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

    /// Материализует string-table IDs в два fixed `char[64]` массива EXE.
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

    /// Возвращает exact signed `short btMaxCharactersNum` ветки create-role.
    pub(crate) fn maximum_characters(&self) -> i16 {
        i16::from_le_bytes(
            self.bytes[MAXIMUM_CHARACTERS_OFFSET..MAXIMUM_CHARACTERS_OFFSET + 2]
                .try_into()
                .expect("PDB-offset находится внутри globe snapshot"),
        )
    }

    /// Возвращает bit-exact `fPlayerSpeed` по PDB-offset `+0x7F8`.
    pub(crate) fn player_speed(&self) -> f32 {
        self.read_f32(PLAYER_SPEED_OFFSET)
    }

    /// Масштаб количества монстров, передаваемый всем region-loader-ам.
    pub(crate) fn monster_number_scale(&self) -> f32 {
        self.read_f32(MONSTER_NUMBER_SCALE_OFFSET)
    }

    pub(crate) fn gold_coin_limit(&self) -> u32 {
        self.read_u32(0x4fc)
    }

    pub(crate) fn increment_log_days(&self) -> u32 {
        self.read_u32(0x80c)
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

    /// Возвращает `m_stSetup.bAuction` из подтверждённого raw snapshot-а.
    pub(crate) const fn auction_enabled(&self) -> bool {
        self.bytes[AUCTION_ENABLED_OFFSET] != 0
    }

    /// Возвращает exact `fSxfJinMax`, используемый комиссией аукциона.
    pub(crate) fn auction_fee_maximum(&self) -> f32 {
        self.read_f32(AUCTION_FEE_MAXIMUM_OFFSET)
    }

    /// Возвращает exact `fSxfJinMin`, используемый комиссией аукциона.
    pub(crate) fn auction_fee_minimum(&self) -> f32 {
        self.read_f32(AUCTION_FEE_MINIMUM_OFFSET)
    }

    /// Возвращает exact `fAuctionFactorC` для выплаты продавцу.
    pub(crate) fn auction_factor_c(&self) -> f32 {
        self.read_f32(AUCTION_FACTOR_C_OFFSET)
    }

    /// Повторяет exact `GetBaseMaxRp`: RP есть только у occupation `0`.
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

    /// Возвращает C-string prefix одного exact `szCountryName[5][0x40]`.
    pub(crate) fn country_name(&self, country_id: u8) -> Option<&[u8]> {
        let index = usize::from(country_id);
        if index >= COUNTRY_NAME_COUNT {
            return None;
        }
        let start = COUNTRY_NAME_OFFSET + index * COUNTRY_NAME_SLOT_LENGTH;
        let slot = &self.bytes[start..start + COUNTRY_NAME_SLOT_LENGTH];
        let visible_len = slot.iter().position(|byte| *byte == 0).unwrap_or(slot.len());
        Some(&slot[..visible_len])
    }

    /// Возвращает C-string prefix exact `szCountryIdentity[8][0x40]`.
    pub(crate) fn country_identity_name(&self, identity: u8) -> Option<&[u8]> {
        let index = usize::from(identity);
        if index >= COUNTRY_IDENTITY_COUNT {
            return None;
        }
        let start = COUNTRY_IDENTITY_OFFSET + index * COUNTRY_NAME_SLOT_LENGTH;
        let slot = &self.bytes[start..start + COUNTRY_NAME_SLOT_LENGTH];
        let visible_len = slot.iter().position(|byte| *byte == 0).unwrap_or(slot.len());
        Some(&slot[..visible_len])
    }

    /// Возвращает C-string prefix exact `strSpeStr[0x40]` по PDB `+0x520`.
    pub(crate) fn special_string(&self) -> &[u8] {
        let slot =
            &self.bytes[SPECIAL_STRING_OFFSET..SPECIAL_STRING_OFFSET + SPECIAL_STRING_LENGTH];
        let visible_len = slot.iter().position(|byte| *byte == 0).unwrap_or(slot.len());
        &slot[..visible_len]
    }

    /// Возвращает exact `dwDelDays` перед `strSpeStr` по PDB-offset `+0x51C`.
    pub(crate) fn deletion_days(&self) -> u32 {
        u32::from_le_bytes(
            self.bytes[DELETION_DAYS_OFFSET..DELETION_DAYS_OFFSET + 4]
                .try_into()
                .expect("PDB-offset находится внутри globe snapshot"),
        )
    }

    /// Возвращает exact `wTotalJingLiDanCnt` по PDB-offset `+0x1110`.
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
            Self::Record { line, label, reason } => write!(
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
        172, 204, 236, 396, 300, 332, 364, 396, 428, 460, 492, 524, 556, 588, 620,
        652, 684, 716,
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
        records.push(vec![field(1020 + index * 4, K::F32), field(1044 + index * 2, K::U16)]);
    }
    add_one(&mut records, 1056, K::I32);
    add_one(&mut records, 1060, K::I32);
    add_one(&mut records, 1064, K::F32);
    add_one(&mut records, 1068, K::F32);
    for index in 0..12 {
        records.push(vec![field(1072 + index * 4, K::U32), field(1120 + index * 4, K::U32)]);
    }
    add_array(&mut records, 1168, K::I32, 3, 4);
    add_array(&mut records, 1180, K::I32, 3, 4);
    add_array(&mut records, 1192, K::I32, 3, 4);
    add_array(&mut records, 0, K::Ignore, 3, 0);
    add_one(&mut records, 1216, K::U16);
    for index in 0..4 {
        records.push(vec![field(1220 + index * 4, K::F32), field(1236 + index * 2, K::U16)]);
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
    for offset in [1392, 1432, 1472, 1512, 1552, 1592, 1632, 1672, 1712, 1752, 1792] {
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
        K::U8 => write_number::<u8>(&mut candidate.bytes, field.offset, value, "ожидалось unsigned 8-bit число"),
        K::Bool => {
            if !matches!(value, b"0" | b"1") {
                return Err("ожидалось boolean 0 или 1");
            }
            write_number::<u8>(&mut candidate.bytes, field.offset, value, "ожидалось boolean 0 или 1")
        }
        K::U16 => write_number::<u16>(&mut candidate.bytes, field.offset, value, "ожидалось unsigned 16-bit число"),
        K::I32 => write_number::<i32>(&mut candidate.bytes, field.offset, value, "ожидалось signed 32-bit число"),
        K::U32 => write_number::<u32>(&mut candidate.bytes, field.offset, value, "ожидалось unsigned 32-bit число"),
        K::F32 => {
            let parsed = parse_number::<f32>(value)
                .filter(|number| number.is_finite())
                .ok_or("ожидалось конечное 32-bit floating-point число")?;
            candidate.bytes[field.offset..field.offset + 4]
                .copy_from_slice(&parsed.to_le_bytes());
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

// Сырой C++ ниже сохранён как локальная документация loaders, accessors и
// Game decoder side effects, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp

// ============================================================================
// FUNCTION: CGlobeSetup::GetBaseMaxRp
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:865
// RVA: 0x0001D610
// ADDRESS: 0041d610
// PROTOTYPE: ushort __cdecl GetBaseMaxRp(uchar param_1, uchar param_2)
//
// IMPLEMENTED_OWNER: `GlobeSetupSnapshot::base_max_rp` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGlobeSetup::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:844
// RVA: 0x0001E4A0
// ADDRESS: 0041e4a0
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
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp

// ============================================================================
// FUNCTION: CGlobeSetup::GetBaseMaxRp
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:865
// RVA: 0x0002EFA0
// ADDRESS: 0042efa0
// PROTOTYPE: ushort __cdecl GetBaseMaxRp(uchar param_1, uchar param_2)
//
// IMPLEMENTED_OWNER: `GlobeSetupSnapshot::base_max_rp` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGlobeSetup::LoadAuctionGoodsList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:706
// RVA: 0x0002F1C0
// ADDRESS: 0042f1c0
// PROTOTYPE: int __cdecl LoadAuctionGoodsList(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGlobeSetup::LoadGameSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:739
// RVA: 0x0002F330
// ADDRESS: 0042f330
// PROTOTYPE: int __cdecl LoadGameSetup(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGlobeSetup::Load
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:27
// RVA: 0x00030150
// ADDRESS: 00430150
// PROTOTYPE: int __cdecl Load(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGlobeSetup::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:836
// RVA: 0x00033470
// ADDRESS: 00433470
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00498dce
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp
// RVA: 0x00098DCE
// ADDRESS: 00498dce
// PROTOTYPE: undefined Catch@00498dce()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Unwind@00531e60
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp
// RVA: 0x00131E60
// ADDRESS: 00531e60
// PROTOTYPE: undefined Unwind@00531e60()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// COMPONENT_VARIANT_END: WorldServer
