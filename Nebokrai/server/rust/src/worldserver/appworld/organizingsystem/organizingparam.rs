//! Параметры организаций исторического WorldServer.
//!
//! Статус `GetMaxNumberByLvl` RVA `0x00040B70`, `GetLvlParamByLvl` RVA
//! `0x00040BA0`, constructor/destructor/singleton RVA `0x00041340..0x000413E0`,
//! `OnGetTodayTax` RVA `0x00041510`, `Load` RVA `0x000416D0` и thunk
//! `Initialize` RVA `0x00041C80`: `IMPLEMENTED`, кроме прямого shutdown-вызова
//! `Release`. Точная пара: `WorldServer/Nworldserver.exe +
//! WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный owner PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingparam.cpp`.
//!
//! PDB задаёт размер owner-а `0x88`, `tagLvlParam` `0x2C` и точный порядок
//! полей. Rust не имитирует MSVC layout: `Vec`, `Vec<u8>`, `Option<TimerId>` и
//! явная передача owner-а заменяют `std::vector`, `std::string`,
//! неинициализированный event ID и nullable singleton. Числовые поля исходный
//! constructor оставлял неинициализированными; безопасный `Default` обнуляет
//! их, а malformed input возвращает typed error вместо чтения случайной памяти.
//!
//! `data/FactionParam.ini` вопреки имени является позиционным whitespace-
//! форматом: двенадцать header-записей, затем записи после каждого точного `*`.
//! Названия header-полей и явный номер уровня исходник не проверял; Rust тоже
//! не приписывает им новую семантику. Строки остаются байтами, включая literal
//! `"0"`. Стандартное файловое чтение заменяет `CRFile`, а маленький parser
//! сохраняет специфический `ReadTo("*")` контракт; обычная INI-библиотека для
//! этого формата непригодна.
//!
//! Exact disassembly `0x00441B1F..0x00441B61` и
//! `0x00441590..0x004415D1` исправляет ошибку decompiler-а: перед сравнением
//! либо `AddDay(1)` в налоговую копию подставляются текущие year/month/day,
//! но не weekday. Первое событие переносится только при strict `< now`, callback
//! всегда ставит следующий день. Broadcast `0x7FE26`, регистрация следующего
//! события, lookup `WS0263` и append в `war` сохраняют исходный порядок.
//! Небезопасный `strcpy` в 256-byte local заменён записью всей owned строки:
//! переполнение локального буфера не является требуемой семантикой Miracle.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::nets::servers::ServerCommandHandle;
use crate::public::date::{TagTime, TagTimeArithmeticBlock, TagTimeParseBlock};
use crate::public::readwrite::read_to;
use crate::public::timer::{CTimer, TimerId};
use crate::public::tools::put_string_to_file;

const FACTION_PARAM_PATH: &str = "data/FactionParam.ini";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct OrganizingLevelParam {
    pub(crate) maximum_members: i32,
    pub(crate) master_level: i32,
    pub(crate) experience: i32,
    pub(crate) money: i32,
    pub(crate) goods: Vec<u8>,
}

#[derive(Debug, Default)]
pub(crate) struct COrganizingParam {
    upload_icon_minimum_level: i32,
    upload_icon_interval_minutes: i32,
    pronounce_minimum_level: i32,
    leave_word_minimum_level: i32,
    endue_right_minimum_level: i32,
    create_union_minimum_level: i32,
    attack_village_minimum_level: i32,
    attack_city_minimum_level: i32,
    disband_faction_minimum_members: i32,
    disband_faction_minutes: i32,
    maximum_contributors: i32,
    create_faction_goods: Vec<u8>,
    create_faction_player_level: i32,
    create_faction_money: i32,
    tax_time: TagTime,
    stat_player_ranks_time: TagTime,
    player_ranks_count: i32,
    levels: Vec<OrganizingLevelParam>,
    latest_tax_event_id: Option<TimerId>,
    tax_event_ids: BTreeSet<TimerId>,
}

#[derive(Debug)]
pub(crate) enum OrganizingParamLoadError {
    Open { path: PathBuf, source: io::Error },
    MissingToken { field: &'static str },
    InvalidInteger { field: &'static str, value: Vec<u8> },
    InvalidTime { field: &'static str, source: TagTimeParseBlock },
    DateArithmetic(TagTimeArithmeticBlock),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct OrganizingParamLoadReport {
    pub(crate) level_records: usize,
    pub(crate) scheduled_tax_time: TagTime,
    pub(crate) tax_event_id: TimerId,
    pub(crate) legacy_result: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingTaxScheduleBlock {
    DateArithmetic(TagTimeArithmeticBlock),
}

#[derive(Debug)]
pub(crate) struct PreparedTodayTaxRefresh {
    current_event_id: TimerId,
    pub(crate) delivery: Result<i32, SendMessageError>,
    pub(crate) next_time: TagTime,
}

#[derive(Debug)]
pub(crate) struct OrganizingTodayTaxRefreshReport {
    pub(crate) delivery: Result<i32, SendMessageError>,
    pub(crate) next_time: TagTime,
    pub(crate) next_event_id: TimerId,
    pub(crate) logged_bytes: usize,
}

#[derive(Debug, Default)]
struct ParsedOrganizingParam {
    upload_icon_minimum_level: i32,
    upload_icon_interval_minutes: i32,
    pronounce_minimum_level: i32,
    leave_word_minimum_level: i32,
    endue_right_minimum_level: i32,
    create_union_minimum_level: i32,
    attack_village_minimum_level: i32,
    attack_city_minimum_level: i32,
    disband_faction_minimum_members: i32,
    disband_faction_minutes: i32,
    maximum_contributors: i32,
    create_faction_goods: Vec<u8>,
    create_faction_player_level: i32,
    create_faction_money: i32,
    tax_time: TagTime,
    stat_player_ranks_time: TagTime,
    player_ranks_count: i32,
    levels: Vec<OrganizingLevelParam>,
}

impl COrganizingParam {
    /// Открывает точный runtime-path, разбирает owner и ставит первое tax-событие.
    pub(crate) fn initialize<Callback: Copy>(
        &mut self,
        runtime_directory: &Path,
        current_time: TagTime,
        timer: &mut CTimer<Callback>,
        callback: Callback,
    ) -> Result<OrganizingParamLoadReport, OrganizingParamLoadError> {
        self.load(runtime_directory, current_time, timer, callback)
    }

    /// `Load` очищает level-vector ещё до попытки открыть файл, как exact owner.
    pub(crate) fn load<Callback: Copy>(
        &mut self,
        runtime_directory: &Path,
        current_time: TagTime,
        timer: &mut CTimer<Callback>,
        callback: Callback,
    ) -> Result<OrganizingParamLoadReport, OrganizingParamLoadError> {
        self.levels.clear();
        let path = runtime_directory.join(FACTION_PARAM_PATH);
        let source = fs::read(&path).map_err(|source| OrganizingParamLoadError::Open {
            path,
            source,
        })?;
        let parsed = parse_organizing_param(&source)?;
        self.apply_parsed(parsed);

        let scheduled_tax_time = self
            .initial_tax_time(current_time)
            .map_err(OrganizingParamLoadError::DateArithmetic)?;
        let tax_event_id = timer.set_time_event(scheduled_tax_time, callback, 0);
        self.record_tax_event(tax_event_id);
        Ok(OrganizingParamLoadReport {
            level_records: self.levels.len(),
            scheduled_tax_time,
            tax_event_id,
            legacy_result: true,
        })
    }

    pub(crate) fn get_max_number_by_level(&self, level: i32) -> i32 {
        self.get_level_param(level)
            .map_or(0, |parameters| parameters.maximum_members)
    }

    pub(crate) fn get_level_param(&self, level: i32) -> Option<&OrganizingLevelParam> {
        if !(1..13).contains(&level) {
            return None;
        }
        self.levels.get((level - 1) as usize)
    }

    pub(crate) fn get_level_param_mut(
        &mut self,
        level: i32,
    ) -> Option<&mut OrganizingLevelParam> {
        if !(1..13).contains(&level) {
            return None;
        }
        self.levels.get_mut((level - 1) as usize)
    }

    pub(crate) const fn stat_player_ranks_time(&self) -> TagTime {
        self.stat_player_ranks_time
    }

    pub(crate) const fn player_ranks_count(&self) -> i32 {
        self.player_ranks_count
    }

    pub(crate) const fn latest_tax_event_id(&self) -> Option<TimerId> {
        self.latest_tax_event_id
    }

    pub(crate) fn is_tax_event(&self, event_id: TimerId) -> bool {
        self.tax_event_ids.contains(&event_id)
    }

    /// Выполняет broadcast и вычисление следующего дня до timer-регистрации.
    pub(crate) fn prepare_today_tax_refresh(
        &self,
        current_event_id: TimerId,
        current_time: TagTime,
        sender: Option<&ServerCommandHandle>,
    ) -> Result<PreparedTodayTaxRefresh, OrganizingTaxScheduleBlock> {
        let delivery = CMessage::new(0x0007_FE26).send_all(sender);
        let mut next_time = self.tax_time_for_current_date(current_time);
        next_time
            .add_day(1)
            .map_err(OrganizingTaxScheduleBlock::DateArithmetic)?;
        Ok(PreparedTodayTaxRefresh {
            current_event_id,
            delivery,
            next_time,
        })
    }

    /// Фиксирует `SetTimeEvent`, затем выполняет поздний `WS0263` war-log.
    pub(crate) fn finish_today_tax_refresh(
        &mut self,
        prepared: PreparedTodayTaxRefresh,
        next_event_id: TimerId,
        world_string_by_id: &mut dyn FnMut(&[u8]) -> Vec<u8>,
    ) -> OrganizingTodayTaxRefreshReport {
        self.tax_event_ids.remove(&prepared.current_event_id);
        self.record_tax_event(next_event_id);
        let localized_text = world_string_by_id(b"WS0263");
        put_string_to_file("war", &localized_text);
        OrganizingTodayTaxRefreshReport {
            delivery: prepared.delivery,
            next_time: prepared.next_time,
            next_event_id,
            logged_bytes: localized_text.len(),
        }
    }

    fn initial_tax_time(
        &self,
        current_time: TagTime,
    ) -> Result<TagTime, TagTimeArithmeticBlock> {
        let mut scheduled = self.tax_time_for_current_date(current_time);
        if scheduled.legacy_lt(current_time) {
            scheduled.add_day(1)?;
        }
        Ok(scheduled)
    }

    fn tax_time_for_current_date(&self, current_time: TagTime) -> TagTime {
        let mut scheduled = self.tax_time;
        scheduled.year = current_time.year;
        scheduled.month = current_time.month;
        scheduled.day = current_time.day;
        scheduled
    }

    fn record_tax_event(&mut self, event_id: TimerId) {
        self.latest_tax_event_id = Some(event_id);
        self.tax_event_ids.insert(event_id);
    }

    fn apply_parsed(&mut self, parsed: ParsedOrganizingParam) {
        self.upload_icon_minimum_level = parsed.upload_icon_minimum_level;
        self.upload_icon_interval_minutes = parsed.upload_icon_interval_minutes;
        self.pronounce_minimum_level = parsed.pronounce_minimum_level;
        self.leave_word_minimum_level = parsed.leave_word_minimum_level;
        self.endue_right_minimum_level = parsed.endue_right_minimum_level;
        self.create_union_minimum_level = parsed.create_union_minimum_level;
        self.attack_village_minimum_level = parsed.attack_village_minimum_level;
        self.attack_city_minimum_level = parsed.attack_city_minimum_level;
        self.disband_faction_minimum_members = parsed.disband_faction_minimum_members;
        self.disband_faction_minutes = parsed.disband_faction_minutes;
        self.maximum_contributors = parsed.maximum_contributors;
        self.create_faction_goods = parsed.create_faction_goods;
        self.create_faction_player_level = parsed.create_faction_player_level;
        self.create_faction_money = parsed.create_faction_money;
        self.tax_time = parsed.tax_time;
        self.stat_player_ranks_time = parsed.stat_player_ranks_time;
        self.player_ranks_count = parsed.player_ranks_count;
        self.levels = parsed.levels;
    }
}

fn parse_organizing_param(source: &[u8]) -> Result<ParsedOrganizingParam, OrganizingParamLoadError> {
    let mut tokens = source
        .split(|byte| byte.is_ascii_whitespace())
        .filter(|token| !token.is_empty());
    let mut parsed = ParsedOrganizingParam::default();

    skip_token(&mut tokens, "upload icon label")?;
    parsed.upload_icon_minimum_level = next_i32(&mut tokens, "upload icon minimum level")?;
    parsed.upload_icon_interval_minutes = next_i32(&mut tokens, "upload icon interval")?;
    skip_token(&mut tokens, "pronounce label")?;
    parsed.pronounce_minimum_level = next_i32(&mut tokens, "pronounce minimum level")?;
    skip_token(&mut tokens, "leave-word label")?;
    parsed.leave_word_minimum_level = next_i32(&mut tokens, "leave-word minimum level")?;
    skip_token(&mut tokens, "endue-right label")?;
    parsed.endue_right_minimum_level = next_i32(&mut tokens, "endue-right minimum level")?;
    skip_token(&mut tokens, "create-union label")?;
    parsed.create_union_minimum_level = next_i32(&mut tokens, "create-union minimum level")?;
    skip_token(&mut tokens, "attack-village label")?;
    parsed.attack_village_minimum_level = next_i32(&mut tokens, "attack-village minimum level")?;
    skip_token(&mut tokens, "attack-city label")?;
    parsed.attack_city_minimum_level = next_i32(&mut tokens, "attack-city minimum level")?;
    skip_token(&mut tokens, "disband label")?;
    parsed.disband_faction_minimum_members =
        next_i32(&mut tokens, "disband minimum members")?;
    parsed.disband_faction_minutes = next_i32(&mut tokens, "disband minutes")?;
    skip_token(&mut tokens, "maximum contributors label")?;
    parsed.maximum_contributors = next_i32(&mut tokens, "maximum contributors")?;
    skip_token(&mut tokens, "create-faction label")?;
    parsed.create_faction_player_level = next_i32(&mut tokens, "create-faction player level")?;
    parsed.create_faction_goods = next_token(&mut tokens, "create-faction goods")?.to_vec();
    parsed.create_faction_money = next_i32(&mut tokens, "create-faction money")?;
    skip_token(&mut tokens, "tax time label")?;
    parsed.tax_time = next_time(&mut tokens, "tax time")?;
    skip_token(&mut tokens, "player ranks label")?;
    parsed.stat_player_ranks_time = next_time(&mut tokens, "player ranks time")?;
    parsed.player_ranks_count = next_i32(&mut tokens, "player ranks count")?;

    while read_to(&mut tokens, b"*") {
        let _ignored_level = next_i32(&mut tokens, "level record number")?;
        parsed.levels.push(OrganizingLevelParam {
            maximum_members: next_i32(&mut tokens, "level maximum members")?,
            master_level: next_i32(&mut tokens, "level master level")?,
            experience: next_i32(&mut tokens, "level experience")?,
            money: next_i32(&mut tokens, "level money")?,
            goods: next_token(&mut tokens, "level goods")?.to_vec(),
        });
    }
    Ok(parsed)
}

fn skip_token<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    field: &'static str,
) -> Result<(), OrganizingParamLoadError> {
    let _ = next_token(tokens, field)?;
    Ok(())
}

fn next_token<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    field: &'static str,
) -> Result<&'a [u8], OrganizingParamLoadError> {
    tokens
        .next()
        .ok_or(OrganizingParamLoadError::MissingToken { field })
}

fn next_i32<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    field: &'static str,
) -> Result<i32, OrganizingParamLoadError> {
    let value = next_token(tokens, field)?;
    std::str::from_utf8(value)
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| OrganizingParamLoadError::InvalidInteger {
            field,
            value: value.to_vec(),
        })
}

fn next_time<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    field: &'static str,
) -> Result<TagTime, OrganizingParamLoadError> {
    let value = next_token(tokens, field)?;
    TagTime::from_legacy_string(value)
        .map_err(|source| OrganizingParamLoadError::InvalidTime { field, source })
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingparam.cpp

// ============================================================================
// FUNCTION: COrganizingParam::GetMaxNumberByLvl
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingparam.cpp:152
// RVA: 0x00040B70
// ADDRESS: 00440b70
// PROTOTYPE: long __thiscall GetMaxNumberByLvl(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingParam::GetLvlParamByLvl
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingparam.cpp:159
// RVA: 0x00040BA0
// ADDRESS: 00440ba0
// PROTOTYPE: tagLvlParam * __thiscall GetLvlParamByLvl(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingParam::COrganizingParam
// STATUS: IMPLEMENTED
// Safe `Default` заменяет неполную inline-инициализацию исходника.
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingparam.cpp:20
// RVA: 0x00041340
// ADDRESS: 00441340
// PROTOTYPE: undefined __thiscall COrganizingParam(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingParam::~COrganizingParam
// STATUS: IMPLEMENTED
// Owned `Vec` и byte strings освобождаются обычным Rust `Drop`.
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingparam.cpp:25
// RVA: 0x000413B0
// ADDRESS: 004413b0
// PROTOTYPE: void __thiscall ~COrganizingParam(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingParam::getInstance
// STATUS: IMPLEMENTED
// Единственный owner передаётся явно вместо nullable process-global pointer-а.
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingparam.cpp:31
// RVA: 0x000413E0
// ADDRESS: 004413e0
// PROTOTYPE: COrganizingParam * __cdecl getInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingParam::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingparam.cpp:45
// RVA: 0x000414E0
// ADDRESS: 004414e0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingParam::OnGetTodayTax
// STATUS: IMPLEMENTED + VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingparam.cpp:120
// RVA: 0x00041510
// ADDRESS: 00441510
// PROTOTYPE: void __stdcall OnGetTodayTax(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingParam::Load
// STATUS: IMPLEMENTED + VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingparam.cpp:53
// RVA: 0x000416D0
// ADDRESS: 004416d0
// PROTOTYPE: bool __thiscall Load(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizingParam::Initialize
// STATUS: IMPLEMENTED + VERIFIED_DISASSEMBLY
// Exact body — единственный `JMP 0x004416D0` в `0x00441C80`.
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingparam.cpp:40
// RVA: 0x00041C80
// ADDRESS: 00441c80
// PROTOTYPE: bool __thiscall Initialize(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_bool___thiscall_CUnion::Invite(long,long)'::__l22::InviteJoinConfeder::Release`adjustor{4}'
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizingparam.cpp
// RVA: 0x000C19F0
// ADDRESS: 004c19f0
// PROTOTYPE: void __thiscall Release`adjustor{4}'(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//















// COMPONENT_VARIANT_END: WorldServer
