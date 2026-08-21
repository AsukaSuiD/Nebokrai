//! Save-владелец `CCountry` исторического `WorldServer`.
//!
//! Статус `CCountry::SetCountryPower/SetCountryTreasury/SetCountryTech` RVA
//! `0x000A4750/0x000A4790/0x000A47D0`, `CCountry::AddToByteArray` RVA `0x000C6E30`,
//! `CCountry::IsKing/CanOperate/Exile/SuccessExiled/Silence/Absolve` RVA
//! `0x000C7160/0x000C7520/0x000C7AD0/0x000C7EE0/0x000C81D0/0x000C8570`,
//! governance-цепочка `CanAscend/CanDemise/DeposeKing/RegisterKing/Demise`
//! RVA `0x000CA830/0x000CAC30/0x000CB030/0x000CB8F0/0x000CC320`,
//! `CCountry::CloneCountryData` RVA `0x000C9CE0` и
//! `CCountry::CloneSaveData` RVA `0x000CC470` — `IMPLEMENTED`; остальной корпус
//! ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.h`
//! и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1564,1598`.
//!
//! Exact PDB задаёт `CCountry` размером `0xB8`, country/treasury/power/tech
//! поля по `+0x4..+0x18`, `CKing` по `+0x24`, minister-map по `+0x5C` и signed
//! `m_lCountryWarRes` по `+0xA8`, а `map<long,long> ExileMap` по `+0xAC`.
//! `GetExileResTime` RVA `0x000C6D70` сначала снимает 32-битный
//! `timeGetTime`, затем ищет signed player ID и считает
//! `_exile_time - now + started_at` с машинным wrapping, делением на 1000 к
//! нулю и нижней границей ноль. Точный `SuccessExiled`
//! `0x004C7EE0..0x004C81C5` вызывает `timeGetTime`, но не использует результат
//! и не наполняет `ExileMap`; Linux-донор добавлял `try_emplace`, то есть
//! исправлял наблюдаемую ошибку оригинала. Rust сохраняет exact поведение и
//! не выдумывает запись, пока её не подтвердит другой машинный owner.
//! Успешная ветка сначала списывает king control point с исходным upper-only
//! clamp, публикует `0x7FF10`, затем wrapping увеличивает `m_lExileNum`,
//! форматирует `WS0077` и рассылает `0x7FF11` каждому connected GameServer.
//! Отказ форматирует `WS0078` и пишет королю `0x7FF13`; отсутствующий map-player
//! использует `WS0072`, причём порядок там обратный: king-log до private send.
//! Старое переполнение `char[260]` не является протоколом: Rust сохраняет не
//! более 259 видимых bytes и единственный wire-NUL.
//! Для входного `0x6030D` exact `CanOperate(4)` сначала читает минимальные
//! control points, затем проверяет `m_bIsWarring`, минимум и wrapping
//! `_max_exile_num - 1`; `0x004C7850/0x004C7AA6` машинно подтверждают false/
//! true. `Exile` последовательно проверяет короля, online-player, страну,
//! наличие exile-rect, unsigned PK и map-route. Только успешный путь отправляет
//! `0x7FF0E { country:u8, player:i32 }`; `0x004C7EAF` возвращает player ID,
//! все отказные ветки возвращают ноль. Linux-донор добавлял socket-correlation,
//! request registry и source/tail validation — их нет в поставочном EXE.
//! Для входного `0x6030C` exact сохраняет тот же порядок `IsKing`, затем
//! `CanOperate(5)`: minimum читается до проверки войны, а daily-limit
//! сравнивается как `m_lSilenceNum <= wrapping(_max_silence_num - 1)`.
//! `Silence` отклоняет короля (`WS0079`, только log), отсутствующего игрока,
//! чужую страну и inherited `m_bIsGod` (`WS0080..WS0082`, log затем private).
//! Успех wrapping увеличивает счётчик, списывает control point с upper-only
//! clamp, пишет `WS0083`, затем отправляет `0x7FF10` королю,
//! `0x7FF0D { country:u8, player:i32, silence_count:i32 }` на маршрут цели и
//! `0x7FF11` всем connected GameServer. Exact `0x004C853E` возвращает ID цели;
//! все отказы возвращают ноль. Linux-донор здесь не добавляет контрактов.
//! `0x6030B -> CanOperate(3) -> Absolve` сначала обнуляет у online-player
//! `wPkCount`, затем `dwKillCount`, и лишь после этого читает цену операции.
//! Успех списывает control point, публикует `0x7FF10`, wrapping увеличивает
//! `m_lAbsolveNum`, пишет `WS0086`, рассылает `0x7FF0C {country, player}` через
//! `SendAll` и затем `0x7FF11`. `0x004C87EE` возвращает ID цели; `WS0084/85`
//! дают log и private, а все отказы возвращают ноль.
//! `0x6030A` читает `target:i32, job:i8, king:i32, country:i8` и строго идёт
//! через `IsKing -> CanOperate(2) -> IsMinister -> DeposeMinister(job, 7)`.
//! `IsMinister` ищет target среди всех министров, не сверяя job. Exact
//! `DeposeMinister` использует `map::operator[]`: неверный job после успешной
//! identity-проверки оставляет null slot, который меняет последующие map-count
//! wire. Rust сохраняет quirk отдельным `BTreeSet`, не вводя nullable owner.
//! Успешная mode-7 ветка `AppointMinister(0, job, 7)` рассылает `WS0068`,
//! очищает ID/имя, публикует `0x7FF04 {country, old_player, job, 2}`, затем
//! `0x7FF10` и полный `0x7FF07` королю; appointed/salary flags не очищаются.
//! `CCountry::NewTerm` RVA `0x000C6F40` — `IMPLEMENTED`: он очищает DB/live
//! appointed/salary flags короля и всех non-null minister owner-ов, рассылает
//! пустой `0x7FF14`, затем обнуляет четыре дневных счётчика.
//! Последующий original clone разыменовывал null slot; безопасный save-clone
//! его пропускает как внутренний UB, но два наблюдаемых wire-count сохраняет.
//! `0x60309` использует тот же target/job/king/country wire и selector `1`.
//! Positive `AppointMinister(..., 6)` проверяет king, slot, pending-флаг,
//! online/country и `HasJob`; последний намеренно вызывает полный `IsKing` и
//! оставляет `WS0034` даже для обычного кандидата. Успех ставит appointed до
//! списания points, пишет `WS0066`, затем публикует `0x7FF04` kind `1`,
//! `0x7FF10` и `0x7FF07`. Exact disassembly `0x004C6B5B/0x004C6BDA/0x004C6BF3`
//! подтвердил, что base-info wire несёт quest-switch короля и пары
//! quest-switch/appointed министров, а не DB salary flags.
//! `0x60308` читает `target:i32, king:i32, country:i8` и строго идёт через
//! `IsKing -> CanOperate(0) -> Demise`. Selector `0` после безусловного чтения
//! общего minimum проверяет войну, `_def_king_control_point_demise_need` и
//! именно DB/live appointed-флаг короля по `+0x4A`. `CanDemise` требует обоих
//! online-player и сохраняет `m_bDemiseFaction=true`, если кандидат не master;
//! этот флаг не откатывается при последующем отказе `CanAscend`. Последний
//! сохраняет машинную ошибку `IsFreeFaction(king_player_id)` вместо faction ID.
//! `RegisterKing(1)` либо запоминает первый город, очищает весь city-list старой
//! faction virtual-вызовом `+0x88`, отдаёт новой только запомненный город и
//! публикует `0x7FE27`,
//! либо вызывает полный уже восстановленный `CFaction::demise`; затем
//! `DeposeKing(2)` снимает министров в unsigned job-order, назначает нового
//! короля, списывает demise-cost и публикует `0x7FF04/0x7FF12`. Внешний
//! `Demise` не проверяет результат регистрации: всегда отправляет `0x7FF10`
//! текущему королю и при пройденном `CanDemise` возвращает ID прежнего. Exact
//! `0x004CC416/0x004CC44B` подтверждает отдельный ноль для self-target и этот
//! необычный return. Linux-донор ошибочно возвращал target для self-target и
//! опускал часть `0x7FE27`/вложенных side effect; Rust следует EXE/PDB.
//! `InitialOLPlayersList/Sort/GetPlayersList` RVA
//! `0x000CA360/0x000CA6E0/0x000CAD90` каждый раз пересобирают
//! online-player срез: только своя страна, level `>= 10` и
//! `m_bIsGod == false`. Exact multimap с level-key и обратным обходом
//! даёт убывание level и обратный online-order для равных level;
//! owned `Vec` и standard sort заменяют только MSVC STL/allocation.
//! Page-start сохраняет wrapping формулу `(page * 3 - 3) * 4`, а
//! `0x7FF08` несёт не более 12 записей. Exact `0x004CAFF0..0x004CB004`
//! возвращает king ID, а не count, как Linux-донор; donor также
//! опускал GM-фильтр.
//! Административный `0x60304` ведёт короля через `SetKing` RVA
//! `0x000CC290`: `DeposeKing(3)` возврат игнорируется, ID заменяется
//! безусловно и публикуется `0x7FF05`, после чего `RegisterKing(0)`
//! проверяет online/country/faction, выставляет default control point,
//! имя/timestamp и `0x7FF04/0x7FF12`. Exact failure returns ноль;
//! Linux-донор ошибочно возвращал player ID и добавлял dispatcher gates.
//! Clone копирует country ID, treasury, power,
//! current/level-up tech exp, tech level, king identity/flags и war-result.
//! Три king-point ограничиваются соответствующими максимумами `CCountryParam`.
//! `COfficer::_bQuestSwitch` по exact PDB находится отдельно от
//! `_bAppointed/_bSalary`; поэтому live quest-флаги короля и министров не
//! смешиваются с их DB save-проекцией.
//! Для достигнутого `CountryWarSys::player_declare` nonzero-ветки
//! `IsKing/IsMinister` сведены к typed identity-проверкам; исходные WS0034/
//! WS0037 log-side-effects остаются у caller-а. Exact `IsMinister(player, 5)`
//! ищет player среди всех министров и использует job `5` только в fail-log —
//! Rust намеренно не ужесточает это до проверки конкретной должности.
//!
//! Minister-map обходится в unsigned key-order, но ключ источника не копируется:
//! максимум первые шесть non-null `CMinister` вставляются по собственному
//! `_byteIDType`. Повторный `_byteIDType` оставляет первую запись, как
//! `std::map::insert`; последующий `CDBCountry::Save` наблюдает только позиции
//! `2..=7`. Rust `BTreeMap`, owned byte names и `Clone` заменяют только MSVC
//! map/string/object allocation. Live null minister исходник разыменовывал;
//! safe reached-state не назначает этому UB новое поведение и представляет
//! только живого owner-а. Allocation failure остаётся политикой стандартного
//! allocator-а.
//!
//! `CloneSaveData` вызывается только `CCountryHandler::GenerateSaveData`, после
//! чего копию наблюдает `CDBCountry::Save`. Поэтому Rust меняет форму API и
//! сразу возвращает полный `CountrySaveSnapshot`; скопированный
//! `_tech_lelup_exp`, который DB-owner не читает, остаётся локально
//! зафиксированным полем live save-state, но не выдумывается в DB-контракте.
//! Raw тела двух заменённых функций и compiler/STL cleanup удалены.
//!
//! Initial-config record сохраняет только наблюдаемую Game-проекцию: country
//! ID, четыре country scalars, три king points, king ID, war-result и ordered
//! minister map `job:u8 -> player_id:i32`. `tech_level_up_exp`, имена и flags
//! в этот wire не входят. `BTreeMap` сохраняет unsigned порядок; signed count
//! проверяется до записи вместо неограниченного `size_t -> long` narrowing.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::ffi::CString;
use std::fmt;

use crate::dbaccess::worlddb::dbcountry::{
    CountryKingSaveSnapshot, CountryMinisterSaveSnapshot, CountrySaveSnapshot,
};
use crate::nets::networld::message::{CMessage, SendMessageError};

use super::countryparam::{CCountryParam, CountryParameterUnavailable};
use super::king::{KingPointUpdate, set_control_point, set_material_point, set_war_point};

/// Три текущих максимума `CCountryParam`, читаемые во время clone.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CountryKingSaveLimits {
    pub(crate) control_point: i32,
    pub(crate) material_point: i32,
    pub(crate) war_point: i32,
}

/// Достигнутая live-форма одного minister owner-а.
#[derive(Clone, Debug)]
pub(crate) struct CountryMinisterState {
    pub(crate) id_type: u8,
    pub(crate) quest_switch: bool,
    pub(crate) snapshot: CountryMinisterSaveSnapshot,
}

/// Достигнутая save-часть живого `CCountry` без копирования MSVC layout.
#[derive(Clone, Debug)]
pub(crate) struct CCountry {
    pub(crate) country_id: u8,
    pub(crate) treasury: i32,
    pub(crate) power: i32,
    pub(crate) tech_current_exp: i32,
    pub(crate) tech_level_up_exp: i32,
    pub(crate) tech_level: i32,
    pub(crate) king: CountryKingSaveSnapshot,
    pub(crate) king_quest_switch: bool,
    pub(crate) country_war_result: i32,
    pub(crate) ministers: BTreeMap<u8, CountryMinisterState>,
    pub(crate) null_minister_slots: BTreeSet<u8>,
    pub(crate) city_id: i32,
    pub(crate) demise_faction: bool,
    pub(crate) king_timestamp_ms: u32,
    pub(crate) is_warring: bool,
    pub(crate) silence_count: i32,
    pub(crate) pk_count: i32,
    pub(crate) exile_count: i32,
    pub(crate) absolve_count: i32,
    pub(crate) exile_started_at_ms: BTreeMap<i32, i32>,
}

/// Наблюдаемый результат exact `CCountry::GetExileResTime`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryExileTimeLookup {
    pub(crate) started_at_ms: Option<i32>,
    pub(crate) sampled_at_ms: u32,
    pub(crate) remaining_ms: i32,
    pub(crate) remaining_seconds: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryQuestSwitchTarget {
    King,
    Minister { job: u8 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryQuestSwitchUpdate {
    pub(crate) target: CountryQuestSwitchTarget,
    pub(crate) previous: bool,
    pub(crate) applied: bool,
}

pub(crate) trait CountryNewTermContext {
    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryMinisterTermReset {
    pub(crate) job: u8,
    pub(crate) previous_appointed: bool,
    pub(crate) previous_salary_received: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryNewTermReport {
    pub(crate) previous_king_appointed: bool,
    pub(crate) previous_king_salary_received: bool,
    pub(crate) minister_resets: Vec<CountryMinisterTermReset>,
    pub(crate) previous_silence_count: i32,
    pub(crate) previous_pk_count: i32,
    pub(crate) previous_exile_count: i32,
    pub(crate) previous_absolve_count: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryScalarUpdate {
    Treasury {
        requested: i32,
        previous: i32,
        applied: i32,
    },
    Power {
        requested: i32,
        previous: i32,
        applied: i32,
    },
    TechnologyExperience {
        requested: i32,
        previous: i32,
        applied: i32,
    },
    TechnologyLevel {
        requested: i32,
        previous: i32,
        applied: i32,
    },
    KingPoint(KingPointUpdate),
}

/// Один аргумент exact `_sprintf` внутри `CCountry::SuccessExiled`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryExileTextArgument<'a> {
    Text(&'a [u8]),
    Signed(i32),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryExileMessageDelivery {
    pub(crate) map_id: i32,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountryExileTarget {
    pub(crate) name: Vec<u8>,
    pub(crate) country: Option<u8>,
    pub(crate) level: u8,
    pub(crate) credit: u32,
    pub(crate) pk_count: u16,
    pub(crate) is_god: bool,
}

/// Достигнутый online-player до фильтрации `InitialOLPlayersList`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountryOnlinePlayer {
    pub(crate) id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) country: Option<u8>,
    pub(crate) occupation: u8,
    pub(crate) level: u8,
    pub(crate) is_god: bool,
}

/// Одна wire-запись exact `tagPlayerInfo` без MSVC string/pointer layout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountryPlayerInfo {
    pub(crate) id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) occupation: u8,
    pub(crate) level: u8,
    pub(crate) faction_name: Vec<u8>,
    pub(crate) is_faction_master: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryPlayersListContextBlock {
    PlayerFactionLookup,
    FactionMasterLookup,
}

/// Полный наблюдаемый результат `GetPlayersList`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryPlayersListReport {
    pub(crate) page: i32,
    pub(crate) start_index: u32,
    pub(crate) total: u32,
    pub(crate) entries: Vec<CountryPlayerInfo>,
    pub(crate) map_id: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
    pub(crate) logs: Vec<Vec<u8>>,
    pub(crate) legacy_result: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryInitialKingRejection {
    PlayerMissing,
    TargetCountryUnavailable,
    TargetFromAnotherCountry,
    FactionMissing,
    KingMismatch,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryInitialKingDisposition {
    Rejected {
        reason: CountryInitialKingRejection,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
    ParameterUnavailable(CountryParameterUnavailable),
    ContextBlocked(CountryGovernanceContextBlock),
    Applied {
        faction_id: i32,
        control_point_update: KingPointUpdate,
        appointment_wire: Vec<u8>,
        appointment_delivery: Result<i32, SendMessageError>,
        world_wire: Option<Vec<u8>>,
        world_delivery: Option<Result<i32, SendMessageError>>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryInitialKingReport {
    pub(crate) player_id: i32,
    pub(crate) text: Vec<u8>,
    pub(crate) legacy_result: i32,
    pub(crate) disposition: CountryInitialKingDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountrySetKingReport {
    pub(crate) player_id: i32,
    pub(crate) depose: CountryDeposeKingReport,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
    pub(crate) legacy_result: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountryFactionSnapshot {
    pub(crate) faction_id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) owned_cities: Vec<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryGovernanceContextBlock {
    FactionMasterLookup,
    PlayerFactionLookup,
    UnionLookup,
    OwnedCityMutation,
    FactionDemise,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryAbsolveCounterReset {
    pub(crate) previous_kill_count: u32,
    pub(crate) previous_pk_count: u16,
}

/// Узкая граница player/localization/network/log эффектов исходного owner-а.
pub(crate) trait CountryExileResultContext {
    fn map_player_name(&mut self, player_id: i32) -> Option<Vec<u8>>;
    fn online_player(&mut self, player_id: i32) -> Option<CountryExileTarget>;
    fn reset_online_player_murder_counters(
        &mut self,
        player_id: i32,
    ) -> Option<CountryAbsolveCounterReset>;
    fn faction_id_by_master_player(
        &mut self,
        _player_id: i32,
    ) -> Result<i32, CountryGovernanceContextBlock> {
        Err(CountryGovernanceContextBlock::FactionMasterLookup)
    }
    fn faction_id_by_player(
        &mut self,
        _player_id: i32,
    ) -> Result<i32, CountryGovernanceContextBlock> {
        Err(CountryGovernanceContextBlock::PlayerFactionLookup)
    }
    fn faction_snapshot(&mut self, _faction_id: i32) -> Option<CountryFactionSnapshot> {
        None
    }
    fn union_id_for_faction(
        &mut self,
        _faction_id: i32,
    ) -> Result<i32, CountryGovernanceContextBlock> {
        Err(CountryGovernanceContextBlock::UnionLookup)
    }
    fn clear_faction_owned_cities(
        &mut self,
        _faction_id: i32,
    ) -> Result<(), CountryGovernanceContextBlock> {
        Err(CountryGovernanceContextBlock::OwnedCityMutation)
    }
    fn add_faction_owned_city(
        &mut self,
        _faction_id: i32,
        _city_id: i32,
    ) -> Result<(), CountryGovernanceContextBlock> {
        Err(CountryGovernanceContextBlock::OwnedCityMutation)
    }
    fn refresh_owned_city(&mut self, _city_id: i32, _faction_id: i32, _union_id: i32) {}
    fn demise_faction(
        &mut self,
        _faction_id: i32,
        _old_master_id: i32,
        _new_master_id: i32,
        _country_id: u8,
        _king_id: i32,
        _demise_faction: bool,
    ) -> Result<bool, CountryGovernanceContextBlock> {
        Err(CountryGovernanceContextBlock::FactionDemise)
    }
    fn current_tick_ms(&mut self) -> u32 {
        0
    }
    fn country_name(&mut self, country_id: u8) -> Vec<u8>;
    fn country_identity_name(&mut self, identity: u8) -> Vec<u8>;
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8>;
    fn game_server_number_by_player_id(&mut self, player_id: i32) -> i32;
    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError>;
    fn send_to_connected_game_servers(
        &mut self,
        message: &CMessage,
    ) -> Vec<CountryExileMessageDelivery>;
    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError>;
    fn put_king_log(&mut self, text: &[u8]);
}

/// Узкая граница online/organizing/network эффектов player-list owner-а.
pub(crate) trait CountryPlayersListContext {
    fn online_players(&mut self) -> Vec<CountryOnlinePlayer>;
    fn player_faction(
        &mut self,
        player_id: i32,
    ) -> Result<(Vec<u8>, bool), CountryPlayersListContextBlock>;
    fn country_name(&mut self, country_id: u8) -> Vec<u8>;
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8>;
    fn game_server_number_by_player_id(&mut self, player_id: i32) -> i32;
    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError>;
    fn put_king_log(&mut self, text: &[u8]);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryDemiseRejection {
    CountryAtWar,
    InsufficientControlPoint,
    AppointmentPending,
    SamePlayer,
    PlayerMissing,
    KingMissing,
    InsufficientCredit,
    InsufficientLevel,
    FactionMissing,
    OwnedCityConflict,
    TargetCountryUnavailable,
    TargetFromAnotherCountry,
    OldKingNameMissing,
    OldKingNotFactionMaster,
    OldFactionMissing,
    OldFactionCityMissing,
    FactionTransferRejected,
    OldKingDeposeFailed,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryCanDemiseDisposition {
    Allowed,
    ParameterUnavailable(CountryParameterUnavailable),
    Rejected {
        reason: CountryDemiseRejection,
        text: Vec<u8>,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryCityTransferReport {
    pub(crate) city_id: i32,
    pub(crate) old_faction_id: i32,
    pub(crate) cleared_old_cities: Vec<i32>,
    pub(crate) new_faction_id: i32,
    pub(crate) new_union_id: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryDeposeKingReport {
    pub(crate) old_king_id: i32,
    pub(crate) legacy_result: i32,
    pub(crate) mode: u8,
    pub(crate) minister_reports: Vec<CountryDeposeMinisterReport>,
    pub(crate) appointment_wire: Option<Vec<u8>>,
    pub(crate) appointment_delivery: Option<Result<i32, SendMessageError>>,
    pub(crate) world_wire: Option<Vec<u8>>,
    pub(crate) world_delivery: Option<Result<i32, SendMessageError>>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryRegisterKingDisposition {
    Rejected(CountryDemiseRejection),
    ParameterUnavailable(CountryParameterUnavailable),
    ContextBlocked(CountryGovernanceContextBlock),
    Applied {
        city_transfer: Option<CountryCityTransferReport>,
        faction_transferred: bool,
        depose: CountryDeposeKingReport,
        control_point_update: KingPointUpdate,
        appointment_wire: Vec<u8>,
        appointment_delivery: Result<i32, SendMessageError>,
        world_wire: Option<Vec<u8>>,
        world_delivery: Option<Result<i32, SendMessageError>>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryRegisterKingReport {
    pub(crate) player_id: i32,
    pub(crate) text: Vec<u8>,
    pub(crate) disposition: CountryRegisterKingDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryDemiseDisposition {
    Rejected(CountryDemiseRejection),
    ParameterUnavailable(CountryParameterUnavailable),
    ContextBlocked(CountryGovernanceContextBlock),
    Applied {
        old_king_id: i32,
        register: CountryRegisterKingReport,
        control_point_delivery: CountryExileMessageDelivery,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryDemiseReport {
    pub(crate) target_player_id: i32,
    pub(crate) legacy_result: i32,
    pub(crate) text: Vec<u8>,
    pub(crate) disposition: CountryDemiseDisposition,
}

enum DemiseTargetBlock {
    Rejected(CountryDemiseRejection, Vec<u8>),
    Parameter(CountryParameterUnavailable),
    Context(CountryGovernanceContextBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountrySuccessExiledDisposition {
    PlayerMissing {
        private_delivery: Option<CountryExileMessageDelivery>,
    },
    ParameterUnavailable {
        block: CountryParameterUnavailable,
        king_map_id: i32,
        control_point_update: Option<KingPointUpdate>,
        control_point_delivery: Option<CountryExileMessageDelivery>,
    },
    Successful {
        control_point_update: KingPointUpdate,
        control_point_delivery: CountryExileMessageDelivery,
        previous_exile_count: i32,
        applied_exile_count: i32,
        country_deliveries: Vec<CountryExileMessageDelivery>,
    },
    Failed {
        private_delivery: Option<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountrySuccessExiledReport {
    pub(crate) player_id: i32,
    pub(crate) success: bool,
    pub(crate) text: Vec<u8>,
    pub(crate) disposition: CountrySuccessExiledDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryExileRejection {
    CountryAtWar,
    InsufficientControlPoint,
    DailyLimitReached,
    TargetIsKing,
    TargetMissing,
    TargetCountryUnavailable,
    TargetFromAnotherCountry,
    ExileRectMissing,
    TargetPkTooHigh,
    TargetRouteMissing,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryCanExileDisposition {
    Allowed,
    ParameterUnavailable(CountryParameterUnavailable),
    Rejected {
        reason: CountryExileRejection,
        text: Vec<u8>,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryExileRequestDisposition {
    ParameterUnavailable(CountryParameterUnavailable),
    Rejected {
        reason: CountryExileRejection,
        text: Vec<u8>,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
    Sent {
        map_id: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountrySilenceRejection {
    CountryAtWar,
    InsufficientControlPoint,
    DailyLimitReached,
    TargetIsKing,
    TargetMissing,
    TargetCountryUnavailable,
    TargetFromAnotherCountry,
    TargetIsGod,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryCanSilenceDisposition {
    Allowed,
    ParameterUnavailable(CountryParameterUnavailable),
    Rejected {
        reason: CountrySilenceRejection,
        text: Vec<u8>,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountrySilenceDisposition {
    Rejected {
        reason: CountrySilenceRejection,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
    ParameterUnavailable {
        block: CountryParameterUnavailable,
        previous_silence_count: i32,
        applied_silence_count: i32,
        control_point_update: Option<KingPointUpdate>,
    },
    Applied {
        previous_silence_count: i32,
        applied_silence_count: i32,
        control_point_update: KingPointUpdate,
        control_point_delivery: CountryExileMessageDelivery,
        target_delivery: CountryExileMessageDelivery,
        target_wire: Vec<u8>,
        country_deliveries: Vec<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountrySilenceReport {
    pub(crate) player_id: i32,
    pub(crate) legacy_result: i32,
    pub(crate) text: Vec<u8>,
    pub(crate) disposition: CountrySilenceDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryAbsolveRejection {
    CountryAtWar,
    InsufficientControlPoint,
    DailyLimitReached,
    TargetMissing,
    TargetCountryUnavailable,
    TargetFromAnotherCountry,
    TargetMutationUnavailable,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryCanAbsolveDisposition {
    Allowed,
    ParameterUnavailable(CountryParameterUnavailable),
    Rejected {
        reason: CountryAbsolveRejection,
        text: Vec<u8>,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryAbsolveDisposition {
    Rejected {
        reason: CountryAbsolveRejection,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
    ParameterUnavailable {
        block: CountryParameterUnavailable,
        counter_reset: CountryAbsolveCounterReset,
        control_point_update: Option<KingPointUpdate>,
    },
    Applied {
        counter_reset: CountryAbsolveCounterReset,
        control_point_update: KingPointUpdate,
        control_point_delivery: CountryExileMessageDelivery,
        previous_absolve_count: i32,
        applied_absolve_count: i32,
        broadcast_wire: Vec<u8>,
        broadcast_delivery: Result<i32, SendMessageError>,
        country_deliveries: Vec<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryAbsolveReport {
    pub(crate) player_id: i32,
    pub(crate) legacy_result: i32,
    pub(crate) text: Vec<u8>,
    pub(crate) disposition: CountryAbsolveDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryCanDeposeMinisterDisposition {
    Allowed,
    ParameterUnavailable(CountryParameterUnavailable),
    Rejected {
        text: Vec<u8>,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryBaseInfoDisposition {
    KingMissing,
    ParameterUnavailable(CountryParameterUnavailable),
    MinisterCountOutOfRange { minister_count: usize },
    Sent {
        map_id: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
        log: Vec<u8>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryDeposeMinisterDisposition {
    SlotMissing { inserted_null_slot: bool },
    Applied {
        previous_player_id: i32,
        previous_name: Vec<u8>,
        country_deliveries: Vec<CountryExileMessageDelivery>,
        appointment_wire: Vec<u8>,
        appointment_delivery: Result<i32, SendMessageError>,
        control_point_delivery: CountryExileMessageDelivery,
        base_info: CountryBaseInfoDisposition,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryDeposeMinisterReport {
    pub(crate) job: u8,
    pub(crate) mode: u8,
    pub(crate) legacy_result: i32,
    pub(crate) text: Vec<u8>,
    pub(crate) disposition: CountryDeposeMinisterDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryAppointMinisterRejection {
    CountryAtWar,
    InsufficientControlPoint,
    TargetIsKing,
    JobUnavailable,
    JobOccupied,
    AppointmentPending,
    TargetMissing,
    TargetCountryUnavailable,
    TargetFromAnotherCountry,
    TargetAlreadyHasJob { job: u8 },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryCanAppointMinisterDisposition {
    Allowed,
    ParameterUnavailable(CountryParameterUnavailable),
    Rejected {
        reason: CountryAppointMinisterRejection,
        text: Vec<u8>,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryAppointMinisterDisposition {
    Rejected {
        reason: CountryAppointMinisterRejection,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
    ParameterUnavailable {
        block: CountryParameterUnavailable,
        appointment_flag_set: bool,
        control_point_update: Option<KingPointUpdate>,
    },
    Applied {
        control_point_update: KingPointUpdate,
        country_deliveries: Vec<CountryExileMessageDelivery>,
        appointment_wire: Vec<u8>,
        appointment_delivery: Result<i32, SendMessageError>,
        control_point_delivery: CountryExileMessageDelivery,
        base_info: CountryBaseInfoDisposition,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryAppointMinisterReport {
    pub(crate) player_id: i32,
    pub(crate) job: u8,
    pub(crate) mode: u8,
    pub(crate) legacy_result: i32,
    pub(crate) text: Vec<u8>,
    pub(crate) disposition: CountryAppointMinisterDisposition,
}

impl CCountry {
    /// Повторяет exact `CCountry::NewTerm` без MSVC map/message plumbing.
    pub(crate) fn new_term<Context: CountryNewTermContext + ?Sized>(
        &mut self,
        context: &mut Context,
    ) -> CountryNewTermReport {
        let previous_king_appointed = self.king.appointed;
        let previous_king_salary_received = self.king.salary_received;
        self.king.appointed = false;
        self.king.salary_received = false;

        let minister_resets = self
            .ministers
            .iter_mut()
            .map(|(&job, minister)| {
                let reset = CountryMinisterTermReset {
                    job,
                    previous_appointed: minister.snapshot.appointed,
                    previous_salary_received: minister.snapshot.salary_received,
                };
                minister.snapshot.appointed = false;
                minister.snapshot.salary_received = false;
                reset
            })
            .collect();

        let message = CMessage::new(0x0007_FF14);
        let wire = message.as_wire_bytes().to_vec();
        let delivery = context.send_all(&message);

        let previous_silence_count = std::mem::replace(&mut self.silence_count, 0);
        let previous_pk_count = std::mem::replace(&mut self.pk_count, 0);
        let previous_exile_count = std::mem::replace(&mut self.exile_count, 0);
        let previous_absolve_count = std::mem::replace(&mut self.absolve_count, 0);
        CountryNewTermReport {
            previous_king_appointed,
            previous_king_salary_received,
            minister_resets,
            previous_silence_count,
            previous_pk_count,
            previous_exile_count,
            previous_absolve_count,
            wire,
            delivery,
        }
    }

    /// Exact `GetInfo`: wrapper без дополнительных side effect.
    pub(crate) fn get_info<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryBaseInfoDisposition {
        self.send_base_info_to_client(parameters, context)
    }

    /// Exact `SetKing`: return `DeposeKing(3)` игнорируется перед `0x7FF05`.
    pub(crate) fn set_king<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> Result<CountrySetKingReport, CountryGovernanceContextBlock> {
        let depose = self.depose_king(3, parameters, context)?;
        self.king.id = player_id;
        let mut message = CMessage::new(0x0007_FF05);
        message.base_mut().add_byte(self.country_id);
        message.base_mut().add_long(player_id);
        let wire = message.as_wire_bytes().to_vec();
        let delivery = context.send_all(&message);
        Ok(CountrySetKingReport {
            player_id,
            depose,
            wire,
            delivery,
            legacy_result: player_id,
        })
    }

    /// Exact mode `0` исходного `RegisterKing` после административного `SetKing`.
    pub(crate) fn register_initial_king<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryInitialKingReport {
        let Some(player) = context.online_player(player_id) else {
            return self.reject_initial_king(
                player_id,
                CountryInitialKingRejection::PlayerMissing,
                b"WS0016",
                &[],
                true,
                context,
            );
        };
        let Some(player_country) = player.country else {
            return CountryInitialKingReport {
                player_id,
                text: Vec::new(),
                legacy_result: 0,
                disposition: CountryInitialKingDisposition::Rejected {
                    reason: CountryInitialKingRejection::TargetCountryUnavailable,
                    private_delivery: None,
                },
            };
        };
        if player_country != self.country_id {
            return self.reject_initial_king(
                player_id,
                CountryInitialKingRejection::TargetFromAnotherCountry,
                b"WS0022",
                &[],
                true,
                context,
            );
        }
        let faction_id = match context.faction_id_by_player(player_id) {
            Ok(faction_id) => faction_id,
            Err(block) => {
                return CountryInitialKingReport {
                    player_id,
                    text: Vec::new(),
                    legacy_result: 0,
                    disposition: CountryInitialKingDisposition::ContextBlocked(block),
                };
            }
        };
        let Some(faction) = context.faction_snapshot(faction_id).filter(|_| faction_id > 0) else {
            let country_name = context.country_name(self.country_id);
            return self.reject_initial_king(
                player_id,
                CountryInitialKingRejection::FactionMissing,
                b"WS0023",
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&player.name),
                ],
                false,
                context,
            );
        };
        if self.king.id != player_id {
            let country_name = context.country_name(self.country_id);
            return self.reject_initial_king(
                player_id,
                CountryInitialKingRejection::KingMismatch,
                b"WS0024",
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&player.name),
                ],
                false,
                context,
            );
        }
        let Some(default_control_point) = parameters.default_king_control_point() else {
            return CountryInitialKingReport {
                player_id,
                text: Vec::new(),
                legacy_result: 0,
                disposition: CountryInitialKingDisposition::ParameterUnavailable(
                    CountryParameterUnavailable { field: "_def_king_control_point" },
                ),
            };
        };
        let control_point_update = match set_control_point(
            &mut self.king,
            default_control_point,
            parameters,
        ) {
            Ok(update) => update,
            Err(block) => {
                return CountryInitialKingReport {
                    player_id,
                    text: Vec::new(),
                    legacy_result: 0,
                    disposition: CountryInitialKingDisposition::ParameterUnavailable(block),
                };
            }
        };
        let country_name = context.country_name(self.country_id);
        let text = legacy_country_text(context.format_world_string(
            b"WS0025",
            &[
                CountryExileTextArgument::Text(&faction.name),
                CountryExileTextArgument::Text(&player.name),
                CountryExileTextArgument::Text(&country_name),
            ],
        ));
        context.put_king_log(&text);
        self.king.name = player.name;
        self.king_timestamp_ms = context.current_tick_ms();
        let mut appointment = CMessage::new(0x0007_FF04);
        appointment.base_mut().add_byte(self.country_id);
        appointment.base_mut().add_long(player_id);
        appointment.base_mut().add_byte(1);
        appointment.base_mut().add_byte(1);
        let appointment_wire = appointment.as_wire_bytes().to_vec();
        let appointment_delivery = context.send_all(&appointment);
        let world = self.send_world_message(&text, context);
        CountryInitialKingReport {
            player_id,
            text,
            legacy_result: player_id,
            disposition: CountryInitialKingDisposition::Applied {
                faction_id,
                control_point_update,
                appointment_wire,
                appointment_delivery,
                world_wire: world.as_ref().map(|(wire, _)| wire.clone()),
                world_delivery: world.map(|(_, delivery)| delivery),
            },
        }
    }

    fn reject_initial_king<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        reason: CountryInitialKingRejection,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
        notify_king: bool,
        context: &mut Context,
    ) -> CountryInitialKingReport {
        let text = legacy_country_text(context.format_world_string(string_id, arguments));
        context.put_king_log(&text);
        let private_delivery = notify_king
            .then(|| self.send_private_message(&text, 0, context))
            .flatten();
        CountryInitialKingReport {
            player_id,
            text,
            legacy_result: 0,
            disposition: CountryInitialKingDisposition::Rejected {
                reason,
                private_delivery,
            },
        }
    }

    /// Exact `IsKing` для player-list owner-а с тем же `WS0033/WS0034`.
    pub(crate) fn authorize_king_for_players<Context: CountryPlayersListContext + ?Sized>(
        &self,
        candidate: i32,
        context: &mut Context,
    ) -> bool {
        if candidate != 0 && self.king.id == candidate {
            return true;
        }
        let country_name = context.country_name(self.country_id);
        let string_id = if candidate == 0 { b"WS0033" } else { b"WS0034" };
        let text = legacy_country_text(context.format_world_string(
            string_id,
            &[CountryExileTextArgument::Text(&country_name)],
        ));
        context.put_king_log(&text);
        false
    }

    /// Exact `InitialOLPlayersList -> Sort -> GetPlayersList` без MSVC owner-указателей.
    pub(crate) fn get_players_list<Context: CountryPlayersListContext + ?Sized>(
        &self,
        page: i32,
        context: &mut Context,
    ) -> Result<CountryPlayersListReport, CountryPlayersListContextBlock> {
        let mut sorted_players = Vec::new();
        for (online_index, player) in context.online_players().into_iter().enumerate() {
            if player.country != Some(self.country_id) || player.level < 10 || player.is_god {
                continue;
            }
            let (faction_name, is_faction_master) = context.player_faction(player.id)?;
            sorted_players.push((
                online_index,
                CountryPlayerInfo {
                    id: player.id,
                    name: player.name,
                    occupation: player.occupation,
                    level: player.level,
                    faction_name,
                    is_faction_master,
                },
            ));
        }

        let country_name = context.country_name(self.country_id);
        let mut initial_log = country_name.clone();
        initial_log.extend_from_slice(b" : Successfully InitialOLPlayersList!");
        let initial_log = legacy_country_text(initial_log);
        context.put_king_log(&initial_log);

        // Reverse multimap traversal: level по убыванию, равные ключи в
        // обратном порядке исходного online-list.
        sorted_players.sort_by(|(left_index, left), (right_index, right)| {
            right
                .level
                .cmp(&left.level)
                .then_with(|| right_index.cmp(left_index))
        });
        let mut sort_log = country_name.clone();
        sort_log.extend_from_slice(b" : Successfully Sort!");
        let sort_log = legacy_country_text(sort_log);
        context.put_king_log(&sort_log);

        let total = sorted_players.len() as u32;
        let total_signed = total as i32;
        let list_log = legacy_country_text(context.format_world_string(
            b"WS0021",
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Signed(total_signed),
            ],
        ));
        context.put_king_log(&list_log);

        let start_index = page
            .wrapping_mul(3)
            .wrapping_sub(3)
            .wrapping_mul(4) as u32;
        let (end_index, entries) = if start_index < total {
            let end_index = start_index.wrapping_add(12).min(total);
            let entries = sorted_players[start_index as usize..end_index as usize]
                .iter()
                .map(|(_, player)| player.clone())
                .collect::<Vec<_>>();
            (end_index, entries)
        } else {
            (start_index, Vec::new())
        };
        let count = end_index.wrapping_sub(start_index) as i32;

        let mut message = CMessage::new(0x0007_FF08);
        message.base_mut().add_long(self.king.id);
        message.base_mut().add_long(count);
        message.base_mut().add_long(total_signed);
        for player in &entries {
            let name = CString::new(legacy_c_string_prefix(&player.name))
                .expect("player-name C-string prefix не содержит NUL");
            let faction_name = CString::new(legacy_c_string_prefix(&player.faction_name))
                .expect("faction-name C-string prefix не содержит NUL");
            message.base_mut().add_long(player.id);
            message.base_mut().add_str(Some(&name));
            message.base_mut().add_byte(player.occupation);
            message.base_mut().add_byte(player.level);
            message.base_mut().add_str(Some(&faction_name));
            message
                .base_mut()
                .add_byte(u8::from(player.is_faction_master));
        }
        let map_id = context.game_server_number_by_player_id(self.king.id);
        let wire = message.as_wire_bytes().to_vec();
        let delivery = context.send_to_map_id(&message, map_id);
        Ok(CountryPlayersListReport {
            page,
            start_index,
            total,
            entries,
            map_id,
            wire,
            delivery,
            logs: vec![initial_log, sort_log, list_log],
            legacy_result: self.king.id,
        })
    }

    /// Exact `IsKing`: нулевой candidate всегда отклоняется с `WS0033`.
    pub(crate) fn authorize_king<Context: CountryExileResultContext + ?Sized>(
        &self,
        candidate: i32,
        context: &mut Context,
    ) -> bool {
        if candidate != 0 && self.king.id == candidate {
            return true;
        }
        let country_name = context.country_name(self.country_id);
        let string_id = if candidate == 0 { b"WS0033" } else { b"WS0034" };
        let text = legacy_country_text(context.format_world_string(
            string_id,
            &[CountryExileTextArgument::Text(&country_name)],
        ));
        context.put_king_log(&text);
        false
    }

    /// Exact `IsMinister`: job участвует только в отрицательном king-log.
    pub(crate) fn authorize_minister<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        job: u8,
        context: &mut Context,
    ) -> bool {
        if player_id != 0 && self.has_minister_id(player_id) {
            return true;
        }
        let country_name = context.country_name(self.country_id);
        let identity_name = context.country_identity_name(job);
        let string_id = if player_id == 0 { b"WS0036" } else { b"WS0037" };
        let text = legacy_country_text(context.format_world_string(
            string_id,
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&identity_name),
            ],
        ));
        context.put_king_log(&text);
        false
    }

    /// Exact selector `CanOperate(0)` перед передачей престола.
    pub(crate) fn can_demise<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryCanDemiseDisposition {
        if parameters.min_king_control_point().is_none() {
            return CountryCanDemiseDisposition::ParameterUnavailable(
                CountryParameterUnavailable { field: "_min_king_control_point" },
            );
        }
        let Some(required) = parameters.demise_required_control_point() else {
            return CountryCanDemiseDisposition::ParameterUnavailable(
                CountryParameterUnavailable { field: "_def_king_control_point_demise_need" },
            );
        };
        let rejection = if self.is_warring {
            Some((CountryDemiseRejection::CountryAtWar, b"WS0038" as &'static [u8], None))
        } else if self.king.control_point < required {
            Some((
                CountryDemiseRejection::InsufficientControlPoint,
                b"WS0039" as &'static [u8],
                Some(required),
            ))
        } else if self.king.appointed {
            Some((CountryDemiseRejection::AppointmentPending, b"WS0040" as &'static [u8], None))
        } else {
            None
        };
        let Some((reason, string_id, argument)) = rejection else {
            return CountryCanDemiseDisposition::Allowed;
        };
        let arguments = argument
            .as_ref()
            .map(|value| [CountryExileTextArgument::Signed(*value)]);
        let text = legacy_country_text(context.format_world_string(
            string_id,
            arguments.as_ref().map_or(&[], |arguments| arguments.as_slice()),
        ));
        let private_delivery = self.send_private_message(&text, 0, context);
        CountryCanDemiseDisposition::Rejected { reason, text, private_delivery }
    }

    /// Exact `0x60308 -> Demise`: возвращает прежнего короля даже если
    /// вложенный `RegisterKing` отказал, потому что старый caller его результат
    /// не проверял.
    pub(crate) fn demise<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryDemiseReport {
        if self.king.id == player_id {
            let country_name = context.country_name(self.country_id);
            let text = legacy_country_text(context.format_world_string(
                b"WS0058",
                &[CountryExileTextArgument::Text(&country_name)],
            ));
            context.put_king_log(&text);
            return CountryDemiseReport {
                target_player_id: player_id,
                legacy_result: 0,
                text,
                disposition: CountryDemiseDisposition::Rejected(
                    CountryDemiseRejection::SamePlayer,
                ),
            };
        }

        if let Err(block) = self.can_demise_target(player_id, parameters, context) {
            return match block {
                DemiseTargetBlock::Rejected(reason, text) => CountryDemiseReport {
                    target_player_id: player_id,
                    legacy_result: 0,
                    text,
                    disposition: CountryDemiseDisposition::Rejected(reason),
                },
                DemiseTargetBlock::Parameter(block) => CountryDemiseReport {
                    target_player_id: player_id,
                    legacy_result: 0,
                    text: Vec::new(),
                    disposition: CountryDemiseDisposition::ParameterUnavailable(block),
                },
                DemiseTargetBlock::Context(block) => CountryDemiseReport {
                    target_player_id: player_id,
                    legacy_result: 0,
                    text: Vec::new(),
                    disposition: CountryDemiseDisposition::ContextBlocked(block),
                },
            };
        }

        let old_king_id = self.king.id;
        let register = self.register_king_demise(player_id, parameters, context);
        let map_id = context.game_server_number_by_player_id(self.king.id);
        let control_point_delivery = self.send_king_control_point(map_id, context);
        CountryDemiseReport {
            target_player_id: player_id,
            legacy_result: old_king_id,
            text: register.text.clone(),
            disposition: CountryDemiseDisposition::Applied {
                old_king_id,
                register,
                control_point_delivery,
            },
        }
    }

    fn can_demise_target<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> Result<(), DemiseTargetBlock> {
        let old_king = context.online_player(self.king.id);
        let candidate = context.online_player(player_id);
        if old_king.is_none() || candidate.is_none() {
            let text = legacy_country_text(context.format_world_string(b"WS0016", &[]));
            context.put_king_log(&text);
            let _ = self.send_private_message(&text, 0, context);
            return Err(DemiseTargetBlock::Rejected(
                CountryDemiseRejection::PlayerMissing,
                text,
            ));
        }
        let target_master_faction = context
            .faction_id_by_master_player(player_id)
            .map_err(DemiseTargetBlock::Context)?;
        if target_master_faction == 0 {
            self.demise_faction = true;
        }
        self.can_ascend_for_demise(player_id, parameters, context)
    }

    fn can_ascend_for_demise<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> Result<(), DemiseTargetBlock> {
        let Some(player) = context.online_player(player_id) else {
            let text = legacy_country_text(context.format_world_string(b"WS0016", &[]));
            context.put_king_log(&text);
            let _ = self.send_private_message(&text, 0, context);
            return Err(DemiseTargetBlock::Rejected(
                CountryDemiseRejection::PlayerMissing,
                text,
            ));
        };
        if self.king.id == 0 {
            let mut text = context.country_name(self.country_id);
            text.extend_from_slice(b" : [Fatal ERROR] Ascend. KingID ==0!");
            let text = legacy_country_text(text);
            context.put_king_log(&text);
            return Err(DemiseTargetBlock::Rejected(
                CountryDemiseRejection::KingMissing,
                text,
            ));
        }
        let required_credit = parameters
            .king_need_credit()
            .ok_or(DemiseTargetBlock::Parameter(CountryParameterUnavailable {
                field: "_king_need_credit",
            }))?;
        if player.credit < required_credit as u32 {
            let country_name = context.country_name(self.country_id);
            let text = legacy_country_text(context.format_world_string(
                b"WS0017",
                &[CountryExileTextArgument::Text(&country_name)],
            ));
            context.put_king_log(&text);
            let _ = self.send_private_message(&text, 0, context);
            return Err(DemiseTargetBlock::Rejected(
                CountryDemiseRejection::InsufficientCredit,
                text,
            ));
        }
        let required_level = parameters
            .king_need_level()
            .ok_or(DemiseTargetBlock::Parameter(CountryParameterUnavailable {
                field: "_king_need_level",
            }))?;
        if i32::from(player.level) < required_level {
            let text = legacy_country_text(context.format_world_string(
                b"WS0018",
                &[CountryExileTextArgument::Signed(required_level)],
            ));
            context.put_king_log(&text);
            let _ = self.send_private_message(&text, 0, context);
            return Err(DemiseTargetBlock::Rejected(
                CountryDemiseRejection::InsufficientLevel,
                text,
            ));
        }

        let mut faction_id = context
            .faction_id_by_master_player(player_id)
            .map_err(DemiseTargetBlock::Context)?;
        if faction_id == 0 {
            faction_id = context
                .faction_id_by_player(player_id)
                .map_err(DemiseTargetBlock::Context)?;
            let king_faction_id = context
                .faction_id_by_player(self.king.id)
                .map_err(DemiseTargetBlock::Context)?;
            if faction_id != king_faction_id {
                faction_id = 0;
            }
        }
        let Some(faction) = context.faction_snapshot(faction_id).filter(|_| faction_id != 0) else {
            return Err(DemiseTargetBlock::Rejected(
                CountryDemiseRejection::FactionMissing,
                Vec::new(),
            ));
        };
        if !self.demise_faction && !faction.owned_cities.is_empty() {
            // В EXE сюда ошибочно передаётся player ID короля, а не faction ID.
            let old_king_union = context
                .union_id_for_faction(self.king.id)
                .map_err(DemiseTargetBlock::Context)?;
            if old_king_union != faction_id {
                let text = legacy_country_text(context.format_world_string(b"WS0019", &[]));
                context.put_king_log(&text);
                let _ = self.send_private_message(&text, 0, context);
                return Err(DemiseTargetBlock::Rejected(
                    CountryDemiseRejection::OwnedCityConflict,
                    text,
                ));
            }
        }
        let country_name = context.country_name(self.country_id);
        let text = legacy_country_text(context.format_world_string(
            b"WS0020",
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&player.name),
            ],
        ));
        context.put_king_log(&text);
        Ok(())
    }

    fn register_king_demise<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryRegisterKingReport {
        let Some(player) = context.online_player(player_id) else {
            return self.reject_register_king(
                player_id,
                CountryDemiseRejection::PlayerMissing,
                b"WS0016",
                &[],
                true,
                context,
            );
        };
        let Some(player_country) = player.country else {
            return CountryRegisterKingReport {
                player_id,
                text: Vec::new(),
                disposition: CountryRegisterKingDisposition::Rejected(
                    CountryDemiseRejection::TargetCountryUnavailable,
                ),
            };
        };
        if player_country != self.country_id {
            return self.reject_register_king(
                player_id,
                CountryDemiseRejection::TargetFromAnotherCountry,
                b"WS0022",
                &[],
                true,
                context,
            );
        }
        let faction_id = match context.faction_id_by_player(player_id) {
            Ok(faction_id) => faction_id,
            Err(block) => {
                return CountryRegisterKingReport {
                    player_id,
                    text: Vec::new(),
                    disposition: CountryRegisterKingDisposition::ContextBlocked(block),
                };
            }
        };
        let Some(candidate_faction) = context
            .faction_snapshot(faction_id)
            .filter(|_| faction_id != 0)
        else {
            let country_name = context.country_name(self.country_id);
            return self.reject_register_king(
                player_id,
                CountryDemiseRejection::FactionMissing,
                b"WS0023",
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&player.name),
                ],
                false,
                context,
            );
        };

        let old_king_name = self.king.name.clone();
        if old_king_name.is_empty() {
            let country_name = context.country_name(self.country_id);
            return self.reject_register_king(
                player_id,
                CountryDemiseRejection::OldKingNameMissing,
                b"WS0026",
                &[CountryExileTextArgument::Text(&country_name)],
                false,
                context,
            );
        }
        let old_faction_id = match context.faction_id_by_master_player(self.king.id) {
            Ok(faction_id) => faction_id,
            Err(block) => {
                return CountryRegisterKingReport {
                    player_id,
                    text: Vec::new(),
                    disposition: CountryRegisterKingDisposition::ContextBlocked(block),
                };
            }
        };
        if old_faction_id == 0 {
            let country_name = context.country_name(self.country_id);
            return self.reject_register_king(
                player_id,
                CountryDemiseRejection::OldKingNotFactionMaster,
                b"WS0027",
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&old_king_name),
                ],
                false,
                context,
            );
        }

        let mut city_transfer = None;
        let faction_transferred;
        if !self.demise_faction {
            faction_transferred = false;
            let Some(old_faction) = context.faction_snapshot(old_faction_id) else {
                return CountryRegisterKingReport {
                    player_id,
                    text: Vec::new(),
                    disposition: CountryRegisterKingDisposition::Rejected(
                        CountryDemiseRejection::OldFactionMissing,
                    ),
                };
            };
            let city_id = old_faction.owned_cities.first().copied().unwrap_or(0);
            self.city_id = city_id;
            if city_id == 0 {
                let country_name = context.country_name(self.country_id);
                return self.reject_register_king(
                    player_id,
                    CountryDemiseRejection::OldFactionCityMissing,
                    b"WS0029",
                    &[
                        CountryExileTextArgument::Text(&country_name),
                        CountryExileTextArgument::Text(&old_king_name),
                    ],
                    false,
                    context,
                );
            }
            if let Err(block) = context.clear_faction_owned_cities(old_faction_id) {
                return CountryRegisterKingReport {
                    player_id,
                    text: Vec::new(),
                    disposition: CountryRegisterKingDisposition::ContextBlocked(block),
                };
            }
            if let Err(block) = context.add_faction_owned_city(faction_id, city_id) {
                return CountryRegisterKingReport {
                    player_id,
                    text: Vec::new(),
                    disposition: CountryRegisterKingDisposition::ContextBlocked(block),
                };
            }
            let new_union_id = match context.union_id_for_faction(candidate_faction.faction_id) {
                Ok(union_id) => union_id,
                Err(block) => {
                    return CountryRegisterKingReport {
                        player_id,
                        text: Vec::new(),
                        disposition: CountryRegisterKingDisposition::ContextBlocked(block),
                    };
                }
            };
            context.refresh_owned_city(city_id, faction_id, new_union_id);
            let mut city_message = CMessage::new(0x0007_FE27);
            city_message.base_mut().add_long(city_id);
            city_message.base_mut().add_long(faction_id);
            city_message.base_mut().add_long(0);
            city_message.base_mut().add_byte(self.country_id);
            let wire = city_message.as_wire_bytes().to_vec();
            let delivery = context.send_all(&city_message);
            city_transfer = Some(CountryCityTransferReport {
                city_id,
                old_faction_id,
                cleared_old_cities: old_faction.owned_cities,
                new_faction_id: faction_id,
                new_union_id,
                wire,
                delivery,
            });
        } else {
            faction_transferred = match context.demise_faction(
                faction_id,
                self.king.id,
                player_id,
                self.country_id,
                self.king.id,
                self.demise_faction,
            ) {
                Ok(transferred) => transferred,
                Err(block) => {
                    return CountryRegisterKingReport {
                        player_id,
                        text: Vec::new(),
                        disposition: CountryRegisterKingDisposition::ContextBlocked(block),
                    };
                }
            };
            if !faction_transferred {
                let country_name = context.country_name(self.country_id);
                return self.reject_register_king(
                    player_id,
                    CountryDemiseRejection::FactionTransferRejected,
                    b"WS0028",
                    &[
                        CountryExileTextArgument::Text(&country_name),
                        CountryExileTextArgument::Text(&player.name),
                    ],
                    false,
                    context,
                );
            }
        }

        let depose = match self.depose_king(2, parameters, context) {
            Ok(report) => report,
            Err(block) => {
                return CountryRegisterKingReport {
                    player_id,
                    text: Vec::new(),
                    disposition: CountryRegisterKingDisposition::ContextBlocked(block),
                };
            }
        };
        if depose.legacy_result == 0 {
            let country_name = context.country_name(self.country_id);
            return self.reject_register_king(
                player_id,
                CountryDemiseRejection::OldKingDeposeFailed,
                b"WS0030",
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&old_king_name),
                ],
                false,
                context,
            );
        }

        self.king.id = player_id;
        self.king.appointed = true;
        let Some(cost) = parameters.demise_control_point_cost() else {
            return CountryRegisterKingReport {
                player_id,
                text: Vec::new(),
                disposition: CountryRegisterKingDisposition::ParameterUnavailable(
                    CountryParameterUnavailable { field: "_dec_king_control_point_demise" },
                ),
            };
        };
        let requested = self.king.control_point.wrapping_sub(cost);
        let control_point_update = match set_control_point(&mut self.king, requested, parameters) {
            Ok(update) => update,
            Err(block) => {
                return CountryRegisterKingReport {
                    player_id,
                    text: Vec::new(),
                    disposition: CountryRegisterKingDisposition::ParameterUnavailable(block),
                };
            }
        };
        let country_name = context.country_name(self.country_id);
        let text = legacy_country_text(context.format_world_string(
            b"WS0031",
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&old_king_name),
                CountryExileTextArgument::Text(&player.name),
            ],
        ));
        context.put_king_log(&text);
        self.demise_faction = false;
        self.king.name = player.name;
        self.king_timestamp_ms = context.current_tick_ms();

        let mut appointment = CMessage::new(0x0007_FF04);
        appointment.base_mut().add_byte(self.country_id);
        appointment.base_mut().add_long(player_id);
        appointment.base_mut().add_byte(1);
        appointment.base_mut().add_byte(1);
        let appointment_wire = appointment.as_wire_bytes().to_vec();
        let appointment_delivery = context.send_all(&appointment);
        let world = self.send_world_message(&text, context);
        CountryRegisterKingReport {
            player_id,
            text,
            disposition: CountryRegisterKingDisposition::Applied {
                city_transfer,
                faction_transferred,
                depose,
                control_point_update,
                appointment_wire,
                appointment_delivery,
                world_wire: world.as_ref().map(|(wire, _)| wire.clone()),
                world_delivery: world.map(|(_, delivery)| delivery),
            },
        }
    }

    fn reject_register_king<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        reason: CountryDemiseRejection,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
        notify_king: bool,
        context: &mut Context,
    ) -> CountryRegisterKingReport {
        let text = legacy_country_text(context.format_world_string(string_id, arguments));
        context.put_king_log(&text);
        if notify_king {
            let _ = self.send_private_message(&text, 0, context);
        }
        CountryRegisterKingReport {
            player_id,
            text,
            disposition: CountryRegisterKingDisposition::Rejected(reason),
        }
    }

    fn depose_king<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        mode: u8,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> Result<CountryDeposeKingReport, CountryGovernanceContextBlock> {
        let old_king_id = self.king.id;
        if old_king_id == 0 && self.king.name.is_empty() {
            let country_name = context.country_name(self.country_id);
            let text = legacy_country_text(context.format_world_string(
                b"WS0053",
                &[CountryExileTextArgument::Text(&country_name)],
            ));
            context.put_king_log(&text);
            return Ok(CountryDeposeKingReport {
                old_king_id,
                legacy_result: 0,
                mode,
                minister_reports: Vec::new(),
                appointment_wire: None,
                appointment_delivery: None,
                world_wire: None,
                world_delivery: None,
            });
        }

        let mut appointment = CMessage::new(0x0007_FF04);
        appointment.base_mut().add_byte(self.country_id);
        appointment.base_mut().add_long(old_king_id);
        appointment.base_mut().add_byte(1);
        appointment.base_mut().add_byte(2);
        let appointment_wire = appointment.as_wire_bytes().to_vec();
        let appointment_delivery = context.send_all(&appointment);

        if !self.demise_faction {
            let faction_id = context.faction_id_by_player(old_king_id)?;
            if faction_id == 0 {
                self.king.id = 0;
                self.king.name.clear();
                let country_name = context.country_name(self.country_id);
                let text = legacy_country_text(context.format_world_string(
                    b"WS0054",
                    &[
                        CountryExileTextArgument::Text(&country_name),
                        CountryExileTextArgument::Signed(old_king_id),
                    ],
                ));
                context.put_king_log(&text);
            }
            if faction_id <= 0 || context.faction_snapshot(faction_id).is_none() {
                let country_name = context.country_name(self.country_id);
                let text = legacy_country_text(context.format_world_string(
                    b"WS0055",
                    &[CountryExileTextArgument::Text(&country_name)],
                ));
                context.put_king_log(&text);
                return Ok(CountryDeposeKingReport {
                    old_king_id,
                    legacy_result: 0,
                    mode,
                    minister_reports: Vec::new(),
                    appointment_wire: Some(appointment_wire),
                    appointment_delivery: Some(appointment_delivery),
                    world_wire: None,
                    world_delivery: None,
                });
            }
        }

        let old_king_name = self.king.name.clone();
        let text = if old_king_name.is_empty() {
            Vec::new()
        } else if mode == 3 || mode == 4 {
            let country_name = context.country_name(self.country_id);
            legacy_country_text(context.format_world_string(
                if mode == 3 { b"WS0056" } else { b"WS0057" },
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&old_king_name),
                ],
            ))
        } else {
            Vec::new()
        };
        let jobs = self
            .ministers
            .iter()
            .filter_map(|(&job, minister)| (minister.snapshot.id != 0).then_some(job))
            .collect::<Vec<_>>();
        let mut minister_reports = Vec::with_capacity(jobs.len());
        for job in jobs {
            minister_reports.push(self.depose_minister(job, mode, parameters, context));
        }
        context.put_king_log(&text);
        let world = self.send_world_message(&text, context);
        self.king.id = 0;
        self.king.name.clear();
        Ok(CountryDeposeKingReport {
            old_king_id,
            legacy_result: old_king_id,
            mode,
            minister_reports,
            appointment_wire: Some(appointment_wire),
            appointment_delivery: Some(appointment_delivery),
            world_wire: world.as_ref().map(|(wire, _)| wire.clone()),
            world_delivery: world.map(|(_, delivery)| delivery),
        })
    }

    pub(crate) fn can_depose_minister<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryCanDeposeMinisterDisposition {
        if parameters.min_king_control_point().is_none() {
            return CountryCanDeposeMinisterDisposition::ParameterUnavailable(
                CountryParameterUnavailable { field: "_min_king_control_point" },
            );
        }
        if !self.is_warring {
            return CountryCanDeposeMinisterDisposition::Allowed;
        }
        let text = legacy_country_text(context.format_world_string(b"WS0043", &[]));
        let private_delivery = self.send_private_message(&text, 0, context);
        CountryCanDeposeMinisterDisposition::Rejected { text, private_delivery }
    }

    pub(crate) fn can_appoint_minister<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryCanAppointMinisterDisposition {
        let Some(minimum) = parameters.min_king_control_point() else {
            return CountryCanAppointMinisterDisposition::ParameterUnavailable(
                CountryParameterUnavailable { field: "_min_king_control_point" },
            );
        };
        let rejection = if self.is_warring {
            Some((CountryAppointMinisterRejection::CountryAtWar, b"WS0041" as &'static [u8], None))
        } else if self.king.control_point < minimum {
            Some((
                CountryAppointMinisterRejection::InsufficientControlPoint,
                b"WS0042" as &'static [u8],
                Some(minimum),
            ))
        } else {
            None
        };
        let Some((reason, string_id, argument)) = rejection else {
            return CountryCanAppointMinisterDisposition::Allowed;
        };
        let arguments = argument
            .as_ref()
            .map(|value| [CountryExileTextArgument::Signed(*value)]);
        let text = legacy_country_text(context.format_world_string(
            string_id,
            arguments.as_ref().map_or(&[], |arguments| arguments.as_slice()),
        ));
        let private_delivery = self.send_private_message(&text, 0, context);
        CountryCanAppointMinisterDisposition::Rejected { reason, text, private_delivery }
    }

    /// Exact positive-player ветка `AppointMinister(player, job, 6)`.
    pub(crate) fn appoint_minister<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        job: u8,
        mode: u8,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryAppointMinisterReport {
        if player_id != 0 && self.king.id == player_id {
            let country_name = context.country_name(self.country_id);
            return self.reject_appoint_minister(
                player_id,
                job,
                mode,
                CountryAppointMinisterRejection::TargetIsKing,
                b"WS0059",
                &[CountryExileTextArgument::Text(&country_name)],
                false,
                context,
            );
        }
        let Some(minister) = self.ministers.get(&job) else {
            let country_name = context.country_name(self.country_id);
            let identity_name = context.country_identity_name(job);
            return self.reject_appoint_minister(
                player_id,
                job,
                mode,
                CountryAppointMinisterRejection::JobUnavailable,
                b"WS0060",
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&identity_name),
                ],
                false,
                context,
            );
        };
        if minister.snapshot.id != 0 {
            let identity_name = context.country_identity_name(job);
            return self.reject_appoint_minister(
                player_id,
                job,
                mode,
                CountryAppointMinisterRejection::JobOccupied,
                b"WS0061",
                &[CountryExileTextArgument::Text(&identity_name)],
                true,
                context,
            );
        }
        if minister.snapshot.appointed {
            let identity_name = context.country_identity_name(job);
            return self.reject_appoint_minister(
                player_id,
                job,
                mode,
                CountryAppointMinisterRejection::AppointmentPending,
                b"WS0062",
                &[CountryExileTextArgument::Text(&identity_name)],
                true,
                context,
            );
        }
        let Some(player) = context.online_player(player_id) else {
            return self.reject_appoint_minister(
                player_id,
                job,
                mode,
                CountryAppointMinisterRejection::TargetMissing,
                b"WS0063",
                &[],
                true,
                context,
            );
        };
        let Some(player_country) = player.country else {
            return CountryAppointMinisterReport {
                player_id,
                job,
                mode,
                legacy_result: player_id,
                text: Vec::new(),
                disposition: CountryAppointMinisterDisposition::Rejected {
                    reason: CountryAppointMinisterRejection::TargetCountryUnavailable,
                    private_delivery: None,
                },
            };
        };
        if player_country != self.country_id {
            return self.reject_appoint_minister(
                player_id,
                job,
                mode,
                CountryAppointMinisterRejection::TargetFromAnotherCountry,
                b"WS0064",
                &[],
                true,
                context,
            );
        }
        let existing_job = self.has_job_with_legacy_king_check(player_id, context);
        if existing_job != 0 {
            let identity_name = context.country_identity_name(existing_job);
            return self.reject_appoint_minister(
                player_id,
                job,
                mode,
                CountryAppointMinisterRejection::TargetAlreadyHasJob { job: existing_job },
                b"WS0065",
                &[
                    CountryExileTextArgument::Text(&player.name),
                    CountryExileTextArgument::Text(&identity_name),
                ],
                true,
                context,
            );
        }
        self.ministers.get_mut(&job).expect("minister проверен выше").snapshot.appointed = true;
        let Some(cost) = parameters.appoint_control_point_cost() else {
            return self.appoint_parameter_unavailable(
                player_id,
                job,
                mode,
                CountryParameterUnavailable { field: "_dec_king_control_point_appoint" },
                None,
            );
        };
        let requested = self.king.control_point.wrapping_sub(cost);
        let control_point_update = match set_control_point(&mut self.king, requested, parameters) {
            Ok(update) => update,
            Err(block) => {
                return self.appoint_parameter_unavailable(player_id, job, mode, block, None);
            }
        };
        let country_name = context.country_name(self.country_id);
        let identity_name = context.country_identity_name(job);
        let text = legacy_country_text(context.format_world_string(
            b"WS0066",
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&player.name),
                CountryExileTextArgument::Text(&identity_name),
            ],
        ));
        let country_deliveries = self.send_country_message(&text, context);
        context.put_king_log(&text);
        let minister = self.ministers.get_mut(&job).expect("minister проверен выше");
        minister.snapshot.id = player_id;
        minister.snapshot.name = player.name;
        let king_map_id = context.game_server_number_by_player_id(self.king.id);
        let mut appointment = CMessage::new(0x0007_FF04);
        appointment.base_mut().add_byte(self.country_id);
        appointment.base_mut().add_long(player_id);
        appointment.base_mut().add_byte(job);
        appointment.base_mut().add_byte(1);
        let appointment_wire = appointment.as_wire_bytes().to_vec();
        let appointment_delivery = context.send_all(&appointment);
        let control_point_delivery = self.send_king_control_point(king_map_id, context);
        let base_info = self.send_base_info_to_client(parameters, context);
        CountryAppointMinisterReport {
            player_id,
            job,
            mode,
            legacy_result: player_id,
            text,
            disposition: CountryAppointMinisterDisposition::Applied {
                control_point_update,
                country_deliveries,
                appointment_wire,
                appointment_delivery,
                control_point_delivery,
                base_info,
            },
        }
    }

    fn has_job_with_legacy_king_check<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        context: &mut Context,
    ) -> u8 {
        if self.authorize_king(player_id, context) {
            return 1;
        }
        self.ministers
            .iter()
            .find_map(|(&job, minister)| (minister.snapshot.id == player_id).then_some(job))
            .unwrap_or(0)
    }

    fn reject_appoint_minister<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        job: u8,
        mode: u8,
        reason: CountryAppointMinisterRejection,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
        notify_king: bool,
        context: &mut Context,
    ) -> CountryAppointMinisterReport {
        let text = legacy_country_text(context.format_world_string(string_id, arguments));
        context.put_king_log(&text);
        let private_delivery = notify_king
            .then(|| self.send_private_message(&text, 0, context))
            .flatten();
        CountryAppointMinisterReport {
            player_id,
            job,
            mode,
            legacy_result: player_id,
            text,
            disposition: CountryAppointMinisterDisposition::Rejected { reason, private_delivery },
        }
    }

    fn appoint_parameter_unavailable(
        &self,
        player_id: i32,
        job: u8,
        mode: u8,
        block: CountryParameterUnavailable,
        control_point_update: Option<KingPointUpdate>,
    ) -> CountryAppointMinisterReport {
        CountryAppointMinisterReport {
            player_id,
            job,
            mode,
            legacy_result: player_id,
            text: Vec::new(),
            disposition: CountryAppointMinisterDisposition::ParameterUnavailable {
                block,
                appointment_flag_set: true,
                control_point_update,
            },
        }
    }

    /// Exact `DeposeMinister(job, mode)` и достигнутая ветка `AppointMinister(0, job, mode)`.
    pub(crate) fn depose_minister<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        job: u8,
        mode: u8,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryDeposeMinisterReport {
        let Some(minister) = self.ministers.get(&job) else {
            let inserted_null_slot = self.null_minister_slots.insert(job);
            return CountryDeposeMinisterReport {
                job,
                mode,
                legacy_result: 0,
                text: Vec::new(),
                disposition: CountryDeposeMinisterDisposition::SlotMissing { inserted_null_slot },
            };
        };
        if minister.snapshot.id == 0 {
            return CountryDeposeMinisterReport {
                job,
                mode,
                legacy_result: 0,
                text: Vec::new(),
                disposition: CountryDeposeMinisterDisposition::SlotMissing {
                    inserted_null_slot: false,
                },
            };
        }
        let previous_player_id = minister.snapshot.id;
        let previous_name = minister.snapshot.name.clone();
        let country_name = context.country_name(self.country_id);
        let identity_name = context.country_identity_name(job);
        let string_id = if mode == 8 { b"WS0067" } else { b"WS0068" };
        let arguments = if mode == 8 {
            [
                CountryExileTextArgument::Text(&previous_name),
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&identity_name),
            ]
        } else {
            [
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&previous_name),
                CountryExileTextArgument::Text(&identity_name),
            ]
        };
        let text = legacy_country_text(context.format_world_string(string_id, &arguments));
        let country_deliveries = self.send_country_message(&text, context);
        context.put_king_log(&text);
        let minister = self.ministers.get_mut(&job).expect("minister проверен выше");
        minister.snapshot.id = 0;
        minister.snapshot.name.clear();

        let king_map_id = context.game_server_number_by_player_id(self.king.id);
        let mut appointment = CMessage::new(0x0007_FF04);
        appointment.base_mut().add_byte(self.country_id);
        appointment.base_mut().add_long(previous_player_id);
        appointment.base_mut().add_byte(job);
        appointment.base_mut().add_byte(2);
        let appointment_wire = appointment.as_wire_bytes().to_vec();
        let appointment_delivery = context.send_all(&appointment);
        let control_point_delivery = self.send_king_control_point(king_map_id, context);
        let base_info = self.send_base_info_to_client(parameters, context);
        CountryDeposeMinisterReport {
            job,
            mode,
            legacy_result: 0,
            text,
            disposition: CountryDeposeMinisterDisposition::Applied {
                previous_player_id,
                previous_name,
                country_deliveries,
                appointment_wire,
                appointment_delivery,
                control_point_delivery,
                base_info,
            },
        }
    }

    /// Exact `CanOperate(3)` с исходным порядком warring/points/daily-limit.
    pub(crate) fn can_absolve<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryCanAbsolveDisposition {
        let Some(minimum) = parameters.min_king_control_point() else {
            return CountryCanAbsolveDisposition::ParameterUnavailable(
                CountryParameterUnavailable { field: "_min_king_control_point" },
            );
        };
        let rejection = if self.is_warring {
            Some((CountryAbsolveRejection::CountryAtWar, b"WS0044" as &'static [u8], None))
        } else if self.king.control_point < minimum {
            Some((
                CountryAbsolveRejection::InsufficientControlPoint,
                b"WS0045" as &'static [u8],
                Some(minimum),
            ))
        } else {
            let Some(maximum) = parameters.max_absolve_count() else {
                return CountryCanAbsolveDisposition::ParameterUnavailable(
                    CountryParameterUnavailable { field: "_max_absolve_num" },
                );
            };
            if self.absolve_count <= maximum.wrapping_sub(1) {
                None
            } else {
                Some((
                    CountryAbsolveRejection::DailyLimitReached,
                    b"WS0046" as &'static [u8],
                    Some(maximum),
                ))
            }
        };
        let Some((reason, string_id, argument)) = rejection else {
            return CountryCanAbsolveDisposition::Allowed;
        };
        let arguments = argument
            .as_ref()
            .map(|value| [CountryExileTextArgument::Signed(*value)]);
        let text = legacy_country_text(context.format_world_string(
            string_id,
            arguments.as_ref().map_or(&[], |arguments| arguments.as_slice()),
        ));
        let private_delivery = self.send_private_message(&text, 0, context);
        CountryCanAbsolveDisposition::Rejected { reason, text, private_delivery }
    }

    /// Exact `CCountry::Absolve`: сбрасывает crime counters и публикует `0x7FF0C`.
    pub(crate) fn absolve<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryAbsolveReport {
        let Some(player) = context.online_player(player_id) else {
            return self.reject_absolve(
                player_id,
                CountryAbsolveRejection::TargetMissing,
                b"WS0084",
                context,
            );
        };
        let Some(player_country) = player.country else {
            return CountryAbsolveReport {
                player_id,
                legacy_result: 0,
                text: Vec::new(),
                disposition: CountryAbsolveDisposition::Rejected {
                    reason: CountryAbsolveRejection::TargetCountryUnavailable,
                    private_delivery: None,
                },
            };
        };
        if player_country != self.country_id {
            return self.reject_absolve(
                player_id,
                CountryAbsolveRejection::TargetFromAnotherCountry,
                b"WS0085",
                context,
            );
        }
        let Some(counter_reset) = context.reset_online_player_murder_counters(player_id) else {
            return CountryAbsolveReport {
                player_id,
                legacy_result: 0,
                text: Vec::new(),
                disposition: CountryAbsolveDisposition::Rejected {
                    reason: CountryAbsolveRejection::TargetMutationUnavailable,
                    private_delivery: None,
                },
            };
        };
        let Some(control_point_cost) = parameters.absolve_control_point_cost() else {
            return self.absolve_parameter_unavailable(
                player_id,
                counter_reset,
                CountryParameterUnavailable { field: "_dec_king_control_point_absolve" },
                None,
            );
        };
        let requested_control_point = self.king.control_point.wrapping_sub(control_point_cost);
        let control_point_update = match set_control_point(
            &mut self.king,
            requested_control_point,
            parameters,
        ) {
            Ok(update) => update,
            Err(block) => {
                return self.absolve_parameter_unavailable(
                    player_id,
                    counter_reset,
                    block,
                    None,
                );
            }
        };

        let king_map_id = context.game_server_number_by_player_id(self.king.id);
        let mut control_point_message = CMessage::new(0x0007_FF10);
        control_point_message.base_mut().add_long(self.king.id);
        control_point_message.base_mut().add_byte(self.country_id);
        control_point_message.base_mut().add_long(self.king.control_point);
        let control_point_delivery = CountryExileMessageDelivery {
            map_id: king_map_id,
            delivery: context.send_to_map_id(&control_point_message, king_map_id),
        };

        let previous_absolve_count = self.absolve_count;
        self.absolve_count = self.absolve_count.wrapping_add(1);
        let country_name = context.country_name(self.country_id);
        let text = legacy_country_text(context.format_world_string(
            b"WS0086",
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&player.name),
            ],
        ));
        context.put_king_log(&text);
        let mut broadcast = CMessage::new(0x0007_FF0C);
        broadcast.base_mut().add_byte(self.country_id);
        broadcast.base_mut().add_long(player_id);
        let broadcast_wire = broadcast.as_wire_bytes().to_vec();
        let broadcast_delivery = context.send_all(&broadcast);
        let country_deliveries = self.send_country_message(&text, context);
        CountryAbsolveReport {
            player_id,
            legacy_result: player_id,
            text,
            disposition: CountryAbsolveDisposition::Applied {
                counter_reset,
                control_point_update,
                control_point_delivery,
                previous_absolve_count,
                applied_absolve_count: self.absolve_count,
                broadcast_wire,
                broadcast_delivery,
                country_deliveries,
            },
        }
    }

    fn reject_absolve<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        reason: CountryAbsolveRejection,
        string_id: &'static [u8],
        context: &mut Context,
    ) -> CountryAbsolveReport {
        let text = legacy_country_text(context.format_world_string(string_id, &[]));
        context.put_king_log(&text);
        let private_delivery = self.send_private_message(&text, 0, context);
        CountryAbsolveReport {
            player_id,
            legacy_result: 0,
            text,
            disposition: CountryAbsolveDisposition::Rejected { reason, private_delivery },
        }
    }

    fn absolve_parameter_unavailable(
        &self,
        player_id: i32,
        counter_reset: CountryAbsolveCounterReset,
        block: CountryParameterUnavailable,
        control_point_update: Option<KingPointUpdate>,
    ) -> CountryAbsolveReport {
        CountryAbsolveReport {
            player_id,
            legacy_result: 0,
            text: Vec::new(),
            disposition: CountryAbsolveDisposition::ParameterUnavailable {
                block,
                counter_reset,
                control_point_update,
            },
        }
    }

    /// Exact `CanOperate(5)` с исходным порядком warring/points/daily-limit.
    pub(crate) fn can_silence<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryCanSilenceDisposition {
        let Some(minimum) = parameters.min_king_control_point() else {
            return CountryCanSilenceDisposition::ParameterUnavailable(
                CountryParameterUnavailable { field: "_min_king_control_point" },
            );
        };
        let rejection = if self.is_warring {
            Some((CountrySilenceRejection::CountryAtWar, b"WS0050" as &'static [u8], None))
        } else if self.king.control_point < minimum {
            Some((
                CountrySilenceRejection::InsufficientControlPoint,
                b"WS0051" as &'static [u8],
                Some(minimum),
            ))
        } else {
            let Some(maximum) = parameters.max_silence_count() else {
                return CountryCanSilenceDisposition::ParameterUnavailable(
                    CountryParameterUnavailable { field: "_max_silence_num" },
                );
            };
            if self.silence_count <= maximum.wrapping_sub(1) {
                None
            } else {
                Some((
                    CountrySilenceRejection::DailyLimitReached,
                    b"WS0052" as &'static [u8],
                    Some(maximum),
                ))
            }
        };
        let Some((reason, string_id, argument)) = rejection else {
            return CountryCanSilenceDisposition::Allowed;
        };
        let arguments = argument
            .as_ref()
            .map(|value| [CountryExileTextArgument::Signed(*value)]);
        let text = legacy_country_text(context.format_world_string(
            string_id,
            arguments.as_ref().map_or(&[], |arguments| arguments.as_slice()),
        ));
        let private_delivery = self.send_private_message(&text, 0, context);
        CountryCanSilenceDisposition::Rejected { reason, text, private_delivery }
    }

    /// Exact `CCountry::Silence`: mutating success и три исходных wire-effect.
    pub(crate) fn silence<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountrySilenceReport {
        if player_id == self.king.id {
            let country_name = context.country_name(self.country_id);
            return self.reject_silence(
                player_id,
                CountrySilenceRejection::TargetIsKing,
                b"WS0079",
                &[CountryExileTextArgument::Text(&country_name)],
                false,
                context,
            );
        }
        let Some(player) = context.online_player(player_id) else {
            return self.reject_silence(
                player_id,
                CountrySilenceRejection::TargetMissing,
                b"WS0080",
                &[],
                true,
                context,
            );
        };
        let Some(player_country) = player.country else {
            return CountrySilenceReport {
                player_id,
                legacy_result: 0,
                text: Vec::new(),
                disposition: CountrySilenceDisposition::Rejected {
                    reason: CountrySilenceRejection::TargetCountryUnavailable,
                    private_delivery: None,
                },
            };
        };
        if player_country != self.country_id {
            return self.reject_silence(
                player_id,
                CountrySilenceRejection::TargetFromAnotherCountry,
                b"WS0081",
                &[],
                true,
                context,
            );
        }
        if player.is_god {
            return self.reject_silence(
                player_id,
                CountrySilenceRejection::TargetIsGod,
                b"WS0082",
                &[],
                true,
                context,
            );
        }

        let previous_silence_count = self.silence_count;
        self.silence_count = self.silence_count.wrapping_add(1);
        let Some(control_point_cost) = parameters.silence_control_point_cost() else {
            return self.silence_parameter_unavailable(
                player_id,
                previous_silence_count,
                CountryParameterUnavailable { field: "_dec_king_control_point_silence" },
                None,
            );
        };
        let requested_control_point = self.king.control_point.wrapping_sub(control_point_cost);
        let control_point_update = match set_control_point(
            &mut self.king,
            requested_control_point,
            parameters,
        ) {
            Ok(update) => update,
            Err(block) => {
                return self.silence_parameter_unavailable(
                    player_id,
                    previous_silence_count,
                    block,
                    None,
                );
            }
        };
        let Some(silence_time) = parameters.silence_time() else {
            return self.silence_parameter_unavailable(
                player_id,
                previous_silence_count,
                CountryParameterUnavailable { field: "_silence_time" },
                Some(control_point_update),
            );
        };

        let country_name = context.country_name(self.country_id);
        let text = legacy_country_text(context.format_world_string(
            b"WS0083",
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&player.name),
                CountryExileTextArgument::Signed(silence_time),
            ],
        ));
        context.put_king_log(&text);

        let king_map_id = context.game_server_number_by_player_id(self.king.id);
        let mut control_point_message = CMessage::new(0x0007_FF10);
        control_point_message.base_mut().add_long(self.king.id);
        control_point_message.base_mut().add_byte(self.country_id);
        control_point_message.base_mut().add_long(self.king.control_point);
        let control_point_delivery = CountryExileMessageDelivery {
            map_id: king_map_id,
            delivery: context.send_to_map_id(&control_point_message, king_map_id),
        };

        let target_map_id = context.game_server_number_by_player_id(player_id);
        let mut target_message = CMessage::new(0x0007_FF0D);
        target_message.base_mut().add_byte(self.country_id);
        target_message.base_mut().add_long(player_id);
        target_message.base_mut().add_long(self.silence_count);
        let target_wire = target_message.as_wire_bytes().to_vec();
        let target_delivery = CountryExileMessageDelivery {
            map_id: target_map_id,
            delivery: context.send_to_map_id(&target_message, target_map_id),
        };
        let country_deliveries = self.send_country_message(&text, context);
        CountrySilenceReport {
            player_id,
            legacy_result: player_id,
            text,
            disposition: CountrySilenceDisposition::Applied {
                previous_silence_count,
                applied_silence_count: self.silence_count,
                control_point_update,
                control_point_delivery,
                target_delivery,
                target_wire,
                country_deliveries,
            },
        }
    }

    fn reject_silence<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        reason: CountrySilenceRejection,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
        notify_king: bool,
        context: &mut Context,
    ) -> CountrySilenceReport {
        let text = legacy_country_text(context.format_world_string(string_id, arguments));
        context.put_king_log(&text);
        let private_delivery = notify_king
            .then(|| self.send_private_message(&text, 0, context))
            .flatten();
        CountrySilenceReport {
            player_id,
            legacy_result: 0,
            text,
            disposition: CountrySilenceDisposition::Rejected { reason, private_delivery },
        }
    }

    fn silence_parameter_unavailable(
        &self,
        player_id: i32,
        previous_silence_count: i32,
        block: CountryParameterUnavailable,
        control_point_update: Option<KingPointUpdate>,
    ) -> CountrySilenceReport {
        CountrySilenceReport {
            player_id,
            legacy_result: 0,
            text: Vec::new(),
            disposition: CountrySilenceDisposition::ParameterUnavailable {
                block,
                previous_silence_count,
                applied_silence_count: self.silence_count,
                control_point_update,
            },
        }
    }

    /// Exact `CanOperate(4)` с исходным порядком warring/points/daily-limit.
    pub(crate) fn can_exile<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryCanExileDisposition {
        let Some(minimum) = parameters.min_king_control_point() else {
            return CountryCanExileDisposition::ParameterUnavailable(
                CountryParameterUnavailable {
                    field: "_min_king_control_point",
                },
            );
        };
        let rejection = if self.is_warring {
            Some((CountryExileRejection::CountryAtWar, b"WS0047" as &'static [u8], None))
        } else if self.king.control_point < minimum {
            Some((
                CountryExileRejection::InsufficientControlPoint,
                b"WS0048" as &'static [u8],
                Some(minimum),
            ))
        } else {
            let Some(maximum) = parameters.max_exile_count() else {
                return CountryCanExileDisposition::ParameterUnavailable(
                    CountryParameterUnavailable {
                        field: "_max_exile_num",
                    },
                );
            };
            if self.exile_count <= maximum.wrapping_sub(1) {
                None
            } else {
                Some((
                    CountryExileRejection::DailyLimitReached,
                    b"WS0049" as &'static [u8],
                    Some(maximum),
                ))
            }
        };
        let Some((reason, string_id, argument)) = rejection else {
            return CountryCanExileDisposition::Allowed;
        };
        let arguments = argument
            .as_ref()
            .map(|value| [CountryExileTextArgument::Signed(*value)]);
        let text = legacy_country_text(context.format_world_string(
            string_id,
            arguments.as_ref().map_or(&[], |arguments| arguments.as_slice()),
        ));
        let private_delivery = self.send_private_message(&text, 0, context);
        CountryCanExileDisposition::Rejected {
            reason,
            text,
            private_delivery,
        }
    }

    /// Exact `CCountry::Exile`: валидный путь только отправляет `0x7FF0E`.
    pub(crate) fn exile<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryExileRequestDisposition {
        if player_id == self.king.id {
            let country_name = context.country_name(self.country_id);
            return self.reject_exile_request(
                CountryExileRejection::TargetIsKing,
                b"WS0071",
                &[CountryExileTextArgument::Text(&country_name)],
                false,
                context,
            );
        }
        let Some(player) = context.online_player(player_id) else {
            return self.reject_exile_request(
                CountryExileRejection::TargetMissing,
                b"WS0072",
                &[],
                true,
                context,
            );
        };
        let Some(player_country) = player.country else {
            return CountryExileRequestDisposition::Rejected {
                reason: CountryExileRejection::TargetCountryUnavailable,
                text: Vec::new(),
                private_delivery: None,
            };
        };
        if player_country != self.country_id {
            return self.reject_exile_request(
                CountryExileRejection::TargetFromAnotherCountry,
                b"WS0073",
                &[],
                true,
                context,
            );
        }
        if !parameters.has_exile_rect(self.country_id) {
            let country_name = context.country_name(self.country_id);
            return self.reject_exile_request(
                CountryExileRejection::ExileRectMissing,
                b"WS0074",
                &[CountryExileTextArgument::Text(&country_name)],
                false,
                context,
            );
        }
        let Some(maximum_pk) = parameters.max_exile_pk() else {
            return CountryExileRequestDisposition::ParameterUnavailable(
                CountryParameterUnavailable {
                    field: "_max_exile_pk",
                },
            );
        };
        if i32::from(player.pk_count) > maximum_pk {
            return self.reject_exile_request(
                CountryExileRejection::TargetPkTooHigh,
                b"WS0075",
                &[CountryExileTextArgument::Text(&player.name)],
                true,
                context,
            );
        }
        let map_id = context.game_server_number_by_player_id(player_id);
        if map_id == 0 {
            return self.reject_exile_request(
                CountryExileRejection::TargetRouteMissing,
                b"WS0076",
                &[CountryExileTextArgument::Text(&player.name)],
                true,
                context,
            );
        }
        let mut message = CMessage::new(0x0007_FF0E);
        message.base_mut().add_byte(self.country_id);
        message.base_mut().add_long(player_id);
        let wire = message.as_wire_bytes().to_vec();
        CountryExileRequestDisposition::Sent {
            map_id,
            wire,
            delivery: context.send_to_map_id(&message, map_id),
        }
    }

    fn reject_exile_request<Context: CountryExileResultContext + ?Sized>(
        &self,
        reason: CountryExileRejection,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
        notify_king: bool,
        context: &mut Context,
    ) -> CountryExileRequestDisposition {
        let text = legacy_country_text(context.format_world_string(string_id, arguments));
        context.put_king_log(&text);
        let private_delivery = notify_king
            .then(|| self.send_private_message(&text, 0, context))
            .flatten();
        CountryExileRequestDisposition::Rejected {
            reason,
            text,
            private_delivery,
        }
    }

    /// Nonzero-ветка exact `IsKing`; caller отдельно сохраняет исходный log.
    pub(crate) fn has_king_id(&self, player_id: i32) -> bool {
        self.king.id == player_id
    }

    /// Exact `IsMinister` игнорирует входной job при поиске и обходит всех
    /// живых министров; job используется только в сообщении отрицательного log.
    pub(crate) fn has_minister_id(&self, player_id: i32) -> bool {
        self.ministers
            .values()
            .any(|minister| minister.snapshot.id == player_id)
    }

    /// Исполняет exact `CCountry::SuccessExiled` без donor-записи в `ExileMap`.
    pub(crate) fn success_exiled<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        success: bool,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountrySuccessExiledReport {
        let Some(player_name) = context.map_player_name(player_id) else {
            let text = legacy_country_text(context.format_world_string(b"WS0072", &[]));
            context.put_king_log(&text);
            let private_delivery = self.send_private_message(&text, 0, context);
            return CountrySuccessExiledReport {
                player_id,
                success,
                text,
                disposition: CountrySuccessExiledDisposition::PlayerMissing {
                    private_delivery,
                },
            };
        };

        if !success {
            let text = legacy_country_text(context.format_world_string(
                b"WS0078",
                &[CountryExileTextArgument::Text(&player_name)],
            ));
            let private_delivery = self.send_private_message(&text, 0, context);
            context.put_king_log(&text);
            return CountrySuccessExiledReport {
                player_id,
                success,
                text,
                disposition: CountrySuccessExiledDisposition::Failed { private_delivery },
            };
        }

        // EXE вычисляет маршрут короля до чтения country-параметров.
        let king_map_id = context.game_server_number_by_player_id(self.king.id);
        let Some(control_point_cost) = parameters.exile_control_point_cost() else {
            return CountrySuccessExiledReport {
                player_id,
                success,
                text: Vec::new(),
                disposition: CountrySuccessExiledDisposition::ParameterUnavailable {
                    block: CountryParameterUnavailable {
                        field: "_dec_king_control_point_exile",
                    },
                    king_map_id,
                    control_point_update: None,
                    control_point_delivery: None,
                },
            };
        };
        let requested_control_point = self.king.control_point.wrapping_sub(control_point_cost);
        let control_point_update = match set_control_point(
            &mut self.king,
            requested_control_point,
            parameters,
        ) {
            Ok(update) => update,
            Err(block) => {
                return CountrySuccessExiledReport {
                    player_id,
                    success,
                    text: Vec::new(),
                    disposition: CountrySuccessExiledDisposition::ParameterUnavailable {
                        block,
                        king_map_id,
                        control_point_update: None,
                        control_point_delivery: None,
                    },
                };
            }
        };

        let mut control_point_message = CMessage::new(0x0007_FF10);
        control_point_message.base_mut().add_long(self.king.id);
        control_point_message.base_mut().add_byte(self.country_id);
        control_point_message
            .base_mut()
            .add_long(self.king.control_point);
        let control_point_delivery = CountryExileMessageDelivery {
            map_id: king_map_id,
            delivery: context.send_to_map_id(&control_point_message, king_map_id),
        };

        let Some(exile_time_ms) = parameters.exile_time_ms() else {
            return CountrySuccessExiledReport {
                player_id,
                success,
                text: Vec::new(),
                disposition: CountrySuccessExiledDisposition::ParameterUnavailable {
                    block: CountryParameterUnavailable {
                        field: "_exile_time",
                    },
                    king_map_id,
                    control_point_update: Some(control_point_update),
                    control_point_delivery: Some(control_point_delivery),
                },
            };
        };

        // Исходный timeGetTime здесь вызывался, но результат не сохранялся и
        // `ExileMap` не менялся. Чисто технический пустой вызов Rust не имитирует.
        let previous_exile_count = self.exile_count;
        self.exile_count = self.exile_count.wrapping_add(1);
        let country_name = context.country_name(self.country_id);
        let text = legacy_country_text(context.format_world_string(
            b"WS0077",
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&player_name),
                CountryExileTextArgument::Signed(exile_time_ms / 60_000),
            ],
        ));
        let country_deliveries = self.send_country_message(&text, context);
        context.put_king_log(&text);
        CountrySuccessExiledReport {
            player_id,
            success,
            text,
            disposition: CountrySuccessExiledDisposition::Successful {
                control_point_update,
                control_point_delivery,
                previous_exile_count,
                applied_exile_count: self.exile_count,
                country_deliveries,
            },
        }
    }

    fn send_private_message<Context: CountryExileResultContext + ?Sized>(
        &self,
        text: &[u8],
        player_id: i32,
        context: &mut Context,
    ) -> Option<CountryExileMessageDelivery> {
        if text.is_empty() {
            return None;
        }
        let target_player_id = if player_id == 0 {
            self.king.id
        } else {
            player_id
        };
        let map_id = context.game_server_number_by_player_id(target_player_id);
        if map_id == 0 {
            return None;
        }
        let text = CString::new(text).expect("legacy country text не содержит NUL");
        let mut message = CMessage::new(0x0007_FF13);
        message.base_mut().add_long(target_player_id);
        message.base_mut().add_str(Some(&text));
        Some(CountryExileMessageDelivery {
            map_id,
            delivery: context.send_to_map_id(&message, map_id),
        })
    }

    fn send_king_control_point<Context: CountryExileResultContext + ?Sized>(
        &self,
        map_id: i32,
        context: &mut Context,
    ) -> CountryExileMessageDelivery {
        let mut message = CMessage::new(0x0007_FF10);
        message.base_mut().add_long(self.king.id);
        message.base_mut().add_byte(self.country_id);
        message.base_mut().add_long(self.king.control_point);
        CountryExileMessageDelivery {
            map_id,
            delivery: context.send_to_map_id(&message, map_id),
        }
    }

    fn send_base_info_to_client<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryBaseInfoDisposition {
        if self.king.id == 0 {
            return CountryBaseInfoDisposition::KingMissing;
        }
        let parameter = |value: Option<i32>, field: &'static str| {
            value.ok_or(CountryParameterUnavailable { field })
        };
        let demise = match parameter(parameters.demise_control_point_cost(), "_dec_king_control_point_demise") {
            Ok(value) => value,
            Err(block) => return CountryBaseInfoDisposition::ParameterUnavailable(block),
        };
        let appoint = match parameter(parameters.appoint_control_point_cost(), "_dec_king_control_point_appoint") {
            Ok(value) => value,
            Err(block) => return CountryBaseInfoDisposition::ParameterUnavailable(block),
        };
        let exile = match parameter(parameters.exile_control_point_cost(), "_dec_king_control_point_exile") {
            Ok(value) => value,
            Err(block) => return CountryBaseInfoDisposition::ParameterUnavailable(block),
        };
        let silence = match parameter(parameters.silence_control_point_cost(), "_dec_king_control_point_silence") {
            Ok(value) => value,
            Err(block) => return CountryBaseInfoDisposition::ParameterUnavailable(block),
        };
        let absolve = match parameter(parameters.absolve_control_point_cost(), "_dec_king_control_point_absolve") {
            Ok(value) => value,
            Err(block) => return CountryBaseInfoDisposition::ParameterUnavailable(block),
        };
        let mut jobs = self.ministers.keys().copied().collect::<BTreeSet<_>>();
        jobs.extend(self.null_minister_slots.iter().copied());
        let Ok(minister_count) = i32::try_from(jobs.len()) else {
            return CountryBaseInfoDisposition::MinisterCountOutOfRange {
                minister_count: jobs.len(),
            };
        };
        let mut message = CMessage::new(0x0007_FF07);
        message.base_mut().add_long(self.king.id);
        message.base_mut().add_long(self.king.control_point);
        message.base_mut().add_long(self.treasury);
        message.base_mut().add_long(self.power);
        message.base_mut().add_long(self.king.material_point);
        message.base_mut().add_long(self.king.war_point);
        message.base_mut().add_long(self.tech_current_exp);
        message.base_mut().add_long(self.tech_level_up_exp);
        message.base_mut().add_long(self.tech_level);
        message.base_mut().add_byte(u8::from(self.king_quest_switch));
        message.base_mut().add_long(minister_count);
        for job in jobs {
            let minister = self.ministers.get(&job);
            let player_id = minister.map_or(0, |minister| minister.snapshot.id);
            message.base_mut().add_long(player_id);
            message.base_mut().add_byte(job);
            if let Some(minister) = minister.filter(|minister| minister.snapshot.id != 0) {
                let visible_name = minister
                    .snapshot
                    .name
                    .split(|&byte| byte == 0)
                    .next()
                    .unwrap_or_default();
                let name = CString::new(visible_name)
                    .expect("C-string prefix minister-а не содержит embedded NUL");
                message.base_mut().add_str(Some(&name));
                message.base_mut().add_byte(u8::from(minister.quest_switch));
                message.base_mut().add_byte(u8::from(minister.snapshot.appointed));
            }
        }
        for value in [demise, appoint, exile, silence, absolve] {
            message.base_mut().add_long(value);
        }
        let map_id = context.game_server_number_by_player_id(self.king.id);
        let wire = message.as_wire_bytes().to_vec();
        let delivery = context.send_to_map_id(&message, map_id);
        let mut log = context.country_name(self.country_id);
        log.extend_from_slice(b" : Successfully SendBaseInfoToClient!");
        let log = legacy_country_text(log);
        context.put_king_log(&log);
        CountryBaseInfoDisposition::Sent { map_id, wire, delivery, log }
    }

    fn send_country_message<Context: CountryExileResultContext + ?Sized>(
        &self,
        text: &[u8],
        context: &mut Context,
    ) -> Vec<CountryExileMessageDelivery> {
        if text.is_empty() {
            return Vec::new();
        }
        let text = CString::new(text).expect("legacy country text не содержит NUL");
        let mut message = CMessage::new(0x0007_FF11);
        message.base_mut().add_byte(self.country_id);
        message.base_mut().add_str(Some(&text));
        context.send_to_connected_game_servers(&message)
    }

    fn send_world_message<Context: CountryExileResultContext + ?Sized>(
        &self,
        text: &[u8],
        context: &mut Context,
    ) -> Option<(Vec<u8>, Result<i32, SendMessageError>)> {
        if text.is_empty() {
            return None;
        }
        let text = CString::new(text).expect("legacy country text не содержит NUL");
        let mut message = CMessage::new(0x0007_FF12);
        message.base_mut().add_long(-0x1_0000);
        message.base_mut().add_long(-0x100);
        message.base_mut().add_str(Some(&text));
        let wire = message.as_wire_bytes().to_vec();
        let delivery = context.send_all(&message);
        Some((wire, delivery))
    }

    /// Повторяет signed 32-битную арифметику `GetExileResTime` после уже
    /// снятого `timeGetTime`; отсутствие записи не требует `_exile_time`.
    pub(crate) fn exile_remaining_time(
        &self,
        player_id: i32,
        sampled_at_ms: u32,
        parameters: &CCountryParam,
    ) -> Result<CountryExileTimeLookup, CountryParameterUnavailable> {
        let Some(&started_at_ms) = self.exile_started_at_ms.get(&player_id) else {
            return Ok(CountryExileTimeLookup {
                started_at_ms: None,
                sampled_at_ms,
                remaining_ms: 0,
                remaining_seconds: 0,
            });
        };
        let exile_time_ms = parameters
            .exile_time_ms()
            .ok_or(CountryParameterUnavailable {
                field: "_exile_time",
            })?;
        let remaining_ms = exile_time_ms
            .wrapping_sub(sampled_at_ms as i32)
            .wrapping_add(started_at_ms);
        let remaining_seconds = (remaining_ms / 1_000).max(0);
        Ok(CountryExileTimeLookup {
            started_at_ms: Some(started_at_ms),
            sampled_at_ms,
            remaining_ms,
            remaining_seconds,
        })
    }

    /// Повторяет exact выбор `CKing` либо `GetMinister(2..=7)` opcode `0x60315`.
    pub(crate) fn set_quest_switch(
        &mut self,
        job: u8,
        enabled: bool,
    ) -> Option<CountryQuestSwitchUpdate> {
        if job == 1 {
            let previous = self.king_quest_switch;
            self.king_quest_switch = enabled;
            return Some(CountryQuestSwitchUpdate {
                target: CountryQuestSwitchTarget::King,
                previous,
                applied: enabled,
            });
        }
        if !(2..=7).contains(&job) {
            return None;
        }
        let minister = self.ministers.get_mut(&job)?;
        let previous = minister.quest_switch;
        minister.quest_switch = enabled;
        Some(CountryQuestSwitchUpdate {
            target: CountryQuestSwitchTarget::Minister { job },
            previous,
            applied: enabled,
        })
    }

    /// Применяет selector server opcode `0x60314` к достигнутому live-state.
    pub(crate) fn apply_server_scalar(
        &mut self,
        selector: i8,
        requested: i32,
        parameters: &CCountryParam,
    ) -> Result<Option<CountryScalarUpdate>, CountryParameterUnavailable> {
        let update = match selector {
            1 => self.set_country_treasury(requested, parameters)?,
            2 => self.set_country_power(requested, parameters)?,
            3 => self.set_country_technology(requested),
            4 => {
                let previous = self.tech_level;
                let applied = requested.max(0);
                self.tech_level = applied;
                CountryScalarUpdate::TechnologyLevel {
                    requested,
                    previous,
                    applied,
                }
            }
            5 => CountryScalarUpdate::KingPoint(set_control_point(
                &mut self.king,
                requested,
                parameters,
            )?),
            6 => CountryScalarUpdate::KingPoint(set_material_point(
                &mut self.king,
                requested,
                parameters,
            )?),
            7 => CountryScalarUpdate::KingPoint(set_war_point(
                &mut self.king,
                requested,
                parameters,
            )?),
            _ => return Ok(None),
        };
        Ok(Some(update))
    }

    pub(crate) fn set_country_power(
        &mut self,
        requested: i32,
        parameters: &CCountryParam,
    ) -> Result<CountryScalarUpdate, CountryParameterUnavailable> {
        let maximum = parameters
            .max_country_power()
            .ok_or(CountryParameterUnavailable {
                field: "_max_country_power",
            })?;
        let previous = self.power;
        let applied = requested.max(0).min(maximum);
        self.power = applied;
        Ok(CountryScalarUpdate::Power {
            requested,
            previous,
            applied,
        })
    }

    pub(crate) fn set_country_treasury(
        &mut self,
        requested: i32,
        parameters: &CCountryParam,
    ) -> Result<CountryScalarUpdate, CountryParameterUnavailable> {
        let maximum = parameters
            .max_country_treasury()
            .ok_or(CountryParameterUnavailable {
                field: "_max_country_treasury",
            })?;
        let previous = self.treasury;
        let applied = requested.max(0).min(maximum);
        self.treasury = applied;
        Ok(CountryScalarUpdate::Treasury {
            requested,
            previous,
            applied,
        })
    }

    pub(crate) fn set_country_technology(&mut self, requested: i32) -> CountryScalarUpdate {
        let previous = self.tech_current_exp;
        let applied = requested.min(self.tech_level_up_exp);
        self.tech_current_exp = applied;
        CountryScalarUpdate::TechnologyExperience {
            requested,
            previous,
            applied,
        }
    }

    /// Дописывает один точный country record для `CCountryHandler` wire.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), CountrySerializeError> {
        let mut minister_jobs = self.ministers.keys().copied().collect::<BTreeSet<_>>();
        minister_jobs.extend(self.null_minister_slots.iter().copied());
        let minister_count = minister_jobs.len();
        let minister_count_i32 = i32::try_from(minister_count)
            .map_err(|_| CountrySerializeError::MinisterCountOutOfRange { minister_count })?;

        destination.push(self.country_id);
        destination.extend_from_slice(&self.treasury.to_le_bytes());
        destination.extend_from_slice(&self.power.to_le_bytes());
        destination.extend_from_slice(&self.tech_current_exp.to_le_bytes());
        destination.extend_from_slice(&self.tech_level.to_le_bytes());
        destination.extend_from_slice(&self.king.control_point.to_le_bytes());
        destination.extend_from_slice(&self.king.material_point.to_le_bytes());
        destination.extend_from_slice(&self.king.war_point.to_le_bytes());
        destination.extend_from_slice(&self.king.id.to_le_bytes());
        destination.extend_from_slice(&self.country_war_result.to_le_bytes());
        destination.extend_from_slice(&minister_count_i32.to_le_bytes());
        for job in minister_jobs {
            destination.push(job);
            let player_id = self
                .ministers
                .get(&job)
                .map_or(0, |minister| minister.snapshot.id);
            destination.extend_from_slice(&player_id.to_le_bytes());
        }
        Ok(())
    }

    /// Создаёт отдельную DB-наблюдаемую копию country state.
    pub(crate) fn clone_save_data(&self, limits: CountryKingSaveLimits) -> CountrySaveSnapshot {
        let mut cloned_ministers = BTreeMap::new();
        for minister in self.ministers.values().take(6) {
            cloned_ministers
                .entry(minister.id_type)
                .or_insert_with(|| minister.snapshot.clone());
        }

        let ministers = std::array::from_fn(|index| {
            let id_type = index as u8 + 2;
            cloned_ministers.remove(&id_type)
        });

        // CloneCountryData копировал это поле, хотя единственный следующий
        // consumer CDBCountry::Save его не читал.
        let _tech_level_up_exp = self.tech_level_up_exp;

        CountrySaveSnapshot {
            country_id: self.country_id,
            treasury: self.treasury,
            power: self.power,
            tech_current_exp: self.tech_current_exp,
            tech_level: self.tech_level,
            king: CountryKingSaveSnapshot {
                id: self.king.id,
                name: self.king.name.clone(),
                appointed: self.king.appointed,
                salary_received: self.king.salary_received,
                control_point: self.king.control_point.min(limits.control_point),
                material_point: self.king.material_point.min(limits.material_point),
                war_point: self.king.war_point.min(limits.war_point),
            },
            country_war_result: self.country_war_result,
            ministers,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountrySerializeError {
    MinisterCountOutOfRange { minister_count: usize },
}

impl fmt::Display for CountrySerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MinisterCountOutOfRange { minister_count } => write!(
                formatter,
                "CCountry содержит {minister_count} министров вне signed 32-битного диапазона"
            ),
        }
    }
}

impl Error for CountrySerializeError {}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    value
        .split(|byte| *byte == 0)
        .next()
        .unwrap_or_default()
}

fn legacy_country_text(mut text: Vec<u8>) -> Vec<u8> {
    if let Some(terminator) = text.iter().position(|byte| *byte == 0) {
        text.truncate(terminator);
    }
    // Старый `_sprintf` писал в `char[260]`; переполнение и последующий
    // overread были внутренним UB, а не Miracle wire-контрактом.
    text.truncate(259);
    text
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp

// ============================================================================
// FUNCTION: Catch@00444c1a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp
// RVA: 0x00044C1A
// ADDRESS: 00444c1a
// PROTOTYPE: undefined Catch@00444c1a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetCountryPower
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.h:57
// RVA: 0x000A4750
// ADDRESS: 004a4750
// PROTOTYPE: long __thiscall SetCountryPower(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetCountryTreasury
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.h:82
// RVA: 0x000A4790
// ADDRESS: 004a4790
// PROTOTYPE: long __thiscall SetCountryTreasury(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetCountryTech
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.h:118
// RVA: 0x000A47D0
// ADDRESS: 004a47d0
// PROTOTYPE: long __thiscall SetCountryTech(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::ChangeKingControlPoint
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1552
// RVA: 0x000C6740
// ADDRESS: 004c6740
// PROTOTYPE: long __thiscall ChangeKingControlPoint(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SendWorldMsg
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1670
// RVA: 0x000C67D0
// ADDRESS: 004c67d0
// PROTOTYPE: void __thiscall SendWorldMsg(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SendPrivateMsg
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1712
// RVA: 0x000C6870
// ADDRESS: 004c6870
// PROTOTYPE: void __thiscall SendPrivateMsg(char * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SendBaseInfoToClient
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:52
// RVA: 0x000C6A80
// ADDRESS: 004c6a80
// PROTOTYPE: bool __thiscall SendBaseInfoToClient(void)
//
// Реализация полного `0x7FF07` wire и king-log находится выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::GetInfo
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:332
// RVA: 0x000C6D30
// ADDRESS: 004c6d30
// PROTOTYPE: long __thiscall GetInfo(void)
//
// Достигнутый wrapper сведён к `send_base_info_to_client` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::GetExileResTime
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1319
// RVA: 0x000C6D70
// ADDRESS: 004c6d70
// PROTOTYPE: long __thiscall GetExileResTime(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::GetMinister
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1466
// RVA: 0x000C6DE0
// ADDRESS: 004c6de0
// PROTOTYPE: CMinister * __thiscall GetMinister(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: CCountry::AddToByteArray, WorldServer RVA 0x000C6E30.
// Реализация находится выше; STL traversal свёрнут в provenance.

// ============================================================================
// FUNCTION: CCountry::NewTerm
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1764
// RVA: 0x000C6F40
// ADDRESS: 004c6f40
// PROTOTYPE: void __thiscall NewTerm(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SendCountryMsg
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1686
// RVA: 0x000C7090
// ADDRESS: 004c7090
// PROTOTYPE: void __thiscall SendCountryMsg(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::IsKing
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:566
// RVA: 0x000C7160
// ADDRESS: 004c7160
// PROTOTYPE: bool __thiscall IsKing(long param_1)
//
// Реализовано выше как `authorize_king`; тело сохранено как source-reference.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::IsMinister
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:598
// RVA: 0x000C7320
// ADDRESS: 004c7320
// PROTOTYPE: bool __thiscall IsMinister(long param_1, uchar param_2)
//
// Полная identity-проверка и WS0036/WS0037 log-ветки реализованы выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::CanOperate
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:638
// RVA: 0x000C7520
// ADDRESS: 004c7520
// PROTOTYPE: bool __thiscall CanOperate(uchar param_1)
//
// Ветки operation `1/2/3/4/5` реализованы выше как `can_appoint_minister/`
// `can_depose_minister/can_absolve/can_exile/can_silence`; остальные — reference.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::Exile
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1156
// RVA: 0x000C7AD0
// ADDRESS: 004c7ad0
// PROTOTYPE: long __thiscall Exile(long param_1)
//
// Реализовано выше; тело сохранено как машинная source-reference.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SuccessExiled
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1254
// RVA: 0x000C7EE0
// ADDRESS: 004c7ee0
// PROTOTYPE: bool __thiscall SuccessExiled(long param_1, bool param_2)
//
// Реализовано выше; тело ниже сохранено как машинная source-reference.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::Silence
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1342
// RVA: 0x000C81D0
// ADDRESS: 004c81d0
// PROTOTYPE: long __thiscall Silence(long param_1)
//
// Реализация выше сохраняет exact проверки, side-effect order, wire и возврат;
// raw оставлен только как локальная документация исходного owner-а.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::Absolve
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1411
// RVA: 0x000C8570
// ADDRESS: 004c8570
// PROTOTYPE: long __thiscall Absolve(long param_1)
//
// Реализация выше сохраняет exact player mutations, side-effect order и wire;
// raw оставлен только как локальная документация исходного owner-а.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::HasJob
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1790
// RVA: 0x000C8820
// ADDRESS: 004c8820
// PROTOTYPE: uchar __thiscall HasJob(long param_1)
//
// Реализация с исходным вложенным `IsKing` log-side-effect находится выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::AddVilTax2Treasure
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1810
// RVA: 0x000C8880
// ADDRESS: 004c8880
// PROTOTYPE: void __thiscall AddVilTax2Treasure(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::AppointMinister
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:956
// RVA: 0x000C8FA0
// ADDRESS: 004c8fa0
// PROTOTYPE: long __thiscall AppointMinister(long param_1, uchar param_2, uchar param_3)
//
// Positive-player назначение и player=0 снятие реализованы выше; mode влияет
// только на WS0067/WS0068 в исходной ветке снятия.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: CCountry::CloneCountryData, WorldServer RVA 0x000C9CE0.
// Реализация достигнутой save-проекции находится выше.

// ============================================================================
// FUNCTION: CCountry::SetMinisterfromDB
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1648
// RVA: 0x000C9EB0
// ADDRESS: 004c9eb0
// PROTOTYPE: void __thiscall SetMinisterfromDB(uchar param_1, CMinister * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::DeposeMinister
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:943
// RVA: 0x000C9F90
// ADDRESS: 004c9f90
// PROTOTYPE: long __thiscall DeposeMinister(uchar param_1, uchar param_2)
//
// Exact operator[] quirk и вызов reached AppointMinister реализованы выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetNewDay
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1733
// RVA: 0x000C9FD0
// ADDRESS: 004c9fd0
// PROTOTYPE: void __thiscall SetNewDay(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::~CCountry
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:35
// RVA: 0x000CA0C0
// ADDRESS: 004ca0c0
// PROTOTYPE: void __thiscall ~CCountry(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::InitialOLPlayersList
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:114
// RVA: 0x000CA360
// ADDRESS: 004ca360
// PROTOTYPE: bool __thiscall InitialOLPlayersList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::Sort
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:174
// RVA: 0x000CA6E0
// ADDRESS: 004ca6e0
// PROTOTYPE: bool __thiscall Sort(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::CanAscend
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:204
// RVA: 0x000CA830
// ADDRESS: 004ca830
// PROTOTYPE: bool __thiscall CanAscend(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::CanDemise
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:294
// RVA: 0x000CAC30
// ADDRESS: 004cac30
// PROTOTYPE: bool __thiscall CanDemise(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::GetPlayersList
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:345
// RVA: 0x000CAD90
// ADDRESS: 004cad90
// PROTOTYPE: long __thiscall GetPlayersList(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::DeposeKing
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:811
// RVA: 0x000CB030
// ADDRESS: 004cb030
// PROTOTYPE: long __thiscall DeposeKing(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1526
// RVA: 0x000CB710
// ADDRESS: 004cb710
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::CCountry
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:28
// RVA: 0x000CB760
// ADDRESS: 004cb760
// PROTOTYPE: undefined __thiscall CCountry(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::RegisterKing
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:395
// RVA: 0x000CB8F0
// ADDRESS: 004cb8f0
// PROTOTYPE: long __thiscall RegisterKing(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetKing
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:796
// RVA: 0x000CC290
// ADDRESS: 004cc290
// PROTOTYPE: long __thiscall SetKing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::Demise
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:916
// RVA: 0x000CC320
// ADDRESS: 004cc320
// PROTOTYPE: long __thiscall Demise(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: CCountry::CloneSaveData, WorldServer RVA 0x000CC470.
// Отдельный heap-owner заменён готовым `CountrySaveSnapshot` выше.


// ============================================================================
// FUNCTION: Unwind@0052e2b0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp
// RVA: 0x0012E2B0
// ADDRESS: 0052e2b0
// PROTOTYPE: undefined Unwind@0052e2b0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// COMPONENT_VARIANT_END: WorldServer
