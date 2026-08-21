//! WorldServer dispatcher-owner country messages `OnCountryMessage`.
//!
//! Dispatcher RVA `0x000A47F0` остаётся `IMPLEMENTED_PARTIAL`: country relays
//! `0x60310 -> 0x7FF11` и `0x60311 -> 0x7FF12`, а также вход country victory
//! `0x60318`, scalar-sync `0x60314`, quest-switch `0x60315`, depose-minister
//! `0x6030A -> 0x7FF04/0x7FF10/0x7FF07`, absolve
//! `0x6030B -> 0x7FF10/0x7FF0C/0x7FF11`, silence
//! `0x6030C -> 0x7FF10/0x7FF0D/0x7FF11`, exile
//! `0x6030D -> 0x7FF0E`, `0x6030E -> 0x7FF15`, `0x60316 -> 0x7FF15`, war-declare
//! `0x60317 -> 0x7FF16` и four-nation result
//! `0x60319 -> 0x7FE49`, `0x6031A -> 0x7FE46/DB`, no-op `0x6031B` и
//! `0x6031C -> 0x7FE47`, `0x6031D -> 0x7FA04` имеют статус
//! `IMPLEMENTED`. Victory читает один
//! unsigned country byte и вызывает исходно
//! названный `CountryWarSys::on_flag_destory`; соседние opcodes helper не
//! интерпретирует. Точная пара
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, исходник
//! `appworld/message/countrymessage.cpp`. Exact `0x004A4FA3..0x004A4FC4`
//! подтверждает, что оба relay меняют type исходного сообщения и вызывают
//! общий `SendAll`, не читая payload, не вызывая `Update` и не добавляя
//! ownership/tail gates. Exact `0x004A48C6..0x004A496E` задаёт для `0x60314`
//! wire `unsigned char country, signed char selector, signed long value` и
//! тихие no-op на отсутствующей стране или неизвестном selector. Scalar-setter
//! сохраняет несимметричные исходные ограничения: treasury/power ограничены
//! снизу нулём и сверху максимумом, tech-exp только сверху, tech-level только
//! снизу, king points только сверху. Вместо singleton `CCountryParam` Rust
//! принимает уже принадлежащий main-loop параметр явно.
//! Exact `0x004A4EF1..0x004A4F9E` и PDB layout `COfficer` подтверждают для
//! `0x60315` три unsigned byte `country/job/raw switch`, выбор встроенного king
//! при job `1`, `GetMinister` только для `2..=7` и запись именно
//! `_bQuestSwitch +0x25`. Строка `king`-лога сохраняет исходный raw switch и
//! byte-exact хвост `A1 A3`; безопасный accessor `CGlobeSetup` заменяет только
//! старое адресное вычисление country-name slot.
//! Exact `0x004A4E43..0x004A4ED9` задаёт для `0x60316` signed player ID и
//! country byte, вызов `GetExileResTime` до проверки online-player и общий
//! ответ `0x7FF15 { remaining_seconds:i32, player_id:i32 }`. Exact
//! `GetExileResTime` снимает tick до поиска `ExileMap`, использует wrapping
//! signed 32-bit milliseconds, делит к нулю и зажимает отрицательный результат.
//! `SuccessExiled` в точном EXE не наполняет map, хотя Linux-донор это исправил;
//! dispatcher сохраняет машинную ошибку, а не принимает donor fix за контракт.
//! Exact `0x004A4C57..0x004A4E3E` задаёт `0x6030E`: signed player ID, два
//! signed `char` success/country, ранний stop до чтения списка при отсутствующей
//! стране, затем синхронный `SuccessExiled` и `0x7FF15 { country:u8,
//! count:i32, player_ids:i32[] }`. Signed count `<= 0` не читает элементы, но
//! всё равно публикуется; source metadata и хвост не проверяются. Для
//! положительного count Rust требует фактически присутствующие DWORD: старый
//! цикл дополнял оборванный inter-server payload нулями до заявленного размера
//! и мог выделять до `INT_MAX` элементов, что является внутренним malformed-
//! input дефектом, а не Miracle-протоколом.
//! Exact `0x6030D` читает target/king как signed long и country через signed
//! `char -> unsigned char`, затем строго вызывает `GetCountry -> IsKing ->
//! CanOperate(4) -> Exile`. Missing country и любой false gate останавливают
//! цепочку; source map/socket и хвост не участвуют. Donor ownership/socket
//! gates и pending-request registry поэтому не перенесены.
//! `0x6030C` имеет тот же wire target/king/country, но selector `5` и вызов
//! `Silence`; exact owner возвращает target ID только после всех трёх success-
//! рассылок. Source metadata и хвост также не проверяются.
//! `0x6030B` декодирует тот же target/king/country wire, вызывает selector `3`
//! и `Absolve`; donor payload/ownership gates в exact dispatcher отсутствуют.
//! `0x6030A` дополнительно читает signed job-byte между target и king, затем
//! вызывает selector `2`, exact `IsMinister` и mode `7`; source/tail/job-range
//! gates старого Linux-донора не переносятся.
//! Exact `0x004A4FC9..0x004A5049` задаёт `0x60317`: два signed long,
//! синхронный `player_declare`, затем ответ `char accepted, player, target` в
//! исходный `m_lMapID`. Проверок socket-owner и полного tail здесь нет; они
//! были добавлены Linux-донором и не являются поведением поставленного EXE.
//! Exact `0x004A506A..0x004A5078` для `0x60319` только получает singleton и
//! передаёт исходное сообщение static `RecvResultFromGS`: source metadata и
//! хвост не проверяются, отдельного ответа источнику нет.
//! Exact `0x004A5096..0x004A50B4` для `0x6031A` читает два signed long и
//! вызывает `ConvertMoraleToExploit(player_id, increment)` без source/tail
//! gate. Последующий DB/online-маршрут материализован в общем async
//! `ProcessMessage`, поскольку `tiberius` требует await.
//! Exact `0x004A507D..0x004A5094` для `0x6031B` читает один signed country и
//! вызывает static `OneCountrySignUp`. Сам owner `0x00494340..0x00494431`
//! только для `1..=4` форматирует неиспользуемый локальный текст; состояния,
//! log-а и network side effect нет, поэтому Rust сохраняет typed no-op без
//! мёртвого `_snprintf`.
//! Exact `0x004A50B6..0x004A50DE` для `0x6031C` читает signed player ID,
//! 32-bit war-time и signed country; route/wire выполняет подтверждённый
//! `SendPlayerWarTimeToGS` без source/tail gate.
//! Exact `0x004A50E0..0x004A50FE` для `0x6031D` читает две signed страны и
//! вызывает `OneCountryFail`; source metadata и хвост не проверяются.
//! Exact switch target `0x004A504E..0x004A5065` подтверждает, что `0x60318`
//! читает один unsigned country byte и сразу передаёт его достигнутому
//! `CountryWarSys`; конкретный region/country/localization/network context
//! подключён в общем `ProcessMessage`, а не оставлен отдельным helper-ом.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::public::tools::put_string_to_file;
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::worldserver::appworld::country::country::{
    CountryAbsolveReport, CountryCanAbsolveDisposition, CountryCanExileDisposition,
    CountryCanDeposeMinisterDisposition, CountryCanSilenceDisposition,
    CountryDeposeMinisterReport, CountryExileRequestDisposition,
    CountryExileResultContext, CountryExileTimeLookup, CountryQuestSwitchUpdate,
    CountryScalarUpdate, CountrySilenceReport, CountrySuccessExiledReport,
};
use crate::worldserver::appworld::country::countryhandler::CCountryHandler;
use crate::worldserver::appworld::country::countryparam::{
    CCountryParam, CountryParameterUnavailable,
};
use crate::worldserver::appworld::organizingsystem::fournationwarsys::{
    CFourNationWarSys, FourNationCountryFailContext, FourNationCountryFailReport,
    FourNationExploitLoadedReport, FourNationSignUpDisposition, FourNationWarResultContext,
    FourNationWarResultReport, FourNationWarTimeReport,
};
use crate::worldserver::worldserver::game::{CGame, legacy_tick_ms};
use crate::worldserver::worldserver::worldserver::AddLogTextDisposition;

use super::super::country::countrywarsys::{
    CountryWarDeclarationContext, CountryWarDeclarationReport, CountryWarSys,
    CountryWarVictoryContext, CountryWarVictoryReport,
};

const COUNTRY_RELAY_FIRST: i32 = 0x0006_0310;
const COUNTRY_RELAY_SECOND: i32 = 0x0006_0311;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryRelayOutcome {
    pub(crate) request_type: i32,
    pub(crate) response_type: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryScalarDisposition {
    CountryMissing,
    SelectorIgnored,
    ParameterUnavailable(CountryParameterUnavailable),
    Updated(CountryScalarUpdate),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryScalarSync {
    pub(crate) country_id: u8,
    pub(crate) country_id_complete: bool,
    pub(crate) selector: i8,
    pub(crate) selector_complete: bool,
    pub(crate) value: i32,
    pub(crate) value_complete: bool,
    pub(crate) disposition: WorldCountryScalarDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryQuestSwitchDisposition {
    CountryMissing,
    OfficerMissing,
    Updated(CountryQuestSwitchUpdate),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryQuestSwitchLog {
    pub(crate) country_name_complete: bool,
    pub(crate) line: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryQuestSwitchSync {
    pub(crate) country_id: u8,
    pub(crate) country_id_complete: bool,
    pub(crate) job: u8,
    pub(crate) job_complete: bool,
    pub(crate) raw_switch: u8,
    pub(crate) raw_switch_complete: bool,
    pub(crate) disposition: WorldCountryQuestSwitchDisposition,
    pub(crate) log: Option<WorldCountryQuestSwitchLog>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryExileTimeDisposition {
    CountryMissing,
    ParameterUnavailable(CountryParameterUnavailable),
    PlayerMissing {
        lookup: CountryExileTimeLookup,
    },
    Broadcast {
        lookup: CountryExileTimeLookup,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryExileTimeSync {
    pub(crate) player_id: i32,
    pub(crate) player_id_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_id_complete: bool,
    pub(crate) disposition: WorldCountryExileTimeDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryExileResultDisposition {
    CountryMissing,
    PlayerListTruncated {
        advertised_count: i32,
        available_complete_ids: usize,
    },
    PlayerListAllocationBlocked {
        advertised_count: i32,
    },
    Broadcast {
        advertised_count: i32,
        player_ids: Vec<i32>,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryExileResultSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) player_id: i32,
    pub(crate) player_id_complete: bool,
    pub(crate) raw_success: i8,
    pub(crate) success_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_id_complete: bool,
    pub(crate) country_report: Option<CountrySuccessExiledReport>,
    pub(crate) count_complete: Option<bool>,
    pub(crate) disposition: WorldCountryExileResultDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryExileRequestDisposition {
    CountryMissing,
    KingRejected,
    OperationRejected(CountryCanExileDisposition),
    Requested {
        operation: CountryCanExileDisposition,
        legacy_result: i32,
        request: CountryExileRequestDisposition,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryExileRequestSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) target_player_id: i32,
    pub(crate) target_complete: bool,
    pub(crate) king_player_id: i32,
    pub(crate) king_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_complete: bool,
    pub(crate) disposition: WorldCountryExileRequestDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountrySilenceRequestDisposition {
    CountryMissing,
    KingRejected,
    OperationRejected(CountryCanSilenceDisposition),
    Applied {
        operation: CountryCanSilenceDisposition,
        report: CountrySilenceReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountrySilenceRequestSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) target_player_id: i32,
    pub(crate) target_complete: bool,
    pub(crate) king_player_id: i32,
    pub(crate) king_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_complete: bool,
    pub(crate) disposition: WorldCountrySilenceRequestDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryAbsolveRequestDisposition {
    CountryMissing,
    KingRejected,
    OperationRejected(CountryCanAbsolveDisposition),
    Applied {
        operation: CountryCanAbsolveDisposition,
        report: CountryAbsolveReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryAbsolveRequestSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) target_player_id: i32,
    pub(crate) target_complete: bool,
    pub(crate) king_player_id: i32,
    pub(crate) king_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_complete: bool,
    pub(crate) disposition: WorldCountryAbsolveRequestDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryDeposeMinisterDisposition {
    CountryMissing,
    KingRejected,
    OperationRejected(CountryCanDeposeMinisterDisposition),
    MinisterRejected,
    Applied {
        operation: CountryCanDeposeMinisterDisposition,
        report: CountryDeposeMinisterReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryDeposeMinisterSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) target_player_id: i32,
    pub(crate) target_complete: bool,
    pub(crate) job: u8,
    pub(crate) job_complete: bool,
    pub(crate) king_player_id: i32,
    pub(crate) king_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_complete: bool,
    pub(crate) disposition: WorldCountryDeposeMinisterDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryWarDeclarationSync {
    pub(crate) player_id: i32,
    pub(crate) player_id_complete: bool,
    pub(crate) target_country: i32,
    pub(crate) target_country_complete: bool,
    pub(crate) source_map_id: i32,
    pub(crate) declaration: CountryWarDeclarationReport,
    pub(crate) response_wire: Vec<u8>,
    pub(crate) response_delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldFourNationWarResultSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) report: FourNationWarResultReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldFourNationExploitRequest {
    pub(crate) player_id: i32,
    pub(crate) player_id_complete: bool,
    pub(crate) increment: i32,
    pub(crate) increment_complete: bool,
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldFourNationExploitDatabaseDisposition {
    NotRequired,
    ConnectionUnavailable { log: AddLogTextDisposition },
    Applied,
    ExecutionFailed { error: String },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldFourNationExploitSync {
    pub(crate) request: WorldFourNationExploitRequest,
    pub(crate) initial: FourNationExploitLoadedReport,
    pub(crate) database: WorldFourNationExploitDatabaseDisposition,
    pub(crate) after_database: Option<FourNationExploitLoadedReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldFourNationSignUpSync {
    pub(crate) country: i32,
    pub(crate) country_complete: bool,
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) disposition: FourNationSignUpDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldFourNationWarTimeSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) numeric_payload_complete: [bool; 3],
    pub(crate) report: FourNationWarTimeReport,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldFourNationCountryFailSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) numeric_payload_complete: [bool; 2],
    pub(crate) report: FourNationCountryFailReport,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryMessageOutcome {
    Relay(WorldCountryRelayOutcome),
    ScalarSynchronized(WorldCountryScalarSync),
    QuestSwitchSynchronized(WorldCountryQuestSwitchSync),
    ExileTimeSynchronized(WorldCountryExileTimeSync),
    ExileRequested(WorldCountryExileRequestSync),
    ExileResultSynchronized(WorldCountryExileResultSync),
    SilenceRequested(WorldCountrySilenceRequestSync),
    AbsolveRequested(WorldCountryAbsolveRequestSync),
    MinisterDeposed(WorldCountryDeposeMinisterSync),
    CountryWarDeclared(WorldCountryWarDeclarationSync),
    CountryWarVictory(WorldCountryWarVictorySync),
    FourNationWarResult(WorldFourNationWarResultSync),
    FourNationExploit(WorldFourNationExploitSync),
    FourNationSignUp(WorldFourNationSignUpSync),
    FourNationWarTime(WorldFourNationWarTimeSync),
    FourNationCountryFail(WorldFourNationCountryFailSync),
}

pub(crate) enum WorldCountryMessageDispatch {
    Handled(WorldCountryMessageOutcome),
    Pending(CMessage),
}

/// Исполняет достигнутые scalar-sync и in-place relay ветви `OnCountryMessage`.
pub(crate) fn on_country_message(
    game: &CGame,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    globe_setup: &GlobeSetupSnapshot,
    mut message: CMessage,
) -> WorldCountryMessageDispatch {
    let request_type = message.message_type();
    if request_type == 0x0006_031b {
        let source_map_id = message.map_id();
        let source_socket_id = message.socket_id();
        let decoded_country = message.base_mut().get_long();
        let country = decoded_country.unwrap_or(0);
        return WorldCountryMessageDispatch::Handled(
            WorldCountryMessageOutcome::FourNationSignUp(WorldFourNationSignUpSync {
                country,
                country_complete: decoded_country.is_some(),
                source_map_id,
                source_socket_id,
                disposition: CFourNationWarSys::one_country_sign_up(country),
            }),
        );
    }
    if request_type == 0x0006_0314 {
        let decoded_country_id = message.base_mut().get_byte();
        let country_id = decoded_country_id.unwrap_or(0);
        let decoded_selector = message.base_mut().get_char();
        let selector = decoded_selector.unwrap_or(0);
        let decoded_value = message.base_mut().get_long();
        let value = decoded_value.unwrap_or(0);
        let disposition = match country_handler.get_country_mut(country_id) {
            None => WorldCountryScalarDisposition::CountryMissing,
            Some(country) => match country.apply_server_scalar(selector, value, country_parameters)
            {
                Ok(Some(update)) => WorldCountryScalarDisposition::Updated(update),
                Ok(None) => WorldCountryScalarDisposition::SelectorIgnored,
                Err(block) => WorldCountryScalarDisposition::ParameterUnavailable(block),
            },
        };
        return WorldCountryMessageDispatch::Handled(
            WorldCountryMessageOutcome::ScalarSynchronized(WorldCountryScalarSync {
                country_id,
                country_id_complete: decoded_country_id.is_some(),
                selector,
                selector_complete: decoded_selector.is_some(),
                value,
                value_complete: decoded_value.is_some(),
                disposition,
            }),
        );
    }
    if request_type == 0x0006_0315 {
        let decoded_country_id = message.base_mut().get_byte();
        let country_id = decoded_country_id.unwrap_or(0);
        let decoded_job = message.base_mut().get_byte();
        let job = decoded_job.unwrap_or(0);
        let decoded_raw_switch = message.base_mut().get_byte();
        let raw_switch = decoded_raw_switch.unwrap_or(0);

        let update = country_handler
            .get_country_mut(country_id)
            .map(|country| country.set_quest_switch(job, raw_switch != 0));
        let (disposition, log) = match update {
            None => (WorldCountryQuestSwitchDisposition::CountryMissing, None),
            Some(None) => (WorldCountryQuestSwitchDisposition::OfficerMissing, None),
            Some(Some(update)) => {
                let country_name = globe_setup.country_name(country_id);
                let line = country_quest_switch_log_line(
                    country_name.unwrap_or_default(),
                    job,
                    raw_switch,
                );
                put_string_to_file("king", &line);
                (
                    WorldCountryQuestSwitchDisposition::Updated(update),
                    Some(WorldCountryQuestSwitchLog {
                        country_name_complete: country_name.is_some(),
                        line,
                    }),
                )
            }
        };
        return WorldCountryMessageDispatch::Handled(
            WorldCountryMessageOutcome::QuestSwitchSynchronized(
                WorldCountryQuestSwitchSync {
                    country_id,
                    country_id_complete: decoded_country_id.is_some(),
                    job,
                    job_complete: decoded_job.is_some(),
                    raw_switch,
                    raw_switch_complete: decoded_raw_switch.is_some(),
                    disposition,
                    log,
                },
            ),
        );
    }
    if request_type == 0x0006_0316 {
        let decoded_player_id = message.base_mut().get_long();
        let player_id = decoded_player_id.unwrap_or(0);
        let decoded_country_id = message.base_mut().get_byte();
        let country_id = decoded_country_id.unwrap_or(0);

        let disposition = match country_handler.get_country(country_id) {
            None => WorldCountryExileTimeDisposition::CountryMissing,
            Some(country) => {
                let sampled_at_ms = legacy_tick_ms();
                match country.exile_remaining_time(
                    player_id,
                    sampled_at_ms,
                    country_parameters,
                ) {
                    Err(block) => {
                        WorldCountryExileTimeDisposition::ParameterUnavailable(block)
                    }
                    Ok(lookup) if game.online_player_by_id(player_id as u32).is_none() => {
                        WorldCountryExileTimeDisposition::PlayerMissing { lookup }
                    }
                    Ok(lookup) => {
                        let mut response = CMessage::new(0x0007_FF15);
                        response.base_mut().add_long(lookup.remaining_seconds);
                        response.base_mut().add_long(player_id);
                        let wire = response.as_wire_bytes().to_vec();
                        let delivery =
                            response.send_all(game.current_game_server_sender().as_ref());
                        WorldCountryExileTimeDisposition::Broadcast {
                            lookup,
                            wire,
                            delivery,
                        }
                    }
                }
            }
        };
        return WorldCountryMessageDispatch::Handled(
            WorldCountryMessageOutcome::ExileTimeSynchronized(WorldCountryExileTimeSync {
                player_id,
                player_id_complete: decoded_player_id.is_some(),
                country_id,
                country_id_complete: decoded_country_id.is_some(),
                disposition,
            }),
        );
    }
    let response_type = match request_type {
        COUNTRY_RELAY_FIRST => 0x0007_FF11,
        COUNTRY_RELAY_SECOND => 0x0007_FF12,
        _ => return WorldCountryMessageDispatch::Pending(message),
    };
    message.set_message_type(response_type);
    let wire = message.as_wire_bytes().to_vec();
    let delivery = message.send_all(game.current_game_server_sender().as_ref());
    WorldCountryMessageDispatch::Handled(WorldCountryMessageOutcome::Relay(
        WorldCountryRelayOutcome {
            request_type,
            response_type,
            wire,
            delivery,
        },
    ))
}

fn country_quest_switch_log_line(country_name: &[u8], job: u8, raw_switch: u8) -> Vec<u8> {
    let mut line = Vec::with_capacity(country_name.len() + 40);
    line.extend_from_slice(country_name);
    line.extend_from_slice(b" : Country Task: ");
    line.extend_from_slice(job.to_string().as_bytes());
    line.extend_from_slice(b" ( ");
    line.extend_from_slice(raw_switch.to_string().as_bytes());
    line.extend_from_slice(b" )");
    line.extend_from_slice(&[0xa1, 0xa3]);
    line
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryWarVictorySync {
    pub(crate) country: u8,
    pub(crate) country_complete: bool,
    pub(crate) report: CountryWarVictoryReport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountryWarVictoryDispatchError<ContextBlock> {
    pub(crate) source: ContextBlock,
}

pub(crate) fn dispatch_country_exile_result_message<
    Context: CountryExileResultContext + ?Sized,
>(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountryExileResultSync> {
    if message.message_type() != 0x6030e {
        return None;
    }

    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let decoded_success = message.base_mut().get_char();
    let raw_success = decoded_success.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;

    let Some(country) = country_handler.get_country_mut(country_id) else {
        return Some(WorldCountryExileResultSync {
            source_map_id,
            source_socket_id,
            player_id,
            player_id_complete: decoded_player_id.is_some(),
            raw_success,
            success_complete: decoded_success.is_some(),
            country_id,
            country_id_complete: decoded_country.is_some(),
            country_report: None,
            count_complete: None,
            disposition: WorldCountryExileResultDisposition::CountryMissing,
        });
    };

    let country_report = country.success_exiled(
        player_id,
        raw_success != 0,
        country_parameters,
        context,
    );
    let decoded_count = message.base_mut().get_long();
    let advertised_count = decoded_count.unwrap_or(0);

    let mut player_ids = Vec::new();
    if advertised_count > 0 {
        let cursor = message.base_mut().cursor();
        let remaining_bytes = message.as_wire_bytes().len().saturating_sub(cursor);
        let advertised_count_usize = advertised_count as usize;
        let required_bytes = advertised_count_usize.saturating_mul(size_of::<i32>());
        if required_bytes > remaining_bytes {
            return Some(WorldCountryExileResultSync {
                source_map_id,
                source_socket_id,
                player_id,
                player_id_complete: decoded_player_id.is_some(),
                raw_success,
                success_complete: decoded_success.is_some(),
                country_id,
                country_id_complete: decoded_country.is_some(),
                country_report: Some(country_report),
                count_complete: Some(decoded_count.is_some()),
                disposition: WorldCountryExileResultDisposition::PlayerListTruncated {
                    advertised_count,
                    available_complete_ids: remaining_bytes / size_of::<i32>(),
                },
            });
        }
        if player_ids.try_reserve_exact(advertised_count_usize).is_err() {
            return Some(WorldCountryExileResultSync {
                source_map_id,
                source_socket_id,
                player_id,
                player_id_complete: decoded_player_id.is_some(),
                raw_success,
                success_complete: decoded_success.is_some(),
                country_id,
                country_id_complete: decoded_country.is_some(),
                country_report: Some(country_report),
                count_complete: Some(decoded_count.is_some()),
                disposition: WorldCountryExileResultDisposition::PlayerListAllocationBlocked {
                    advertised_count,
                },
            });
        }
        for _ in 0..advertised_count_usize {
            player_ids.push(
                message
                    .base_mut()
                    .get_long()
                    .expect("полнота exile player-list проверена до декодирования"),
            );
        }
    }

    let mut response = CMessage::new(0x0007_FF15);
    response.base_mut().add_byte(country_id);
    response.base_mut().add_long(advertised_count);
    for &exiled_player_id in &player_ids {
        response.base_mut().add_long(exiled_player_id);
    }
    let wire = response.as_wire_bytes().to_vec();
    let delivery = context.send_all(&response);
    Some(WorldCountryExileResultSync {
        source_map_id,
        source_socket_id,
        player_id,
        player_id_complete: decoded_player_id.is_some(),
        raw_success,
        success_complete: decoded_success.is_some(),
        country_id,
        country_id_complete: decoded_country.is_some(),
        country_report: Some(country_report),
        count_complete: Some(decoded_count.is_some()),
        disposition: WorldCountryExileResultDisposition::Broadcast {
            advertised_count,
            player_ids,
            wire,
            delivery,
        },
    })
}

pub(crate) fn dispatch_country_exile_request_message<
    Context: CountryExileResultContext + ?Sized,
>(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountryExileRequestSync> {
    if message.message_type() != 0x6030d {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_target = message.base_mut().get_long();
    let target_player_id = decoded_target.unwrap_or(0);
    let decoded_king = message.base_mut().get_long();
    let king_player_id = decoded_king.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;

    let disposition = match country_handler.get_country(country_id) {
        None => WorldCountryExileRequestDisposition::CountryMissing,
        Some(country) if !country.authorize_king(king_player_id, context) => {
            WorldCountryExileRequestDisposition::KingRejected
        }
        Some(country) => {
            let operation = country.can_exile(country_parameters, context);
            if !matches!(operation, CountryCanExileDisposition::Allowed) {
                WorldCountryExileRequestDisposition::OperationRejected(operation)
            } else {
                let request = country.exile(target_player_id, country_parameters, context);
                let legacy_result = if matches!(request, CountryExileRequestDisposition::Sent { .. }) {
                    target_player_id
                } else {
                    0
                };
                WorldCountryExileRequestDisposition::Requested {
                    operation,
                    legacy_result,
                    request,
                }
            }
        }
    };
    Some(WorldCountryExileRequestSync {
        source_map_id,
        source_socket_id,
        target_player_id,
        target_complete: decoded_target.is_some(),
        king_player_id,
        king_complete: decoded_king.is_some(),
        country_id,
        country_complete: decoded_country.is_some(),
        disposition,
    })
}

pub(crate) fn dispatch_country_silence_request_message<
    Context: CountryExileResultContext + ?Sized,
>(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountrySilenceRequestSync> {
    if message.message_type() != 0x6030c {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_target = message.base_mut().get_long();
    let target_player_id = decoded_target.unwrap_or(0);
    let decoded_king = message.base_mut().get_long();
    let king_player_id = decoded_king.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;

    let disposition = match country_handler.get_country_mut(country_id) {
        None => WorldCountrySilenceRequestDisposition::CountryMissing,
        Some(country) if !country.authorize_king(king_player_id, context) => {
            WorldCountrySilenceRequestDisposition::KingRejected
        }
        Some(country) => {
            let operation = country.can_silence(country_parameters, context);
            if !matches!(operation, CountryCanSilenceDisposition::Allowed) {
                WorldCountrySilenceRequestDisposition::OperationRejected(operation)
            } else {
                let report = country.silence(target_player_id, country_parameters, context);
                WorldCountrySilenceRequestDisposition::Applied { operation, report }
            }
        }
    };
    Some(WorldCountrySilenceRequestSync {
        source_map_id,
        source_socket_id,
        target_player_id,
        target_complete: decoded_target.is_some(),
        king_player_id,
        king_complete: decoded_king.is_some(),
        country_id,
        country_complete: decoded_country.is_some(),
        disposition,
    })
}

pub(crate) fn dispatch_country_absolve_request_message<
    Context: CountryExileResultContext + ?Sized,
>(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountryAbsolveRequestSync> {
    if message.message_type() != 0x6030b {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_target = message.base_mut().get_long();
    let target_player_id = decoded_target.unwrap_or(0);
    let decoded_king = message.base_mut().get_long();
    let king_player_id = decoded_king.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;

    let disposition = match country_handler.get_country_mut(country_id) {
        None => WorldCountryAbsolveRequestDisposition::CountryMissing,
        Some(country) if !country.authorize_king(king_player_id, context) => {
            WorldCountryAbsolveRequestDisposition::KingRejected
        }
        Some(country) => {
            let operation = country.can_absolve(country_parameters, context);
            if !matches!(operation, CountryCanAbsolveDisposition::Allowed) {
                WorldCountryAbsolveRequestDisposition::OperationRejected(operation)
            } else {
                let report = country.absolve(target_player_id, country_parameters, context);
                WorldCountryAbsolveRequestDisposition::Applied { operation, report }
            }
        }
    };
    Some(WorldCountryAbsolveRequestSync {
        source_map_id,
        source_socket_id,
        target_player_id,
        target_complete: decoded_target.is_some(),
        king_player_id,
        king_complete: decoded_king.is_some(),
        country_id,
        country_complete: decoded_country.is_some(),
        disposition,
    })
}

pub(crate) fn dispatch_country_depose_minister_message<
    Context: CountryExileResultContext + ?Sized,
>(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountryDeposeMinisterSync> {
    if message.message_type() != 0x6030a {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_target = message.base_mut().get_long();
    let target_player_id = decoded_target.unwrap_or(0);
    let decoded_job = message.base_mut().get_char();
    let job = decoded_job.unwrap_or(0) as u8;
    let decoded_king = message.base_mut().get_long();
    let king_player_id = decoded_king.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;
    let disposition = match country_handler.get_country_mut(country_id) {
        None => WorldCountryDeposeMinisterDisposition::CountryMissing,
        Some(country) if !country.authorize_king(king_player_id, context) => {
            WorldCountryDeposeMinisterDisposition::KingRejected
        }
        Some(country) => {
            let operation = country.can_depose_minister(country_parameters, context);
            if !matches!(operation, CountryCanDeposeMinisterDisposition::Allowed) {
                WorldCountryDeposeMinisterDisposition::OperationRejected(operation)
            } else if !country.authorize_minister(target_player_id, job, context) {
                WorldCountryDeposeMinisterDisposition::MinisterRejected
            } else {
                let report = country.depose_minister(job, 7, country_parameters, context);
                WorldCountryDeposeMinisterDisposition::Applied { operation, report }
            }
        }
    };
    Some(WorldCountryDeposeMinisterSync {
        source_map_id,
        source_socket_id,
        target_player_id,
        target_complete: decoded_target.is_some(),
        job,
        job_complete: decoded_job.is_some(),
        king_player_id,
        king_complete: decoded_king.is_some(),
        country_id,
        country_complete: decoded_country.is_some(),
        disposition,
    })
}

pub(crate) fn dispatch_country_war_victory_message<Context: CountryWarVictoryContext + ?Sized>(
    message: &mut CMessage,
    country_war_sys: &mut CountryWarSys,
    context: &mut Context,
) -> Result<Option<WorldCountryWarVictorySync>, CountryWarVictoryDispatchError<Context::Block>> {
    if message.message_type() != 0x60318 {
        return Ok(None);
    }

    let decoded_country = message.base_mut().get_byte();
    let country = decoded_country.unwrap_or(0);
    let report = country_war_sys
        .on_flag_destory(i32::from(country), context)
        .map_err(|source| CountryWarVictoryDispatchError { source })?;
    Ok(Some(WorldCountryWarVictorySync {
        country,
        country_complete: decoded_country.is_some(),
        report,
    }))
}

pub(crate) fn dispatch_country_war_declaration_message<
    Context: CountryWarDeclarationContext + ?Sized,
>(
    message: &mut CMessage,
    country_war_sys: &mut CountryWarSys,
    context: &mut Context,
) -> Option<WorldCountryWarDeclarationSync> {
    if message.message_type() != 0x60317 {
        return None;
    }

    let source_map_id = message.map_id();
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let decoded_target_country = message.base_mut().get_long();
    let target_country = decoded_target_country.unwrap_or(0);
    let declaration = country_war_sys.player_declare(player_id, target_country, context);

    let mut response = CMessage::new(0x7ff16);
    response
        .base_mut()
        .add_char(if declaration.accepted() { 1 } else { 0 });
    response.base_mut().add_long(player_id);
    response.base_mut().add_long(target_country);
    let response_wire = response.as_wire_bytes().to_vec();
    let response_delivery = context.send_to_map_id(&response, source_map_id);

    Some(WorldCountryWarDeclarationSync {
        player_id,
        player_id_complete: decoded_player_id.is_some(),
        target_country,
        target_country_complete: decoded_target_country.is_some(),
        source_map_id,
        declaration,
        response_wire,
        response_delivery,
    })
}

pub(crate) fn dispatch_four_nation_war_result_message<
    Context: FourNationWarResultContext + ?Sized,
>(
    message: &mut CMessage,
    four_nation_war: &mut CFourNationWarSys,
    context: &mut Context,
) -> Option<WorldFourNationWarResultSync> {
    if message.message_type() != 0x60319 {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let report = four_nation_war.receive_result_from_game_server(message, context);
    Some(WorldFourNationWarResultSync {
        source_map_id,
        source_socket_id,
        report,
    })
}

pub(crate) fn dispatch_four_nation_war_time_message<
    Context: FourNationWarResultContext + ?Sized,
>(
    message: &mut CMessage,
    four_nation_war: &mut CFourNationWarSys,
    context: &mut Context,
) -> Option<WorldFourNationWarTimeSync> {
    if message.message_type() != 0x6031c {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let decoded_war_time = message.base_mut().get_long();
    let war_time = decoded_war_time.unwrap_or(0) as u32;
    let decoded_country = message.base_mut().get_long();
    let country = decoded_country.unwrap_or(0);
    let report = four_nation_war.send_player_war_time_to_game_server(
        player_id, war_time, country, context,
    );
    Some(WorldFourNationWarTimeSync {
        source_map_id,
        source_socket_id,
        numeric_payload_complete: [
            decoded_player_id.is_some(),
            decoded_war_time.is_some(),
            decoded_country.is_some(),
        ],
        report,
    })
}

pub(crate) fn dispatch_four_nation_country_fail_message<
    Context: FourNationCountryFailContext + ?Sized,
>(
    message: &mut CMessage,
    context: &mut Context,
) -> Option<WorldFourNationCountryFailSync> {
    if message.message_type() != 0x6031d {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_country = message.base_mut().get_long();
    let country = decoded_country.unwrap_or(0);
    let decoded_failed_country = message.base_mut().get_long();
    let failed_country = decoded_failed_country.unwrap_or(0);
    let report = CFourNationWarSys::one_country_fail(country, failed_country, context);
    Some(WorldFourNationCountryFailSync {
        source_map_id,
        source_socket_id,
        numeric_payload_complete: [
            decoded_country.is_some(),
            decoded_failed_country.is_some(),
        ],
        report,
    })
}

pub(crate) fn decode_four_nation_exploit_message(
    message: &mut CMessage,
) -> Option<WorldFourNationExploitRequest> {
    if message.message_type() != 0x6031a {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let decoded_increment = message.base_mut().get_long();
    let increment = decoded_increment.unwrap_or(0);
    Some(WorldFourNationExploitRequest {
        player_id,
        player_id_complete: decoded_player_id.is_some(),
        increment,
        increment_complete: decoded_increment.is_some(),
        source_map_id,
        source_socket_id,
    })
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\countrymessage.cpp

// ============================================================================
// FUNCTION: OnCountryMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\countrymessage.cpp:21
// RVA: 0x000A47F0
// ADDRESS: 004a47f0
// PROTOTYPE: void __cdecl OnCountryMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: WorldServer
