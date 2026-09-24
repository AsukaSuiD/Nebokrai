//! Расписание деревенских войн `CVillageWarSys` из `villagewarsys.cpp/.h`,
//! подтверждённое `worldserver.exe` и `worldserver.pdb`, перенесённый в Realm `activities/`.
//!
//! Loader дважды читает token stream: weekly `#`, затем single `*`. Weekly
//! rollover, неполная overlap-проверка и порядок шести фазовых timers
//! сохраняются. Неназначенный event ID хранится как `None` и не используется
//! для случайного `KillEvent` при reload.
//!
//! Callbacks сначала меняют state, затем рассылают `0x7FE2F..0x7FE33` и
//! публикуют журналы. Countdown учитывает только минуты и секунды. Weekly end
//! очищает список заявок и ищет не более пяти следующих недель; промежуточные
//! events перед повторным `Initialize` намеренно остаются.
//!
//! Победа и заявка сохраняют исходные gates, порядок ownership, country,
//! top-info и war-log эффектов. Snapshot пишет prefix записи, отдельный count
//! и ordered faction IDs; allocator padding и неинициализированные event IDs
//! нормализованы нулями, поскольку парный decoder их не использует.

use std::collections::BTreeMap;

use nebokrai_shared::resources::read_to_marker as read_to;
use nebokrai_shared::runtime::{CTimer, TimerId};
use nebokrai_shared::values::{TagTime, TagTimeArithmeticBlock, TagTimeParseBlock};

use crate::app::world_message::CMessage;
use crate::content::organizing::ECityState;

#[derive(Clone, Copy, Debug)]
pub struct VillageWarCallbacks<Callback> {
    pub declare: Callback,
    pub start_info: Callback,
    pub start: Callback,
    pub end_info: Callback,
    pub end: Callback,
    pub clear_player: Callback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VillageWarCallbackKind {
    Declare,
    StartInfo,
    Start,
    EndInfo,
    End,
    ClearPlayer,
}

impl<Callback: PartialEq> VillageWarCallbacks<Callback> {
    pub fn kind(&self, callback: &Callback) -> Option<VillageWarCallbackKind> {
        [
            (&self.declare, VillageWarCallbackKind::Declare),
            (&self.start_info, VillageWarCallbackKind::StartInfo),
            (&self.start, VillageWarCallbackKind::Start),
            (&self.end_info, VillageWarCallbackKind::EndInfo),
            (&self.end, VillageWarCallbackKind::End),
            (&self.clear_player, VillageWarCallbackKind::ClearPlayer),
        ]
        .into_iter()
        .find_map(|(candidate, kind)| (candidate == callback).then_some(kind))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VillageWarAnnouncement {
    pub war_number: i32,
    pub war_region_id: i32,
    pub world_string_id: &'static [u8],
    pub set_all_countries_warring: bool,
}

pub trait VillageWarPhaseContext {
    type Block;

    fn send_all(&mut self, message: &CMessage) -> i32;

    /// При существующем region форматирует `world_string_id` его именем,
    /// отправляет organizing-info с `-366/`, затем при указанном
    /// флаге ставит `m_bIsWarring=true` странам `1..=4` и пишет строку в `war`.
    fn announce(&mut self, request: VillageWarAnnouncement) -> Result<(), Self::Block>;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VillageWarPhaseReport {
    pub schedule_found: bool,
    pub delivery: Option<i32>,
    pub announcement_requested: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VillageWarCountdownRequest {
    pub war_number: i32,
    pub war_region_id: i32,
    pub world_string_id: &'static [u8],
    pub duration_ms: i32,
}

pub trait VillageWarCountdownContext {
    type Block;

    fn region_exists(&mut self, region_id: i32) -> Result<bool, Self::Block>;

    /// Разрешает `world_string_id`, вызывает `AddOneTopInfo(2, duration, text)`,
    /// затем `SendTopInfoToClient` с возвращённым ID и теми же полями.
    fn publish_countdown(&mut self, request: VillageWarCountdownRequest)
        -> Result<(), Self::Block>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VillageWarCountdownBlock<ContextBlock> {
    Context(ContextBlock),
    Arithmetic(TagTimeArithmeticBlock),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VillageWarCountdownReport {
    pub schedule_found: bool,
    pub region_found: bool,
    pub target_in_future: bool,
    pub duration_ms: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VillageWarEndBlock {
    Arithmetic(TagTimeArithmeticBlock),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VillageWarEndReport {
    pub schedule_found: bool,
    pub delivery: Option<i32>,
    pub cleared_factions: usize,
    pub rearmed_after_weeks: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VillageWarResultRegion {
    pub name: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VillageWarResultFaction {
    pub organizing_id: i32,
    pub name: Vec<u8>,
}

pub trait VillageWarResultContext {
    type Block;

    fn region(&mut self, region_id: i32) -> Result<Option<VillageWarResultRegion>, Self::Block>;

    fn region_owner_faction_id(&mut self, region_id: i32) -> Result<i32, Self::Block>;

    fn faction(&mut self, faction_id: i32) -> Result<Option<VillageWarResultFaction>, Self::Block>;

    fn union_for_faction(&mut self, faction_id: i32) -> Result<i32, Self::Block>;

    fn refresh_owned_city_org(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
    ) -> Result<(), Self::Block>;

    fn add_owned_city(&mut self, faction_id: i32, region_id: i32) -> Result<(), Self::Block>;

    fn clear_owned_city(&mut self, faction_id: i32) -> Result<(), Self::Block>;

    fn add_village_war_victor_count(&mut self, faction_id: i32) -> Result<(), Self::Block>;

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[&[u8]],
    ) -> Result<Vec<u8>, Self::Block>;

    fn send_organizing_info(&mut self, text: &[u8]) -> Result<(), Self::Block>;

    fn write_war_log(&mut self, text: &[u8]) -> Result<(), Self::Block>;

    fn send_top_info(
        &mut self,
        top_info_id: i32,
        timer_flag: i32,
        parameter: i32,
        text: &[u8],
    ) -> Result<(), Self::Block>;

    fn send_all(&mut self, message: &CMessage) -> i32;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VillageWarResultBlock<ContextBlock> {
    MissingEndInfoEventId { war_number: i32 },
    MissingEndEventId { war_number: i32 },
    End(VillageWarEndBlock),
    Context(ContextBlock),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VillageWarResultReport {
    pub schedule_found: bool,
    pub end_info_event_killed: Option<bool>,
    pub end_event_killed: Option<bool>,
    pub war_region_found: bool,
    pub village_region_found: bool,
    pub war_end: Option<VillageWarEndReport>,
    pub winner_faction_found: Option<bool>,
    pub ownership_changed: bool,
    pub top_info_published: bool,
}

pub trait VillageWarApplicationContext {
    type Block;

    fn faction_master_for_player(&mut self, player_id: i32) -> Result<i32, Self::Block>;

    fn faction_exists(&mut self, faction_id: i32) -> Result<bool, Self::Block>;

    fn region_exists(&mut self, region_id: i32) -> Result<bool, Self::Block>;

    fn attack_village_min_level(&mut self) -> Result<i32, Self::Block>;

    fn faction_level(&mut self, faction_id: i32) -> Result<i32, Self::Block>;

    fn region_owner_faction_id(&mut self, region_id: i32) -> Result<i32, Self::Block>;

    fn union_for_faction(&mut self, faction_id: i32) -> Result<i32, Self::Block>;

    fn faction_owned_city_count(&mut self, faction_id: i32) -> Result<usize, Self::Block>;

    fn already_declared_for_city_war(&mut self, faction_id: i32) -> Result<bool, Self::Block>;

    fn faction_name(&mut self, faction_id: i32) -> Result<Vec<u8>, Self::Block>;

    fn region_name(&mut self, region_id: i32) -> Result<Vec<u8>, Self::Block>;

    /// Локализует title/text по порядку и отправляет player-targeted overload
    /// с legacy arguments `-1, -1, `.
    fn send_level_rejection(
        &mut self,
        player_id: i32,
        title_string_id: &'static [u8],
        text_string_id: &'static [u8],
    ) -> Result<(), Self::Block>;

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[&[u8]],
    ) -> Result<Vec<u8>, Self::Block>;

    fn send_organizing_info(&mut self, text: &[u8]) -> Result<(), Self::Block>;

    fn write_war_log(&mut self, text: &[u8]) -> Result<(), Self::Block>;

    fn send_all(&mut self, message: &CMessage) -> i32;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VillageWarApplicationReport {
    pub schedule_found: bool,
    pub master_faction_id: Option<i32>,
    pub faction_found: bool,
    pub war_region_found: bool,
    pub village_region_found: bool,
    pub level_rejected: bool,
    pub accepted: bool,
    pub delivery: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct VillageWarSetup {
    id: i32,
    war_region_id: i32,
    declare_event_id: Option<TimerId>,
    declare_time: TagTime,
    start_info_event_id: Option<TimerId>,
    start_info_time: TagTime,
    start_event_id: Option<TimerId>,
    start_time: TagTime,
    end_info_event_id: Option<TimerId>,
    end_info_time: TagTime,
    end_event_id: Option<TimerId>,
    end_time: TagTime,
    clear_event_id: Option<TimerId>,
    clear_player_time: TagTime,
    village_region_id: i32,
    region_state: ECityState,
    declaring_factions: Vec<i32>,
    is_every_week: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VillageWarLoadReport {
    pub resource_found: bool,
    pub accepted: u32,
    pub ignored_time_order: u32,
    pub ignored_time_conflict: u32,
    pub ignored_overdue: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VillageWarLoadError {
    ResourceMissing,
    MissingValue { field: &'static str },
    InvalidValue { field: &'static str },
    TimeParse(TagTimeParseBlock),
    Arithmetic(TagTimeArithmeticBlock),
    ScheduleIdOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VillageWarReloadEvent {
    Declare,
    StartInfo,
    Start,
    EndInfo,
    End,
    ClearPlayer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VillageWarReloadBlock {
    MissingEventId {
        war_number: i32,
        event: VillageWarReloadEvent,
    },
    End {
        war_number: i32,
        block: VillageWarEndBlock,
    },
    Load(VillageWarLoadError),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VillageWarReloadReport {
    pub previous_schedules: usize,
    pub kill_requests: usize,
    pub killed_events: usize,
    pub active_schedules_ended: usize,
    pub load: VillageWarLoadReport,
}

impl From<TagTimeArithmeticBlock> for VillageWarLoadError {
    fn from(value: TagTimeArithmeticBlock) -> Self {
        Self::Arithmetic(value)
    }
}

impl From<TagTimeParseBlock> for VillageWarLoadError {
    fn from(value: TagTimeParseBlock) -> Self {
        Self::TimeParse(value)
    }
}

#[derive(Default)]
pub struct CVillageWarSys {
    village_wars: BTreeMap<i32, VillageWarSetup>,
}

impl CVillageWarSys {
    pub const fn new() -> Self {
        Self {
            village_wars: BTreeMap::new(),
        }
    }

    pub fn initialize<Callback: Copy>(
        &mut self,
        source: Option<&[u8]>,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: VillageWarCallbacks<Callback>,
    ) -> Result<VillageWarLoadReport, VillageWarLoadError> {
        self.village_wars.clear();
        let Some(source) = source else {
            return Err(VillageWarLoadError::ResourceMissing);
        };
        let mut report = VillageWarLoadReport {
            resource_found: true,
            ..VillageWarLoadReport::default()
        };
        let mut next_id = 0i32;

        let mut weekly_tokens = war_tokens(source);
        while read_to(&mut weekly_tokens, b"#") {
            next_id = next_id
                .checked_add(1)
                .ok_or(VillageWarLoadError::ScheduleIdOverflow)?;
            let candidate = read_weekly_village_schedule(&mut weekly_tokens, next_id, now)?;
            if !candidate.has_valid_time_order() {
                report.ignored_time_order = report.ignored_time_order.wrapping_add(1);
            } else if self.conflicts_with(&candidate) {
                report.ignored_time_conflict = report.ignored_time_conflict.wrapping_add(1);
            } else {
                self.village_wars.insert(next_id, candidate);
                report.accepted = report.accepted.wrapping_add(1);
            }
        }

        let mut single_tokens = war_tokens(source);
        while read_to(&mut single_tokens, b"*") {
            next_id = next_id
                .checked_add(1)
                .ok_or(VillageWarLoadError::ScheduleIdOverflow)?;
            let candidate = read_single_village_schedule(&mut single_tokens, next_id)?;
            if candidate.declare_time.legacy_lt(now) {
                report.ignored_overdue = report.ignored_overdue.wrapping_add(1);
            } else if !candidate.has_valid_time_order() {
                report.ignored_time_order = report.ignored_time_order.wrapping_add(1);
            } else if self.conflicts_with(&candidate) {
                report.ignored_time_conflict = report.ignored_time_conflict.wrapping_add(1);
            } else {
                self.village_wars.insert(next_id, candidate);
                report.accepted = report.accepted.wrapping_add(1);
            }
        }

        for setup in self.village_wars.values_mut() {
            setup.register_initial_events(now, timer, callbacks);
        }
        Ok(report)
    }

    pub fn reload<Callback, SendAll>(
        &mut self,
        source: Option<&[u8]>,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: VillageWarCallbacks<Callback>,
        mut send_all: SendAll,
    ) -> Result<VillageWarReloadReport, VillageWarReloadBlock>
    where
        Callback: Copy,
        SendAll: FnMut(&CMessage) -> i32,
    {
        let mut report = VillageWarReloadReport {
            previous_schedules: self.village_wars.len(),
            ..VillageWarReloadReport::default()
        };

        for (&war_number, setup) in &self.village_wars {
            let events = [
                (VillageWarReloadEvent::Declare, setup.declare_event_id),
                (VillageWarReloadEvent::StartInfo, setup.start_info_event_id),
                (VillageWarReloadEvent::Start, setup.start_event_id),
                (VillageWarReloadEvent::EndInfo, setup.end_info_event_id),
                (VillageWarReloadEvent::End, setup.end_event_id),
                (VillageWarReloadEvent::ClearPlayer, setup.clear_event_id),
            ];
            for (event, event_id) in events {
                let Some(event_id) = event_id else {
                    return Err(VillageWarReloadBlock::MissingEventId { war_number, event });
                };
                report.kill_requests = report.kill_requests.wrapping_add(1);
                if timer.kill_time_event(event_id) {
                    report.killed_events = report.killed_events.wrapping_add(1);
                }
            }
        }

        let previous = self.village_wars.clone();
        for (&war_number, setup) in &previous {
            if setup.region_state != ECityState::No {
                self.on_village_war_end(
                    war_number,
                    setup.war_region_id,
                    timer,
                    callbacks,
                    &mut send_all,
                )
                .map_err(|block| VillageWarReloadBlock::End { war_number, block })?;
                report.active_schedules_ended = report.active_schedules_ended.wrapping_add(1);
            }
        }

        report.load = self
            .initialize(source, now, timer, callbacks)
            .map_err(VillageWarReloadBlock::Load)?;
        Ok(report)
    }

    pub fn add_to_byte_array(&self, output: &mut Vec<u8>) -> bool {
        append_u32(output, self.village_wars.len() as u32);
        for setup in self.village_wars.values() {
            append_i32(output, setup.id);
            append_i32(output, setup.war_region_id);
            append_timer_id(output, setup.declare_event_id);
            append_tag_time(output, setup.declare_time);
            append_timer_id(output, setup.start_info_event_id);
            append_tag_time(output, setup.start_info_time);
            append_timer_id(output, setup.start_event_id);
            append_tag_time(output, setup.start_time);
            append_timer_id(output, setup.end_info_event_id);
            append_tag_time(output, setup.end_info_time);
            append_timer_id(output, setup.end_event_id);
            append_tag_time(output, setup.end_time);
            append_timer_id(output, setup.clear_event_id);
            append_tag_time(output, setup.clear_player_time);
            append_i32(output, setup.village_region_id);
            append_i32(output, setup.region_state as i32);

            // копирует ещё четыре неинициализированных bytes
            // пустого allocator-base `std::list`. GameServer
            // перезаписывает ими тот же stateless allocator, но отдельно
            // создаёт `_Myhead`, читает count и ни разу не читает event IDs.
            // Нули — узкий compatibility-layer, а не заявленные старые bytes.
            output.extend_from_slice(&[0; 4]);
            append_u32(output, setup.declaring_factions.len() as u32);
            for &faction_id in &setup.declaring_factions {
                append_i32(output, faction_id);
            }
        }
        true
    }

    pub fn get_region_state_at(&self, region_id: i32, now: TagTime) -> ECityState {
        self.village_wars
            .values()
            .find(|setup| {
                (setup.war_region_id == region_id || setup.village_region_id == region_id)
                    && now.legacy_ge(setup.declare_time)
                    && now.legacy_le(setup.end_time)
            })
            .map_or(ECityState::No, |setup| setup.region_state)
    }

    pub fn get_region_state(&self, region_id: i32) -> ECityState {
        self.get_region_state_at(region_id, TagTime::local_now())
    }

    pub fn is_village_region_left_clear_time_at(&self, region_id: i32, now: TagTime) -> bool {
        self.village_wars.values().any(|setup| {
            setup.village_region_id == region_id
                && now.legacy_ge(setup.clear_player_time)
                && now.legacy_lt(setup.end_time)
        })
    }

    pub fn is_village_region_left_clear_time(&self, region_id: i32) -> bool {
        self.is_village_region_left_clear_time_at(region_id, TagTime::local_now())
    }

    pub fn get_city_state_by_war_num(&self, war_number: i32) -> ECityState {
        self.village_wars
            .get(&war_number)
            .map_or(ECityState::No, |setup| setup.region_state)
    }

    pub fn update_apply_war_factions_to_game_server(
        &self,
        war_number: i32,
        output: &mut Vec<u8>,
    ) -> bool {
        append_i32(output, war_number);
        let factions = self
            .village_wars
            .get(&war_number)
            .map_or(&[][..], |setup| setup.declaring_factions.as_slice());
        output.extend_from_slice(&(factions.len() as u32).to_le_bytes());
        for &faction_id in factions {
            append_i32(output, faction_id);
        }
        true
    }

    pub fn is_already_declared_for_war(&self, faction_id: i32) -> bool {
        self.village_wars.values().any(|setup| {
            setup.region_state != ECityState::No && setup.declaring_factions.contains(&faction_id)
        })
    }

    pub fn apply_for_village_war<Context>(
        &mut self,
        player_id: i32,
        war_number: i32,
        _legacy_third_parameter: i32,
        context: &mut Context,
    ) -> Result<VillageWarApplicationReport, Context::Block>
    where
        Context: VillageWarApplicationContext + ?Sized,
    {
        let mut report = VillageWarApplicationReport::default();
        let Some(setup) = self.village_wars.get(&war_number).cloned() else {
            return Ok(report);
        };
        report.schedule_found = true;

        let faction_id = context.faction_master_for_player(player_id)?;
        report.master_faction_id = Some(faction_id);
        if faction_id == 0 || !context.faction_exists(faction_id)? {
            return Ok(report);
        }
        report.faction_found = true;
        if setup.war_region_id == 0 || !context.region_exists(setup.war_region_id)? {
            return Ok(report);
        }
        report.war_region_found = true;
        if !context.region_exists(setup.village_region_id)? {
            return Ok(report);
        }
        report.village_region_found = true;

        let minimum_level = context.attack_village_min_level()?;
        if context.faction_level(faction_id)? < minimum_level {
            context.send_level_rejection(player_id, b"WS0121", b"WS0288")?;
            report.level_rejected = true;
            return Ok(report);
        }
        if setup.region_state == ECityState::No {
            return Ok(report);
        }

        let owner_faction_id = context.region_owner_faction_id(setup.village_region_id)?;
        if owner_faction_id == faction_id {
            return Ok(report);
        }
        let faction_union_id = context.union_for_faction(faction_id)?;
        let owner_union_id = context.union_for_faction(owner_faction_id)?;
        if faction_union_id != 0 && owner_union_id == faction_union_id {
            return Ok(report);
        }
        if context.faction_owned_city_count(faction_id)? != 0
            || self.is_already_declared_for_war(faction_id)
            || context.already_declared_for_city_war(faction_id)?
        {
            return Ok(report);
        }

        self.village_wars
            .get_mut(&war_number)
            .expect("schedule проверен и не удаляется внутри owner-а")
            .declaring_factions
            .push(faction_id);
        let mut payload = Vec::new();
        self.update_apply_war_factions_to_game_server(war_number, &mut payload);
        let mut message = CMessage::new(0x7FE36);
        message.base_mut().add(&payload);
        report.delivery = Some(context.send_all(&message));

        let faction_name = context.faction_name(faction_id)?;
        let war_region_name = context.region_name(setup.war_region_id)?;
        let text = context.format_world_string(
            b"WS0289",
            &[faction_name.as_slice(), war_region_name.as_slice()],
        )?;
        context.send_organizing_info(&text)?;
        context.write_war_log(&text)?;
        report.accepted = true;
        Ok(report)
    }

    pub fn get_village_war_setup_by_time(&self, war_number: i32) -> Option<VillageWarSetup> {
        self.village_wars.get(&war_number).cloned()
    }

    pub fn on_declare_war<Context: VillageWarPhaseContext + ?Sized>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> Result<VillageWarPhaseReport, Context::Block> {
        self.apply_phase(
            war_number,
            ECityState::Duth,
            0x7FE2F,
            b"WS0283",
            true,
            context,
        )
    }

    pub fn on_attack_village_start<Context: VillageWarPhaseContext + ?Sized>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> Result<VillageWarPhaseReport, Context::Block> {
        self.apply_phase(
            war_number,
            ECityState::Fight,
            0x7FE30,
            b"WS0284",
            false,
            context,
        )
    }

    pub fn on_attack_village_end<Context: VillageWarPhaseContext + ?Sized>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> Result<VillageWarPhaseReport, Context::Block> {
        // снова ставит `true` всем четырём странам. Это
        // подтверждённая странность callback-а, а не исправляемый typo.
        self.apply_phase(
            war_number,
            ECityState::No,
            0x7FE31,
            b"WS0285",
            true,
            context,
        )
    }

    pub fn on_clear_player<Context: VillageWarPhaseContext + ?Sized>(
        &self,
        war_number: i32,
        context: &mut Context,
    ) -> VillageWarPhaseReport {
        if !self.village_wars.contains_key(&war_number) {
            return VillageWarPhaseReport::default();
        }
        VillageWarPhaseReport {
            schedule_found: true,
            delivery: Some(send_phase_message(context, 0x7FE33, war_number)),
            announcement_requested: false,
        }
    }

    pub fn on_attack_village_start_info<Context: VillageWarCountdownContext + ?Sized>(
        &self,
        war_number: i32,
        now: TagTime,
        context: &mut Context,
    ) -> Result<VillageWarCountdownReport, VillageWarCountdownBlock<Context::Block>> {
        self.publish_countdown(
            war_number,
            now,
            |setup| setup.start_time,
            b"WS0286",
            context,
        )
    }

    pub fn on_attack_village_end_info<Context: VillageWarCountdownContext + ?Sized>(
        &self,
        war_number: i32,
        now: TagTime,
        context: &mut Context,
    ) -> Result<VillageWarCountdownReport, VillageWarCountdownBlock<Context::Block>> {
        self.publish_countdown(war_number, now, |setup| setup.end_time, b"WS0287", context)
    }

    fn publish_countdown<Context, Target>(
        &self,
        war_number: i32,
        now: TagTime,
        target: Target,
        world_string_id: &'static [u8],
        context: &mut Context,
    ) -> Result<VillageWarCountdownReport, VillageWarCountdownBlock<Context::Block>>
    where
        Context: VillageWarCountdownContext + ?Sized,
        Target: FnOnce(&VillageWarSetup) -> TagTime,
    {
        let Some(current) = self.village_wars.get(&war_number) else {
            return Ok(VillageWarCountdownReport::default());
        };
        let war_region_id = current.war_region_id;
        if war_region_id == 0
            || !context
                .region_exists(war_region_id)
                .map_err(VillageWarCountdownBlock::Context)?
        {
            return Ok(VillageWarCountdownReport {
                schedule_found: true,
                ..VillageWarCountdownReport::default()
            });
        }

        let copied = self
            .get_village_war_setup_by_time(war_number)
            .expect("ID найден до неизменяемого повторного lookup");
        let target = target(&copied);
        if target.legacy_le(now) {
            return Ok(VillageWarCountdownReport {
                schedule_found: true,
                region_found: true,
                ..VillageWarCountdownReport::default()
            });
        }
        let difference = target
            .get_time_difference(now)
            .map_err(VillageWarCountdownBlock::Arithmetic)?;
        let duration_ms = countdown_duration_ms(difference);
        context
            .publish_countdown(VillageWarCountdownRequest {
                war_number,
                war_region_id,
                world_string_id,
                duration_ms,
            })
            .map_err(VillageWarCountdownBlock::Context)?;
        Ok(VillageWarCountdownReport {
            schedule_found: true,
            region_found: true,
            target_in_future: true,
            duration_ms: Some(duration_ms),
        })
    }

    pub fn on_village_war_end<Callback, SendAll>(
        &mut self,
        war_number: i32,
        _legacy_second_parameter: i32,
        timer: &mut CTimer<Callback>,
        callbacks: VillageWarCallbacks<Callback>,
        mut send_all: SendAll,
    ) -> Result<VillageWarEndReport, VillageWarEndBlock>
    where
        Callback: Copy,
        SendAll: FnMut(&CMessage) -> i32,
    {
        let Some(setup) = self.village_wars.get_mut(&war_number) else {
            return Ok(VillageWarEndReport::default());
        };
        let mut message = CMessage::new(0x7FE32);
        message.base_mut().add_long(war_number);
        let delivery = send_all(&message);
        let cleared_factions = setup.declaring_factions.len();
        setup.declaring_factions.clear();
        if !setup.is_every_week {
            return Ok(VillageWarEndReport {
                schedule_found: true,
                delivery: Some(delivery),
                cleared_factions,
                rearmed_after_weeks: None,
            });
        }

        let region_id = setup.war_region_id;
        let mut candidate_declare = setup.declare_time;
        let mut candidate_end = setup.end_time;
        let mut selected_weeks = None;
        for weeks in 1..=5 {
            candidate_declare
                .add_day(7)
                .map_err(VillageWarEndBlock::Arithmetic)?;
            candidate_end
                .add_day(7)
                .map_err(VillageWarEndBlock::Arithmetic)?;
            let conflicts = self.village_wars.iter().any(|(&other_id, other)| {
                other_id != war_number
                    && other.war_region_id == region_id
                    && (inside_closed(candidate_declare, other.declare_time, other.end_time)
                        || inside_closed(candidate_end, other.declare_time, other.end_time))
            });
            if !conflicts {
                selected_weeks = Some(weeks);
                break;
            }
        }

        if let Some(weeks) = selected_weeks {
            let setup = self
                .village_wars
                .get_mut(&war_number)
                .expect("weekly schedule не удаляется во время conflict scan");
            setup.rearm_after_weeks(weeks, timer, callbacks)?;
        }
        Ok(VillageWarEndReport {
            schedule_found: true,
            delivery: Some(delivery),
            cleared_factions,
            rearmed_after_weeks: selected_weeks,
        })
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "четыре legacy long и timer/context сохраняют исходную callback-границу"
    )]
    pub fn on_faction_win_village<Callback, Context>(
        &mut self,
        war_number: i32,
        war_region_id: i32,
        winner_faction_id: i32,
        _legacy_fourth_parameter: i32,
        timer: &mut CTimer<Callback>,
        callbacks: VillageWarCallbacks<Callback>,
        context: &mut Context,
    ) -> Result<VillageWarResultReport, VillageWarResultBlock<Context::Block>>
    where
        Callback: Copy,
        Context: VillageWarResultContext + ?Sized,
    {
        let mut report = VillageWarResultReport::default();
        if war_number == 0 {
            return Ok(report);
        }
        let Some(setup) = self.village_wars.get(&war_number).cloned() else {
            return Ok(report);
        };
        report.schedule_found = true;

        let Some(end_info_event_id) = setup.end_info_event_id else {
            return Err(VillageWarResultBlock::MissingEndInfoEventId { war_number });
        };
        report.end_info_event_killed = Some(timer.kill_time_event(end_info_event_id));
        let Some(end_event_id) = setup.end_event_id else {
            return Err(VillageWarResultBlock::MissingEndEventId { war_number });
        };
        report.end_event_killed = Some(timer.kill_time_event(end_event_id));

        let Some(war_region) = context
            .region(war_region_id)
            .map_err(VillageWarResultBlock::Context)?
        else {
            return Ok(report);
        };
        report.war_region_found = true;
        let Some(village_region) = context
            .region(setup.village_region_id)
            .map_err(VillageWarResultBlock::Context)?
        else {
            return Ok(report);
        };
        report.village_region_found = true;

        let village_owner_faction_id = context
            .region_owner_faction_id(setup.village_region_id)
            .map_err(VillageWarResultBlock::Context)?;
        let old_owner = if village_owner_faction_id > 0 {
            context
                .faction(village_owner_faction_id)
                .map_err(VillageWarResultBlock::Context)?
        } else {
            None
        };
        let war_end = self
            .on_village_war_end(war_number, war_region_id, timer, callbacks, |message| {
                context.send_all(message)
            })
            .map_err(VillageWarResultBlock::End)?;
        report.war_end = Some(war_end);

        let top_info = if winner_faction_id == 0 {
            if let Some(old_owner) = &old_owner {
                context
                    .clear_owned_city(village_owner_faction_id)
                    .map_err(VillageWarResultBlock::Context)?;
                let text = context
                    .format_world_string(
                        b"WS0290",
                        &[old_owner.name.as_slice(), village_region.name.as_slice()],
                    )
                    .map_err(VillageWarResultBlock::Context)?;
                context
                    .write_war_log(&text)
                    .map_err(VillageWarResultBlock::Context)?;
            }
            let text = context
                .format_world_string(b"WS0291", &[war_region.name.as_slice()])
                .map_err(VillageWarResultBlock::Context)?;
            context
                .send_organizing_info(&text)
                .map_err(VillageWarResultBlock::Context)?;
            context
                .write_war_log(&text)
                .map_err(VillageWarResultBlock::Context)?;
            text
        } else {
            let Some(winner) = context
                .faction(winner_faction_id)
                .map_err(VillageWarResultBlock::Context)?
            else {
                report.winner_faction_found = Some(false);
                return Ok(report);
            };
            report.winner_faction_found = Some(true);
            let winner_union_id = context
                .union_for_faction(winner_faction_id)
                .map_err(VillageWarResultBlock::Context)?;
            if winner.organizing_id != village_owner_faction_id {
                context
                    .refresh_owned_city_org(
                        setup.village_region_id,
                        winner_faction_id,
                        winner_union_id,
                    )
                    .map_err(VillageWarResultBlock::Context)?;
                context
                    .add_owned_city(winner_faction_id, setup.village_region_id)
                    .map_err(VillageWarResultBlock::Context)?;
                report.ownership_changed = true;

                let text = context
                    .format_world_string(
                        b"WS0292",
                        &[winner.name.as_slice(), village_region.name.as_slice()],
                    )
                    .map_err(VillageWarResultBlock::Context)?;
                context
                    .write_war_log(&text)
                    .map_err(VillageWarResultBlock::Context)?;
                if let Some(old_owner) = &old_owner {
                    context
                        .clear_owned_city(village_owner_faction_id)
                        .map_err(VillageWarResultBlock::Context)?;
                    let text = context
                        .format_world_string(
                            b"WS0290",
                            &[old_owner.name.as_slice(), village_region.name.as_slice()],
                        )
                        .map_err(VillageWarResultBlock::Context)?;
                    context
                        .write_war_log(&text)
                        .map_err(VillageWarResultBlock::Context)?;
                }
            }

            context
                .add_village_war_victor_count(winner_faction_id)
                .map_err(VillageWarResultBlock::Context)?;
            let text = context
                .format_world_string(
                    b"WS0293",
                    &[
                        winner.name.as_slice(),
                        war_region.name.as_slice(),
                        village_region.name.as_slice(),
                    ],
                )
                .map_err(VillageWarResultBlock::Context)?;
            context
                .send_organizing_info(&text)
                .map_err(VillageWarResultBlock::Context)?;
            context
                .write_war_log(&text)
                .map_err(VillageWarResultBlock::Context)?;
            context
                .format_world_string(
                    b"WS0294",
                    &[winner.name.as_slice(), war_region.name.as_slice()],
                )
                .map_err(VillageWarResultBlock::Context)?
        };

        context
            .send_top_info(-1, 1, 2, &top_info)
            .map_err(VillageWarResultBlock::Context)?;
        report.top_info_published = true;
        Ok(report)
    }

    fn apply_phase<Context: VillageWarPhaseContext + ?Sized>(
        &mut self,
        war_number: i32,
        state: ECityState,
        message_type: i32,
        world_string_id: &'static [u8],
        set_all_countries_warring: bool,
        context: &mut Context,
    ) -> Result<VillageWarPhaseReport, Context::Block> {
        let Some(setup) = self.village_wars.get_mut(&war_number) else {
            return Ok(VillageWarPhaseReport::default());
        };
        setup.region_state = state;
        let war_region_id = setup.war_region_id;
        let delivery = send_phase_message(context, message_type, war_number);
        let announcement_requested = war_region_id != 0;
        if announcement_requested {
            context.announce(VillageWarAnnouncement {
                war_number,
                war_region_id,
                world_string_id,
                set_all_countries_warring,
            })?;
        }
        Ok(VillageWarPhaseReport {
            schedule_found: true,
            delivery: Some(delivery),
            announcement_requested,
        })
    }

    fn conflicts_with(&self, candidate: &VillageWarSetup) -> bool {
        self.village_wars.values().any(|current| {
            current.war_region_id == candidate.war_region_id
                && (inside_closed(
                    candidate.declare_time,
                    current.declare_time,
                    current.end_time,
                ) || inside_closed(candidate.end_time, current.declare_time, current.end_time))
        })
    }
}

fn send_phase_message<Context: VillageWarPhaseContext + ?Sized>(
    context: &mut Context,
    message_type: i32,
    war_number: i32,
) -> i32 {
    let mut message = CMessage::new(message_type);
    message.base_mut().add_long(war_number);
    context.send_all(&message)
}

fn countdown_duration_ms(difference: TagTime) -> i32 {
    (u32::from(difference.second)
        .wrapping_add(u32::from(difference.minute).wrapping_mul(60))
        .wrapping_mul(1000)) as i32
}

fn append_i32(output: &mut Vec<u8>, value: i32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn append_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn append_timer_id(output: &mut Vec<u8>, event_id: Option<TimerId>) {
    append_u32(output, event_id.map_or(0, TimerId::get));
}

fn append_tag_time(output: &mut Vec<u8>, time: TagTime) {
    for field in [
        time.year,
        time.month,
        time.day_of_week,
        time.day,
        time.hour,
        time.minute,
        time.second,
        time.milliseconds,
    ] {
        output.extend_from_slice(&field.to_le_bytes());
    }
}

impl VillageWarSetup {
    fn has_valid_time_order(&self) -> bool {
        self.declare_time.legacy_lt(self.start_info_time)
            && self.start_info_time.legacy_lt(self.clear_player_time)
            && self.clear_player_time.legacy_lt(self.start_time)
            && self.start_time.legacy_lt(self.end_info_time)
            && self.end_info_time.legacy_lt(self.end_time)
    }

    fn register_initial_events<Callback: Copy>(
        &mut self,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: VillageWarCallbacks<Callback>,
    ) {
        if !self.end_time.legacy_gt(now) {
            return;
        }
        self.end_info_event_id = Some(timer.set_time_event(
            if self.end_info_time.legacy_ge(now) {
                self.end_info_time
            } else {
                now
            },
            callbacks.end_info,
            self.id,
        ));
        self.end_event_id = Some(timer.set_time_event(self.end_time, callbacks.end, self.id));
        if !self.start_time.legacy_gt(now) {
            self.region_state = ECityState::Fight;
            return;
        }
        self.clear_event_id =
            Some(timer.set_time_event(self.clear_player_time, callbacks.clear_player, self.id));
        self.start_event_id = Some(timer.set_time_event(self.start_time, callbacks.start, self.id));
        if self.declare_time.legacy_gt(now) {
            self.declare_event_id =
                Some(timer.set_time_event(self.declare_time, callbacks.declare, self.id));
        } else {
            self.region_state = ECityState::Duth;
        }
        self.start_info_event_id = Some(timer.set_time_event(
            if self.start_info_time.legacy_ge(now) {
                self.start_info_time
            } else {
                now
            },
            callbacks.start_info,
            self.id,
        ));
    }

    fn rearm_after_weeks<Callback: Copy>(
        &mut self,
        weeks: i32,
        timer: &mut CTimer<Callback>,
        callbacks: VillageWarCallbacks<Callback>,
    ) -> Result<(), VillageWarEndBlock> {
        let days = weeks * 7;
        self.declare_time
            .add_day(days)
            .map_err(VillageWarEndBlock::Arithmetic)?;
        self.declare_event_id =
            Some(timer.set_time_event(self.declare_time, callbacks.declare, self.id));
        self.start_info_time
            .add_day(days)
            .map_err(VillageWarEndBlock::Arithmetic)?;
        self.start_info_event_id =
            Some(timer.set_time_event(self.start_info_time, callbacks.start_info, self.id));
        self.clear_player_time
            .add_day(days)
            .map_err(VillageWarEndBlock::Arithmetic)?;
        self.clear_event_id =
            Some(timer.set_time_event(self.clear_player_time, callbacks.clear_player, self.id));
        self.start_time
            .add_day(days)
            .map_err(VillageWarEndBlock::Arithmetic)?;
        self.start_event_id = Some(timer.set_time_event(self.start_time, callbacks.start, self.id));
        self.end_info_time
            .add_day(days)
            .map_err(VillageWarEndBlock::Arithmetic)?;
        self.end_info_event_id =
            Some(timer.set_time_event(self.end_info_time, callbacks.end_info, self.id));
        self.end_time
            .add_day(days)
            .map_err(VillageWarEndBlock::Arithmetic)?;
        self.end_event_id = Some(timer.set_time_event(self.end_time, callbacks.end, self.id));
        self.region_state = ECityState::No;
        Ok(())
    }
}

fn read_weekly_village_schedule<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    id: i32,
    now: TagTime,
) -> Result<VillageWarSetup, VillageWarLoadError> {
    let war_region_id = next_war_i32(tokens, "lWarRegionID")?;
    let weekday = next_war_i32(tokens, "weekday")?;
    let hour = next_war_i32(tokens, "hour")?;
    let minute = next_war_i32(tokens, "minute")?;
    let second = next_war_i32(tokens, "second")?;
    let start_info_offset = next_war_i32(tokens, "WarStartInfoTime offset")?;
    let declare_offset = next_war_i32(tokens, "DeclarWarTime offset")?;
    let end_info_offset = next_war_i32(tokens, "WarEndInfoTime offset")?;
    let end_offset = next_war_i32(tokens, "WarEndTime offset")?;
    let clear_offset = next_war_i32(tokens, "ClearPlayerTime offset")?;
    let village_region_id = next_war_i32(tokens, "lVilRegionID")?;

    let delta = weekday - i32::from(now.day_of_week);
    let mut start_time = now;
    let _ = start_time.add_day(if delta < 0 { delta + 7 } else { delta })?;
    start_time.hour = hour as u16;
    start_time.minute = minute as u16;
    start_time.second = second as u16;
    let mut declare_time = shifted_minutes(start_time, declare_offset)?;
    if declare_time.legacy_lt(now) {
        let _ = start_time.add_day(7)?;
        declare_time = shifted_minutes(start_time, declare_offset)?;
    }
    new_village_schedule(
        id,
        war_region_id,
        village_region_id,
        start_time,
        declare_time,
        start_info_offset,
        end_info_offset,
        end_offset,
        clear_offset,
        true,
    )
}

fn read_single_village_schedule<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    id: i32,
) -> Result<VillageWarSetup, VillageWarLoadError> {
    let war_region_id = next_war_i32(tokens, "lWarRegionID")?;
    let start_time = TagTime::from_legacy_string(next_war_token(tokens, "WarStartTime")?)?;
    let start_info_offset = next_war_i32(tokens, "WarStartInfoTime offset")?;
    let declare_offset = next_war_i32(tokens, "DeclarWarTime offset")?;
    let end_info_offset = next_war_i32(tokens, "WarEndInfoTime offset")?;
    let end_offset = next_war_i32(tokens, "WarEndTime offset")?;
    let clear_offset = next_war_i32(tokens, "ClearPlayerTime offset")?;
    let village_region_id = next_war_i32(tokens, "lVilRegionID")?;
    let declare_time = shifted_minutes(start_time, declare_offset)?;
    new_village_schedule(
        id,
        war_region_id,
        village_region_id,
        start_time,
        declare_time,
        start_info_offset,
        end_info_offset,
        end_offset,
        clear_offset,
        false,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "аргументы буквально следуют полям одной INI-записи"
)]
fn new_village_schedule(
    id: i32,
    war_region_id: i32,
    village_region_id: i32,
    start_time: TagTime,
    declare_time: TagTime,
    start_info_offset: i32,
    end_info_offset: i32,
    end_offset: i32,
    clear_offset: i32,
    is_every_week: bool,
) -> Result<VillageWarSetup, VillageWarLoadError> {
    // Вместо конструктора по умолчанию, оставлявшего ID полей и событий
    // неинициализированными до записи парсером, Rust создаёт запись только с
    // уже известными полями. Это устраняет внутреннее UB, не меняя действующее
    // расписание, его Clone/присваивание или wire-снимок.
    Ok(VillageWarSetup {
        id,
        war_region_id,
        declare_event_id: None,
        declare_time,
        start_info_event_id: None,
        start_info_time: shifted_minutes(start_time, start_info_offset)?,
        start_event_id: None,
        start_time,
        end_info_event_id: None,
        end_info_time: shifted_minutes(start_time, end_info_offset)?,
        end_event_id: None,
        end_time: shifted_minutes(start_time, end_offset)?,
        clear_event_id: None,
        clear_player_time: shifted_minutes(start_time, clear_offset)?,
        village_region_id,
        region_state: ECityState::No,
        declaring_factions: Vec::new(),
        is_every_week,
    })
}

fn shifted_minutes(mut time: TagTime, offset: i32) -> Result<TagTime, TagTimeArithmeticBlock> {
    let _ = time.add_minute(offset)?;
    Ok(time)
}

fn inside_closed(value: TagTime, start: TagTime, end: TagTime) -> bool {
    value.legacy_ge(start) && value.legacy_le(end)
}

fn war_tokens(source: &[u8]) -> impl Iterator<Item = &[u8]> {
    source
        .split(u8::is_ascii_whitespace)
        .filter(|token| !token.is_empty())
}

fn next_war_token<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    field: &'static str,
) -> Result<&'a [u8], VillageWarLoadError> {
    tokens
        .next()
        .ok_or(VillageWarLoadError::MissingValue { field })
}

fn next_war_i32<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    field: &'static str,
) -> Result<i32, VillageWarLoadError> {
    std::str::from_utf8(next_war_token(tokens, field)?)
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or(VillageWarLoadError::InvalidValue { field })
}
