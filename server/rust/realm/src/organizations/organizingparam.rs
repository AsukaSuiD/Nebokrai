//! Параметры организаций `COrganizingParam` из `organizingparam.cpp/.h`,
//! подтверждённые `worldserver.exe` и `worldserver.pdb`, перенесённый в Realm `organizations/`.
//!
//! `FactionParam.ini` — позиционный whitespace-формат: 12 header values и
//! level records после `*`; labels и номер уровня не проверяются. Byte-строки
//! и literal `"0"` сохраняются, malformed input оставляет уже прочитанный
//! префикс.
//!
//! Tax timer сравнивает только подставленные year/month/day и переносится на
//! следующий день при strict `< now`. Callback сохраняет порядок broadcast
//! `0x7FE26`, регистрации следующего события, StringTable lookup и war-log.
//! `Release` снимает только принадлежащие owner-у calendar events.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::app::world_message::{CMessage, SendMessageError};
use nebokrai_shared::network::ServerCommandHandle;
use nebokrai_shared::resources::read_to_marker as read_to;
use nebokrai_shared::runtime::put_string_to_file;
use nebokrai_shared::runtime::{CTimer, TimerId};
use nebokrai_shared::values::{TagTime, TagTimeArithmeticBlock, TagTimeParseBlock};

const FACTION_PARAM_PATH: &str = "data/FactionParam.ini";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OrganizingLevelParam {
    pub maximum_members: i32,
    pub master_level: i32,
    pub experience: i32,
    pub money: i32,
    pub goods: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct COrganizingParam {
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
pub enum OrganizingParamLoadError {
    Open {
        path: PathBuf,
        source: io::Error,
    },
    MissingToken {
        field: &'static str,
    },
    InvalidInteger {
        field: &'static str,
        value: Vec<u8>,
    },
    InvalidTime {
        field: &'static str,
        source: TagTimeParseBlock,
    },
    DateArithmetic(TagTimeArithmeticBlock),
}

#[derive(Clone, Copy, Debug)]
pub struct OrganizingParamLoadReport {
    pub level_records: usize,
    pub scheduled_tax_time: TagTime,
    pub tax_event_id: TimerId,
    pub legacy_result: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizingTaxScheduleBlock {
    DateArithmetic(TagTimeArithmeticBlock),
}

#[derive(Debug)]
pub struct PreparedTodayTaxRefresh {
    current_event_id: TimerId,
    pub delivery: Result<i32, SendMessageError>,
    pub next_time: TagTime,
}

#[derive(Debug)]
pub struct OrganizingTodayTaxRefreshReport {
    pub delivery: Result<i32, SendMessageError>,
    pub next_time: TagTime,
    pub next_event_id: TimerId,
    pub logged_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrganizingParamReleaseReport {
    pub latest_tax_event_id: Option<TimerId>,
    pub registered_tax_events: usize,
    pub cancelled_tax_events: usize,
    pub released_level_records: usize,
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
    /// Завершает owner и снимает его calendar callbacks до обычного `Drop`.
    ///
    /// `Release` удалял только singleton: в исходном shutdown timer
    /// освобождался отдельно. Явные Rust owners могут жить раздельно, поэтому
    /// registrations отменяются здесь, чтобы callback не пережил параметры.
    pub fn release<Callback>(self, timer: &mut CTimer<Callback>) -> OrganizingParamReleaseReport {
        let registered_tax_events = self.tax_event_ids.len();
        let cancelled_tax_events = self
            .tax_event_ids
            .iter()
            .copied()
            .filter(|event_id| timer.kill_time_event(*event_id))
            .count();
        OrganizingParamReleaseReport {
            latest_tax_event_id: self.latest_tax_event_id,
            registered_tax_events,
            cancelled_tax_events,
            released_level_records: self.levels.len(),
        }
    }

    pub fn initialize<Callback: Copy>(
        &mut self,
        runtime_directory: &Path,
        current_time: TagTime,
        timer: &mut CTimer<Callback>,
        callback: Callback,
    ) -> Result<OrganizingParamLoadReport, OrganizingParamLoadError> {
        self.load(runtime_directory, current_time, timer, callback)
    }

    pub fn load<Callback: Copy>(
        &mut self,
        runtime_directory: &Path,
        current_time: TagTime,
        timer: &mut CTimer<Callback>,
        callback: Callback,
    ) -> Result<OrganizingParamLoadReport, OrganizingParamLoadError> {
        self.levels.clear();
        let path = runtime_directory.join(FACTION_PARAM_PATH);
        let source =
            fs::read(&path).map_err(|source| OrganizingParamLoadError::Open { path, source })?;
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

    pub fn get_max_number_by_level(&self, level: i32) -> i32 {
        self.get_level_param(level)
            .map_or(0, |parameters| parameters.maximum_members)
    }

    pub fn get_level_param(&self, level: i32) -> Option<&OrganizingLevelParam> {
        if !(1..13).contains(&level) {
            return None;
        }
        self.levels.get((level - 1) as usize)
    }

    pub fn get_level_param_mut(&mut self, level: i32) -> Option<&mut OrganizingLevelParam> {
        if !(1..13).contains(&level) {
            return None;
        }
        self.levels.get_mut((level - 1) as usize)
    }

    pub const fn stat_player_ranks_time(&self) -> TagTime {
        self.stat_player_ranks_time
    }

    pub const fn player_ranks_count(&self) -> i32 {
        self.player_ranks_count
    }

    pub const fn upload_icon_interval_minutes(&self) -> i32 {
        self.upload_icon_interval_minutes
    }

    pub const fn pronounce_minimum_level(&self) -> i32 {
        self.pronounce_minimum_level
    }

    pub const fn leave_word_minimum_level(&self) -> i32 {
        self.leave_word_minimum_level
    }

    pub const fn endue_right_minimum_level(&self) -> i32 {
        self.endue_right_minimum_level
    }

    pub const fn create_union_minimum_level(&self) -> i32 {
        self.create_union_minimum_level
    }

    pub const fn attack_village_minimum_level(&self) -> i32 {
        self.attack_village_minimum_level
    }

    pub const fn attack_city_minimum_level(&self) -> i32 {
        self.attack_city_minimum_level
    }

    pub const fn disband_faction_minimum_members(&self) -> i32 {
        self.disband_faction_minimum_members
    }

    pub const fn disband_faction_minutes(&self) -> i32 {
        self.disband_faction_minutes
    }

    pub const fn maximum_contributors(&self) -> i32 {
        self.maximum_contributors
    }

    pub const fn create_faction_player_level(&self) -> i32 {
        self.create_faction_player_level
    }

    pub fn create_faction_goods(&self) -> &[u8] {
        &self.create_faction_goods
    }

    pub const fn create_faction_money(&self) -> i32 {
        self.create_faction_money
    }

    pub const fn latest_tax_event_id(&self) -> Option<TimerId> {
        self.latest_tax_event_id
    }

    pub fn is_tax_event(&self, event_id: TimerId) -> bool {
        self.tax_event_ids.contains(&event_id)
    }

    pub fn prepare_today_tax_refresh(
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

    pub fn finish_today_tax_refresh(
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

    fn initial_tax_time(&self, current_time: TagTime) -> Result<TagTime, TagTimeArithmeticBlock> {
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

fn parse_organizing_param(
    source: &[u8],
) -> Result<ParsedOrganizingParam, OrganizingParamLoadError> {
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
    parsed.disband_faction_minimum_members = next_i32(&mut tokens, "disband minimum members")?;
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
