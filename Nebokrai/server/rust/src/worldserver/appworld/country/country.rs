//! Save-владелец `CCountry` исторического `WorldServer`.
//!
//! Статус `CCountry::SetCountryPower/SetCountryTreasury/SetCountryTech` RVA
//! `0x000A4750/0x000A4790/0x000A47D0`, `CCountry::AddToByteArray` RVA `0x000C6E30`,
//! `CCountry::IsKing/CanOperate/Exile/SuccessExiled/Silence/Absolve` RVA
//! `0x000C7160/0x000C7520/0x000C7AD0/0x000C7EE0/0x000C81D0/0x000C8570`,
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

use std::collections::BTreeMap;
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
    pub(crate) is_warring: bool,
    pub(crate) silence_count: i32,
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
    pub(crate) pk_count: u16,
    pub(crate) is_god: bool,
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
    fn send_to_connected_game_servers(
        &mut self,
        message: &CMessage,
    ) -> Vec<CountryExileMessageDelivery>;
    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError>;
    fn put_king_log(&mut self, text: &[u8]);
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

impl CCountry {
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
        let minister_count = self.ministers.len();
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
        for (&job, minister) in &self.ministers {
            destination.push(job);
            destination.extend_from_slice(&minister.snapshot.id.to_le_bytes());
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:52
// RVA: 0x000C6A80
// ADDRESS: 004c6a80
// PROTOTYPE: bool __thiscall SendBaseInfoToClient(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::GetInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:332
// RVA: 0x000C6D30
// ADDRESS: 004c6d30
// PROTOTYPE: long __thiscall GetInfo(void)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:598
// RVA: 0x000C7320
// ADDRESS: 004c7320
// PROTOTYPE: bool __thiscall IsMinister(long param_1, uchar param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::CanOperate
// STATUS: IMPLEMENTED_PARTIAL_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:638
// RVA: 0x000C7520
// ADDRESS: 004c7520
// PROTOTYPE: bool __thiscall CanOperate(uchar param_1)
//
// Ветки operation `3/4/5` реализованы выше как `can_absolve/can_exile/`
// `can_silence`; остальные selectors остаются source-reference.
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1790
// RVA: 0x000C8820
// ADDRESS: 004c8820
// PROTOTYPE: uchar __thiscall HasJob(long param_1)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:956
// RVA: 0x000C8FA0
// ADDRESS: 004c8fa0
// PROTOTYPE: long __thiscall AppointMinister(long param_1, uchar param_2, uchar param_3)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:943
// RVA: 0x000C9F90
// ADDRESS: 004c9f90
// PROTOTYPE: long __thiscall DeposeMinister(uchar param_1, uchar param_2)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
