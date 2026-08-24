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
//! Полный expression evaluator и остальные function ID ниже пока остаются RAW.

use crate::gameserver::appserver::country::country::CountryScalarMutationReport;
use crate::gameserver::appserver::session::cequipmentdakong::EquipmentDaKongExternalRefreshReport;
use crate::gameserver::appserver::session::csessionfactory::EquipmentSessionPlugKind;
use crate::gameserver::gameserver::game::{
    CGame, EquipmentDaKongContext, EquipmentSessionOpenContext, EquipmentSessionOpenReport,
};
use crate::nets::netserver::message::SendMessageError;

pub(crate) const SCRIPT_FUNCTION_REFLUSH_EXTERN_PROPERTY: i32 = 9351;
pub(crate) const SCRIPT_FUNCTION_OPEN_DA_KONG: i32 = 9350;
pub(crate) const SCRIPT_FUNCTION_OPEN_EQUIPMENT_COMPOSE: i32 = 9354;
pub(crate) const SCRIPT_FUNCTION_OPEN_EQUIPMENT_UPGRADE: i32 = 2216;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_POWER: i32 = 9001;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_TECH_LEVEL: i32 = 9003;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_TREASURY: i32 = 9009;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_MATERIAL: i32 = 9011;
pub(crate) const SCRIPT_FUNCTION_SET_COUNTRY_TECH: i32 = 9013;
pub(crate) const SCRIPT_FUNCTION_GET_QUEST_SWITCH: i32 = 9018;
pub(crate) const SCRIPT_FUNCTION_SET_QUEST_SWITCH: i32 = 9019;
const SCRIPT_INT_PARAMETER_ERROR: i32 = 0x09ff_fff9;

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
