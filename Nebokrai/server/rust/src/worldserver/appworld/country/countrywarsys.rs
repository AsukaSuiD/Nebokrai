//! WorldServer-владелец country-war state `CountryWarSys`.
//!
//! `AddToByteArray` RVA `0x0008F800`, `player_declare` RVA `0x00091B00`,
//! `on_war_start/on_war_end` RVA `0x00090EC0/0x00091100`,
//! `end_war` RVA `0x0008F890`, пять phase callbacks RVA
//! `0x0008F490/0x0008FA00/0x0008FB40/0x0008FC80/0x00091530`,
//! `on_flag_destory` (исходное PDB-написание) RVA `0x00091E00`, `initialize`
//! RVA `0x00092220`, top-info callbacks RVA `0x000916C0/0x000918E0` и
//! `reload` RVA `0x00092DF0` имеют статус `IMPLEMENTED`; остальной корпус
//! ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара `WorldServer/Nworldserver.exe +
//! WorldServer/WorldServer.pdb`, исходник `appworld/country/countrywarsys.cpp`.
//!
//! PDB/static map хранит `CountryWarRegion` размером `0x0C`: clear-byte с
//! padding, signed defend и attack country. `BTreeMap` сохраняет map-order.
//! Для каждой записи с двумя ненулевыми сторонами callback сначала сбрасывает
//! clear-byte, затем ищет живой region и обе страны. При успехе он рассылает
//! `0x7FF22` с low byte входной страны; full signed equality выбирает defender
//! result `2/1` либо attacker result `1/2`. После optional `WS0105/WS0106`
//! форматирования стороны всегда обнуляются и country-info `0x7FA03` уходит
//! даже при пустом тексте. Потерянный Ghidra stack-key region lookup и порядок
//! state/message/result/clear подтверждены exact EXE `0x00491E69..0x00492142`.
//! Region, localization, country-map и network owners остаются явной context-
//! границей; concrete adapter теперь подключён к `ProcessMessage(0x60318)`.
//! Неизвестность в этих владельцах сохраняет уже выполненные предыдущие эффекты.
//! Переполнение исходного 256-byte `_sprintf` не воспроизводится: нормальный
//! output сохраняется, oversized localization безопасно ограничивается 255
//! байтами под C-string NUL как внутренний UB без доказанного gameplay-эффекта.
//! `player_declare` подтверждён exact `0x00491B00..0x00491DFD`: online-player,
//! чужая target-country, последовательные `IsKing/IsMinister(5)`, первый
//! свободный region, живой `pRegion`, затем проверки только defend-country.
//! При успехе state меняется до `0x7FF1F`, результаты обеих рассылок
//! игнорируются, лишний lookup `WS0103` перезаписывается `WS0104`, и функция
//! возвращает true. Linux-донор добавлял source/tail gates, fallback региона и
//! rollback при отказе очереди; эти полезные, но неоригинальные политики сюда
//! не перенесены. 512-byte `_sprintf` overflow безопасно ограничен нормальным
//! C-string payload в 511 байт без изменения штатного результата.
//!
//! `initialize` подтверждён RAW и точным disassembly
//! `0x00492220..0x00492DCA`. Он очищает только `_country_wars`, читает восемь
//! пар `label + signed long`, затем последовательно сканирует две секции `#`
//! через общий `<end>`-ограничитель. `_war_regions` заранее не очищается, а
//! каждая встреченная запись полностью обнуляет соответствующее состояние.
//! Вторая секция имеет машинную странность: stack key обнуляется один раз и
//! никогда не увеличивается, поэтому все допустимые времена пишутся под ID `0`
//! и последняя запись заменяет предыдущую. Linux-донор выдумывал возрастающие
//! ID; это исправлено по EXE. `TagTime` сохраняет исходный colon/`atoi` parser
//! и минутную арифметику, `BTreeMap`, byte slices и `CTimer` заменяют только
//! `ifstream`, STL и singleton plumbing. Неполные/нечисловые восемь параметров
//! в оригинале оставляли чтение неинициализированного stack `long`; Rust
//! локально блокирует такой malformed input вместо выдуманного значения.
//!
//! Регистрация событий также буквально сохраняет порядок EXE. Просроченный
//! `EndTime` оставляет первый clear-event на `ClearTime`, ставит второй на
//! текущее время и теряет ID первого. Event `DeclarEnd` ошибочно записывается
//! в поле `DeclarBeginEventID`, а достижимый `DeclarBegin` перезаписывает его;
//! отдельное поле `DeclarEndEventID` остаётся неизвестным. `Option<TimerId>`
//! выражает эту constructor/stack-неизвестность без нулевой заглушки. Логи
//! missing-file и восьми проверок порядка передаются существующему World
//! log-owner-у в месте вызова.
//!
//! `end_war` сохраняет ещё одно отличие от Linux-донора: exact EXE обнуляет
//! только defender/attacker, не трогая `state_clear`, и затем безусловно
//! рассылает `0x7FF1D`. `reload` в map-order безусловно читает и передаёт в
//! `KillTimeEvent` девять ID, затем вызывает `end_war -> initialize`.
//! `Option<TimerId>` не пропускает отсутствующее поле как donor helper
//! `KillEvent(0)`: typed block возвращается в точке первого недоказанного ID и
//! сохраняет счётчики уже выполненных kill side effects. Return рассылки
//! `0x7FF1D` оригинал не проверял; typed report хранит полный `Result`, не
//! сворачивая ошибку очереди в придуманный signed код.
//!
//! Phase callbacks сохраняют точный side-effect порядок. `DeclareBegin`
//! сначала обходит country IDs `1..=4`, назначая war-result `0` только живым
//! странам, затем рассылает `0x7FF17` и публикует `WS0095`. Exact
//! `0x00491565..0x0049159F` подтверждает один и тот же stack-byte как ключ
//! `find/operator[]`, исправляя ложное раздвоение переменных в RAW. Остальные
//! пары: `DeclareEnd = 0x7FF18/WS0096`, `PrepareBegin = 0x7FF19/WS0097`,
//! `PrepareEnd = 0x7FF1A/WS0098`; clear callback только рассылает `0x7FF1E`.
//! Timer parameter во всех пяти функциях не читается. Lookup null превращается
//! в пустую строку до старого no-argument `_sprintf`; форматирование и
//! 256-byte UB-граница остаются у явного context-а, а нормальные bytes не
//! требуют собственного formatter-а.
//!
//! `on_war_start_info/on_war_end_info` делают observable `operator[]` по
//! входному war ID до проверки времени; отсутствующий key поэтому вставляет
//! value-initialized zero schedule. Только target строго позже local now
//! достигает публикации. Duration вычисляется exact как
//! `(difference.minute * 60 + difference.second) * 1000` с 32-битным
//! wrapping и намеренно игнорирует hour/day. После `WS0099/WS0100` lookup и
//! форматирования concrete `CCountryHandler::AddOneTopInfo(2, duration, text)`
//! вызывается до concrete `SendTopInfoToClient` с возвращённым ID. Exact wire
//! `0x7FA04` теперь принадлежит `CCountryHandler`, а delivery сохранён в report.
//!
//! `on_war_start` подтверждён exact `0x00490EC0..0x004910F7`: сначала
//! безусловный `0x7FF1B`, затем map-order обход записей с двумя ненулевыми
//! сторонами. Для каждой такой записи clear-byte становится `1` до region
//! lookup; отсутствующий/null region пропускает только `WS0092`, не откатывая
//! state. Exact `0x00490F4A..0x00490F86` использует один region ID и для
//! `find`, и для `operator[]`, исправляя ложный stack-key RAW. В `_sprintf`
//! передавались именно `szCountryName[country][0x40]`, поэтому concrete
//! adapter теперь форматирует именами стран, а не их числовыми ID. Старый
//! 256-byte overflow безопасно ограничен 255 видимыми байтами.
//!
//! `on_war_end` подтверждён exact `0x00491100..0x0049152B`. После `0x7FF1C`
//! он обрабатывает только `state_clear && defend != 0 && attack != 0`: живой
//! region получает `WS0093`, затем результаты существующих defender и attacker
//! сбрасываются в этом порядке. Независимо от region lookup запись после этого
//! очищается целиком. В конце observable `operator[]` вставляет нулевой schedule
//! для отсутствующего war ID и, если `ClearTime > now`, публикует `WS0094` тем
//! же минутно-секундным duration, `AddOneTopInfo` и exact `0x7FA04` owner-ом.
//! Девять callback-token-ов, зарегистрированных `initialize`, теперь
//! распознаются `WorldTimerHandler` по тем же значениям и исполняются внутри
//! `CTimer::Run`; входной signed parameter остаётся war ID. Это заменяет только
//! глобальные static function/singleton lookup: неизвестные callback-и по-
//! прежнему передаются внешнему dispatcher-у, а country-war события остаются
//! one-shot без выдуманной повторной регистрации.
//!
//! Snapshot намеренно сохраняет layout World EXE: `state_clear + 3 bytes
//! padding`, затем defender и attacker. Парный Game EXE RVA `0x000EBD60`
//! трактует те же 12 bytes как defender, attacker, `state_clear + padding` —
//! это подтверждённое несовпадение поставленных бинарников, а не повод молча
//! менять World wire. Неинициализированный padding старого World нормализован
//! нулями: он не несёт семантики Miracle и не должен утекать в сеть.

use std::collections::BTreeMap;

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::public::date::{TagTime, TagTimeArithmeticBlock, TagTimeParseBlock};
use crate::public::readwrite::read_to;
use crate::public::timer::{CTimer, TimerId};

#[derive(Clone, Copy, Debug)]
pub(crate) struct CountryWarCallbacks<Callback> {
    pub(crate) clear: Callback,
    pub(crate) declare_begin: Callback,
    pub(crate) declare_end: Callback,
    pub(crate) prepare_begin: Callback,
    pub(crate) prepare_end: Callback,
    pub(crate) start: Callback,
    pub(crate) end: Callback,
    pub(crate) start_info: Callback,
    pub(crate) end_info: Callback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarCallbackKind {
    Clear,
    DeclareBegin,
    DeclareEnd,
    PrepareBegin,
    PrepareEnd,
    Start,
    End,
    StartInfo,
    EndInfo,
}

impl<Callback: PartialEq> CountryWarCallbacks<Callback> {
    /// Сопоставляет invocation с тем же typed token, который был передан при
    /// регистрации; `PartialEq` заменяет только сравнение callback pointer-а.
    pub(crate) fn kind(&self, callback: Callback) -> Option<CountryWarCallbackKind> {
        if callback == self.clear {
            Some(CountryWarCallbackKind::Clear)
        } else if callback == self.declare_begin {
            Some(CountryWarCallbackKind::DeclareBegin)
        } else if callback == self.declare_end {
            Some(CountryWarCallbackKind::DeclareEnd)
        } else if callback == self.prepare_begin {
            Some(CountryWarCallbackKind::PrepareBegin)
        } else if callback == self.prepare_end {
            Some(CountryWarCallbackKind::PrepareEnd)
        } else if callback == self.start {
            Some(CountryWarCallbackKind::Start)
        } else if callback == self.end {
            Some(CountryWarCallbackKind::End)
        } else if callback == self.start_info {
            Some(CountryWarCallbackKind::StartInfo)
        } else if callback == self.end_info {
            Some(CountryWarCallbackKind::EndInfo)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarLoadNotice {
    PrepareBeginNotBeforePrepareEnd,
    DeclareBeginNotBeforePrepareBegin,
    DeclareBeginNotBeforeDeclareEnd,
    DeclareBeginNotBeforeBegin,
    BeginNotBeforeEnd,
    InfoBeginNotBeforeBegin,
    InfoEndNotBeforeEnd,
    EndNotBeforeClear,
}

impl CountryWarLoadNotice {
    pub(crate) const fn legacy_text(self) -> &'static [u8] {
        match self {
            Self::PrepareBeginNotBeforePrepareEnd => {
                b"PrepareBeginTime >= PrepareEndTime, Ignore!"
            }
            Self::DeclareBeginNotBeforePrepareBegin => {
                b"DeclarBeginTime >= PrepareBeginTime, Ignore!"
            }
            Self::DeclareBeginNotBeforeDeclareEnd => {
                b"DeclarBeginTime >= DeclarEndTime, Ignore!"
            }
            Self::DeclareBeginNotBeforeBegin => b"DeclarBeginTime >= BeginTime, Ignore!",
            Self::BeginNotBeforeEnd => b"BeginTime >= EndTime, Ignore!",
            Self::InfoBeginNotBeforeBegin => b"InfoBeginTime >= BeginTime, Ignore!",
            Self::InfoEndNotBeforeEnd => b"InfoEndTime >= EndTime, Ignore!",
            Self::EndNotBeforeClear => b"EndTime >= ClearTime, Ignore!",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarLoadError {
    MissingValue { field: &'static str },
    InvalidValue { field: &'static str },
    TimeParse(TagTimeParseBlock),
    Arithmetic(TagTimeArithmeticBlock),
}

impl From<TagTimeParseBlock> for CountryWarLoadError {
    fn from(value: TagTimeParseBlock) -> Self {
        Self::TimeParse(value)
    }
}

impl From<TagTimeArithmeticBlock> for CountryWarLoadError {
    fn from(value: TagTimeArithmeticBlock) -> Self {
        Self::Arithmetic(value)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CountryWarLoadReport {
    pub(crate) resource_found: bool,
    pub(crate) legacy_result: bool,
    pub(crate) region_records: u32,
    pub(crate) schedule_records: u32,
    pub(crate) accepted_records: u32,
    pub(crate) notices: Vec<CountryWarLoadNotice>,
    pub(crate) registered_events: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryWarEndReport {
    pub(crate) reset_regions: usize,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarReloadEvent {
    PrepareBegin,
    PrepareEnd,
    DeclareBegin,
    DeclareEnd,
    InfoBegin,
    Begin,
    InfoEnd,
    End,
    Clear,
}

#[derive(Debug)]
pub(crate) struct CountryWarReloadReport {
    pub(crate) previous_schedules: usize,
    pub(crate) kill_requests: u32,
    pub(crate) killed_events: u32,
    pub(crate) end_war: CountryWarEndReport,
    pub(crate) load: CountryWarLoadReport,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryWarReloadBlock {
    MissingEventId {
        war_id: i32,
        event: CountryWarReloadEvent,
        kill_requests: u32,
        killed_events: u32,
    },
    Load {
        source: CountryWarLoadError,
        kill_requests: u32,
        killed_events: u32,
        end_war: CountryWarEndReport,
    },
}

impl Default for CountryWarLoadReport {
    fn default() -> Self {
        Self {
            resource_found: false,
            legacy_result: false,
            region_records: 0,
            schedule_records: 0,
            accepted_records: 0,
            notices: Vec::new(),
            registered_events: 0,
        }
    }
}

#[derive(Clone, Debug)]
struct CountryWarTime {
    // Exact EXE записывает event DeclarEnd в поле DeclarBeginEventID и при
    // достижимом DeclarBegin затем перезаписывает его вторым ID.
    declare_begin_event_id: Option<TimerId>,
    declare_begin_time: TagTime,
    declare_end_event_id: Option<TimerId>,
    declare_end_time: TagTime,
    prepare_begin_event_id: Option<TimerId>,
    prepare_begin_time: TagTime,
    prepare_end_event_id: Option<TimerId>,
    prepare_end_time: TagTime,
    start_info_event_id: Option<TimerId>,
    start_info_time: TagTime,
    start_event_id: Option<TimerId>,
    start_time: TagTime,
    end_info_event_id: Option<TimerId>,
    end_info_time: TagTime,
    end_event_id: Option<TimerId>,
    end_time: TagTime,
    clear_event_id: Option<TimerId>,
    clear_time: TagTime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryWarRegion {
    pub(crate) state_clear: bool,
    pub(crate) defend_country: i32,
    pub(crate) attack_country: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountryWarVictoryRegion {
    pub(crate) name: Vec<u8>,
}

pub(crate) trait CountryWarVictoryContext {
    type Block;

    /// Повторяет `s_mapRegionList.find/operator[]` и non-null `pRegion` gate.
    fn region(&mut self, region_id: i32) -> Result<Option<CountryWarVictoryRegion>, Self::Block>;

    /// Повторяет отдельный `GetCountry(low byte)`; ID `0` даёт false.
    fn country_exists(&mut self, country: u8) -> Result<bool, Self::Block>;

    /// Синхронно повторяет `CMessage::SendAll`; старый return игнорировался.
    fn send_all(&mut self, message: &CMessage) -> i32;

    /// Пишет result уже доказанно существующей стране.
    fn set_country_war_result(&mut self, country: u8, result: i32) -> Result<(), Self::Block>;

    /// Повторяет `GetStringByID` и старую 256-byte `_sprintf` границу.
    fn format_victory_notice(
        &mut self,
        string_id: &'static [u8],
        attack_country: i32,
        defend_country: i32,
        region_name: &[u8],
    ) -> Result<Vec<u8>, Self::Block>;

    /// Вызывает concrete `CCountryHandler::send_info_to_client` с исходными
    /// title `-366` и color `0xFFFF0000`.
    fn send_country_info(
        &mut self,
        text: &[u8],
        title: u32,
        color: u32,
    ) -> Result<i32, Self::Block>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryWarVictoryReport {
    pub(crate) active_regions: usize,
    pub(crate) flag_deliveries: Vec<i32>,
    pub(crate) result_pairs: usize,
    pub(crate) formatted_notices: usize,
    pub(crate) info_deliveries: Vec<i32>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryWarStartReport {
    pub(crate) broadcast_delivery: i32,
    pub(crate) active_regions: usize,
    pub(crate) started_regions: Vec<i32>,
    pub(crate) formatted_notices: usize,
    pub(crate) info_deliveries: Vec<i32>,
}

#[derive(Debug)]
pub(crate) enum CountryWarStartBlock<ContextBlock> {
    Region {
        region_id: i32,
        report: CountryWarStartReport,
        source: ContextBlock,
    },
    FormatNotice {
        region_id: i32,
        report: CountryWarStartReport,
        source: ContextBlock,
    },
    SendInfo {
        region_id: i32,
        report: CountryWarStartReport,
        source: ContextBlock,
    },
}

#[derive(Debug, Default)]
pub(crate) struct CountryWarFinishReport {
    pub(crate) broadcast_delivery: i32,
    pub(crate) active_regions: usize,
    pub(crate) ended_regions: Vec<i32>,
    pub(crate) formatted_notices: usize,
    pub(crate) reset_country_results: Vec<u8>,
    pub(crate) info_deliveries: Vec<i32>,
    pub(crate) clear_top_info: Option<CountryWarTopInfoReport>,
}

#[derive(Debug)]
pub(crate) enum CountryWarFinishBlock<ContextBlock> {
    Region {
        region_id: i32,
        report: CountryWarFinishReport,
        source: ContextBlock,
    },
    FormatNotice {
        region_id: i32,
        report: CountryWarFinishReport,
        source: ContextBlock,
    },
    SendInfo {
        region_id: i32,
        report: CountryWarFinishReport,
        source: ContextBlock,
    },
    CountryExists {
        region_id: i32,
        country: u8,
        report: CountryWarFinishReport,
        source: ContextBlock,
    },
    ResetCountry {
        region_id: i32,
        country: u8,
        report: CountryWarFinishReport,
        source: ContextBlock,
    },
    ClearTopInfo {
        report: CountryWarFinishReport,
        source: CountryWarTopInfoBlock<ContextBlock>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarDeclarationAuthority {
    CountryMissing,
    Rejected,
    Authorized,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarDeclarationPlayer {
    Missing,
    CountryUnavailable,
    Country(u8),
}

pub(crate) trait CountryWarDeclarationContext {
    fn online_player_country(&mut self, player_id: i32) -> CountryWarDeclarationPlayer;

    /// Повторяет последовательные `IsKing`, затем `IsMinister(player, 5)`,
    /// включая их king-log при отрицательных проверках.
    fn declaration_authority(
        &mut self,
        country: u8,
        player_id: i32,
    ) -> CountryWarDeclarationAuthority;

    fn region(&mut self, region_id: i32) -> Option<CountryWarVictoryRegion>;
    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;
    fn format_declaration_notice(
        &mut self,
        attack_country: u8,
        defend_country: i32,
        region_name: &[u8],
    ) -> Vec<u8>;
    fn send_private_to_country_king(
        &mut self,
        country: u8,
        text: &[u8],
    ) -> Option<Result<i32, SendMessageError>>;
    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError>;
    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError>;
    fn send_country_info(&mut self, text: &[u8], title: u32, color: u32) -> i32;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarPhase {
    Clear,
    DeclareBegin,
    DeclareEnd,
    PrepareBegin,
    PrepareEnd,
}

impl CountryWarPhase {
    const fn opcode(self) -> i32 {
        match self {
            Self::Clear => 0x7ff1e,
            Self::DeclareBegin => 0x7ff17,
            Self::DeclareEnd => 0x7ff18,
            Self::PrepareBegin => 0x7ff19,
            Self::PrepareEnd => 0x7ff1a,
        }
    }

    const fn string_id(self) -> Option<&'static [u8]> {
        match self {
            Self::Clear => None,
            Self::DeclareBegin => Some(b"WS0095"),
            Self::DeclareEnd => Some(b"WS0096"),
            Self::PrepareBegin => Some(b"WS0097"),
            Self::PrepareEnd => Some(b"WS0098"),
        }
    }
}

pub(crate) trait CountryWarPhaseContext {
    type Block;

    /// Повторяет `GetCountry(1..=4)` и назначает result только живой стране.
    fn reset_country_war_result_if_present(
        &mut self,
        country: u8,
    ) -> Result<bool, Self::Block>;

    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError>;

    /// Выполняет `GetStringByID`, null -> empty и штатную no-argument ветвь
    /// старого `_sprintf(char[256], localized_format)`.
    fn format_phase_notice(
        &mut self,
        string_id: &'static [u8],
    ) -> Result<Vec<u8>, Self::Block>;

    fn send_country_info(
        &mut self,
        text: &[u8],
        title: u32,
        color: u32,
    ) -> Result<i32, Self::Block>;
}

#[derive(Debug)]
pub(crate) struct CountryWarPhaseReport {
    pub(crate) phase: CountryWarPhase,
    pub(crate) reset_countries: Vec<u8>,
    pub(crate) broadcast_delivery: Option<Result<i32, SendMessageError>>,
    pub(crate) notice: Option<Vec<u8>>,
    pub(crate) info_delivery: Option<i32>,
}

#[derive(Debug)]
pub(crate) enum CountryWarPhaseBlock<ContextBlock> {
    ResetCountry {
        country: u8,
        report: CountryWarPhaseReport,
        source: ContextBlock,
    },
    FormatNotice {
        report: CountryWarPhaseReport,
        source: ContextBlock,
    },
    SendInfo {
        report: CountryWarPhaseReport,
        source: ContextBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarTopInfoKind {
    Start,
    End,
    Clear,
}

impl CountryWarTopInfoKind {
    const fn string_id(self) -> &'static [u8] {
        match self {
            Self::Start => b"WS0099",
            Self::End => b"WS0100",
            Self::Clear => b"WS0094",
        }
    }
}

pub(crate) trait CountryWarTopInfoContext {
    type Block;

    /// Выполняет GetStringByID/null-empty и штатное no-argument форматирование.
    fn format_top_info_notice(
        &mut self,
        string_id: &'static [u8],
    ) -> Result<Vec<u8>, Self::Block>;

    /// Вызывает concrete `CCountryHandler::AddOneTopInfo`.
    fn add_top_info(
        &mut self,
        timer_flag: i32,
        duration_ms: i32,
        text: &[u8],
        get_tick: &mut dyn FnMut() -> u32,
    ) -> i32;

    /// Вызывает concrete `CCountryHandler::SendTopInfoToClient`.
    fn send_top_info(
        &mut self,
        top_info_id: i32,
        timer_flag: i32,
        duration_ms: i32,
        text: &[u8],
    ) -> i32;
}

#[derive(Debug)]
pub(crate) struct CountryWarTopInfoReport {
    pub(crate) kind: CountryWarTopInfoKind,
    pub(crate) war_id: i32,
    pub(crate) schedule_inserted: bool,
    pub(crate) target_time: TagTime,
    pub(crate) now: TagTime,
    pub(crate) target_in_future: bool,
    pub(crate) duration_ms: Option<i32>,
    pub(crate) text: Option<Vec<u8>>,
    pub(crate) top_info_id: Option<i32>,
    pub(crate) delivery: Option<i32>,
}

#[derive(Debug)]
pub(crate) enum CountryWarTopInfoBlock<ContextBlock> {
    Arithmetic {
        report: CountryWarTopInfoReport,
        source: TagTimeArithmeticBlock,
    },
    FormatNotice {
        report: CountryWarTopInfoReport,
        source: ContextBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryWarDeclarationRejection {
    PlayerMissing,
    PlayerCountryUnavailable,
    OwnCountry,
    CountryMissing,
    Unauthorized,
    NoFreeRegion,
    RegionMissing,
    AttackAlreadyDeclared {
        private_delivery: Option<Result<i32, SendMessageError>>,
    },
    TargetAlreadyDeclared {
        private_delivery: Option<Result<i32, SendMessageError>>,
    },
    StateEntryMissing,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryWarDeclarationDisposition {
    Rejected(CountryWarDeclarationRejection),
    Declared {
        region_id: i32,
        state_wire: Vec<u8>,
        state_delivery: Result<i32, SendMessageError>,
        discarded_ws0103: Vec<u8>,
        notice: Vec<u8>,
        info_delivery: i32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryWarDeclarationReport {
    pub(crate) player_id: i32,
    pub(crate) target_country: i32,
    pub(crate) attack_country: Option<u8>,
    pub(crate) disposition: CountryWarDeclarationDisposition,
}

impl CountryWarDeclarationReport {
    pub(crate) const fn accepted(&self) -> bool {
        matches!(self.disposition, CountryWarDeclarationDisposition::Declared { .. })
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CountryWarSys {
    pub(crate) war_regions: BTreeMap<i32, CountryWarRegion>,
    country_wars: BTreeMap<i32, CountryWarTime>,
}

impl CountryWarSys {
    /// Загружает `setup/CountryWarSys.ini` и регистрирует исходные calendar events.
    ///
    /// `source` уже разрешён внешним resource owner-ом. Это заменяет только
    /// `ifstream`; token-order, первый `<end>`, повторное использование stream,
    /// map-key `0` и все timer-ветки сохранены по EXE `0x00492220..0x00492DCA`.
    pub(crate) fn initialize<Callback, Log>(
        &mut self,
        source: Option<&[u8]>,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: CountryWarCallbacks<Callback>,
        mut add_log_text: Log,
    ) -> Result<CountryWarLoadReport, CountryWarLoadError>
    where
        Callback: Copy,
        Log: FnMut(&[u8]),
    {
        self.country_wars.clear();
        let Some(source) = source else {
            add_log_text(b"setup/CountryWarSys.ini can't found!");
            return Ok(CountryWarLoadReport::default());
        };
        let mut report = CountryWarLoadReport {
            resource_found: true,
            legacy_result: true,
            ..CountryWarLoadReport::default()
        };
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());

        let info_begin_minutes = read_labeled_i32(&mut tokens, "InfoBeginTime offset")?;
        let info_end_minutes = read_labeled_i32(&mut tokens, "InfoEndTime offset")?;
        let declare_begin_minutes = read_labeled_i32(&mut tokens, "DeclarBeginTime offset")?;
        let declare_end_minutes = read_labeled_i32(&mut tokens, "DeclarEndTime offset")?;
        let prepare_begin_minutes = read_labeled_i32(&mut tokens, "PrepareBeginTime offset")?;
        let prepare_end_minutes = read_labeled_i32(&mut tokens, "PrepareEndTime offset")?;
        let duration_minutes = read_labeled_i32(&mut tokens, "war duration")?;
        let clear_minutes = read_labeled_i32(&mut tokens, "ClearTime offset")?;

        while read_to(&mut tokens, b"#") {
            let region_id = next_country_war_i32(&mut tokens, "country-war region ID")?;
            self.war_regions.insert(
                region_id,
                CountryWarRegion {
                    state_clear: false,
                    defend_country: 0,
                    attack_country: 0,
                },
            );
            report.region_records = report.region_records.wrapping_add(1);
        }

        while read_to(&mut tokens, b"#") {
            report.schedule_records = report.schedule_records.wrapping_add(1);
            let begin_source = next_country_war_token(&mut tokens, "country-war BeginTime")?;
            let begin_time = TagTime::from_legacy_string(begin_source)?;
            let candidate = CountryWarTime::from_offsets(
                begin_time,
                CountryWarOffsets {
                    info_begin_minutes,
                    info_end_minutes,
                    declare_begin_minutes,
                    declare_end_minutes,
                    prepare_begin_minutes,
                    prepare_end_minutes,
                    duration_minutes,
                    clear_minutes,
                },
            )?;
            if let Some(notice) = candidate.invalid_time_order() {
                add_log_text(notice.legacy_text());
                report.notices.push(notice);
                continue;
            }

            // VERIFIED_DISASSEMBLY: exact `0x00492AE2` всегда передаёт
            // stack long, обнулённый один раз в `0x0049257F`, без increment.
            self.country_wars.insert(0, candidate);
            report.accepted_records = report.accepted_records.wrapping_add(1);
        }

        for (&war_id, war) in &mut self.country_wars {
            report.registered_events = report
                .registered_events
                .wrapping_add(war.register_initial_events(war_id, now, timer, callbacks));
        }
        Ok(report)
    }

    /// Отменяет все старые event-ID, завершает войны и повторяет `initialize`.
    pub(crate) fn reload<Callback, Log, SendAll>(
        &mut self,
        source: Option<&[u8]>,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: CountryWarCallbacks<Callback>,
        add_log_text: Log,
        mut send_all: SendAll,
    ) -> Result<CountryWarReloadReport, CountryWarReloadBlock>
    where
        Callback: Copy,
        Log: FnMut(&[u8]),
        SendAll: FnMut(&CMessage) -> Result<i32, SendMessageError>,
    {
        let previous_schedules = self.country_wars.len();
        let mut kill_requests = 0u32;
        let mut killed_events = 0u32;
        for (&war_id, war) in &self.country_wars {
            for (event, event_id) in war.reload_event_ids() {
                kill_requests = kill_requests.wrapping_add(1);
                let Some(event_id) = event_id else {
                    return Err(CountryWarReloadBlock::MissingEventId {
                        war_id,
                        event,
                        kill_requests,
                        killed_events,
                    });
                };
                if timer.kill_time_event(event_id) {
                    killed_events = killed_events.wrapping_add(1);
                }
            }
        }

        let end_war = self.end_war(&mut send_all);
        let load = match self.initialize(source, now, timer, callbacks, add_log_text) {
            Ok(load) => load,
            Err(source) => {
                return Err(CountryWarReloadBlock::Load {
                    source,
                    kill_requests,
                    killed_events,
                    end_war,
                });
            }
        };
        Ok(CountryWarReloadReport {
            previous_schedules,
            kill_requests,
            killed_events,
            end_war,
            load,
        })
    }

    /// Сбрасывает только обе стороны каждой войны и рассылает exact `0x7FF1D`.
    pub(crate) fn end_war<SendAll>(&mut self, mut send_all: SendAll) -> CountryWarEndReport
    where
        SendAll: FnMut(&CMessage) -> Result<i32, SendMessageError>,
    {
        for state in self.war_regions.values_mut() {
            state.defend_country = 0;
            state.attack_country = 0;
        }
        let message = CMessage::new(0x7ff1d);
        CountryWarEndReport {
            reset_regions: self.war_regions.len(),
            delivery: send_all(&message),
        }
    }

    pub(crate) fn add_to_byte_array(&self, output: &mut Vec<u8>) -> bool {
        output.extend_from_slice(&(self.war_regions.len() as u32).to_le_bytes());
        for (&region_id, state) in &self.war_regions {
            output.extend_from_slice(&region_id.to_le_bytes());
            output.push(u8::from(state.state_clear));
            output.extend_from_slice(&[0; 3]);
            output.extend_from_slice(&state.defend_country.to_le_bytes());
            output.extend_from_slice(&state.attack_country.to_le_bytes());
        }
        true
    }

    fn is_already_declared(&self, country: i32) -> bool {
        self.war_regions
            .values()
            .any(|state| state.defend_country == country)
    }

    fn free_war_region(&self) -> Option<i32> {
        self.war_regions
            .iter()
            .find(|(_, state)| state.defend_country == 0 && state.attack_country == 0)
            .map(|(&region_id, _)| region_id)
    }

    /// Выполняет пять callbacks, чей внешний контракт ограничен country reset,
    /// одним broadcast и optional phase-info.
    pub(crate) fn run_phase<Context>(
        &mut self,
        phase: CountryWarPhase,
        _war_id: i32,
        context: &mut Context,
    ) -> Result<CountryWarPhaseReport, CountryWarPhaseBlock<Context::Block>>
    where
        Context: CountryWarPhaseContext + ?Sized,
    {
        let mut report = CountryWarPhaseReport {
            phase,
            reset_countries: Vec::new(),
            broadcast_delivery: None,
            notice: None,
            info_delivery: None,
        };

        if phase == CountryWarPhase::DeclareBegin {
            for country in 1u8..5 {
                let reset = match context.reset_country_war_result_if_present(country) {
                    Ok(reset) => reset,
                    Err(source) => {
                        return Err(CountryWarPhaseBlock::ResetCountry {
                            country,
                            report,
                            source,
                        });
                    }
                };
                if reset {
                    report.reset_countries.push(country);
                }
            }
        }

        let message = CMessage::new(phase.opcode());
        report.broadcast_delivery = Some(context.send_all(&message));
        let Some(string_id) = phase.string_id() else {
            return Ok(report);
        };
        let notice = match context.format_phase_notice(string_id) {
            Ok(notice) => notice,
            Err(source) => {
                return Err(CountryWarPhaseBlock::FormatNotice { report, source });
            }
        };
        report.notice = Some(notice);
        let info_delivery = match context.send_country_info(
            report.notice.as_deref().unwrap_or_default(),
            0xffff_fe92,
            0xffff_0000,
        ) {
            Ok(delivery) => delivery,
            Err(source) => {
                return Err(CountryWarPhaseBlock::SendInfo { report, source });
            }
        };
        report.info_delivery = Some(info_delivery);
        Ok(report)
    }

    /// Выполняет exact World `on_war_start`: общий broadcast, перевод
    /// назначенных регионов в war-state и optional `WS0092` для живого региона.
    pub(crate) fn run_war_start<Context>(
        &mut self,
        _war_id: i32,
        context: &mut Context,
    ) -> Result<CountryWarStartReport, CountryWarStartBlock<Context::Block>>
    where
        Context: CountryWarVictoryContext + ?Sized,
    {
        let message = CMessage::new(0x7ff1b);
        let mut report = CountryWarStartReport {
            broadcast_delivery: context.send_all(&message),
            ..CountryWarStartReport::default()
        };

        for (&region_id, state) in &mut self.war_regions {
            if state.defend_country == 0 || state.attack_country == 0 {
                continue;
            }
            report.active_regions += 1;
            state.state_clear = true;
            report.started_regions.push(region_id);

            let region = match context.region(region_id) {
                Ok(region) => region,
                Err(source) => {
                    return Err(CountryWarStartBlock::Region {
                        region_id,
                        report,
                        source,
                    });
                }
            };
            let Some(region) = region else {
                continue;
            };
            let notice = match context.format_victory_notice(
                b"WS0092",
                state.attack_country,
                state.defend_country,
                &region.name,
            ) {
                Ok(notice) => notice,
                Err(source) => {
                    return Err(CountryWarStartBlock::FormatNotice {
                        region_id,
                        report,
                        source,
                    });
                }
            };
            report.formatted_notices += 1;
            let delivery =
                match context.send_country_info(&notice, 0xffff_fe92, 0xffff_0000) {
                    Ok(delivery) => delivery,
                    Err(source) => {
                        return Err(CountryWarStartBlock::SendInfo {
                            region_id,
                            report,
                            source,
                        });
                    }
                };
            report.info_deliveries.push(delivery);
        }
        Ok(report)
    }

    /// Выполняет exact World `on_war_end`: завершает только начатые войны,
    /// сбрасывает их стороны и затем публикует отсчёт до `ClearTime`.
    pub(crate) fn run_war_end<Context, GetTick>(
        &mut self,
        war_id: i32,
        now: TagTime,
        mut get_tick: GetTick,
        context: &mut Context,
    ) -> Result<
        CountryWarFinishReport,
        CountryWarFinishBlock<<Context as CountryWarVictoryContext>::Block>,
    >
    where
        Context: CountryWarVictoryContext
            + CountryWarTopInfoContext<Block = <Context as CountryWarVictoryContext>::Block>
            + ?Sized,
        GetTick: FnMut() -> u32,
    {
        let message = CMessage::new(0x7ff1c);
        let mut report = CountryWarFinishReport {
            broadcast_delivery: CountryWarVictoryContext::send_all(context, &message),
            ..CountryWarFinishReport::default()
        };

        for (&region_id, state) in &mut self.war_regions {
            if !state.state_clear
                || state.defend_country == 0
                || state.attack_country == 0
            {
                continue;
            }
            report.active_regions += 1;
            let defend_country = state.defend_country;
            let attack_country = state.attack_country;

            let region = match context.region(region_id) {
                Ok(region) => region,
                Err(source) => {
                    return Err(CountryWarFinishBlock::Region {
                        region_id,
                        report,
                        source,
                    });
                }
            };
            if let Some(region) = region {
                let notice = match context.format_victory_notice(
                    b"WS0093",
                    attack_country,
                    defend_country,
                    &region.name,
                ) {
                    Ok(notice) => notice,
                    Err(source) => {
                        return Err(CountryWarFinishBlock::FormatNotice {
                            region_id,
                            report,
                            source,
                        });
                    }
                };
                report.formatted_notices += 1;
                let delivery = match context.send_country_info(
                    &notice,
                    0xffff_fe92,
                    0xffff_0000,
                ) {
                    Ok(delivery) => delivery,
                    Err(source) => {
                        return Err(CountryWarFinishBlock::SendInfo {
                            region_id,
                            report,
                            source,
                        });
                    }
                };
                report.info_deliveries.push(delivery);

                for country in [defend_country as u8, attack_country as u8] {
                    let exists = match context.country_exists(country) {
                        Ok(exists) => exists,
                        Err(source) => {
                            return Err(CountryWarFinishBlock::CountryExists {
                                region_id,
                                country,
                                report,
                                source,
                            });
                        }
                    };
                    if !exists {
                        continue;
                    }
                    if let Err(source) = context.set_country_war_result(country, 0) {
                        return Err(CountryWarFinishBlock::ResetCountry {
                            region_id,
                            country,
                            report,
                            source,
                        });
                    }
                    report.reset_country_results.push(country);
                }
            }

            state.state_clear = false;
            state.defend_country = 0;
            state.attack_country = 0;
            report.ended_regions.push(region_id);
        }

        let clear_top_info = match self.run_top_info(
            CountryWarTopInfoKind::Clear,
            war_id,
            now,
            &mut get_tick,
            context,
        ) {
            Ok(clear_top_info) => clear_top_info,
            Err(source) => {
                return Err(CountryWarFinishBlock::ClearTopInfo { report, source });
            }
        };
        report.clear_top_info = Some(clear_top_info);
        Ok(report)
    }

    /// Выполняет `on_war_start_info/on_war_end_info` поверх живого top-info owner-а.
    pub(crate) fn run_top_info<Context, GetTick>(
        &mut self,
        kind: CountryWarTopInfoKind,
        war_id: i32,
        now: TagTime,
        mut get_tick: GetTick,
        context: &mut Context,
    ) -> Result<CountryWarTopInfoReport, CountryWarTopInfoBlock<Context::Block>>
    where
        Context: CountryWarTopInfoContext + ?Sized,
        GetTick: FnMut() -> u32,
    {
        let schedule_inserted = !self.country_wars.contains_key(&war_id);
        let schedule = self
            .country_wars
            .entry(war_id)
            .or_insert_with(CountryWarTime::zero_initialized);
        let target_time = match kind {
            CountryWarTopInfoKind::Start => schedule.start_time,
            CountryWarTopInfoKind::End => schedule.end_time,
            CountryWarTopInfoKind::Clear => schedule.clear_time,
        };
        let mut report = CountryWarTopInfoReport {
            kind,
            war_id,
            schedule_inserted,
            target_time,
            now,
            target_in_future: false,
            duration_ms: None,
            text: None,
            top_info_id: None,
            delivery: None,
        };
        if target_time.legacy_le(now) {
            return Ok(report);
        }
        report.target_in_future = true;

        let difference = match target_time.get_time_difference(now) {
            Ok(difference) => difference,
            Err(source) => {
                return Err(CountryWarTopInfoBlock::Arithmetic { report, source });
            }
        };
        let duration_ms = u32::from(difference.second)
            .wrapping_add(u32::from(difference.minute).wrapping_mul(60))
            .wrapping_mul(1_000) as i32;
        report.duration_ms = Some(duration_ms);
        let text = match context.format_top_info_notice(kind.string_id()) {
            Ok(text) => text,
            Err(source) => {
                return Err(CountryWarTopInfoBlock::FormatNotice { report, source });
            }
        };
        report.text = Some(text);
        let top_info_id = context.add_top_info(
            2,
            duration_ms,
            report.text.as_deref().unwrap_or_default(),
            &mut get_tick,
        );
        report.top_info_id = Some(top_info_id);
        report.delivery = Some(context.send_top_info(
            top_info_id,
            2,
            duration_ms,
            report.text.as_deref().unwrap_or_default(),
        ));
        Ok(report)
    }

    pub(crate) fn player_declare<Context: CountryWarDeclarationContext + ?Sized>(
        &mut self,
        player_id: i32,
        target_country: i32,
        context: &mut Context,
    ) -> CountryWarDeclarationReport {
        let rejected = |attack_country, reason| CountryWarDeclarationReport {
            player_id,
            target_country,
            attack_country,
            disposition: CountryWarDeclarationDisposition::Rejected(reason),
        };

        let attack_country = match context.online_player_country(player_id) {
            CountryWarDeclarationPlayer::Missing => {
                return rejected(None, CountryWarDeclarationRejection::PlayerMissing);
            }
            CountryWarDeclarationPlayer::CountryUnavailable => {
                return rejected(
                    None,
                    CountryWarDeclarationRejection::PlayerCountryUnavailable,
                );
            }
            CountryWarDeclarationPlayer::Country(country) => country,
        };
        if i32::from(attack_country) == target_country {
            return rejected(
                Some(attack_country),
                CountryWarDeclarationRejection::OwnCountry,
            );
        }
        match context.declaration_authority(attack_country, player_id) {
            CountryWarDeclarationAuthority::CountryMissing => {
                return rejected(
                    Some(attack_country),
                    CountryWarDeclarationRejection::CountryMissing,
                );
            }
            CountryWarDeclarationAuthority::Rejected => {
                return rejected(
                    Some(attack_country),
                    CountryWarDeclarationRejection::Unauthorized,
                );
            }
            CountryWarDeclarationAuthority::Authorized => {}
        }

        let Some(region_id) = self.free_war_region() else {
            return rejected(
                Some(attack_country),
                CountryWarDeclarationRejection::NoFreeRegion,
            );
        };
        let Some(region) = context.region(region_id) else {
            return rejected(
                Some(attack_country),
                CountryWarDeclarationRejection::RegionMissing,
            );
        };
        if self.is_already_declared(i32::from(attack_country)) {
            let text = context.world_string(b"WS0101");
            let private_delivery = context.send_private_to_country_king(attack_country, &text);
            return rejected(
                Some(attack_country),
                CountryWarDeclarationRejection::AttackAlreadyDeclared { private_delivery },
            );
        }
        if self.is_already_declared(target_country) {
            let text = context.world_string(b"WS0102");
            let private_delivery = context.send_private_to_country_king(attack_country, &text);
            return rejected(
                Some(attack_country),
                CountryWarDeclarationRejection::TargetAlreadyDeclared { private_delivery },
            );
        }

        let Some(state) = self.war_regions.get_mut(&region_id) else {
            return rejected(
                Some(attack_country),
                CountryWarDeclarationRejection::StateEntryMissing,
            );
        };
        state.defend_country = target_country;
        state.attack_country = i32::from(attack_country);

        let mut state_message = CMessage::new(0x7ff1f);
        state_message.base_mut().add_long(region_id);
        state_message.base_mut().add_long(target_country);
        state_message
            .base_mut()
            .add_long(i32::from(attack_country));
        let state_wire = state_message.as_wire_bytes().to_vec();
        let state_delivery = context.send_all(&state_message);

        // Exact сначала копировал WS0103 в 512-byte buffer, затем полностью
        // перезаписывал его результатом sprintf(WS0104). Сам lookup сохраняем.
        let discarded_ws0103 = context.world_string(b"WS0103");
        let notice = context.format_declaration_notice(
            attack_country,
            target_country,
            &region.name,
        );
        let info_delivery = context.send_country_info(&notice, 0xffff_fe92, 0xffff_0000);

        CountryWarDeclarationReport {
            player_id,
            target_country,
            attack_country: Some(attack_country),
            disposition: CountryWarDeclarationDisposition::Declared {
                region_id,
                state_wire,
                state_delivery,
                discarded_ws0103,
                notice,
                info_delivery,
            },
        }
    }

    pub(crate) fn on_flag_destory<Context: CountryWarVictoryContext + ?Sized>(
        &mut self,
        country: i32,
        context: &mut Context,
    ) -> Result<CountryWarVictoryReport, Context::Block> {
        let mut report = CountryWarVictoryReport::default();

        for (&region_id, state) in &mut self.war_regions {
            if state.defend_country == 0 || state.attack_country == 0 {
                continue;
            }
            report.active_regions += 1;
            state.state_clear = false;

            let defend_country = state.defend_country;
            let attack_country = state.attack_country;
            let region = context.region(region_id)?;
            let mut victory_side = None;

            if region.is_some() {
                // Оригинал выполняет оба lookup независимо, затем общий gate.
                let defend_exists = context.country_exists(defend_country as u8)?;
                let attack_exists = context.country_exists(attack_country as u8)?;
                if defend_exists && attack_exists {
                    let mut message = CMessage::new(0x7ff22);
                    message.base_mut().add_byte(country as u8);
                    report.flag_deliveries.push(context.send_all(&message));

                    if defend_country == country {
                        context.set_country_war_result(defend_country as u8, 2)?;
                        context.set_country_war_result(attack_country as u8, 1)?;
                        victory_side = Some(false);
                        report.result_pairs += 1;
                    } else if attack_country == country {
                        context.set_country_war_result(defend_country as u8, 1)?;
                        context.set_country_war_result(attack_country as u8, 2)?;
                        victory_side = Some(true);
                        report.result_pairs += 1;
                    }
                }
            }

            let text = match (victory_side, region.as_ref()) {
                (Some(false), Some(region)) => {
                    report.formatted_notices += 1;
                    context.format_victory_notice(
                        b"WS0105",
                        attack_country,
                        defend_country,
                        &region.name,
                    )?
                }
                (Some(true), Some(region)) => {
                    report.formatted_notices += 1;
                    context.format_victory_notice(
                        b"WS0106",
                        attack_country,
                        defend_country,
                        &region.name,
                    )?
                }
                _ => Vec::new(),
            };

            state.defend_country = 0;
            state.attack_country = 0;
            report.info_deliveries.push(context.send_country_info(
                &text,
                0xffff_fe92,
                0xffff_0000,
            )?);
        }

        Ok(report)
    }
}

#[derive(Clone, Copy)]
struct CountryWarOffsets {
    info_begin_minutes: i32,
    info_end_minutes: i32,
    declare_begin_minutes: i32,
    declare_end_minutes: i32,
    prepare_begin_minutes: i32,
    prepare_end_minutes: i32,
    duration_minutes: i32,
    clear_minutes: i32,
}

impl CountryWarTime {
    /// Повторяет value-insert `map::operator[]` для отсутствующего war ID.
    fn zero_initialized() -> Self {
        let zero_time = TagTime::default();
        let zero_event = Some(TimerId::from_raw(0));
        Self {
            declare_begin_event_id: zero_event,
            declare_begin_time: zero_time,
            declare_end_event_id: zero_event,
            declare_end_time: zero_time,
            prepare_begin_event_id: zero_event,
            prepare_begin_time: zero_time,
            prepare_end_event_id: zero_event,
            prepare_end_time: zero_time,
            start_info_event_id: zero_event,
            start_info_time: zero_time,
            start_event_id: zero_event,
            start_time: zero_time,
            end_info_event_id: zero_event,
            end_info_time: zero_time,
            end_event_id: zero_event,
            end_time: zero_time,
            clear_event_id: zero_event,
            clear_time: zero_time,
        }
    }

    fn from_offsets(
        begin_time: TagTime,
        offsets: CountryWarOffsets,
    ) -> Result<Self, TagTimeArithmeticBlock> {
        Ok(Self {
            declare_begin_event_id: None,
            declare_begin_time: with_minute_offset(begin_time, offsets.declare_begin_minutes)?,
            declare_end_event_id: None,
            declare_end_time: with_minute_offset(begin_time, offsets.declare_end_minutes)?,
            prepare_begin_event_id: None,
            prepare_begin_time: with_minute_offset(begin_time, offsets.prepare_begin_minutes)?,
            prepare_end_event_id: None,
            prepare_end_time: with_minute_offset(begin_time, offsets.prepare_end_minutes)?,
            start_info_event_id: None,
            start_info_time: with_minute_offset(begin_time, offsets.info_begin_minutes)?,
            start_event_id: None,
            start_time: begin_time,
            end_info_event_id: None,
            end_info_time: with_minute_offset(begin_time, offsets.info_end_minutes)?,
            end_event_id: None,
            end_time: with_minute_offset(begin_time, offsets.duration_minutes)?,
            clear_event_id: None,
            clear_time: with_minute_offset(begin_time, offsets.clear_minutes)?,
        })
    }

    fn invalid_time_order(&self) -> Option<CountryWarLoadNotice> {
        if self.prepare_begin_time.legacy_ge(self.prepare_end_time) {
            Some(CountryWarLoadNotice::PrepareBeginNotBeforePrepareEnd)
        } else if self.declare_begin_time.legacy_ge(self.prepare_begin_time) {
            Some(CountryWarLoadNotice::DeclareBeginNotBeforePrepareBegin)
        } else if self.declare_begin_time.legacy_ge(self.declare_end_time) {
            Some(CountryWarLoadNotice::DeclareBeginNotBeforeDeclareEnd)
        } else if self.declare_begin_time.legacy_ge(self.start_time) {
            Some(CountryWarLoadNotice::DeclareBeginNotBeforeBegin)
        } else if self.start_time.legacy_ge(self.end_time) {
            Some(CountryWarLoadNotice::BeginNotBeforeEnd)
        } else if self.start_info_time.legacy_ge(self.start_time) {
            Some(CountryWarLoadNotice::InfoBeginNotBeforeBegin)
        } else if self.end_info_time.legacy_ge(self.end_time) {
            Some(CountryWarLoadNotice::InfoEndNotBeforeEnd)
        } else if self.end_time.legacy_ge(self.clear_time) {
            Some(CountryWarLoadNotice::EndNotBeforeClear)
        } else {
            None
        }
    }

    fn reload_event_ids(
        &self,
    ) -> [(CountryWarReloadEvent, Option<TimerId>); 9] {
        [
            (
                CountryWarReloadEvent::PrepareBegin,
                self.prepare_begin_event_id,
            ),
            (
                CountryWarReloadEvent::PrepareEnd,
                self.prepare_end_event_id,
            ),
            (
                CountryWarReloadEvent::DeclareBegin,
                self.declare_begin_event_id,
            ),
            (
                CountryWarReloadEvent::DeclareEnd,
                self.declare_end_event_id,
            ),
            (CountryWarReloadEvent::InfoBegin, self.start_info_event_id),
            (CountryWarReloadEvent::Begin, self.start_event_id),
            (CountryWarReloadEvent::InfoEnd, self.end_info_event_id),
            (CountryWarReloadEvent::End, self.end_event_id),
            (CountryWarReloadEvent::Clear, self.clear_event_id),
        ]
    }

    fn register_initial_events<Callback: Copy>(
        &mut self,
        war_id: i32,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: CountryWarCallbacks<Callback>,
    ) -> u32 {
        if !self.clear_time.legacy_ge(now) {
            return 0;
        }

        let mut registered = 1u32;
        self.clear_event_id = Some(timer.set_time_event(self.clear_time, callbacks.clear, war_id));
        if self.end_time.legacy_ge(now) {
            self.end_event_id = Some(timer.set_time_event(self.end_time, callbacks.end, war_id));
            registered = registered.wrapping_add(1);
            self.end_info_event_id = Some(timer.set_time_event(
                if self.end_info_time.legacy_ge(now) {
                    self.end_info_time
                } else {
                    now
                },
                callbacks.end_info,
                war_id,
            ));
            registered = registered.wrapping_add(1);

            if self.start_time.legacy_ge(now) {
                self.start_event_id =
                    Some(timer.set_time_event(self.start_time, callbacks.start, war_id));
                registered = registered.wrapping_add(1);
                self.start_info_event_id = Some(timer.set_time_event(
                    if self.start_info_time.legacy_ge(now) {
                        self.start_info_time
                    } else {
                        now
                    },
                    callbacks.start_info,
                    war_id,
                ));
                registered = registered.wrapping_add(1);

                if self.declare_end_time.legacy_ge(now) {
                    self.declare_begin_event_id = Some(timer.set_time_event(
                        self.declare_end_time,
                        callbacks.declare_end,
                        war_id,
                    ));
                    registered = registered.wrapping_add(1);
                    if self.declare_begin_time.legacy_ge(now) {
                        self.declare_begin_event_id = Some(timer.set_time_event(
                            self.declare_begin_time,
                            callbacks.declare_begin,
                            war_id,
                        ));
                        registered = registered.wrapping_add(1);
                    }
                }
            }
        } else {
            // Exact сначала уже оставил clear-event на ClearTime, затем ставит
            // второй на now и теряет ID первого через overwrite поля.
            self.clear_event_id = Some(timer.set_time_event(now, callbacks.clear, war_id));
            registered = registered.wrapping_add(1);
        }

        if self.prepare_end_time.legacy_ge(now) {
            self.prepare_end_event_id = Some(timer.set_time_event(
                self.prepare_end_time,
                callbacks.prepare_end,
                war_id,
            ));
            registered = registered.wrapping_add(1);
            if self.prepare_begin_time.legacy_ge(now) {
                self.prepare_begin_event_id = Some(timer.set_time_event(
                    self.prepare_begin_time,
                    callbacks.prepare_begin,
                    war_id,
                ));
                registered = registered.wrapping_add(1);
            }
        }
        registered
    }
}

fn with_minute_offset(
    mut time: TagTime,
    minutes: i32,
) -> Result<TagTime, TagTimeArithmeticBlock> {
    let _ = time.add_minute(minutes)?;
    Ok(time)
}

fn read_labeled_i32<'a, Tokens>(
    tokens: &mut Tokens,
    field: &'static str,
) -> Result<i32, CountryWarLoadError>
where
    Tokens: Iterator<Item = &'a [u8]>,
{
    let _label = next_country_war_token(tokens, field)?;
    next_country_war_i32(tokens, field)
}

fn next_country_war_i32<'a, Tokens>(
    tokens: &mut Tokens,
    field: &'static str,
) -> Result<i32, CountryWarLoadError>
where
    Tokens: Iterator<Item = &'a [u8]>,
{
    let token = next_country_war_token(tokens, field)?;
    std::str::from_utf8(token)
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or(CountryWarLoadError::InvalidValue { field })
}

fn next_country_war_token<'a, Tokens>(
    tokens: &mut Tokens,
    field: &'static str,
) -> Result<&'a [u8], CountryWarLoadError>
where
    Tokens: Iterator<Item = &'a [u8]>,
{
    tokens
        .next()
        .ok_or(CountryWarLoadError::MissingValue { field })
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp

// ============================================================================
// FUNCTION: CountryWarSys::get_instance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:38
// RVA: 0x0008F410
// ADDRESS: 0048f410
// PROTOTYPE: CountryWarSys * __cdecl get_instance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_war_clear
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:477
// RVA: 0x0008F490
// ADDRESS: 0048f490
// PROTOTYPE: void __stdcall on_war_clear(long param_1)
//
// Реализовано выше через `run_phase(Clear, ...)`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: get_country_war_sys
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:775
// RVA: 0x0008F4F0
// ADDRESS: 0048f4f0
// PROTOTYPE: CountryWarSys * __cdecl get_country_war_sys(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::AddToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:335
// RVA: 0x0008F800
// ADDRESS: 0048f800
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Реализовано выше с exact World layout и нулевым техническим padding.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::end_war
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:482
// RVA: 0x0008F890
// ADDRESS: 0048f890
// PROTOTYPE: void __thiscall end_war(void)
//
// Реализовано выше; raw сохранён как локальная проверка того, что clear-byte
// исходного CountryWarRegion здесь не менялся.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::is_already_declare
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:604
// RVA: 0x0008F940
// ADDRESS: 0048f940
// PROTOTYPE: bool __thiscall is_already_declare(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::get_war_region
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:693
// RVA: 0x0008F990
// ADDRESS: 0048f990
// PROTOTYPE: long __thiscall get_war_region(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_declare_end
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:514
// RVA: 0x0008FA00
// ADDRESS: 0048fa00
// PROTOTYPE: void __stdcall on_declare_end(long param_1)
//
// Реализовано выше через `run_phase(DeclareEnd, ...)`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_prepare_begin
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:526
// RVA: 0x0008FB40
// ADDRESS: 0048fb40
// PROTOTYPE: void __stdcall on_prepare_begin(long param_1)
//
// Реализовано выше через `run_phase(PrepareBegin, ...)`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_prepare_end
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:538
// RVA: 0x0008FC80
// ADDRESS: 0048fc80
// PROTOTYPE: void __stdcall on_prepare_end(long param_1)
//
// Реализовано выше через `run_phase(PrepareEnd, ...)`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_war_start
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:376
// RVA: 0x00090EC0
// ADDRESS: 00490ec0
// PROTOTYPE: void __stdcall on_war_start(long param_1)
//
// Реализовано выше через `run_war_start`; exact disassembly
// `0x00490EC0..0x004910F7` подтверждает общий region key и порядок side effects.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_war_end
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:409
// RVA: 0x00091100
// ADDRESS: 00491100
// PROTOTYPE: void __stdcall on_war_end(long param_1)
//
// Реализовано выше через `run_war_end`; exact disassembly
// `0x00491100..0x0049152B` подтверждает region reset и ClearTime top-info.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_declare_begin
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:495
// RVA: 0x00091530
// ADDRESS: 00491530
// PROTOTYPE: void __stdcall on_declare_begin(long param_1)
//
// Реализовано выше через `run_phase(DeclareBegin, ...)`; exact disassembly
// подтверждает country IDs `1..=4` и единый byte-key обоих map-вызовов.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_war_start_info
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:550
// RVA: 0x000916C0
// ADDRESS: 004916c0
// PROTOTYPE: void __stdcall on_war_start_info(long param_1)
//
// Реализовано выше через `run_top_info(Start, ...)` и concrete AddOneTopInfo.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_war_end_info
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:578
// RVA: 0x000918E0
// ADDRESS: 004918e0
// PROTOTYPE: void __stdcall on_war_end_info(long param_1)
//
// Реализовано выше через `run_top_info(End, ...)` и concrete AddOneTopInfo.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::player_declare
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:617
// RVA: 0x00091B00
// ADDRESS: 00491b00
// PROTOTYPE: bool __thiscall player_declare(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_flag_destory
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:706
// RVA: 0x00091E00
//
// Реализовано выше в связной cross-server country victory chain;
// потерянный stack-key и порядок эффектов имеют статус VERIFIED_DISASSEMBLY.
//

// ============================================================================
// FUNCTION: CountryWarSys::initialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:45
// RVA: 0x00092220
// ADDRESS: 00492220
// PROTOTYPE: bool __thiscall initialize(void)
//
// Реализовано выше по RAW и exact disassembly. Полный псевдокод ниже сохранён
// как локальная provenance-документация timer/map quirks.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::reload
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:351
// RVA: 0x00092DF0
// ADDRESS: 00492df0
// PROTOTYPE: bool __thiscall reload(void)
//
// Реализовано выше с точным kill-order и явной границей неизвестных event-ID.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
