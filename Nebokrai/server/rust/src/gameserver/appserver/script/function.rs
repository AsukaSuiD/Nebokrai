//! Script-function dispatcher исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/script/function.cpp`. Из dense dispatcher-а
//! материализованы ID `9351 / ReflushExternProperty`, `9350 / OpenRolePage`,
//! `9354 / OpenEquipmentCompose` и `2216 / OpenGoodsUpgrade`. Refresh вычисляет первую
//! строка, DaKong gate предшествует lookup выбранного enhancement goods, а
//! gameplay передаётся canonical `CGame`, который сам исполняет localized
//! notices, session/plug lifecycle и client wire; runtime сообщает только
//! ещё не owned team skill-state. Country scalar family `9001/9003/9009/9011/
//! 9013` сохраняет byte country lookup, field-specific clamp, local mutation и
//! World `0x60314`. Quest-switch `9018/9019` сохраняет read-only lookup и
//! странность writer-а: второй аргумент влияет только на country fallback, а
//! применяемое значение всегда `true`; `0x60315` уходит до local map write.
//! Exile-time `9021` остаётся полностью локальным: страна script-player, один
//! runtime clock sample и wrapping `CCountry` calculation. `9317 / AddKingPoint`
//! складывает delta как DWORD, меняет local control point и публикует selector
//! `5`, сохраняя нулевой script result. Government identity `9004/9005/9006`
//! читает mutating CI, отправляет назначение `0x60304` без преждевременной
//! local mutation и читает отдельный краткоживущий king-ID response state.
//! Country scalar query family `9000/9002/9008/9010/9012` одним контрактом
//! сужает explicit country до byte либо использует страну script-player и
//! возвращает `-1` при недоступном owner-е.
//! Aliases `2633/9020` разрешают local target и проходят canonical
//! `CGame::player_country_identity`, который mutating-читает ordered CI `1..8`.
//! Country-war declaration `9100` сохраняет NPC distance gate, фазу объявления,
//! king-CI, обе duplicate-проверки, idle-region gate, `GS0213..GS0218` и
//! terminal World request `0x60317 [player, target country]`.
//! Полный expression evaluator и остальные function ID ниже пока остаются RAW.

use crate::gameserver::appserver::country::country::{
    CountryExileRestTimeReport, CountryScalarMutationReport,
};
use crate::gameserver::appserver::session::cequipmentdakong::EquipmentDaKongExternalRefreshReport;
use crate::gameserver::appserver::session::csessionfactory::EquipmentSessionPlugKind;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeResolver};
use crate::gameserver::gameserver::game::{
    colored_player_notice_message, CGame, EquipmentDaKongContext, EquipmentSessionOpenContext,
    EquipmentSessionOpenReport,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::public::guid::CGuid;

pub(crate) const SCRIPT_FUNCTION_REFLUSH_EXTERN_PROPERTY: i32 = 9351;
pub(crate) const SCRIPT_FUNCTION_OPEN_DA_KONG: i32 = 9350;
pub(crate) const SCRIPT_FUNCTION_OPEN_EQUIPMENT_COMPOSE: i32 = 9354;
pub(crate) const SCRIPT_FUNCTION_OPEN_EQUIPMENT_UPGRADE: i32 = 2216;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_POWER: i32 = 9001;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_POWER: i32 = 9000;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_TECH_LEVEL: i32 = 9002;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_TECH_LEVEL: i32 = 9003;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_TREASURY: i32 = 9009;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_MATERIAL: i32 = 9011;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_TECH: i32 = 9013;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_CI: i32 = 9004;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_CI: i32 = 9005;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_KING_ID: i32 = 9006;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_TREASURY: i32 = 9008;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_MATERIAL: i32 = 9010;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_TECH: i32 = 9012;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_OCCUPATION: i32 = 2633;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_IDENTITY: i32 = 9020;
pub(crate) const SCRIPT_FUNCTION_GET_QUEST_SWITCH: i32 = 9018;
pub(crate) const SCRIPT_FUNCTION_SET_QUEST_SWITCH: i32 = 9019;
pub(crate) const SCRIPT_FUNCTION_EXILE_TIME: i32 = 9021;
pub(crate) const SCRIPT_FUNCTION_ADD_KING_POINT: i32 = 9317;
pub(crate) const SCRIPT_FUNCTION_DECLARE_COUNTRY_WAR: i32 = 9100;
const SCRIPT_INT_PARAMETER_ERROR: i32 = 0x09ff_fff9;
const SCRIPT_PLAYER_TYPE: i32 = 400;
const SCRIPT_NPC_TYPE: i32 = 500;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarDeclarationScriptDisposition {
    CallerMissing,
    CallerShapeMissing,
    TooFar {
        distance: i32,
        maximum: i32,
    },
    DeclarationClosed {
        delivery: i32,
    },
    TargetCountryMissing,
    SameCountry {
        country: u8,
        delivery: i32,
    },
    KingRequired {
        country: u8,
        recorded_king_id: i32,
        delivery: i32,
    },
    AttackerAlreadyDeclared {
        country: u8,
        delivery: i32,
    },
    TargetAlreadyDeclared {
        country: i32,
        delivery: i32,
    },
    IdleRegionMissing {
        delivery: i32,
    },
    Requested {
        player_id: i32,
        target_country: i32,
        idle_region_id: i32,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarDeclarationScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        legacy_return: i32,
        disposition: CountryWarDeclarationScriptDisposition,
    },
}

pub(crate) fn run_country_war_declaration_script_function(
    game: &mut CGame,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
    function_id: i32,
    evaluated_target_country: Option<i32>,
) -> CountryWarDeclarationScriptFunctionOutcome {
    if function_id != SCRIPT_FUNCTION_DECLARE_COUNTRY_WAR {
        return CountryWarDeclarationScriptFunctionOutcome::DifferentFunction;
    }
    let (Some(player_id), Some(npc_id)) = (script_player_id, script_npc_id) else {
        return country_war_declaration_handled(
            1,
            CountryWarDeclarationScriptDisposition::CallerMissing,
        );
    };
    let player_shape = game.resolve_shape(ShapeIdentity {
        object_type: SCRIPT_PLAYER_TYPE,
        id: player_id,
        ex_id: CGuid::GUID_INVALID,
    });
    let npc_shape = game.resolve_shape(ShapeIdentity {
        object_type: SCRIPT_NPC_TYPE,
        id: npc_id,
        ex_id: CGuid::GUID_INVALID,
    });
    let (Some(player_shape), Some(npc_shape)) = (player_shape, npc_shape) else {
        return country_war_declaration_handled(
            1,
            CountryWarDeclarationScriptDisposition::CallerShapeMissing,
        );
    };
    let distance = npc_shape.distance(player_shape);
    let maximum = game
        .globe_setup()
        .area_width()
        .max(game.globe_setup().area_height())
        / 2;
    if maximum < distance {
        return country_war_declaration_handled(
            2,
            CountryWarDeclarationScriptDisposition::TooFar { distance, maximum },
        );
    }
    if !game.country_war_sys().state_declare {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0213");
        return country_war_declaration_handled(
            3,
            CountryWarDeclarationScriptDisposition::DeclarationClosed { delivery },
        );
    }
    let target_country = evaluated_target_country.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    if target_country == SCRIPT_INT_PARAMETER_ERROR {
        return country_war_declaration_handled(
            3,
            CountryWarDeclarationScriptDisposition::TargetCountryMissing,
        );
    }
    let Some(player_country) = game.find_player(player_id).map(|player| player.country()) else {
        return country_war_declaration_handled(
            1,
            CountryWarDeclarationScriptDisposition::CallerMissing,
        );
    };
    if i32::from(player_country) == target_country {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0214");
        return country_war_declaration_handled(
            3,
            CountryWarDeclarationScriptDisposition::SameCountry {
                country: player_country,
                delivery,
            },
        );
    }
    let recorded_king_id = game
        .country_handler_mut()
        .country_mut(player_country)
        .map(|country| country.country_information(1))
        .unwrap_or(0);
    if recorded_king_id != player_id {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0215");
        return country_war_declaration_handled(
            4,
            CountryWarDeclarationScriptDisposition::KingRequired {
                country: player_country,
                recorded_king_id,
                delivery,
            },
        );
    }
    if game
        .country_war_sys()
        .is_already_declar(i32::from(player_country))
    {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0216");
        return country_war_declaration_handled(
            4,
            CountryWarDeclarationScriptDisposition::AttackerAlreadyDeclared {
                country: player_country,
                delivery,
            },
        );
    }
    if game.country_war_sys().is_already_declar(target_country) {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0217");
        return country_war_declaration_handled(
            4,
            CountryWarDeclarationScriptDisposition::TargetAlreadyDeclared {
                country: target_country,
                delivery,
            },
        );
    }
    let idle_region_id = game.country_war_sys().get_idle_war_region();
    if idle_region_id == 0 {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0218");
        return country_war_declaration_handled(
            4,
            CountryWarDeclarationScriptDisposition::IdleRegionMissing { delivery },
        );
    }
    let mut request = CMessage::new(0x0006_0317);
    request.base_mut().add_long(player_id);
    request.base_mut().add_long(target_country);
    let delivery = request.send(game, false);
    country_war_declaration_handled(
        0,
        CountryWarDeclarationScriptDisposition::Requested {
            player_id,
            target_country,
            idle_region_id,
            delivery,
        },
    )
}

fn send_country_war_script_notice(game: &CGame, player_id: i32, string_id: &[u8]) -> i32 {
    colored_player_notice_message(0xffff_ffff, 0xffff_0000, game.get_string_by_id(string_id))
        .send_to_player(game.net_server(), player_id)
}

fn country_war_declaration_handled(
    legacy_return: i32,
    disposition: CountryWarDeclarationScriptDisposition,
) -> CountryWarDeclarationScriptFunctionOutcome {
    CountryWarDeclarationScriptFunctionOutcome::Handled {
        legacy_return,
        disposition,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryScalarQueryField {
    Power,
    TechnologyLevel,
    Treasury,
    Material,
    TechnologyExperience,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryScalarQueryDisposition {
    ScriptPlayerMissing,
    CountryMissing {
        country: u8,
    },
    Completed {
        country: u8,
        field: CountryScalarQueryField,
        value: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryScalarQueryScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        function_id: i32,
        legacy_return: i32,
        disposition: CountryScalarQueryDisposition,
    },
}

pub(crate) fn run_country_scalar_query_script_function(
    game: &CGame,
    script_player_id: Option<i32>,
    function_id: i32,
    evaluated_country: Option<i32>,
) -> CountryScalarQueryScriptFunctionOutcome {
    let field = match function_id {
        SCRIPT_FUNCTION_GET_COUNTRY_POWER => CountryScalarQueryField::Power,
        SCRIPT_FUNCTION_GET_COUNTRY_TECH_LEVEL => CountryScalarQueryField::TechnologyLevel,
        SCRIPT_FUNCTION_GET_COUNTRY_TREASURY => CountryScalarQueryField::Treasury,
        SCRIPT_FUNCTION_GET_COUNTRY_MATERIAL => CountryScalarQueryField::Material,
        SCRIPT_FUNCTION_GET_COUNTRY_TECH => CountryScalarQueryField::TechnologyExperience,
        _ => return CountryScalarQueryScriptFunctionOutcome::DifferentFunction,
    };
    let raw_country = evaluated_country.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    let country = if raw_country == SCRIPT_INT_PARAMETER_ERROR {
        let Some(player) = script_player_id.and_then(|player_id| game.find_player(player_id))
        else {
            return CountryScalarQueryScriptFunctionOutcome::Handled {
                function_id,
                legacy_return: -1,
                disposition: CountryScalarQueryDisposition::ScriptPlayerMissing,
            };
        };
        player.country()
    } else {
        raw_country as u8
    };
    let Some(country_owner) = game.country_handler().country(country) else {
        return CountryScalarQueryScriptFunctionOutcome::Handled {
            function_id,
            legacy_return: -1,
            disposition: CountryScalarQueryDisposition::CountryMissing { country },
        };
    };
    let value = match field {
        CountryScalarQueryField::Power => country_owner.power,
        CountryScalarQueryField::TechnologyLevel => country_owner.tech_level,
        CountryScalarQueryField::Treasury => country_owner.treasury,
        CountryScalarQueryField::Material => country_owner.material_point,
        CountryScalarQueryField::TechnologyExperience => country_owner.tech_current_exp,
    };
    CountryScalarQueryScriptFunctionOutcome::Handled {
        function_id,
        legacy_return: value,
        disposition: CountryScalarQueryDisposition::Completed {
            country,
            field,
            value,
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryIdentityScriptDisposition {
    ArgumentMissing {
        argument: usize,
    },
    ScriptPlayerMissing,
    TargetPlayerMissing {
        player_id: i32,
    },
    IdentityOutOfRange {
        identity: i32,
    },
    CountryMissing {
        country: u8,
    },
    CountryInformationRead {
        country: u8,
        identity: u8,
        player_id: i32,
    },
    AssignmentRequested {
        country: u8,
        identity: u8,
        player_id: i32,
        delivery: Result<i32, SendMessageError>,
    },
    KingIdRead {
        country: u8,
        king_id: i32,
    },
    TargetIdRejected {
        player_id: i32,
    },
    PlayerMissing {
        player_id: Option<i32>,
    },
    PlayerIdentityRead {
        player_id: i32,
        identity: u8,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryIdentityScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        function_id: i32,
        legacy_return: i32,
        disposition: CountryIdentityScriptDisposition,
    },
}

pub(crate) fn run_country_identity_script_function(
    game: &mut CGame,
    script_player_id: Option<i32>,
    function_id: i32,
    evaluated_first: Option<i32>,
    evaluated_second: Option<i32>,
) -> CountryIdentityScriptFunctionOutcome {
    if !matches!(
        function_id,
        SCRIPT_FUNCTION_GET_COUNTRY_CI
            | SCRIPT_FUNCTION_SET_COUNTRY_CI
            | SCRIPT_FUNCTION_GET_COUNTRY_KING_ID
            | SCRIPT_FUNCTION_GET_COUNTRY_OCCUPATION
            | SCRIPT_FUNCTION_GET_COUNTRY_IDENTITY
    ) {
        return CountryIdentityScriptFunctionOutcome::DifferentFunction;
    }
    if matches!(
        function_id,
        SCRIPT_FUNCTION_GET_COUNTRY_OCCUPATION | SCRIPT_FUNCTION_GET_COUNTRY_IDENTITY
    ) {
        let raw_player_id = evaluated_first.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
        let player_id = if raw_player_id == SCRIPT_INT_PARAMETER_ERROR {
            let Some(player_id) = script_player_id else {
                return country_identity_handled(
                    function_id,
                    -1,
                    CountryIdentityScriptDisposition::PlayerMissing { player_id: None },
                );
            };
            player_id
        } else if raw_player_id <= 0 {
            return country_identity_handled(
                function_id,
                -1,
                CountryIdentityScriptDisposition::TargetIdRejected {
                    player_id: raw_player_id,
                },
            );
        } else {
            raw_player_id
        };
        if game.find_player(player_id).is_none() {
            return country_identity_handled(
                function_id,
                -1,
                CountryIdentityScriptDisposition::PlayerMissing {
                    player_id: Some(player_id),
                },
            );
        }
        let identity = game.player_country_identity(player_id);
        return country_identity_handled(
            function_id,
            i32::from(identity),
            CountryIdentityScriptDisposition::PlayerIdentityRead {
                player_id,
                identity,
            },
        );
    }
    if function_id == SCRIPT_FUNCTION_SET_COUNTRY_CI {
        let identity = evaluated_first.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
        if identity == SCRIPT_INT_PARAMETER_ERROR {
            return country_identity_handled(
                function_id,
                -1,
                CountryIdentityScriptDisposition::ArgumentMissing { argument: 0 },
            );
        }
        let raw_target = evaluated_second.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
        let target_id = if raw_target == SCRIPT_INT_PARAMETER_ERROR {
            let Some(player_id) = script_player_id else {
                return country_identity_handled(
                    function_id,
                    -1,
                    CountryIdentityScriptDisposition::ScriptPlayerMissing,
                );
            };
            player_id
        } else {
            raw_target
        };
        let Some(target) = game.find_player(target_id) else {
            return country_identity_handled(
                function_id,
                -1,
                CountryIdentityScriptDisposition::TargetPlayerMissing {
                    player_id: target_id,
                },
            );
        };
        let country = target.country();
        if !(0..=8).contains(&identity) {
            return country_identity_handled(
                function_id,
                -1,
                CountryIdentityScriptDisposition::IdentityOutOfRange { identity },
            );
        }
        let mut request = CMessage::new(0x0006_0304);
        request.base_mut().add_byte(country);
        request.base_mut().add_long(target_id);
        request.base_mut().add_byte(identity as u8);
        return country_identity_handled(
            function_id,
            0,
            CountryIdentityScriptDisposition::AssignmentRequested {
                country,
                identity: identity as u8,
                player_id: target_id,
                delivery: request.send(game, false),
            },
        );
    }

    let raw_country = evaluated_first.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    let country_was_defaulted = raw_country == SCRIPT_INT_PARAMETER_ERROR;
    let country = if raw_country == SCRIPT_INT_PARAMETER_ERROR {
        let Some(player) = script_player_id.and_then(|player_id| game.find_player(player_id))
        else {
            return country_identity_handled(
                function_id,
                -1,
                CountryIdentityScriptDisposition::ScriptPlayerMissing,
            );
        };
        player.country()
    } else {
        raw_country as u8
    };
    if function_id == SCRIPT_FUNCTION_GET_COUNTRY_KING_ID {
        let Some(country_owner) = game.country_handler().country(country) else {
            return country_identity_handled(
                function_id,
                -1,
                CountryIdentityScriptDisposition::CountryMissing { country },
            );
        };
        let king_id = country_owner.king_id();
        return country_identity_handled(
            function_id,
            king_id,
            CountryIdentityScriptDisposition::KingIdRead { country, king_id },
        );
    }

    let identity = if country_was_defaulted {
        1
    } else {
        evaluated_second
            .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            .unwrap_or(1) as u8
    };
    let Some(country_owner) = game.country_handler_mut().country_mut(country) else {
        return country_identity_handled(
            function_id,
            -1,
            CountryIdentityScriptDisposition::CountryMissing { country },
        );
    };
    let player_id = country_owner.country_information(identity);
    country_identity_handled(
        function_id,
        player_id,
        CountryIdentityScriptDisposition::CountryInformationRead {
            country,
            identity,
            player_id,
        },
    )
}

fn country_identity_handled(
    function_id: i32,
    legacy_return: i32,
    disposition: CountryIdentityScriptDisposition,
) -> CountryIdentityScriptFunctionOutcome {
    CountryIdentityScriptFunctionOutcome::Handled {
        function_id,
        legacy_return,
        disposition,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryControlPointScriptDisposition {
    ArgumentMissing {
        argument: usize,
    },
    CountryMissing {
        country: u8,
    },
    Applied {
        country: u8,
        delta: i32,
        mutation: CountryScalarMutationReport,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryControlPointScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        legacy_return: i32,
        disposition: CountryControlPointScriptDisposition,
    },
}

pub(crate) fn run_country_control_point_script_function(
    game: &mut CGame,
    function_id: i32,
    evaluated_delta: Option<i32>,
    evaluated_country: Option<i32>,
) -> CountryControlPointScriptFunctionOutcome {
    if function_id != SCRIPT_FUNCTION_ADD_KING_POINT {
        return CountryControlPointScriptFunctionOutcome::DifferentFunction;
    }
    let delta = evaluated_delta.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    if delta == SCRIPT_INT_PARAMETER_ERROR {
        return CountryControlPointScriptFunctionOutcome::Handled {
            legacy_return: 0,
            disposition: CountryControlPointScriptDisposition::ArgumentMissing { argument: 0 },
        };
    }
    let raw_country = evaluated_country.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    if raw_country == SCRIPT_INT_PARAMETER_ERROR {
        return CountryControlPointScriptFunctionOutcome::Handled {
            legacy_return: 0,
            disposition: CountryControlPointScriptDisposition::ArgumentMissing { argument: 1 },
        };
    }
    let country = raw_country as u8;
    let Some(country_owner) = game.country_handler().country(country) else {
        return CountryControlPointScriptFunctionOutcome::Handled {
            legacy_return: 0,
            disposition: CountryControlPointScriptDisposition::CountryMissing { country },
        };
    };
    let applied = country_owner.control_point.wrapping_add(delta);
    let (mutation, message) = game
        .country_handler_mut()
        .country_mut(country)
        .expect("country owner жив до control-point mutation")
        .set_script_scalar(5, applied);
    let delivery = message.send(game, false);
    CountryControlPointScriptFunctionOutcome::Handled {
        legacy_return: 0,
        disposition: CountryControlPointScriptDisposition::Applied {
            country,
            delta,
            mutation,
            delivery,
        },
    }
}

pub(crate) trait CountryExileTimeScriptContext {
    fn country_exile_time_now_milliseconds(&mut self) -> u32;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryExileTimeScriptDisposition {
    ScriptPlayerMissing,
    CountryMissing {
        country: u8,
    },
    ParameterUnavailable {
        field: &'static str,
        sampled_at_ms: u32,
    },
    Completed(CountryExileRestTimeReport),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryExileTimeScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        player_id: i32,
        legacy_return: i32,
        disposition: CountryExileTimeScriptDisposition,
    },
}

pub(crate) fn run_country_exile_time_script_function<Context: CountryExileTimeScriptContext>(
    game: &CGame,
    script_player_id: Option<i32>,
    function_id: i32,
    evaluated_player_id: Option<i32>,
    context: &mut Context,
) -> CountryExileTimeScriptFunctionOutcome {
    if function_id != SCRIPT_FUNCTION_EXILE_TIME {
        return CountryExileTimeScriptFunctionOutcome::DifferentFunction;
    }
    let Some(script_player) = script_player_id.and_then(|player_id| game.find_player(player_id))
    else {
        return CountryExileTimeScriptFunctionOutcome::Handled {
            player_id: evaluated_player_id.unwrap_or(SCRIPT_INT_PARAMETER_ERROR),
            legacy_return: -1,
            disposition: CountryExileTimeScriptDisposition::ScriptPlayerMissing,
        };
    };
    let player_id = match evaluated_player_id {
        Some(value) if value != SCRIPT_INT_PARAMETER_ERROR => value,
        _ => script_player_id.expect("live script player имеет ID"),
    };
    let country = script_player.country();
    let Some(country_owner) = game.country_handler().country(country) else {
        return CountryExileTimeScriptFunctionOutcome::Handled {
            player_id,
            legacy_return: -1,
            disposition: CountryExileTimeScriptDisposition::CountryMissing { country },
        };
    };
    let sampled_at_ms = context.country_exile_time_now_milliseconds();
    match country_owner.exile_rest_time(player_id, sampled_at_ms, game.country_param().exile_time())
    {
        Ok(report) => CountryExileTimeScriptFunctionOutcome::Handled {
            player_id,
            legacy_return: report.remaining_seconds,
            disposition: CountryExileTimeScriptDisposition::Completed(report),
        },
        Err(field) => CountryExileTimeScriptFunctionOutcome::Handled {
            player_id,
            legacy_return: -1,
            disposition: CountryExileTimeScriptDisposition::ParameterUnavailable {
                field,
                sampled_at_ms,
            },
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryQuestSwitchScriptDisposition {
    IdentityMissing,
    PlayerMissing,
    CountryMissing {
        country: u8,
    },
    Read {
        country: u8,
        enabled: bool,
    },
    Written {
        country: u8,
        raw_switch: i32,
        mutation: crate::gameserver::appserver::country::country::CountryQuestSwitchMutationReport,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryQuestSwitchScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        function_id: i32,
        identity: i32,
        legacy_return: i32,
        disposition: CountryQuestSwitchScriptDisposition,
    },
}

pub(crate) fn run_country_quest_switch_script_function(
    game: &mut CGame,
    script_player_id: Option<i32>,
    function_id: i32,
    evaluated_identity: Option<i32>,
    evaluated_switch: Option<i32>,
    evaluated_country: Option<i32>,
) -> CountryQuestSwitchScriptFunctionOutcome {
    if !matches!(
        function_id,
        SCRIPT_FUNCTION_GET_QUEST_SWITCH | SCRIPT_FUNCTION_SET_QUEST_SWITCH
    ) {
        return CountryQuestSwitchScriptFunctionOutcome::DifferentFunction;
    }
    let identity = evaluated_identity.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    if identity == SCRIPT_INT_PARAMETER_ERROR {
        return CountryQuestSwitchScriptFunctionOutcome::Handled {
            function_id,
            identity,
            legacy_return: -1,
            disposition: CountryQuestSwitchScriptDisposition::IdentityMissing,
        };
    }
    let raw_switch = evaluated_switch.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    let raw_country = evaluated_country.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    let needs_player_country = if function_id == SCRIPT_FUNCTION_GET_QUEST_SWITCH {
        raw_country == SCRIPT_INT_PARAMETER_ERROR
    } else {
        raw_switch == SCRIPT_INT_PARAMETER_ERROR || raw_country == SCRIPT_INT_PARAMETER_ERROR
    };
    let country = if needs_player_country {
        let Some(player) = script_player_id.and_then(|player_id| game.find_player(player_id))
        else {
            return CountryQuestSwitchScriptFunctionOutcome::Handled {
                function_id,
                identity,
                legacy_return: -1,
                disposition: CountryQuestSwitchScriptDisposition::PlayerMissing,
            };
        };
        player.country()
    } else {
        raw_country as u8
    };
    let Some(country_owner) = game.country_handler().country(country) else {
        return CountryQuestSwitchScriptFunctionOutcome::Handled {
            function_id,
            identity,
            legacy_return: -1,
            disposition: CountryQuestSwitchScriptDisposition::CountryMissing { country },
        };
    };
    if function_id == SCRIPT_FUNCTION_GET_QUEST_SWITCH {
        let enabled = country_owner.quest_switch(identity as u8);
        return CountryQuestSwitchScriptFunctionOutcome::Handled {
            function_id,
            identity,
            legacy_return: i32::from(enabled),
            disposition: CountryQuestSwitchScriptDisposition::Read { country, enabled },
        };
    }

    let message = country_owner.quest_switch_message(identity as u8, true);
    let delivery = message.send(game, false);
    let mutation = game
        .country_handler_mut()
        .country_mut(country)
        .expect("country owner жив после quest-switch World enqueue")
        .apply_quest_switch(identity as u8, true);
    CountryQuestSwitchScriptFunctionOutcome::Handled {
        function_id,
        identity,
        legacy_return: if identity as u8 == 0 { -1 } else { identity },
        disposition: CountryQuestSwitchScriptDisposition::Written {
            country,
            raw_switch,
            mutation,
            delivery,
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryScalarScriptDisposition {
    ValueMissing,
    CountryMissing {
        country: u8,
    },
    ParameterUnavailable {
        field: &'static str,
    },
    TechnologyLevelMissing {
        level: i32,
    },
    Applied {
        country: u8,
        requested: i32,
        applied: i32,
        mutation: CountryScalarMutationReport,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryScalarScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        function_id: i32,
        legacy_return: i32,
        disposition: CountryScalarScriptDisposition,
    },
}

pub(crate) fn run_country_scalar_script_function(
    game: &mut CGame,
    function_id: i32,
    evaluated_country: Option<i32>,
    evaluated_value: Option<i32>,
) -> CountryScalarScriptFunctionOutcome {
    let (selector, default_return) = match function_id {
        SCRIPT_FUNCTION_SET_COUNTRY_POWER => (2, -1),
        SCRIPT_FUNCTION_SET_COUNTRY_TECH_LEVEL => (4, -1),
        SCRIPT_FUNCTION_SET_COUNTRY_TREASURY => (1, 0),
        SCRIPT_FUNCTION_SET_COUNTRY_MATERIAL => (6, -1),
        SCRIPT_FUNCTION_SET_COUNTRY_TECH => (3, -1),
        _ => return CountryScalarScriptFunctionOutcome::DifferentFunction,
    };
    let country = evaluated_country.unwrap_or(SCRIPT_INT_PARAMETER_ERROR) as u8;
    let Some(requested) = evaluated_value.filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
    else {
        return CountryScalarScriptFunctionOutcome::Handled {
            function_id,
            legacy_return: default_return,
            disposition: CountryScalarScriptDisposition::ValueMissing,
        };
    };
    if game.country_handler().country(country).is_none() {
        return CountryScalarScriptFunctionOutcome::Handled {
            function_id,
            legacy_return: default_return,
            disposition: CountryScalarScriptDisposition::CountryMissing { country },
        };
    }

    let applied = match function_id {
        SCRIPT_FUNCTION_SET_COUNTRY_POWER => {
            let Some(maximum) = game.country_param().max_country_power() else {
                return scalar_parameter_unavailable(
                    function_id,
                    default_return,
                    "_max_country_power",
                );
            };
            if requested < 0 {
                0
            } else {
                requested.min(maximum)
            }
        }
        SCRIPT_FUNCTION_SET_COUNTRY_TECH_LEVEL => {
            if requested <= 0 {
                1
            } else {
                requested.min(game.country_param().country_tech_level_count())
            }
        }
        SCRIPT_FUNCTION_SET_COUNTRY_TREASURY => {
            let Some(maximum) = game.country_param().max_country_treasury() else {
                return scalar_parameter_unavailable(
                    function_id,
                    default_return,
                    "_max_country_treasury",
                );
            };
            if requested < 0 {
                0
            } else {
                requested.min(maximum)
            }
        }
        SCRIPT_FUNCTION_SET_COUNTRY_MATERIAL => {
            let Some(maximum) = game.country_param().max_king_material_point() else {
                return scalar_parameter_unavailable(
                    function_id,
                    default_return,
                    "_max_king_material_point",
                );
            };
            if requested < 0 {
                0
            } else {
                requested.min(maximum)
            }
        }
        SCRIPT_FUNCTION_SET_COUNTRY_TECH => {
            let next_level = game
                .country_handler()
                .country(country)
                .expect("country owner проверен перед tech-level lookup")
                .tech_level
                .wrapping_add(1);
            let Some(level) = game.country_param().country_tech_level(next_level) else {
                return CountryScalarScriptFunctionOutcome::Handled {
                    function_id,
                    legacy_return: default_return,
                    disposition: CountryScalarScriptDisposition::TechnologyLevelMissing {
                        level: next_level,
                    },
                };
            };
            if requested > level.country_tech_exp {
                level.country_tech_exp
            } else if requested < 0 {
                0
            } else {
                requested
            }
        }
        _ => unreachable!("country scalar function ID проверен перед clamp"),
    };
    let (mutation, message) = game
        .country_handler_mut()
        .country_mut(country)
        .expect("country owner жив до scalar mutation")
        .set_script_scalar(selector, applied);
    let delivery = message.send(game, false);
    CountryScalarScriptFunctionOutcome::Handled {
        function_id,
        legacy_return: applied,
        disposition: CountryScalarScriptDisposition::Applied {
            country,
            requested,
            applied,
            mutation,
            delivery,
        },
    }
}

fn scalar_parameter_unavailable(
    function_id: i32,
    legacy_return: i32,
    field: &'static str,
) -> CountryScalarScriptFunctionOutcome {
    CountryScalarScriptFunctionOutcome::Handled {
        function_id,
        legacy_return,
        disposition: CountryScalarScriptDisposition::ParameterUnavailable { field },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentSessionScriptFunctionOutcome {
    DifferentFunction,
    Opened(EquipmentSessionOpenReport),
}

pub(crate) fn run_equipment_session_script_function<Context: EquipmentSessionOpenContext>(
    game: &mut CGame,
    player_id: i32,
    function_id: i32,
    context: &mut Context,
) -> EquipmentSessionScriptFunctionOutcome {
    let kind = match function_id {
        SCRIPT_FUNCTION_OPEN_DA_KONG => EquipmentSessionPlugKind::DaKong,
        SCRIPT_FUNCTION_OPEN_EQUIPMENT_COMPOSE => EquipmentSessionPlugKind::Compose,
        SCRIPT_FUNCTION_OPEN_EQUIPMENT_UPGRADE => EquipmentSessionPlugKind::Upgrade,
        _ => return EquipmentSessionScriptFunctionOutcome::DifferentFunction,
    };
    EquipmentSessionScriptFunctionOutcome::Opened(
        game.open_equipment_session(player_id, kind, context),
    )
}

#[must_use = "script dispatch отличает чужой ID от handled no-op и выполненного gameplay"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentDaKongScriptFunctionOutcome {
    DifferentFunction,
    HandledWithoutCall,
    Refreshed(EquipmentDaKongExternalRefreshReport),
}

pub(crate) fn run_equipment_da_kong_script_function<Context: EquipmentDaKongContext>(
    game: &mut CGame,
    player_id: i32,
    function_id: i32,
    evaluated_first_string: Option<&[u8]>,
    context: &mut Context,
) -> EquipmentDaKongScriptFunctionOutcome {
    if function_id != SCRIPT_FUNCTION_REFLUSH_EXTERN_PROPERTY {
        return EquipmentDaKongScriptFunctionOutcome::DifferentFunction;
    }
    let Some(cost_original_name) = evaluated_first_string.filter(|value| !value.is_empty()) else {
        return EquipmentDaKongScriptFunctionOutcome::HandledWithoutCall;
    };
    EquipmentDaKongScriptFunctionOutcome::Refreshed(
        game.reflush_equipment_da_kong_external_property(player_id, cost_original_name, context),
    )
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2508::CreateFaction::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6131
// RVA: 0x000AEB90
// ADDRESS: 004aeb90
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2527::ApplyJoinFaction::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6248
// RVA: 0x000AEC30
// ADDRESS: 004aec30
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2712::DeclareFactionWar::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6716
// RVA: 0x000AECE0
// ADDRESS: 004aece0
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2527::ApplyJoinFaction::ApplyJoinFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6242
// RVA: 0x000AEDB0
// ADDRESS: 004aedb0
// PROTOTYPE: undefined __thiscall ApplyJoinFaction(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2712::DeclareFactionWar::DeclareFactionWar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6710
// RVA: 0x000AEDE0
// ADDRESS: 004aede0
// PROTOTYPE: undefined __thiscall DeclareFactionWar(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2508::CreateFaction::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6128
// RVA: 0x000AEE90
// ADDRESS: 004aee90
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2508::CreateFaction::CreateFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6125
// RVA: 0x000AEEB0
// ADDRESS: 004aeeb0
// PROTOTYPE: undefined __thiscall CreateFaction(long param_1, char * param_2, long param_3, uchar param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2508::CreateFaction::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6144
// RVA: 0x000AF250
// ADDRESS: 004af250
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004af3ee
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6181
// RVA: 0x000AF3EE
// ADDRESS: 004af3ee
// PROTOTYPE: undefined Catch@004af3ee()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004af43b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6185
// RVA: 0x000AF43B
// ADDRESS: 004af43b
// PROTOTYPE: undefined FUN_004af43b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2527::ApplyJoinFaction::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6263
// RVA: 0x000AF460
// ADDRESS: 004af460
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_long___thiscall_CScript::RunFunction(char_const*)'::__l2712::DeclareFactionWar::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:6731
// RVA: 0x000AF660
// ADDRESS: 004af660
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::CheckFunctionRunning
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:11947
// RVA: 0x000AF990
// ADDRESS: 004af990
// PROTOTYPE: SCRIPTRETURN __thiscall CheckFunctionRunning(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScript::RunFunction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:90
// RVA: 0x000AFAF0
// ADDRESS: 004afaf0
// PROTOTYPE: long __thiscall RunFunction(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004c3f4b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\function.cpp:137
// RVA: 0x000C3F4B
// ADDRESS: 004c3f4b
// PROTOTYPE: undefined Catch@004c3f4b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
