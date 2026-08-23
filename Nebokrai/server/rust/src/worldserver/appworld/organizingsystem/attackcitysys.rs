//! Владелец расписания городских войн `CAttackCitySys` WorldServer.
//!
//! `Initialize`, `GetCityState` и
//! `GetCityStateByWarNum`, clear/refresh callbacks
//! и фазовые start/end/declare/mass callbacks
//! действуют;
//! countdown и city end/weekly rearm
//! и result callback `OnFacWinCity`, а также заявка
//! `OnPlayerDeclareWar`, `Reload` и полный
//! `AddToByteArray` snapshot действуют. Источник контракта — точная пара WorldServer EXE/PDB.
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`; source
//!
//! Layout сохраняет static `map<long,tagAttackCityTime>`, а запись размером `0xBC`:
//! signed schedule/city IDs, восемь `tagTime`, восемь unsigned event IDs,
//! `eCityState`, ordered list faction IDs и signed weekly flag. `BTreeMap` и
//! `Vec` сохраняют наблюдаемый порядок; ещё не назначенные event IDs выражены
//! через `Option<TimerId>`, потому что старый reload безусловно читал их и мог
//! достигать неинициализированных значений у уже прошедших фаз.
//!
//! `Initialize` дважды сканирует один byte-token stream через точный `ReadTo`:
//! `#` для weekly и после reopen `*` для single schedules. Weekly rollover
//! проверяет `DeclarWarTime < now`; порядок строго равен `declare < start-info
//! < mass < clear < refresh < start < end-info < end`. Исходная overlap-
//! проверка сравнивает лишь два endpoints новой записи с closed interval уже
//! принятой записи. Поставочный `CityWarSys.ini` содержит восемь `#` и ни
//! одного `*`, поэтому его видимый раздел `Danci` оригинал не достигает.
//!
//! Rust регистрирует через `CTimer` те же calendar events и получает callback
//! keys от caller-а. Фазовые callbacks сохраняют `Duth/Mass/Fight/No`, Fight-
//! gate у end, затем World messages `0x7FE1F/20/21/23/24/25` с signed
//! war number. MainLoop передаёт callback-ам живые region, localization,
//! organizing-info, country, top-info и war-log owners через узкий контекст,
//! сохраняя их исходный порядок. Countdown игнорирует hours/days разности. City
//! end очищает заявки, закрывает country/enemy relations и пробует до пяти
//! следующих недель перед восемью ordered registrations. Faction snapshot
//! имеет формат signed war ID/count/ordered IDs и завершает decode значением
//! `true`. Active-state duplicate scan
//! и relation rebuild
//! сохраняют исходные gates, expansion faction/union и взаимный ordered add.
//! `OnFacWinCity` всегда пишет входной diagnostic, затем при точном совпадении
//! schedule/region убивает end events и вызывает оба end-owner-а. Neutral и
//! faction-result сохраняют порядок `WS0147..WS0153`, ownership refresh,
//! defence/offense counters, billboard и передачу master-а в country. Owner
//! region читается только в положительной winner-ветке; четвёртый входной
//! `long` после diagnostic заменяется вычисленным union ID. При same-owner
//! исходник безусловно разыменовывал nullable faction на, поэтому
//! safe Rust блокирует только эту границу, не назначая поведение старому UB.
//! `OnPlayerDeclareWar` сохраняет master/faction/schedule/region/level/state/
//! owner/union/owned-city/duplicate gates, затем дописывает virtual GetID,
//! рассылает `0x7FE35`, пересобирает enemy relations и публикует
//! `WS0103` либо `WS0145`, а также log `WS0146`. `m_btCountry` читается по
//! подтверждённому offset `+0x80`; только `0..=4` индексируют старую таблицу
//! country names. Level-rejection равен `WS0121/WS0144`, третий `long` не
//! читается. EXE возвращает `true` только после log; отказ —
//! `false`. Неиспользуемый nullable lookup прежнего owner-faction не имел
//! эффекта и удалён вместе с STL/string/SEH noise; DB-вызова здесь нет.
//! `Reload` убивает восемь schedule IDs в исходном нестандартном порядке и
//! затем внутри каждой итерации повторно убивает глобальный
//! `m_GetTodayTaxTime.lEventID`. После snapshot-а активные записи проходят
//! `OnCityWarEnd` и `0x7FE22`, затем `Initialize`; weekly end events, ставшие
//! промежуточными перед очисткой map, сознательно не отменяются. Отсутствующий
//! `Option<TimerId>` локально блокирует неизвестный старый оригинал ID.
//! возвращает `true` независимо от старого Initialize-result;
//! сам `Initialize` tax-event не регистрирует.
//! Полный snapshot начинает с 32-битного count, затем для каждой записи в
//! map-order копирует prefix `0xB0`, отдельный count и ordered faction IDs;
//! weekly flag за prefix не попадает. Четыре bytes allocator-base `std::list`
//! и ещё не назначенные event IDs в оригинале могли быть неинициализированы.
//! Они нормализованы к нулю: парный decoder из
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`,,
//! пересоздаёт list, отдельно читает count/IDs и event IDs не использует. Это
//! узкая downstream-compatible замена, не обещание старых residue bytes.
//! При отсутствии записи расписания старый `operator[]` создавал её по
//! умолчанию с недоказанными числовыми полями и временем; это остаётся явной
//! неизвестностью. Некорректное чтение, переполнение ID и недоказанная знаковая
//! календарная арифметика возвращают локальные ошибки.
//! STL/ifstream/SEH/allocator noise Rust-кода не имеет. Деструктор
//! `tagAttackCityTime` очищал только list заявившихся фракций; его заменяет
//! структурный Drop `declaring_factions: Vec<i32>`.

use std::collections::BTreeMap;

use crate::nets::networld::message::CMessage;
use crate::public::date::{TagTime, TagTimeArithmeticBlock, TagTimeParseBlock};
use crate::public::readwrite::read_to;
use crate::public::timer::{CTimer, TimerId};

use super::organizing::ECityState;

#[derive(Clone, Copy, Debug)]
pub(crate) struct AttackCityCallbacks<Callback> {
    pub(crate) declare: Callback,
    pub(crate) start_info: Callback,
    pub(crate) start: Callback,
    pub(crate) end_info: Callback,
    pub(crate) end: Callback,
    pub(crate) mass: Callback,
    pub(crate) clear_other_player: Callback,
    pub(crate) refresh_region: Callback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttackCityCallbackKind {
    Declare,
    StartInfo,
    Start,
    EndInfo,
    End,
    Mass,
    ClearOtherPlayer,
    RefreshRegion,
}

impl<Callback: PartialEq> AttackCityCallbacks<Callback> {
 /// Сопоставляет сработавший типизированный идентификатор с исходной фазой таймера.
    pub(crate) fn kind(&self, callback: &Callback) -> Option<AttackCityCallbackKind> {
        [
            (&self.declare, AttackCityCallbackKind::Declare),
            (&self.start_info, AttackCityCallbackKind::StartInfo),
            (&self.start, AttackCityCallbackKind::Start),
            (&self.end_info, AttackCityCallbackKind::EndInfo),
            (&self.end, AttackCityCallbackKind::End),
            (&self.mass, AttackCityCallbackKind::Mass),
            (
                &self.clear_other_player,
                AttackCityCallbackKind::ClearOtherPlayer,
            ),
            (&self.refresh_region, AttackCityCallbackKind::RefreshRegion),
        ]
        .into_iter()
        .find_map(|(candidate, kind)| (candidate == callback).then_some(kind))
    }
}

/// Внешние region/localization/organizing/country/log эффекты city phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttackCityPhaseEffect {
    RegionAnnouncement {
        war_number: i32,
        city_region_id: i32,
        notice_string_id: &'static [u8],
        log_string_id: &'static [u8],
        set_region_country_warring: bool,
    },
    EndLog {
        war_number: i32,
        log_string_id: &'static [u8],
    },
}

/// Контракт внешних эффектов фаз городской войны.
pub(crate) trait AttackCityPhaseContext {
    type Block;

 /// Синхронно повторяет `CMessage::SendAll`; старый return игнорировался.
    fn send_all(&mut self, message: &CMessage) -> i32;

 /// Region-вариант при живом регионе отправляет organizing-info с
 /// `-366/`, при флаге ставит country региона в war и пишет log;
 /// end-вариант форматирует только war number и пишет log.
    fn apply_effect(&mut self, effect: AttackCityPhaseEffect) -> Result<(), Self::Block>;
}

/// Наблюдаемый результат одного city phase callback-а.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AttackCityPhaseReport {
    pub(crate) schedule_found: bool,
    pub(crate) phase_applied: bool,
    pub(crate) delivery: Option<i32>,
    pub(crate) effect_requested: bool,
}

/// Параметры timer-2 top-info countdown-а городских войн.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AttackCityCountdownRequest {
    pub(crate) war_number: i32,
    pub(crate) city_region_id: i32,
    pub(crate) world_string_id: &'static [u8],
    pub(crate) duration_ms: i32,
}

pub(crate) trait AttackCityCountdownContext {
    type Block;

    fn region_exists(&mut self, region_id: i32) -> Result<bool, Self::Block>;

 /// Разрешает строку, добавляет timer-2 top-info и сразу рассылает её с
 /// возвращённым `AddOneTopInfo` ID.
    fn publish_countdown(&mut self, request: AttackCityCountdownRequest)
    -> Result<(), Self::Block>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AttackCityCountdownBlock<ContextBlock> {
    Context(ContextBlock),
    Arithmetic(TagTimeArithmeticBlock),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AttackCityCountdownReport {
    pub(crate) schedule_found: bool,
    pub(crate) region_found: bool,
    pub(crate) target_in_future: bool,
    pub(crate) phase_allowed: bool,
    pub(crate) duration_ms: Option<i32>,
}

/// Граница region/organizing owners для city-war enemy-list rebuild.
pub(crate) trait AttackCityEnemyRelationContext {
    type Block;

    fn clear_all_city_faction_relations(&mut self) -> Result<(), Self::Block>;

 /// Возвращает owner faction живого region либо `None` для всех оригинал gates.
    fn city_owner_faction_id(&mut self, city_region_id: i32) -> Result<Option<i32>, Self::Block>;

 /// Для свободной faction возвращает её organizing, для faction союза —
 /// ordered `GetAllFacs`; отсутствующие organizing дают пустой список.
    fn expand_faction_organizings(&mut self, faction_id: i32) -> Result<Vec<i32>, Self::Block>;

    fn add_city_war_enemy_organizing(
        &mut self,
        organizing_id: i32,
        enemy_organizing_id: i32,
    ) -> Result<(), Self::Block>;

    fn set_all_city_faction_enemy_changed(&mut self, changed: bool) -> Result<(), Self::Block>;

    fn update_all_city_enemy_faction_relations(&mut self) -> Result<(), Self::Block>;
}

/// Полный результат rebuild-а активных city-war relations.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AttackCityEnemyRelationReport {
    pub(crate) active_schedules: usize,
    pub(crate) directed_additions: usize,
}

/// Дополнительная country-граница `OnCityWarEnd`.
pub(crate) trait AttackCityWarEndContext: AttackCityEnemyRelationContext {
 /// При живом city region и найденной стране ставит `m_bIsWarring=false`.
    fn clear_region_country_warring_if_present(
        &mut self,
        city_region_id: i32,
    ) -> Result<(), Self::Block>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AttackCityWarEndBlock<ContextBlock> {
    MissingScheduleDefaultUnknown { war_number: i32 },
    Context(ContextBlock),
    Arithmetic(TagTimeArithmeticBlock),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AttackCityWarEndReport {
    pub(crate) cleared_factions: usize,
    pub(crate) rearmed_after_weeks: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AttackCityWarResultRegion {
    pub(crate) name: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AttackCityWarResultFaction {
    pub(crate) name: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttackCityWarResultFormatArgument<'a> {
    Text(&'a [u8]),
    Signed(i32),
}

/// Полная внешняя граница результата городской войны.
pub(crate) trait AttackCityWarResultContext: AttackCityWarEndContext {
    fn region(&mut self, region_id: i32) -> Result<Option<AttackCityWarResultRegion>, Self::Block>;

 /// Читает virtual owner getter только в положительной winner-ветке.
    fn region_owner_faction_id(&mut self, region_id: i32) -> Result<i32, Self::Block>;

    fn faction(
        &mut self,
        faction_id: i32,
    ) -> Result<Option<AttackCityWarResultFaction>, Self::Block>;

 /// Повторяет nullable `GetFactionOrganizing` без чтения virtual getters.
    fn faction_exists(&mut self, faction_id: i32) -> Result<bool, Self::Block>;

    fn union_for_faction(&mut self, faction_id: i32) -> Result<i32, Self::Block>;

    fn refresh_owned_city_org(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
    ) -> Result<(), Self::Block>;

    fn add_owned_city(&mut self, faction_id: i32, region_id: i32) -> Result<(), Self::Block>;

    fn clear_owned_city(&mut self, faction_id: i32) -> Result<(), Self::Block>;

    fn faction_master_id(&mut self, faction_id: i32) -> Result<i32, Self::Block>;

    fn add_defence_victor_count(&mut self, faction_id: i32) -> Result<(), Self::Block>;

    fn add_offense_victor_count(&mut self, faction_id: i32) -> Result<(), Self::Block>;

    fn stat_billboard(&mut self) -> Result<(), Self::Block>;

    fn faction_country(&mut self, faction_id: i32) -> Result<u8, Self::Block>;

    fn country_exists(&mut self, country_id: u8) -> Result<bool, Self::Block>;

    fn set_country_king_and_city(
        &mut self,
        country_id: u8,
        master_id: i32,
        city_region_id: i32,
    ) -> Result<(), Self::Block>;

 /// Форматирует `GetStringByID` в старую 256-byte `_sprintf` границу.
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[AttackCityWarResultFormatArgument<'_>],
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
pub(crate) enum AttackCityWarResultBlock<ContextBlock> {
    MissingEndInfoEventId { war_number: i32 },
    MissingEndEventId { war_number: i32 },
    MissingCurrentOwnerForDefence { owner_faction_id: i32 },
    End(AttackCityWarEndBlock<ContextBlock>),
    Context(ContextBlock),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AttackCityWarResultReport {
    pub(crate) schedule_found: bool,
    pub(crate) region_matches_schedule: bool,
    pub(crate) region_found: bool,
    pub(crate) end_info_event_killed: Option<bool>,
    pub(crate) end_event_killed: Option<bool>,
    pub(crate) war_end: Option<AttackCityWarEndReport>,
    pub(crate) end_delivery: Option<i32>,
    pub(crate) winner_faction_found: Option<bool>,
    pub(crate) ownership_changed: bool,
    pub(crate) result_log_written: bool,
}

/// Внешние organizing/region/localization owners подачи city-war заявки.
pub(crate) trait AttackCityApplicationContext: AttackCityEnemyRelationContext {
    fn faction_master_for_player(&mut self, player_id: i32) -> Result<i32, Self::Block>;

    fn faction_exists(&mut self, faction_id: i32) -> Result<bool, Self::Block>;

    fn region_exists(&mut self, region_id: i32) -> Result<bool, Self::Block>;

    fn attack_city_min_level(&mut self) -> Result<i32, Self::Block>;

    fn faction_level(&mut self, faction_id: i32) -> Result<i32, Self::Block>;

    fn region_owner_faction_id(&mut self, region_id: i32) -> Result<i32, Self::Block>;

    fn union_for_faction(&mut self, faction_id: i32) -> Result<i32, Self::Block>;

    fn faction_owned_city_count(&mut self, faction_id: i32) -> Result<usize, Self::Block>;

    fn already_declared_for_village_war(&mut self, faction_id: i32) -> Result<bool, Self::Block>;

    fn faction_organizing_id(&mut self, faction_id: i32) -> Result<i32, Self::Block>;

    fn faction_name(&mut self, faction_id: i32) -> Result<Vec<u8>, Self::Block>;

    fn region_name(&mut self, region_id: i32) -> Result<Vec<u8>, Self::Block>;

    fn region_country(&mut self, region_id: i32) -> Result<u8, Self::Block>;

 /// Локализует title/text по порядку и отправляет player-targeted overload
 /// с legacy arguments `-1, -1, `.
    fn send_level_rejection(
        &mut self,
        player_id: i32,
        title_string_id: &'static [u8],
        text_string_id: &'static [u8],
    ) -> Result<(), Self::Block>;

 /// Возвращает `WS0145(country, faction, country)` для country `0..=4`,
 /// иначе исходный fallback `WS0103`.
    fn format_declaration_notice(
        &mut self,
        country_id: u8,
        faction_name: &[u8],
    ) -> Result<Vec<u8>, Self::Block>;

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[AttackCityWarResultFormatArgument<'_>],
    ) -> Result<Vec<u8>, Self::Block>;

    fn send_organizing_info(&mut self, text: &[u8]) -> Result<(), Self::Block>;

    fn write_war_log(&mut self, text: &[u8]) -> Result<(), Self::Block>;

    fn send_all(&mut self, message: &CMessage) -> i32;
}

/// Доказанные gates и уже выполненные эффекты одной city-war заявки.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AttackCityApplicationReport {
    pub(crate) master_faction_id: Option<i32>,
    pub(crate) faction_found: bool,
    pub(crate) schedule_found: bool,
    pub(crate) region_found: bool,
    pub(crate) level_rejected: bool,
    pub(crate) accepted: bool,
    pub(crate) delivery: Option<i32>,
    pub(crate) enemy_relations: Option<AttackCityEnemyRelationReport>,
}

#[derive(Clone, Debug)]
pub(crate) struct AttackCityTime {
    id: i32,
    city_region_id: i32,
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
    mass_event_id: Option<TimerId>,
    mass_time: TagTime,
    clear_event_id: Option<TimerId>,
    clear_player_time: TagTime,
    refresh_event_id: Option<TimerId>,
    refresh_region_time: TagTime,
    region_state: ECityState,
    declaring_factions: Vec<i32>,
    is_every_week: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AttackCityLoadReport {
    pub(crate) resource_found: bool,
    pub(crate) accepted: u32,
    pub(crate) ignored_time_order: u32,
    pub(crate) ignored_time_conflict: u32,
    pub(crate) ignored_overdue: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttackCityLoadError {
    ResourceMissing,
    MissingValue { field: &'static str },
    InvalidValue { field: &'static str },
    TimeParse(TagTimeParseBlock),
    Arithmetic(TagTimeArithmeticBlock),
    ScheduleIdOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttackCityReloadEvent {
    Declare,
    Mass,
    ClearPlayer,
    RefreshRegion,
    StartInfo,
    Start,
    EndInfo,
    End,
    TodayTax,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AttackCityReloadBlock<ContextBlock> {
    MissingEventId {
        war_number: i32,
        event: AttackCityReloadEvent,
    },
    End {
        war_number: i32,
        block: AttackCityWarEndBlock<ContextBlock>,
    },
    Load(AttackCityLoadError),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AttackCityReloadReport {
    pub(crate) previous_schedules: usize,
    pub(crate) kill_requests: usize,
    pub(crate) killed_events: usize,
    pub(crate) active_schedules_ended: usize,
    pub(crate) end_broadcasts: usize,
    pub(crate) load: AttackCityLoadReport,
}

impl From<TagTimeArithmeticBlock> for AttackCityLoadError {
    fn from(value: TagTimeArithmeticBlock) -> Self {
        Self::Arithmetic(value)
    }
}

impl From<TagTimeParseBlock> for AttackCityLoadError {
    fn from(value: TagTimeParseBlock) -> Self {
        Self::TimeParse(value)
    }
}

#[derive(Default)]
pub(crate) struct CAttackCitySys {
    attacks: BTreeMap<i32, AttackCityTime>,
}

impl CAttackCitySys {
    pub(crate) const fn new() -> Self {
        Self {
            attacks: BTreeMap::new(),
        }
    }

 /// Загружает расписания и регистрирует действующие calendar events.
 ///
 /// Как bool-owner, сначала очищает registry, а отсутствующий resource
 /// возвращает false caller-у. Поэтому `CGame::Init` публикует свой
 /// fail-log и завершает init вместо прежнего ошибочного Rust success-path.
    pub(crate) fn initialize<Callback: Copy>(
        &mut self,
        source: Option<&[u8]>,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: AttackCityCallbacks<Callback>,
    ) -> Result<AttackCityLoadReport, AttackCityLoadError> {
        self.attacks.clear();
        let Some(source) = source else {
            return Err(AttackCityLoadError::ResourceMissing);
        };
        let mut report = AttackCityLoadReport {
            resource_found: true,
            ..AttackCityLoadReport::default()
        };
        let mut next_id = 0i32;

        let mut weekly_tokens = war_tokens(source);
        while read_to(&mut weekly_tokens, b"#") {
            next_id = next_id
                .checked_add(1)
                .ok_or(AttackCityLoadError::ScheduleIdOverflow)?;
            let candidate = read_weekly_attack_schedule(&mut weekly_tokens, next_id, now)?;
            if !candidate.has_valid_time_order() {
                report.ignored_time_order = report.ignored_time_order.wrapping_add(1);
            } else if self.conflicts_with(&candidate) {
                report.ignored_time_conflict = report.ignored_time_conflict.wrapping_add(1);
            } else {
                self.attacks.insert(next_id, candidate);
                report.accepted = report.accepted.wrapping_add(1);
            }
        }

        let mut single_tokens = war_tokens(source);
        while read_to(&mut single_tokens, b"*") {
            next_id = next_id
                .checked_add(1)
                .ok_or(AttackCityLoadError::ScheduleIdOverflow)?;
            let candidate = read_single_attack_schedule(&mut single_tokens, next_id)?;
            if candidate.declare_time.legacy_lt(now) {
                report.ignored_overdue = report.ignored_overdue.wrapping_add(1);
            } else if !candidate.has_valid_time_order() {
                report.ignored_time_order = report.ignored_time_order.wrapping_add(1);
            } else if self.conflicts_with(&candidate) {
                report.ignored_time_conflict = report.ignored_time_conflict.wrapping_add(1);
            } else {
                self.attacks.insert(next_id, candidate);
                report.accepted = report.accepted.wrapping_add(1);
            }
        }

        for setup in self.attacks.values_mut() {
            setup.register_initial_events(now, timer, callbacks);
        }
        Ok(report)
    }

 /// Отменяет старые events, завершает активные копии и заново загружает файл.
    #[allow(
        clippy::too_many_arguments,
        reason = "source/time/timer/callbacks/tax/context/send сохраняют исходные owners"
    )]
    pub(crate) fn reload<Callback, Context, SendAll>(
        &mut self,
        source: Option<&[u8]>,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: AttackCityCallbacks<Callback>,
        today_tax_event_id: Option<TimerId>,
        context: &mut Context,
        mut send_all: SendAll,
    ) -> Result<AttackCityReloadReport, AttackCityReloadBlock<Context::Block>>
    where
        Callback: Copy,
        Context: AttackCityWarEndContext + ?Sized,
        SendAll: FnMut(&CMessage) -> i32,
    {
        let mut report = AttackCityReloadReport {
            previous_schedules: self.attacks.len(),
            ..AttackCityReloadReport::default()
        };

        for (&war_number, setup) in &self.attacks {
            let events = [
                (AttackCityReloadEvent::Declare, setup.declare_event_id),
                (AttackCityReloadEvent::Mass, setup.mass_event_id),
                (AttackCityReloadEvent::ClearPlayer, setup.clear_event_id),
                (AttackCityReloadEvent::RefreshRegion, setup.refresh_event_id),
                (AttackCityReloadEvent::StartInfo, setup.start_info_event_id),
                (AttackCityReloadEvent::Start, setup.start_event_id),
                (AttackCityReloadEvent::EndInfo, setup.end_info_event_id),
                (AttackCityReloadEvent::End, setup.end_event_id),
                (AttackCityReloadEvent::TodayTax, today_tax_event_id),
            ];
            for (event, event_id) in events {
                let Some(event_id) = event_id else {
                    return Err(AttackCityReloadBlock::MissingEventId { war_number, event });
                };
                report.kill_requests = report.kill_requests.wrapping_add(1);
                if timer.kill_time_event(event_id) {
                    report.killed_events = report.killed_events.wrapping_add(1);
                }
            }
        }

        let previous = self.attacks.clone();
        for (&war_number, setup) in &previous {
            if setup.region_state != ECityState::No {
                self.on_city_war_end(war_number, timer, callbacks, context)
                    .map_err(|block| AttackCityReloadBlock::End { war_number, block })?;
                report.active_schedules_ended = report.active_schedules_ended.wrapping_add(1);
                self.on_city_war_end_to_game_server(
                    setup.city_region_id,
                    war_number,
                    &mut send_all,
                );
                report.end_broadcasts = report.end_broadcasts.wrapping_add(1);
            }
        }

        report.load = self
            .initialize(source, now, timer, callbacks)
            .map_err(AttackCityReloadBlock::Load)?;
        Ok(report)
    }

 /// Дописывает полный World->Game snapshot в исходном map/field порядке.
    pub(crate) fn add_to_byte_array(&self, output: &mut Vec<u8>) -> bool {
        append_u32(output, self.attacks.len() as u32);
        for setup in self.attacks.values() {
            append_i32(output, setup.id);
            append_i32(output, setup.city_region_id);
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
            append_timer_id(output, setup.mass_event_id);
            append_tag_time(output, setup.mass_time);
            append_timer_id(output, setup.clear_event_id);
            append_tag_time(output, setup.clear_player_time);
            append_timer_id(output, setup.refresh_event_id);
            append_tag_time(output, setup.refresh_region_time);
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

 /// Возвращает state первой записи в map-order для города и closed периода войны.
    pub(crate) fn get_city_state_at(&self, region_id: i32, now: TagTime) -> ECityState {
        self.attacks
            .values()
            .find(|setup| {
                setup.city_region_id == region_id
                    && now.legacy_ge(setup.declare_time)
                    && now.legacy_le(setup.end_time)
            })
            .map_or(ECityState::No, |setup| setup.region_state)
    }

    pub(crate) fn get_city_state(&self, region_id: i32) -> ECityState {
        self.get_city_state_at(region_id, TagTime::local_now())
    }

    pub(crate) fn get_city_state_by_war_num(&self, war_number: i32) -> ECityState {
        self.attacks
            .get(&war_number)
            .map_or(ECityState::No, |setup| setup.region_state)
    }

 /// Дописывает один faction-list snapshot для GameServer.
    pub(crate) fn update_apply_war_factions_to_game_server(
        &self,
        war_number: i32,
        output: &mut Vec<u8>,
    ) -> bool {
        append_i32(output, war_number);
        let factions = self
            .attacks
            .get(&war_number)
            .map_or(&[][..], |setup| setup.declaring_factions.as_slice());
        output.extend_from_slice(&(factions.len() as u32).to_le_bytes());
        for &faction_id in factions {
            append_i32(output, faction_id);
        }
        true
    }

 /// Проверяет faction во всех не завершённых city-war заявках.
    pub(crate) fn is_already_declared_for_war(&self, faction_id: i32) -> bool {
        self.attacks.values().any(|setup| {
            setup.region_state != ECityState::No && setup.declaring_factions.contains(&faction_id)
        })
    }

 /// Принимает city-war заявку только после всей исходной цепочки gates.
    pub(crate) fn on_player_declare_war<Context>(
        &mut self,
        player_id: i32,
        war_number: i32,
        _legacy_third_parameter: i32,
        context: &mut Context,
    ) -> Result<AttackCityApplicationReport, Context::Block>
    where
        Context: AttackCityApplicationContext + ?Sized,
    {
        let mut report = AttackCityApplicationReport::default();
        let faction_id = context.faction_master_for_player(player_id)?;
        report.master_faction_id = Some(faction_id);
        if faction_id <= 0 || !context.faction_exists(faction_id)? {
            return Ok(report);
        }
        report.faction_found = true;

        let Some(setup) = self.attacks.get(&war_number).cloned() else {
            return Ok(report);
        };
        report.schedule_found = true;
        if setup.city_region_id == 0 || !context.region_exists(setup.city_region_id)? {
            return Ok(report);
        }
        report.region_found = true;

        let minimum_level = context.attack_city_min_level()?;
        if context.faction_level(faction_id)? < minimum_level {
            context.send_level_rejection(player_id, b"WS0121", b"WS0144")?;
            report.level_rejected = true;
            return Ok(report);
        }
        if setup.region_state == ECityState::No {
            return Ok(report);
        }

        let owner_faction_id = context.region_owner_faction_id(setup.city_region_id)?;
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
            || context.already_declared_for_village_war(faction_id)?
        {
            return Ok(report);
        }

        let organizing_id = context.faction_organizing_id(faction_id)?;
        self.attacks
            .get_mut(&war_number)
            .expect("schedule проверен и не удаляется внутри owner-а")
            .declaring_factions
            .push(organizing_id);
        let mut payload = Vec::new();
        self.update_apply_war_factions_to_game_server(war_number, &mut payload);
        let mut message = CMessage::new(0x7FE35);
        message.base_mut().add(&payload);
        report.delivery = Some(context.send_all(&message));
        report.enemy_relations = Some(self.update_city_all_faction_enemy_relation(context)?);

        let faction_name = context.faction_name(faction_id)?;
        let country_id = context.region_country(setup.city_region_id)?;
        let notice = context.format_declaration_notice(country_id, &faction_name)?;
        context.send_organizing_info(&notice)?;

        let city_name = context.region_name(setup.city_region_id)?;
        let log = context.format_world_string(
            b"WS0146",
            &[
                AttackCityWarResultFormatArgument::Signed(war_number),
                AttackCityWarResultFormatArgument::Text(faction_name.as_slice()),
                AttackCityWarResultFormatArgument::Text(city_name.as_slice()),
            ],
        )?;
        context.write_war_log(&log)?;
        report.accepted = true;
        Ok(report)
    }

 /// Строит city-war enemy pairs для всех активных schedules.
    pub(crate) fn initial_city_all_faction_enemy_relation<Context>(
        &self,
        context: &mut Context,
    ) -> Result<AttackCityEnemyRelationReport, Context::Block>
    where
        Context: AttackCityEnemyRelationContext + ?Sized,
    {
        context.clear_all_city_faction_relations()?;
        let mut report = AttackCityEnemyRelationReport::default();
        for setup in self
            .attacks
            .values()
            .filter(|setup| setup.region_state != ECityState::No)
        {
            report.active_schedules = report.active_schedules.wrapping_add(1);
            report.directed_additions =
                report
                    .directed_additions
                    .wrapping_add(Self::set_city_war_enemy_factions(
                        setup.city_region_id,
                        &setup.declaring_factions,
                        context,
                    )?);
        }
        Ok(report)
    }

 /// Сбрасывает changed flags, пересобирает и публикует все city relations.
    pub(crate) fn update_city_all_faction_enemy_relation<Context>(
        &self,
        context: &mut Context,
    ) -> Result<AttackCityEnemyRelationReport, Context::Block>
    where
        Context: AttackCityEnemyRelationContext + ?Sized,
    {
        context.set_all_city_faction_enemy_changed(false)?;
        let report = self.initial_city_all_faction_enemy_relation(context)?;
        context.update_all_city_enemy_faction_relations()?;
        Ok(report)
    }

 /// Рассылает `0x7FE22`; первый legacy parameter не читался.
    pub(crate) fn on_city_war_end_to_game_server<SendAll>(
        &self,
        _legacy_city_region_id: i32,
        war_number: i32,
        mut send_all: SendAll,
    ) -> i32
    where
        SendAll: FnMut(&CMessage) -> i32,
    {
        let mut message = CMessage::new(0x7FE22);
        message.base_mut().add_long(war_number);
        send_all(&message)
    }

    pub(crate) fn on_declare_war<Context: AttackCityPhaseContext + ?Sized>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> Result<AttackCityPhaseReport, Context::Block> {
        self.apply_region_phase(
            war_number,
            ECityState::Duth,
            0x7FE1F,
            b"WS0138",
            b"WS0139",
            true,
            context,
        )
    }

    pub(crate) fn on_mass<Context: AttackCityPhaseContext + ?Sized>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> Result<AttackCityPhaseReport, Context::Block> {
        self.apply_region_phase(
            war_number,
            ECityState::Mass,
            0x7FE23,
            b"WS0140",
            b"WS0141",
            false,
            context,
        )
    }

    pub(crate) fn on_attack_city_start<Context: AttackCityPhaseContext + ?Sized>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> Result<AttackCityPhaseReport, Context::Block> {
        self.apply_region_phase(
            war_number,
            ECityState::Fight,
            0x7FE20,
            b"WS0135",
            b"WS0136",
            false,
            context,
        )
    }

 /// End действует только из точного `CIS_Fight` состояния.
    pub(crate) fn on_attack_city_end<Context: AttackCityPhaseContext + ?Sized>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> Result<AttackCityPhaseReport, Context::Block> {
        let Some(setup) = self.attacks.get_mut(&war_number) else {
            return Ok(AttackCityPhaseReport::default());
        };
        if setup.region_state != ECityState::Fight {
            return Ok(AttackCityPhaseReport {
                schedule_found: true,
                ..AttackCityPhaseReport::default()
            });
        }
        setup.region_state = ECityState::No;
        let delivery = send_phase_message(context, 0x7FE21, war_number);
        context.apply_effect(AttackCityPhaseEffect::EndLog {
            war_number,
            log_string_id: b"WS0137",
        })?;
        Ok(AttackCityPhaseReport {
            schedule_found: true,
            phase_applied: true,
            delivery: Some(delivery),
            effect_requested: true,
        })
    }

    pub(crate) fn on_clear_other_player<Context: AttackCityPhaseContext + ?Sized>(
        &self,
        war_number: i32,
        context: &mut Context,
    ) -> AttackCityPhaseReport {
        self.send_existing_schedule_phase(war_number, 0x7FE24, context)
    }

    pub(crate) fn on_refresh_region<Context: AttackCityPhaseContext + ?Sized>(
        &self,
        war_number: i32,
        context: &mut Context,
    ) -> AttackCityPhaseReport {
        self.send_existing_schedule_phase(war_number, 0x7FE25, context)
    }

    pub(crate) fn on_attack_city_start_info<Context: AttackCityCountdownContext + ?Sized>(
        &self,
        war_number: i32,
        now: TagTime,
        context: &mut Context,
    ) -> Result<AttackCityCountdownReport, AttackCityCountdownBlock<Context::Block>> {
        self.publish_countdown(
            war_number,
            now,
            |setup| setup.start_time,
            |_| true,
            b"WS0142",
            context,
        )
    }

    pub(crate) fn on_attack_city_end_info<Context: AttackCityCountdownContext + ?Sized>(
        &self,
        war_number: i32,
        now: TagTime,
        context: &mut Context,
    ) -> Result<AttackCityCountdownReport, AttackCityCountdownBlock<Context::Block>> {
        self.publish_countdown(
            war_number,
            now,
            |setup| setup.end_time,
            |setup| setup.region_state == ECityState::Fight,
            b"WS0143",
            context,
        )
    }

    fn publish_countdown<Context, Target, PhaseGate>(
        &self,
        war_number: i32,
        now: TagTime,
        target: Target,
        phase_gate: PhaseGate,
        world_string_id: &'static [u8],
        context: &mut Context,
    ) -> Result<AttackCityCountdownReport, AttackCityCountdownBlock<Context::Block>>
    where
        Context: AttackCityCountdownContext + ?Sized,
        Target: FnOnce(&AttackCityTime) -> TagTime,
        PhaseGate: FnOnce(&AttackCityTime) -> bool,
    {
        let Some(current) = self.attacks.get(&war_number) else {
            return Ok(AttackCityCountdownReport::default());
        };
        let city_region_id = current.city_region_id;
        if city_region_id == 0
            || !context
                .region_exists(city_region_id)
                .map_err(AttackCityCountdownBlock::Context)?
        {
            return Ok(AttackCityCountdownReport {
                schedule_found: true,
                ..AttackCityCountdownReport::default()
            });
        }

        let copied = self
            .attacks
            .get(&war_number)
            .cloned()
            .expect("ID найден до неизменяемого повторного lookup");
        let target = target(&copied);
        if target.legacy_le(now) {
            return Ok(AttackCityCountdownReport {
                schedule_found: true,
                region_found: true,
                ..AttackCityCountdownReport::default()
            });
        }
 // End-info существует только для уже начавшейся Fight-фазы; start-info
 // проходит ту же позицию с безусловным предикатом.
        if !phase_gate(&copied) {
            return Ok(AttackCityCountdownReport {
                schedule_found: true,
                region_found: true,
                target_in_future: true,
                ..AttackCityCountdownReport::default()
            });
        }
        let difference = target
            .get_time_difference(now)
            .map_err(AttackCityCountdownBlock::Arithmetic)?;
        let duration_ms = countdown_duration_ms(difference);
        context
            .publish_countdown(AttackCityCountdownRequest {
                war_number,
                city_region_id,
                world_string_id,
                duration_ms,
            })
            .map_err(AttackCityCountdownBlock::Context)?;
        Ok(AttackCityCountdownReport {
            schedule_found: true,
            region_found: true,
            target_in_future: true,
            phase_allowed: true,
            duration_ms: Some(duration_ms),
        })
    }

 /// Выполняет полный действующий end/relation/weekly-rearm порядок.
    pub(crate) fn on_city_war_end<Callback, Context>(
        &mut self,
        war_number: i32,
        timer: &mut CTimer<Callback>,
        callbacks: AttackCityCallbacks<Callback>,
        context: &mut Context,
    ) -> Result<AttackCityWarEndReport, AttackCityWarEndBlock<Context::Block>>
    where
        Callback: Copy,
        Context: AttackCityWarEndContext + ?Sized,
    {
 // Старый `operator[]` создавал отсутствующую запись по умолчанию, но
 // начальные значения её числовых полей и времени не доказаны.
        let (cleared_factions, city_region_id, is_every_week, declare_time, end_time) = {
            let Some(setup) = self.attacks.get_mut(&war_number) else {
                return Err(AttackCityWarEndBlock::MissingScheduleDefaultUnknown { war_number });
            };
            let cleared_factions = setup.declaring_factions.len();
            setup.declaring_factions.clear();
            (
                cleared_factions,
                setup.city_region_id,
                setup.is_every_week,
                setup.declare_time,
                setup.end_time,
            )
        };
        context
            .clear_region_country_warring_if_present(city_region_id)
            .map_err(AttackCityWarEndBlock::Context)?;
        self.update_city_all_faction_enemy_relation(context)
            .map_err(AttackCityWarEndBlock::Context)?;
        if !is_every_week {
            return Ok(AttackCityWarEndReport {
                cleared_factions,
                rearmed_after_weeks: None,
            });
        }

        let mut candidate_declare = declare_time;
        let mut candidate_end = end_time;
        let mut selected_weeks = None;
        for weeks in 1..=5 {
            candidate_declare
                .add_day(7)
                .map_err(AttackCityWarEndBlock::Arithmetic)?;
            candidate_end
                .add_day(7)
                .map_err(AttackCityWarEndBlock::Arithmetic)?;
            let conflicts = self.attacks.iter().any(|(&other_id, other)| {
                other_id != war_number
                    && other.city_region_id == city_region_id
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
                .attacks
                .get_mut(&war_number)
                .expect("weekly schedule не удаляется во время conflict scan");
            setup.rearm_after_weeks(weeks, timer, callbacks)?;
        }
        Ok(AttackCityWarEndReport {
            cleared_factions,
            rearmed_after_weeks: selected_weeks,
        })
    }

 /// Завершает досрочный city-war результат и сохраняет ownership/country order.
    #[allow(
        clippy::too_many_arguments,
        reason = "четыре legacy long и timer/context сохраняют исходную callback-границу"
    )]
    pub(crate) fn on_faction_win_city<Callback, Context>(
        &mut self,
        war_number: i32,
        city_region_id: i32,
        winner_faction_id: i32,
        reported_union_id: i32,
        timer: &mut CTimer<Callback>,
        callbacks: AttackCityCallbacks<Callback>,
        context: &mut Context,
    ) -> Result<AttackCityWarResultReport, AttackCityWarResultBlock<Context::Block>>
    where
        Callback: Copy,
        Context: AttackCityWarResultContext + ?Sized,
    {
        let diagnostic = format!(
            "lWarNum:{war_number},lWarRegionID:{city_region_id},lFactionID:{winner_faction_id},lUnionID:{reported_union_id}"
        );
        context
            .write_war_log(diagnostic.as_bytes())
            .map_err(AttackCityWarResultBlock::Context)?;

        let mut report = AttackCityWarResultReport::default();
        if war_number == 0 || city_region_id == 0 {
            return Ok(report);
        }
        let Some(setup) = self.attacks.get(&war_number).cloned() else {
            return Ok(report);
        };
        report.schedule_found = true;
        if setup.city_region_id != city_region_id {
            return Ok(report);
        }
        report.region_matches_schedule = true;
        let Some(city_region) = context
            .region(city_region_id)
            .map_err(AttackCityWarResultBlock::Context)?
        else {
            return Ok(report);
        };
        report.region_found = true;

        let Some(end_info_event_id) = setup.end_info_event_id else {
            return Err(AttackCityWarResultBlock::MissingEndInfoEventId { war_number });
        };
        report.end_info_event_killed = Some(timer.kill_time_event(end_info_event_id));
        let Some(end_event_id) = setup.end_event_id else {
            return Err(AttackCityWarResultBlock::MissingEndEventId { war_number });
        };
        report.end_event_killed = Some(timer.kill_time_event(end_event_id));

        let war_end = self
            .on_city_war_end(war_number, timer, callbacks, context)
            .map_err(AttackCityWarResultBlock::End)?;
        report.war_end = Some(war_end);
        report.end_delivery = Some(self.on_city_war_end_to_game_server(
            city_region_id,
            war_number,
            |message| context.send_all(message),
        ));

        let result_log = if winner_faction_id <= 0 {
            let text = context
                .format_world_string(
                    b"WS0147",
                    &[AttackCityWarResultFormatArgument::Text(
                        city_region.name.as_slice(),
                    )],
                )
                .map_err(AttackCityWarResultBlock::Context)?;
            context
                .send_organizing_info(&text)
                .map_err(AttackCityWarResultBlock::Context)?;

            let text = context
                .format_world_string(
                    b"WS0148",
                    &[AttackCityWarResultFormatArgument::Text(
                        city_region.name.as_slice(),
                    )],
                )
                .map_err(AttackCityWarResultBlock::Context)?;
            context
                .send_top_info(-1, 1, 2, &text)
                .map_err(AttackCityWarResultBlock::Context)?;
            context
                .send_organizing_info(&text)
                .map_err(AttackCityWarResultBlock::Context)?;

            context
                .format_world_string(
                    b"WS0149",
                    &[
                        AttackCityWarResultFormatArgument::Signed(war_number),
                        AttackCityWarResultFormatArgument::Text(city_region.name.as_slice()),
                    ],
                )
                .map_err(AttackCityWarResultBlock::Context)?
        } else {
            let owner_faction_id = context
                .region_owner_faction_id(city_region_id)
                .map_err(AttackCityWarResultBlock::Context)?;
            let Some(winner) = context
                .faction(winner_faction_id)
                .map_err(AttackCityWarResultBlock::Context)?
            else {
                report.winner_faction_found = Some(false);
                return Ok(report);
            };
            report.winner_faction_found = Some(true);
            let winner_union_id = context
                .union_for_faction(winner_faction_id)
                .map_err(AttackCityWarResultBlock::Context)?;
            let old_owner_exists = context
                .faction_exists(owner_faction_id)
                .map_err(AttackCityWarResultBlock::Context)?;
            let mut winner_master_id = 0;

            if owner_faction_id == winner_faction_id {
                if !old_owner_exists {
 //,: null result
 // `GetFactionOrganizing(owner)` разыменовывался без gate.
                    return Err(AttackCityWarResultBlock::MissingCurrentOwnerForDefence {
                        owner_faction_id,
                    });
                }
                context
                    .add_defence_victor_count(owner_faction_id)
                    .map_err(AttackCityWarResultBlock::Context)?;
            } else {
                context
                    .refresh_owned_city_org(city_region_id, winner_faction_id, winner_union_id)
                    .map_err(AttackCityWarResultBlock::Context)?;
                context
                    .add_owned_city(winner_faction_id, city_region_id)
                    .map_err(AttackCityWarResultBlock::Context)?;
                if old_owner_exists {
                    context
                        .clear_owned_city(owner_faction_id)
                        .map_err(AttackCityWarResultBlock::Context)?;
                }
                winner_master_id = context
                    .faction_master_id(winner_faction_id)
                    .map_err(AttackCityWarResultBlock::Context)?;
                context
                    .add_offense_victor_count(winner_faction_id)
                    .map_err(AttackCityWarResultBlock::Context)?;
                report.ownership_changed = true;
            }
            context
                .stat_billboard()
                .map_err(AttackCityWarResultBlock::Context)?;

            let text = context
                .format_world_string(
                    b"WS0150",
                    &[
                        AttackCityWarResultFormatArgument::Text(city_region.name.as_slice()),
                        AttackCityWarResultFormatArgument::Text(winner.name.as_slice()),
                        AttackCityWarResultFormatArgument::Text(city_region.name.as_slice()),
                        AttackCityWarResultFormatArgument::Text(city_region.name.as_slice()),
                    ],
                )
                .map_err(AttackCityWarResultBlock::Context)?;
            context
                .send_organizing_info(&text)
                .map_err(AttackCityWarResultBlock::Context)?;

            let text = context
                .format_world_string(
                    b"WS0151",
                    &[
                        AttackCityWarResultFormatArgument::Text(winner.name.as_slice()),
                        AttackCityWarResultFormatArgument::Text(city_region.name.as_slice()),
                    ],
                )
                .map_err(AttackCityWarResultBlock::Context)?;
            context
                .send_top_info(-1, 1, 2, &text)
                .map_err(AttackCityWarResultBlock::Context)?;

            let result_log = context
                .format_world_string(
                    b"WS0152",
                    &[
                        AttackCityWarResultFormatArgument::Signed(war_number),
                        AttackCityWarResultFormatArgument::Text(winner.name.as_slice()),
                        AttackCityWarResultFormatArgument::Text(city_region.name.as_slice()),
                        AttackCityWarResultFormatArgument::Text(city_region.name.as_slice()),
                    ],
                )
                .map_err(AttackCityWarResultBlock::Context)?;

            if winner_master_id != 0 {
                let country_id = context
                    .faction_country(winner_faction_id)
                    .map_err(AttackCityWarResultBlock::Context)?;
                if context
                    .country_exists(country_id)
                    .map_err(AttackCityWarResultBlock::Context)?
                {
                    let text = context
                        .format_world_string(
                            b"WS0153",
                            &[AttackCityWarResultFormatArgument::Text(
                                winner.name.as_slice(),
                            )],
                        )
                        .map_err(AttackCityWarResultBlock::Context)?;
                    context
                        .send_organizing_info(&text)
                        .map_err(AttackCityWarResultBlock::Context)?;
                    context
                        .set_country_king_and_city(country_id, winner_master_id, city_region_id)
                        .map_err(AttackCityWarResultBlock::Context)?;
                }
            }
            result_log
        };

        context
            .write_war_log(&result_log)
            .map_err(AttackCityWarResultBlock::Context)?;
        report.result_log_written = true;
        Ok(report)
    }

 /// Добавляет взаимные city-war enemy IDs для одной city/faction заявки.
    pub(crate) fn set_city_war_enemy_factions<Context>(
        city_region_id: i32,
        declaring_factions: &[i32],
        context: &mut Context,
    ) -> Result<usize, Context::Block>
    where
        Context: AttackCityEnemyRelationContext + ?Sized,
    {
        if declaring_factions.is_empty() {
            return Ok(0);
        }
        let Some(owner_faction_id) = context.city_owner_faction_id(city_region_id)? else {
            return Ok(0);
        };
        let owner_organizings = context.expand_faction_organizings(owner_faction_id)?;
        let mut attacker_organizings = Vec::new();
        for &faction_id in declaring_factions {
            attacker_organizings.extend(context.expand_faction_organizings(faction_id)?);
        }

        let mut directed_additions = 0usize;
        for &attacker_id in &attacker_organizings {
            for &owner_id in &owner_organizings {
                context.add_city_war_enemy_organizing(attacker_id, owner_id)?;
                directed_additions = directed_additions.wrapping_add(1);
                context.add_city_war_enemy_organizing(owner_id, attacker_id)?;
                directed_additions = directed_additions.wrapping_add(1);
            }
        }
        Ok(directed_additions)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "поля буквально задают state, wire и две локализованные фазы callback-а"
    )]
    fn apply_region_phase<Context: AttackCityPhaseContext + ?Sized>(
        &mut self,
        war_number: i32,
        state: ECityState,
        message_type: i32,
        notice_string_id: &'static [u8],
        log_string_id: &'static [u8],
        set_region_country_warring: bool,
        context: &mut Context,
    ) -> Result<AttackCityPhaseReport, Context::Block> {
        let Some(setup) = self.attacks.get_mut(&war_number) else {
            return Ok(AttackCityPhaseReport::default());
        };
        setup.region_state = state;
        let city_region_id = setup.city_region_id;
        let delivery = send_phase_message(context, message_type, war_number);
        let effect_requested = city_region_id != 0;
        if effect_requested {
            context.apply_effect(AttackCityPhaseEffect::RegionAnnouncement {
                war_number,
                city_region_id,
                notice_string_id,
                log_string_id,
                set_region_country_warring,
            })?;
        }
        Ok(AttackCityPhaseReport {
            schedule_found: true,
            phase_applied: true,
            delivery: Some(delivery),
            effect_requested,
        })
    }

    fn send_existing_schedule_phase<Context: AttackCityPhaseContext + ?Sized>(
        &self,
        war_number: i32,
        message_type: i32,
        context: &mut Context,
    ) -> AttackCityPhaseReport {
        if !self.attacks.contains_key(&war_number) {
            return AttackCityPhaseReport::default();
        }
        AttackCityPhaseReport {
            schedule_found: true,
            phase_applied: true,
            delivery: Some(send_phase_message(context, message_type, war_number)),
            effect_requested: false,
        }
    }

    fn conflicts_with(&self, candidate: &AttackCityTime) -> bool {
        self.attacks.values().any(|current| {
            current.city_region_id == candidate.city_region_id
                && (inside_closed(
                    candidate.declare_time,
                    current.declare_time,
                    current.end_time,
                ) || inside_closed(candidate.end_time, current.declare_time, current.end_time))
        })
    }
}

fn send_phase_message<Context: AttackCityPhaseContext + ?Sized>(
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

impl AttackCityTime {
    fn has_valid_time_order(&self) -> bool {
        self.declare_time.legacy_lt(self.start_info_time)
            && self.start_info_time.legacy_lt(self.mass_time)
            && self.mass_time.legacy_lt(self.clear_player_time)
            && self.clear_player_time.legacy_lt(self.refresh_region_time)
            && self.refresh_region_time.legacy_lt(self.start_time)
            && self.start_time.legacy_lt(self.end_info_time)
            && self.end_info_time.legacy_lt(self.end_time)
    }

    fn register_initial_events<Callback: Copy>(
        &mut self,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: AttackCityCallbacks<Callback>,
    ) {
        if !self.end_time.legacy_ge(now) {
            return;
        }
        self.end_event_id = Some(timer.set_time_event(self.end_time, callbacks.end, self.id));
        self.end_info_event_id = Some(timer.set_time_event(
            if self.end_info_time.legacy_ge(now) {
                self.end_info_time
            } else {
                now
            },
            callbacks.end_info,
            self.id,
        ));
        if !self.start_time.legacy_ge(now) {
            self.region_state = ECityState::Fight;
            return;
        }
        self.clear_event_id = Some(timer.set_time_event(
            self.clear_player_time,
            callbacks.clear_other_player,
            self.id,
        ));
        self.start_event_id = Some(timer.set_time_event(self.start_time, callbacks.start, self.id));
        self.refresh_event_id =
            Some(timer.set_time_event(self.refresh_region_time, callbacks.refresh_region, self.id));
        self.start_info_event_id = Some(timer.set_time_event(
            if self.start_info_time.legacy_ge(now) {
                self.start_info_time
            } else {
                now
            },
            callbacks.start_info,
            self.id,
        ));
        if !self.mass_time.legacy_ge(now) {
            self.region_state = ECityState::Mass;
            return;
        }
        self.mass_event_id = Some(timer.set_time_event(self.mass_time, callbacks.mass, self.id));
        if self.declare_time.legacy_ge(now) {
            self.declare_event_id =
                Some(timer.set_time_event(self.declare_time, callbacks.declare, self.id));
        } else {
            self.region_state = ECityState::Duth;
        }
    }

    fn rearm_after_weeks<Callback: Copy, ContextBlock>(
        &mut self,
        weeks: i32,
        timer: &mut CTimer<Callback>,
        callbacks: AttackCityCallbacks<Callback>,
    ) -> Result<(), AttackCityWarEndBlock<ContextBlock>> {
        let days = weeks * 7;
        self.declare_time
            .add_day(days)
            .map_err(AttackCityWarEndBlock::Arithmetic)?;
        self.declare_event_id =
            Some(timer.set_time_event(self.declare_time, callbacks.declare, self.id));
        self.start_info_time
            .add_day(days)
            .map_err(AttackCityWarEndBlock::Arithmetic)?;
        self.start_info_event_id =
            Some(timer.set_time_event(self.start_info_time, callbacks.start_info, self.id));
        self.mass_time
            .add_day(days)
            .map_err(AttackCityWarEndBlock::Arithmetic)?;
        self.mass_event_id = Some(timer.set_time_event(self.mass_time, callbacks.mass, self.id));
        self.clear_player_time
            .add_day(days)
            .map_err(AttackCityWarEndBlock::Arithmetic)?;
        self.clear_event_id = Some(timer.set_time_event(
            self.clear_player_time,
            callbacks.clear_other_player,
            self.id,
        ));
        self.refresh_region_time
            .add_day(days)
            .map_err(AttackCityWarEndBlock::Arithmetic)?;
        self.refresh_event_id =
            Some(timer.set_time_event(self.refresh_region_time, callbacks.refresh_region, self.id));
        self.start_time
            .add_day(days)
            .map_err(AttackCityWarEndBlock::Arithmetic)?;
        self.start_event_id = Some(timer.set_time_event(self.start_time, callbacks.start, self.id));
        self.end_info_time
            .add_day(days)
            .map_err(AttackCityWarEndBlock::Arithmetic)?;
        self.end_info_event_id =
            Some(timer.set_time_event(self.end_info_time, callbacks.end_info, self.id));
        self.end_time
            .add_day(days)
            .map_err(AttackCityWarEndBlock::Arithmetic)?;
        self.end_event_id = Some(timer.set_time_event(self.end_time, callbacks.end, self.id));
        self.region_state = ECityState::No;
        Ok(())
    }
}

fn read_weekly_attack_schedule<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    id: i32,
    now: TagTime,
) -> Result<AttackCityTime, AttackCityLoadError> {
    let city_region_id = next_war_i32(tokens, "lCityRegionID")?;
    let weekday = next_war_i32(tokens, "weekday")?;
    let hour = next_war_i32(tokens, "hour")?;
    let minute = next_war_i32(tokens, "minute")?;
    let second = next_war_i32(tokens, "second")?;
    let start_info_offset = next_war_i32(tokens, "AttackCityStartInfoTime offset")?;
    let declare_offset = next_war_i32(tokens, "DeclarWarTime offset")?;
    let end_info_offset = next_war_i32(tokens, "AttackCityEndInfoTime offset")?;
    let end_offset = next_war_i32(tokens, "AttackCityEndTime offset")?;
    let mass_offset = next_war_i32(tokens, "MassTime offset")?;
    let clear_offset = next_war_i32(tokens, "ClearPlayerTime offset")?;
    let refresh_offset = next_war_i32(tokens, "RefreshRegionTime offset")?;

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
    new_attack_schedule(
        id,
        city_region_id,
        start_time,
        declare_time,
        start_info_offset,
        end_info_offset,
        end_offset,
        mass_offset,
        clear_offset,
        refresh_offset,
        true,
    )
}

fn read_single_attack_schedule<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    id: i32,
) -> Result<AttackCityTime, AttackCityLoadError> {
    let city_region_id = next_war_i32(tokens, "lCityRegionID")?;
    let start_time = TagTime::from_legacy_string(next_war_token(tokens, "AttackCityStartTime")?)?;
    let start_info_offset = next_war_i32(tokens, "AttackCityStartInfoTime offset")?;
    let declare_offset = next_war_i32(tokens, "DeclarWarTime offset")?;
    let end_info_offset = next_war_i32(tokens, "AttackCityEndInfoTime offset")?;
    let end_offset = next_war_i32(tokens, "AttackCityEndTime offset")?;
    let mass_offset = next_war_i32(tokens, "MassTime offset")?;
    let clear_offset = next_war_i32(tokens, "ClearPlayerTime offset")?;
    let refresh_offset = next_war_i32(tokens, "RefreshRegionTime offset")?;
    let declare_time = shifted_minutes(start_time, declare_offset)?;
    new_attack_schedule(
        id,
        city_region_id,
        start_time,
        declare_time,
        start_info_offset,
        end_info_offset,
        end_offset,
        mass_offset,
        clear_offset,
        refresh_offset,
        false,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "аргументы буквально следуют полям одной INI-записи"
)]
fn new_attack_schedule(
    id: i32,
    city_region_id: i32,
    start_time: TagTime,
    declare_time: TagTime,
    start_info_offset: i32,
    end_info_offset: i32,
    end_offset: i32,
    mass_offset: i32,
    clear_offset: i32,
    refresh_offset: i32,
    is_every_week: bool,
) -> Result<AttackCityTime, AttackCityLoadError> {
    Ok(AttackCityTime {
        id,
        city_region_id,
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
        mass_event_id: None,
        mass_time: shifted_minutes(start_time, mass_offset)?,
        clear_event_id: None,
        clear_player_time: shifted_seconds(start_time, clear_offset)?,
        refresh_event_id: None,
        refresh_region_time: shifted_seconds(start_time, refresh_offset)?,
        region_state: ECityState::No,
        declaring_factions: Vec::new(),
        is_every_week,
    })
}

fn shifted_minutes(mut time: TagTime, offset: i32) -> Result<TagTime, TagTimeArithmeticBlock> {
    let _ = time.add_minute(offset)?;
    Ok(time)
}

fn shifted_seconds(mut time: TagTime, offset: i32) -> Result<TagTime, TagTimeArithmeticBlock> {
    let _ = time.add_second(offset)?;
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
) -> Result<&'a [u8], AttackCityLoadError> {
    tokens
        .next()
        .ok_or(AttackCityLoadError::MissingValue { field })
}

fn next_war_i32<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    field: &'static str,
) -> Result<i32, AttackCityLoadError> {
    std::str::from_utf8(next_war_token(tokens, field)?)
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or(AttackCityLoadError::InvalidValue { field })
}
