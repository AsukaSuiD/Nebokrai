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
//! Соседняя read-only family `9101..9104/9106..9109/9111` одним dispatcher-ом
//! читает phase flags, declaration membership, country-region win symbols,
//! ordered war region/camp/opponent и сохранённый country result.
//! Action family `9105/9110` сохраняет war/distance/argument ordering, вызывает
//! concrete `ServerCountryRegion::OnEnterContend` с миллисекундным duration,
//! публикует player state `0xBFF28`, timer `0xBFF29`, localized result и
//! отправляет byte-narrowed победу World сообщением `0x60318`.
//! Nation-war family `9304..9313/9315/9316` связывает current Nation region,
//! timing/carriage/contender player-effects, FourNation seconds/time/morale,
//! weak-state и World signup `0x6031B`; debug `GS1053..GS1056` остаётся в
//! точном порядке вокруг соответствующих concrete вызовов.
//! Battle-fairy family `9400..9411` сохраняет mixed string/integer parameter
//! routing, name/current-player lookup, skill/equipment mutation, reset RNG,
//! revive, experience/recreate RNG, old-client `0xBF918`, properties wire и
//! локальные журналы `BattleFairy`.
//! `2249 / FairyExpUp` разрешает enhancement-shadow обратно в live packet или
//! equipment goods, сохраняет grow-log, replacement ownership и concrete
//! delete/new-object wire, включая необратимый late packet-add failure.
//! Предметная family `2231..2236/2243..2246` получает GUID запускающего goods
//! из того же CScript instance, ограничивает used-item lookup настоящей
//! сумкой, изменяет addon/durability storage и публикует delete/amount/update
//! wire; selected durability разрешается через live enhancement-shadow.
//! PreciousBox `2221/2222/2237` сохраняет trusted action-script у player,
//! client open/result/close wire, общий Game RNG, configuration roll,
//! goods factory/upgrade/packet ownership и optional World announcement;
//! повторный запуск приходит из живого goods opcode `0x8FC12`.
//! Numeric selector получает вычисленные параметры из reached synchronous
//! `CScript`; остальные function ID и асинхронный dialog/wait lifecycle ниже
//! пока остаются RAW.

use crate::gameserver::appserver::country::country::{
    CountryExileRestTimeReport, CountryScalarMutationReport,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::servercountryregion::{
    CountryContendEntryContext, CountryContendPlayer, CountryNullPlayerCancelBlock,
};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::session::cequipmentdakong::EquipmentDaKongExternalRefreshReport;
use crate::gameserver::appserver::session::csessionfactory::EquipmentSessionPlugKind;
use crate::gameserver::appserver::shape::{ShapeCoordinateBlock, ShapeIdentity, ShapeResolver};
use crate::gameserver::gameserver::game::{
    BattleFairyDeathContext, BattleFairyScriptAction, BattleFairySkillResetContext, CGame,
    EquipmentDaKongContext, EquipmentSessionOpenContext, EquipmentSessionOpenReport,
    GameContainerMessageRuntime, NationCarriageReturnReport, NationCombatContext,
    NationContendEnterReport, ServerRegionOwner, colored_player_notice_message,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::public::guid::CGuid;

pub(crate) const SCRIPT_FUNCTION_REFLUSH_EXTERN_PROPERTY: i32 = 9351;
pub(crate) const SCRIPT_FUNCTION_OPEN_DA_KONG: i32 = 9350;
pub(crate) const SCRIPT_FUNCTION_OPEN_EQUIPMENT_COMPOSE: i32 = 9354;
pub(crate) const SCRIPT_FUNCTION_OPEN_EQUIPMENT_UPGRADE: i32 = 2216;
pub(crate) const SCRIPT_FUNCTION_OPEN_PRECIOUS_BOX: i32 = 2221;
pub(crate) const SCRIPT_FUNCTION_GET_PRECIOUS_ITEM: i32 = 2222;
pub(crate) const SCRIPT_FUNCTION_DELETE_USED_GOODS: i32 = 2231;
pub(crate) const SCRIPT_FUNCTION_CHECK_USED_GOODS: i32 = 2232;
pub(crate) const SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_1: i32 = 2233;
pub(crate) const SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_2: i32 = 2234;
pub(crate) const SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_1: i32 = 2235;
pub(crate) const SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_2: i32 = 2236;
pub(crate) const SCRIPT_FUNCTION_CLOSE_PRECIOUS_BOX: i32 = 2237;
pub(crate) const SCRIPT_FUNCTION_GET_CURRENT_DURABILITY: i32 = 2243;
pub(crate) const SCRIPT_FUNCTION_SET_CURRENT_DURABILITY: i32 = 2244;
pub(crate) const SCRIPT_FUNCTION_GET_SELECTED_DURABILITY: i32 = 2245;
pub(crate) const SCRIPT_FUNCTION_SET_SELECTED_DURABILITY: i32 = 2246;
pub(crate) const SCRIPT_FUNCTION_FAIRY_EXP_UP: i32 = 2249;
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
pub(crate) const SCRIPT_FUNCTION_IS_COUNTRY_WAR_DECLARE: i32 = 9101;
pub(crate) const SCRIPT_FUNCTION_IS_COUNTRY_DECLARED: i32 = 9102;
pub(crate) const SCRIPT_FUNCTION_IS_COUNTRY_WAR_PREPARE: i32 = 9103;
pub(crate) const SCRIPT_FUNCTION_IS_COUNTRY_WAR: i32 = 9104;
pub(crate) const SCRIPT_FUNCTION_ENTER_COUNTRY_CONTEND: i32 = 9105;
pub(crate) const SCRIPT_FUNCTION_IS_COUNTRY_WIN_SYMBOL: i32 = 9106;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_WAR_REGION: i32 = 9107;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_WAR_CAMP: i32 = 9108;
pub(crate) const SCRIPT_FUNCTION_GET_OTHER_WAR_COUNTRY: i32 = 9109;
pub(crate) const SCRIPT_FUNCTION_COUNTRY_WAR_VICTORY: i32 = 9110;
pub(crate) const SCRIPT_FUNCTION_GET_COUNTRY_WAR_RESULT: i32 = 9111;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_SEND_PLAYER_ID: i32 = 9304;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_CARRIAGE_BACK_TOWN: i32 = 9305;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_GET_FLAG_STATUS: i32 = 9306;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_GET_TIME: i32 = 9307;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_ENTER_CONTEND: i32 = 9308;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_COUNTRY_SIGN_UP: i32 = 9309;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_CLEAR_PLAYER_TIME: i32 = 9310;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_GET_NATION_STATUS: i32 = 9311;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_SET_PLAYER_TIME: i32 = 9312;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_IS_PLAYER_WEAK: i32 = 9313;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_GET_MORALE: i32 = 9315;
pub(crate) const SCRIPT_FUNCTION_NATION_WAR_CLEAR_MORALE: i32 = 9316;
pub(crate) const SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_SKILL: i32 = 9400;
pub(crate) const SCRIPT_FUNCTION_GET_FETCH_POWER: i32 = 9401;
pub(crate) const SCRIPT_FUNCTION_SET_BATTLE_FAIRY_ATTRIBUTE: i32 = 9402;
pub(crate) const SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SKILL: i32 = 9403;
pub(crate) const SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SPECIAL_SKILL: i32 = 9404;
pub(crate) const SCRIPT_FUNCTION_REVIVE_BATTLE_FAIRY: i32 = 9406;
pub(crate) const SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_ID: i32 = 9407;
pub(crate) const SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_LEVEL: i32 = 9408;
pub(crate) const SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_EXPERIENCE: i32 = 9409;
pub(crate) const SCRIPT_FUNCTION_GET_BATTLE_FAIRY_ATTRIBUTE: i32 = 9410;
pub(crate) const SCRIPT_FUNCTION_RECREATE_BATTLE_FAIRY_ATTRIBUTES: i32 = 9411;
const SCRIPT_INT_PARAMETER_ERROR: i32 = 0x09ff_fff9;
const SCRIPT_PLAYER_TYPE: i32 = 400;
const SCRIPT_NPC_TYPE: i32 = 500;

pub(crate) trait CountryWarActionScriptRuntime {
    /// Exact lower DWORD process tick, sampled only when a contender is added.
    fn country_contend_now_milliseconds(&mut self) -> u32;
}

pub(crate) trait ScriptFunctionRuntime:
    CountryWarActionScriptRuntime
    + CountryExileTimeScriptContext
    + NationCombatContext
    + EquipmentSessionOpenContext
    + EquipmentDaKongContext
    + GameContainerMessageRuntime
    + BattleFairyDeathContext
    + BattleFairySkillResetContext
{
}

impl<T> ScriptFunctionRuntime for T where
    T: CountryWarActionScriptRuntime
        + CountryExileTimeScriptContext
        + NationCombatContext
        + EquipmentSessionOpenContext
        + EquipmentDaKongContext
        + GameContainerMessageRuntime
        + BattleFairyDeathContext
        + BattleFairySkillResetContext
{
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarActionKind {
    EnterContend,
    PublishVictory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarContendEffect {
    PlayerState {
        player_id: i32,
        state: bool,
        changed: bool,
        around_delivery: Option<Result<i32, ShapeCoordinateBlock>>,
    },
    Time {
        player_id: i32,
        percentage: i32,
        delivery: i32,
    },
    Notice {
        player_id: i32,
        string_id: &'static str,
        delivery: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarActionScriptDisposition {
    CallerMissing,
    CallerShapeMissing,
    WarClosed {
        delivery: i32,
    },
    TooFar {
        distance: i32,
        maximum: i32,
        delivery: i32,
    },
    ArgumentMissing {
        argument: usize,
    },
    RegionMissing {
        region_id: Option<i32>,
    },
    RegionNotCountry {
        region_id: i32,
    },
    NpcMissing {
        region_id: i32,
        npc_id: i32,
    },
    ContendInvoked {
        region_id: i32,
        player_id: i32,
        symbol_id: i32,
        duration_ms: i32,
        result: Result<(), CountryNullPlayerCancelBlock>,
        effects: Vec<CountryWarContendEffect>,
    },
    VictoryNotPublished,
    VictoryPublished {
        country: u8,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarActionScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        function_id: i32,
        kind: CountryWarActionKind,
        legacy_return: i32,
        disposition: CountryWarActionScriptDisposition,
    },
}

struct GameCountryContendEntryContext<'a, Runtime> {
    game: &'a mut CGame,
    runtime: &'a mut Runtime,
    region: CServerRegion,
    effects: Vec<CountryWarContendEffect>,
}

impl<Runtime: CountryWarActionScriptRuntime> CountryContendEntryContext
    for GameCountryContendEntryContext<'_, Runtime>
{
    fn now_millis(&mut self) -> u32 {
        self.runtime.country_contend_now_milliseconds()
    }

    fn send_contend_time(&mut self, player_id: i32, time: i32) {
        let mut message = CMessage::new(0x000b_ff29);
        message.base_mut().add_long(time);
        let delivery = message.send_to_player(self.game.net_server(), player_id);
        self.effects.push(CountryWarContendEffect::Time {
            player_id,
            percentage: time,
            delivery,
        });
    }

    fn set_known_player_contend_state(&mut self, player_id: i32, state: bool) {
        let around_delivery =
            self.game
                .publish_country_player_contend_state(&self.region, player_id, state);
        self.effects.push(CountryWarContendEffect::PlayerState {
            player_id,
            state,
            changed: around_delivery.is_some(),
            around_delivery,
        });
    }

    fn notify_player(&mut self, player_id: i32, string_id: &'static str) {
        let delivery = send_country_war_script_notice(self.game, player_id, string_id.as_bytes());
        self.effects.push(CountryWarContendEffect::Notice {
            player_id,
            string_id,
            delivery,
        });
    }
}

pub(crate) fn run_country_war_action_script_function<Runtime: CountryWarActionScriptRuntime>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
    script_region_id: Option<i32>,
    function_id: i32,
    evaluated_arguments: [Option<i32>; 2],
) -> CountryWarActionScriptFunctionOutcome {
    if function_id == SCRIPT_FUNCTION_COUNTRY_WAR_VICTORY {
        let country = evaluated_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
        if country == SCRIPT_INT_PARAMETER_ERROR {
            return country_war_action_handled(
                function_id,
                CountryWarActionKind::PublishVictory,
                -1,
                CountryWarActionScriptDisposition::ArgumentMissing { argument: 0 },
            );
        }
        if country == 0 {
            return country_war_action_handled(
                function_id,
                CountryWarActionKind::PublishVictory,
                0,
                CountryWarActionScriptDisposition::VictoryNotPublished,
            );
        }
        let country = country as u8;
        let mut request = CMessage::new(0x0006_0318);
        request.base_mut().add_byte(country);
        let delivery = request.send(game, false);
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::PublishVictory,
            0,
            CountryWarActionScriptDisposition::VictoryPublished { country, delivery },
        );
    }
    if function_id != SCRIPT_FUNCTION_ENTER_COUNTRY_CONTEND {
        return CountryWarActionScriptFunctionOutcome::DifferentFunction;
    }

    let (Some(player_id), Some(npc_id)) = (script_player_id, script_npc_id) else {
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::CallerMissing,
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
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::CallerShapeMissing,
        );
    };
    if !game.country_war_sys().state_war {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0219");
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::WarClosed { delivery },
        );
    }
    let distance = npc_shape.distance(player_shape);
    if distance > 2 {
        let delivery = send_country_war_script_notice(game, player_id, b"GS0208");
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::TooFar {
                distance,
                maximum: 2,
                delivery,
            },
        );
    }
    let symbol_id = evaluated_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    if symbol_id == SCRIPT_INT_PARAMETER_ERROR {
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::ArgumentMissing { argument: 0 },
        );
    }
    let duration = evaluated_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    if duration == SCRIPT_INT_PARAMETER_ERROR {
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::ArgumentMissing { argument: 1 },
        );
    }
    let Some(region_id) = script_region_id else {
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::RegionMissing { region_id: None },
        );
    };
    let Some(owner) = game.take_region_owner(region_id) else {
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::RegionMissing {
                region_id: Some(region_id),
            },
        );
    };
    let ServerRegionOwner::Country(mut region) = owner else {
        game.restore_region_owner(owner);
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::RegionNotCountry { region_id },
        );
    };
    let Some(symbol_name) = region
        .base
        .find_npc_by_id(npc_id)
        .map(|npc| String::from_utf8_lossy(npc.name()).into_owned())
    else {
        game.restore_region_owner(ServerRegionOwner::Country(region));
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::NpcMissing { region_id, npc_id },
        );
    };
    let Some(player) = game
        .find_player(player_id)
        .map(|player| CountryContendPlayer {
            player_id,
            faction_id: player.faction_id(),
            country: player.country(),
            shape_type: player_shape.identity.object_type,
            is_dead: player.is_dead(),
        })
    else {
        game.restore_region_owner(ServerRegionOwner::Country(region));
        return country_war_action_handled(
            function_id,
            CountryWarActionKind::EnterContend,
            0,
            CountryWarActionScriptDisposition::CallerMissing,
        );
    };
    let duration_ms = duration.wrapping_mul(1_000);
    let region_projection = region.base.clone();
    let mut context = GameCountryContendEntryContext {
        game,
        runtime,
        region: region_projection,
        effects: Vec::new(),
    };
    let result = region.on_enter_contend(
        Some(&player),
        symbol_id,
        &symbol_name,
        duration_ms,
        &mut context,
    );
    let effects = std::mem::take(&mut context.effects);
    drop(context);
    game.restore_region_owner(ServerRegionOwner::Country(region));
    country_war_action_handled(
        function_id,
        CountryWarActionKind::EnterContend,
        0,
        CountryWarActionScriptDisposition::ContendInvoked {
            region_id,
            player_id,
            symbol_id,
            duration_ms,
            result,
            effects,
        },
    )
}

fn country_war_action_handled(
    function_id: i32,
    kind: CountryWarActionKind,
    legacy_return: i32,
    disposition: CountryWarActionScriptDisposition,
) -> CountryWarActionScriptFunctionOutcome {
    CountryWarActionScriptFunctionOutcome::Handled {
        function_id,
        kind,
        legacy_return,
        disposition,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NationWarScriptKind {
    SendPlayerId,
    CarriageBackTown,
    GetFlagStatus,
    GetTime,
    EnterContend,
    CountrySignUp,
    ClearPlayerTime,
    GetNationStatus,
    SetPlayerTime,
    IsPlayerWeak,
    GetMorale,
    ClearMorale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NationWarScriptDisposition {
    ArgumentMissing {
        argument: usize,
    },
    PlayerMissing,
    TimingStart {
        target_player_id: i32,
        started: bool,
    },
    CarriageBackTown {
        enter_debug: Vec<u8>,
        report: Option<NationCarriageReturnReport>,
        completed_debug: Option<Vec<u8>>,
    },
    Scalar {
        region_id: Option<i32>,
        player_id: Option<i32>,
        value: i32,
    },
    PlayerWarTime {
        player_id: i32,
        enter_debug: Vec<u8>,
        value_debug: Vec<u8>,
        value: i32,
    },
    Contend {
        player_id: i32,
        duration_ms: u32,
        report: Option<NationContendEnterReport>,
    },
    SignUpSkipped,
    SignUpRequested {
        country: i32,
        delivery: Result<i32, SendMessageError>,
    },
    PlayerTimeCleared {
        player_id: i32,
        previous_ms: Option<u32>,
    },
    PlayerTimeSet {
        player_id: i32,
        time_ms: u32,
        previous_ms: Option<u32>,
    },
    MoraleCleared {
        previous: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NationWarScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        function_id: i32,
        kind: NationWarScriptKind,
        legacy_return: i32,
        disposition: NationWarScriptDisposition,
    },
}

pub(crate) fn run_nation_war_script_function<Runtime: NationCombatContext>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    function_id: i32,
    evaluated_arguments: [Option<i32>; 2],
) -> NationWarScriptFunctionOutcome {
    let kind = match function_id {
        SCRIPT_FUNCTION_NATION_WAR_SEND_PLAYER_ID => NationWarScriptKind::SendPlayerId,
        SCRIPT_FUNCTION_NATION_WAR_CARRIAGE_BACK_TOWN => NationWarScriptKind::CarriageBackTown,
        SCRIPT_FUNCTION_NATION_WAR_GET_FLAG_STATUS => NationWarScriptKind::GetFlagStatus,
        SCRIPT_FUNCTION_NATION_WAR_GET_TIME => NationWarScriptKind::GetTime,
        SCRIPT_FUNCTION_NATION_WAR_ENTER_CONTEND => NationWarScriptKind::EnterContend,
        SCRIPT_FUNCTION_NATION_WAR_COUNTRY_SIGN_UP => NationWarScriptKind::CountrySignUp,
        SCRIPT_FUNCTION_NATION_WAR_CLEAR_PLAYER_TIME => NationWarScriptKind::ClearPlayerTime,
        SCRIPT_FUNCTION_NATION_WAR_GET_NATION_STATUS => NationWarScriptKind::GetNationStatus,
        SCRIPT_FUNCTION_NATION_WAR_SET_PLAYER_TIME => NationWarScriptKind::SetPlayerTime,
        SCRIPT_FUNCTION_NATION_WAR_IS_PLAYER_WEAK => NationWarScriptKind::IsPlayerWeak,
        SCRIPT_FUNCTION_NATION_WAR_GET_MORALE => NationWarScriptKind::GetMorale,
        SCRIPT_FUNCTION_NATION_WAR_CLEAR_MORALE => NationWarScriptKind::ClearMorale,
        _ => return NationWarScriptFunctionOutcome::DifferentFunction,
    };
    let argument = |index: usize| {
        evaluated_arguments[index]
            .filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
            .ok_or(NationWarScriptDisposition::ArgumentMissing { argument: index })
    };

    match kind {
        NationWarScriptKind::SendPlayerId => {
            let target_player_id = match argument(0) {
                Ok(value) => value,
                Err(disposition) => {
                    return nation_war_script_handled(function_id, kind, 0, disposition);
                }
            };
            let Some(script_player_id) = script_player_id else {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::PlayerMissing,
                );
            };
            let started =
                game.script_nation_war_send_player_id(script_player_id, target_player_id, || {
                    runtime.now_milliseconds()
                });
            nation_war_script_handled(
                function_id,
                kind,
                0,
                NationWarScriptDisposition::TimingStart {
                    target_player_id,
                    started,
                },
            )
        }
        NationWarScriptKind::CarriageBackTown => {
            let enter_debug = nation_war_script_debug(game, runtime, b"GS1053");
            let Some((region_id, country)) = script_player_id.and_then(|player_id| {
                let player = game.find_player(player_id)?;
                Some((player.server_region_id()?, i32::from(player.country())))
            }) else {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::CarriageBackTown {
                        enter_debug,
                        report: None,
                        completed_debug: None,
                    },
                );
            };
            let report = game.script_nation_carriage_back_town(region_id, country);
            let completed_debug = report
                .is_some()
                .then(|| nation_war_script_debug(game, runtime, b"GS1054"));
            nation_war_script_handled(
                function_id,
                kind,
                0,
                NationWarScriptDisposition::CarriageBackTown {
                    enter_debug,
                    report,
                    completed_debug,
                },
            )
        }
        NationWarScriptKind::GetFlagStatus => {
            let region_id = nation_script_player_region_id(game, script_player_id);
            let value = region_id
                .and_then(|region_id| match game.find_region(region_id) {
                    Some(ServerRegionOwner::Nation(region)) => Some(region.flag_belong_to_id()),
                    _ => None,
                })
                .unwrap_or(0);
            nation_war_script_handled(
                function_id,
                kind,
                value,
                NationWarScriptDisposition::Scalar {
                    region_id,
                    player_id: script_player_id,
                    value,
                },
            )
        }
        NationWarScriptKind::GetTime => {
            let enter_debug = nation_war_script_debug(game, runtime, b"GS1055");
            let Some(player_id) = script_player_id else {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::PlayerMissing,
                );
            };
            let debug_time = game
                .four_nation_war_sys()
                .player_war_time_seconds(player_id) as i32;
            let Some(player_name) = game
                .find_player(player_id)
                .map(|player| player.player_name().to_vec())
            else {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::PlayerMissing,
                );
            };
            let value_debug = format_nation_war_time_debug(
                game.get_string_by_id(b"GS1056"),
                &player_name,
                debug_time,
            );
            runtime.put_debug_string(&value_debug);
            let value = game
                .four_nation_war_sys()
                .player_war_time_seconds(player_id) as i32;
            nation_war_script_handled(
                function_id,
                kind,
                value,
                NationWarScriptDisposition::PlayerWarTime {
                    player_id,
                    enter_debug,
                    value_debug,
                    value,
                },
            )
        }
        NationWarScriptKind::EnterContend => {
            let seconds = match argument(0) {
                Ok(value) => value,
                Err(disposition) => {
                    return nation_war_script_handled(function_id, kind, 0, disposition);
                }
            };
            let Some(player_id) = script_player_id else {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::PlayerMissing,
                );
            };
            if game.find_player(player_id).is_none() {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::PlayerMissing,
                );
            }
            let duration_ms = (seconds as u32).wrapping_mul(1_000);
            let report =
                nation_script_player_region_id(game, Some(player_id)).and_then(|region_id| {
                    game.nation_enter_contend(region_id, player_id, duration_ms, runtime)
                });
            nation_war_script_handled(
                function_id,
                kind,
                0,
                NationWarScriptDisposition::Contend {
                    player_id,
                    duration_ms,
                    report,
                },
            )
        }
        NationWarScriptKind::CountrySignUp => {
            let operation = match argument(0) {
                Ok(value) => value,
                Err(disposition) => {
                    return nation_war_script_handled(function_id, kind, 0, disposition);
                }
            };
            if operation == 0 {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::SignUpSkipped,
                );
            }
            let Some(country) = script_player_id
                .and_then(|player_id| game.find_player(player_id))
                .map(|player| i32::from(player.country()))
            else {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::PlayerMissing,
                );
            };
            let mut request = CMessage::new(0x0006_031b);
            request.base_mut().add_long(country);
            let delivery = request.send(game, false);
            let legacy_return = match delivery {
                Ok(value) => value,
                Err(_) => 0,
            };
            nation_war_script_handled(
                function_id,
                kind,
                legacy_return,
                NationWarScriptDisposition::SignUpRequested { country, delivery },
            )
        }
        NationWarScriptKind::ClearPlayerTime => {
            let player_id = match argument(0) {
                Ok(value) => value,
                Err(disposition) => {
                    return nation_war_script_handled(function_id, kind, 0, disposition);
                }
            };
            let previous_ms = game
                .four_nation_war_sys_mut()
                .clear_one_player_war_time(player_id);
            nation_war_script_handled(
                function_id,
                kind,
                0,
                NationWarScriptDisposition::PlayerTimeCleared {
                    player_id,
                    previous_ms,
                },
            )
        }
        NationWarScriptKind::GetNationStatus => {
            let region_id = nation_script_player_region_id(game, script_player_id);
            let country = script_player_id
                .and_then(|player_id| game.find_player(player_id))
                .map(|player| i32::from(player.country()));
            let value = match (region_id, country) {
                (Some(region_id), Some(country)) => match game.find_region(region_id) {
                    Some(ServerRegionOwner::Nation(region)) if region.is_nation_fail(country) => 1,
                    _ => 0,
                },
                _ => 0,
            };
            nation_war_script_handled(
                function_id,
                kind,
                value,
                NationWarScriptDisposition::Scalar {
                    region_id,
                    player_id: script_player_id,
                    value,
                },
            )
        }
        NationWarScriptKind::SetPlayerTime => {
            // Exact dispatcher вычисляет оба expression до любой sentinel-проверки.
            let player_id = evaluated_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let time_ms = evaluated_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            if player_id == SCRIPT_INT_PARAMETER_ERROR {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::ArgumentMissing { argument: 0 },
                );
            }
            if time_ms == SCRIPT_INT_PARAMETER_ERROR {
                return nation_war_script_handled(
                    function_id,
                    kind,
                    0,
                    NationWarScriptDisposition::ArgumentMissing { argument: 1 },
                );
            }
            let time_ms = time_ms as u32;
            let previous_ms = game
                .four_nation_war_sys_mut()
                .set_one_player_war_time(player_id, time_ms);
            nation_war_script_handled(
                function_id,
                kind,
                0,
                NationWarScriptDisposition::PlayerTimeSet {
                    player_id,
                    time_ms,
                    previous_ms,
                },
            )
        }
        NationWarScriptKind::IsPlayerWeak => {
            let value = script_player_id
                .and_then(|player_id| game.find_player(player_id))
                .map(|player| i32::from(player.is_nation_war_player_weak()))
                .unwrap_or(0);
            nation_war_script_handled(
                function_id,
                kind,
                value,
                NationWarScriptDisposition::Scalar {
                    region_id: None,
                    player_id: script_player_id,
                    value,
                },
            )
        }
        NationWarScriptKind::GetMorale => {
            let value = game.four_nation_war_sys().morale();
            nation_war_script_handled(
                function_id,
                kind,
                value,
                NationWarScriptDisposition::Scalar {
                    region_id: None,
                    player_id: None,
                    value,
                },
            )
        }
        NationWarScriptKind::ClearMorale => {
            let previous = game.four_nation_war_sys_mut().clear_morale_value();
            nation_war_script_handled(
                function_id,
                kind,
                0,
                NationWarScriptDisposition::MoraleCleared { previous },
            )
        }
    }
}

fn nation_script_player_region_id(game: &CGame, player_id: Option<i32>) -> Option<i32> {
    player_id
        .and_then(|player_id| game.find_player(player_id))
        .and_then(|player| player.server_region_id())
}

fn nation_war_script_debug<Runtime: NationCombatContext>(
    game: &CGame,
    runtime: &mut Runtime,
    string_id: &[u8],
) -> Vec<u8> {
    let text = game.get_string_by_id(string_id).to_vec();
    runtime.put_debug_string(&text);
    text
}

fn format_nation_war_time_debug(template: &[u8], player_name: &[u8], time: i32) -> Vec<u8> {
    enum Argument<'a> {
        Bytes(&'a [u8]),
        Signed(i32),
    }
    let arguments = [Argument::Bytes(player_name), Argument::Signed(time)];
    let template = template.split(|byte| *byte == 0).next().unwrap_or_default();
    let mut output = Vec::new();
    let mut argument = 0usize;
    let mut offset = 0usize;
    // Safe replacement for zeroed `char[64] + _snprintf(..., 64, ...)`:
    // one byte remains reserved for the terminator instead of reproducing
    // the legacy unterminated-buffer UB on a fully truncated result.
    while offset < template.len() && output.len() < 63 {
        if template[offset] != b'%' {
            output.push(template[offset]);
            offset += 1;
            continue;
        }
        if template.get(offset + 1) == Some(&b'%') {
            output.push(b'%');
            offset += 2;
            continue;
        }
        let rendered = match (template.get(offset + 1).copied(), arguments.get(argument)) {
            (Some(b's'), Some(Argument::Bytes(value))) => Some(
                value
                    .split(|byte| *byte == 0)
                    .next()
                    .unwrap_or_default()
                    .to_vec(),
            ),
            (Some(b'd' | b'i'), Some(Argument::Signed(value))) => {
                Some(value.to_string().into_bytes())
            }
            _ => None,
        };
        let Some(rendered) = rendered else {
            output.push(b'%');
            offset += 1;
            continue;
        };
        let remaining = 63usize.saturating_sub(output.len());
        output.extend_from_slice(&rendered[..rendered.len().min(remaining)]);
        argument += 1;
        offset += 2;
    }
    output
}

fn nation_war_script_handled(
    function_id: i32,
    kind: NationWarScriptKind,
    legacy_return: i32,
    disposition: NationWarScriptDisposition,
) -> NationWarScriptFunctionOutcome {
    NationWarScriptFunctionOutcome::Handled {
        function_id,
        kind,
        legacy_return,
        disposition,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarQueryKind {
    DeclarationOpen,
    CountryDeclared,
    PreparationOpen,
    WarOpen,
    WinSymbol,
    Region,
    Camp,
    OtherCountry,
    Result,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarQueryScriptDisposition {
    CallerMissing,
    CallerShapeMissing,
    TooFar {
        distance: i32,
        maximum: i32,
    },
    ArgumentMissing {
        argument: usize,
    },
    RegionMissing {
        region_id: i32,
    },
    CountryMissing {
        country: u8,
    },
    Completed {
        kind: CountryWarQueryKind,
        country: Option<i32>,
        value: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarQueryScriptFunctionOutcome {
    DifferentFunction,
    Handled {
        function_id: i32,
        legacy_return: i32,
        disposition: CountryWarQueryScriptDisposition,
    },
}

pub(crate) fn run_country_war_query_script_function(
    game: &CGame,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
    function_id: i32,
    evaluated_arguments: [Option<i32>; 3],
) -> CountryWarQueryScriptFunctionOutcome {
    let kind = match function_id {
        SCRIPT_FUNCTION_IS_COUNTRY_WAR_DECLARE => CountryWarQueryKind::DeclarationOpen,
        SCRIPT_FUNCTION_IS_COUNTRY_DECLARED => CountryWarQueryKind::CountryDeclared,
        SCRIPT_FUNCTION_IS_COUNTRY_WAR_PREPARE => CountryWarQueryKind::PreparationOpen,
        SCRIPT_FUNCTION_IS_COUNTRY_WAR => CountryWarQueryKind::WarOpen,
        SCRIPT_FUNCTION_IS_COUNTRY_WIN_SYMBOL => CountryWarQueryKind::WinSymbol,
        SCRIPT_FUNCTION_GET_COUNTRY_WAR_REGION => CountryWarQueryKind::Region,
        SCRIPT_FUNCTION_GET_COUNTRY_WAR_CAMP => CountryWarQueryKind::Camp,
        SCRIPT_FUNCTION_GET_OTHER_WAR_COUNTRY => CountryWarQueryKind::OtherCountry,
        SCRIPT_FUNCTION_GET_COUNTRY_WAR_RESULT => CountryWarQueryKind::Result,
        _ => return CountryWarQueryScriptFunctionOutcome::DifferentFunction,
    };
    if matches!(
        kind,
        CountryWarQueryKind::DeclarationOpen
            | CountryWarQueryKind::CountryDeclared
            | CountryWarQueryKind::PreparationOpen
            | CountryWarQueryKind::WarOpen
    ) {
        let gate = country_war_script_caller_gate(game, script_player_id, script_npc_id);
        if let Err(disposition) = gate {
            return country_war_query_handled(function_id, 0, disposition);
        }
    }
    let default_country = || {
        script_player_id
            .and_then(|player_id| game.find_player(player_id))
            .map(|player| i32::from(player.country()))
    };
    let optional_country = || {
        let raw = evaluated_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
        if raw == SCRIPT_INT_PARAMETER_ERROR {
            default_country()
        } else {
            Some(raw)
        }
    };
    match kind {
        CountryWarQueryKind::DeclarationOpen => country_war_query_completed(
            function_id,
            kind,
            None,
            i32::from(game.country_war_sys().state_declare),
        ),
        CountryWarQueryKind::CountryDeclared => {
            let Some(country) = optional_country() else {
                return country_war_query_handled(
                    function_id,
                    0,
                    CountryWarQueryScriptDisposition::CallerMissing,
                );
            };
            country_war_query_completed(
                function_id,
                kind,
                Some(country),
                i32::from(game.country_war_sys().is_already_declar(country)),
            )
        }
        CountryWarQueryKind::PreparationOpen => country_war_query_completed(
            function_id,
            kind,
            None,
            i32::from(game.country_war_sys().state_prepare),
        ),
        CountryWarQueryKind::WarOpen => country_war_query_completed(
            function_id,
            kind,
            None,
            i32::from(game.country_war_sys().state_war),
        ),
        CountryWarQueryKind::WinSymbol => {
            let mut values = [0; 3];
            for (index, argument) in evaluated_arguments.into_iter().enumerate() {
                let Some(value) = argument.filter(|value| *value != SCRIPT_INT_PARAMETER_ERROR)
                else {
                    return country_war_query_handled(
                        function_id,
                        0,
                        CountryWarQueryScriptDisposition::ArgumentMissing { argument: index },
                    );
                };
                values[index] = value;
            }
            let Some(ServerRegionOwner::Country(region)) = game.find_region(values[0]) else {
                return country_war_query_handled(
                    function_id,
                    0,
                    CountryWarQueryScriptDisposition::RegionMissing {
                        region_id: values[0],
                    },
                );
            };
            country_war_query_completed(
                function_id,
                kind,
                None,
                i32::from(region.is_win_symbol(values[1], values[2])),
            )
        }
        CountryWarQueryKind::Region
        | CountryWarQueryKind::Camp
        | CountryWarQueryKind::OtherCountry => {
            let Some(country) = optional_country() else {
                return country_war_query_handled(
                    function_id,
                    0,
                    CountryWarQueryScriptDisposition::CallerMissing,
                );
            };
            let value = match kind {
                CountryWarQueryKind::Region => {
                    game.country_war_sys().get_war_region_for_country(country)
                }
                CountryWarQueryKind::Camp => game.country_war_sys().get_war_camp(country),
                CountryWarQueryKind::OtherCountry => {
                    game.country_war_sys().get_other_country(country)
                }
                _ => unreachable!(),
            };
            country_war_query_completed(function_id, kind, Some(country), value)
        }
        CountryWarQueryKind::Result => {
            let Some(country) = optional_country() else {
                return country_war_query_handled(
                    function_id,
                    0,
                    CountryWarQueryScriptDisposition::CallerMissing,
                );
            };
            let country = country as u8;
            let Some(owner) = game.country_handler().country(country) else {
                return country_war_query_handled(
                    function_id,
                    0,
                    CountryWarQueryScriptDisposition::CountryMissing { country },
                );
            };
            country_war_query_completed(
                function_id,
                kind,
                Some(i32::from(country)),
                owner.country_war_result,
            )
        }
    }
}

fn country_war_script_caller_gate(
    game: &CGame,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
) -> Result<(i32, i32), CountryWarQueryScriptDisposition> {
    let (Some(player_id), Some(npc_id)) = (script_player_id, script_npc_id) else {
        return Err(CountryWarQueryScriptDisposition::CallerMissing);
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
        return Err(CountryWarQueryScriptDisposition::CallerShapeMissing);
    };
    let distance = npc_shape.distance(player_shape);
    let maximum = game
        .globe_setup()
        .area_width()
        .max(game.globe_setup().area_height())
        / 2;
    if maximum < distance {
        return Err(CountryWarQueryScriptDisposition::TooFar { distance, maximum });
    }
    Ok((distance, maximum))
}

fn country_war_query_completed(
    function_id: i32,
    kind: CountryWarQueryKind,
    country: Option<i32>,
    value: i32,
) -> CountryWarQueryScriptFunctionOutcome {
    country_war_query_handled(
        function_id,
        value,
        CountryWarQueryScriptDisposition::Completed {
            kind,
            country,
            value,
        },
    )
}

fn country_war_query_handled(
    function_id: i32,
    legacy_return: i32,
    disposition: CountryWarQueryScriptDisposition,
) -> CountryWarQueryScriptFunctionOutcome {
    CountryWarQueryScriptFunctionOutcome::Handled {
        function_id,
        legacy_return,
        disposition,
    }
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScriptFunctionDispatchOutcome {
    DifferentFunction,
    Invalid,
    Handled { legacy_return: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScriptFunctionParameterKind {
    Integer,
    String,
    Unused,
}

/// `GetStringParam`/`GetIntParam` routing исторического dense owner-а.
/// CScript использует таблицу до вычисления выражения, сохраняя positional
/// string arguments вместо прежнего special-case только для 9351.
pub(crate) fn script_function_parameter_kind(
    function_id: i32,
    index: usize,
) -> ScriptFunctionParameterKind {
    use ScriptFunctionParameterKind::{Integer, String, Unused};
    match function_id {
        SCRIPT_FUNCTION_DELETE_USED_GOODS
        | SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_1
        | SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_2
        | SCRIPT_FUNCTION_SET_CURRENT_DURABILITY
        | SCRIPT_FUNCTION_SET_SELECTED_DURABILITY => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_1 | SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_2 => {
            match index {
                0 | 1 => Integer,
                _ => Unused,
            }
        }
        SCRIPT_FUNCTION_CHECK_USED_GOODS
        | SCRIPT_FUNCTION_GET_CURRENT_DURABILITY
        | SCRIPT_FUNCTION_GET_SELECTED_DURABILITY => Unused,
        SCRIPT_FUNCTION_FAIRY_EXP_UP => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_REFLUSH_EXTERN_PROPERTY => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_OPEN_PRECIOUS_BOX => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_PRECIOUS_ITEM => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_SKILL => match index {
            0 | 1 => String,
            2 | 3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_FETCH_POWER
        | SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SPECIAL_SKILL
        | SCRIPT_FUNCTION_REVIVE_BATTLE_FAIRY => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_BATTLE_FAIRY_ATTRIBUTE => match index {
            0 => String,
            1 | 2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SKILL
        | SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_ID
        | SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_LEVEL
        | SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_EXPERIENCE
        | SCRIPT_FUNCTION_GET_BATTLE_FAIRY_ATTRIBUTE => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_RECREATE_BATTLE_FAIRY_ATTRIBUTES => match index {
            0 => String,
            1..=3 => Integer,
            _ => Unused,
        },
        _ if index < 3 => Integer,
        _ => Unused,
    }
}

fn run_battle_fairy_script_function<Runtime: ScriptFunctionRuntime>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    function_id: i32,
    integer_arguments: [Option<i32>; 4],
    string_arguments: [Option<&[u8]>; 2],
) -> Option<i32> {
    let string = |index: usize| string_arguments[index].map(<[u8]>::to_vec);
    let integer = |index: usize| integer_arguments[index].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    let action = match function_id {
        SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_SKILL => {
            let (Some(player_name), Some(skill_name)) = (string(0), string(1)) else {
                return Some(-1);
            };
            BattleFairyScriptAction::AddSkill {
                player_name,
                skill_name,
                skill_level: integer(2),
                position: integer_arguments[3],
            }
        }
        SCRIPT_FUNCTION_GET_FETCH_POWER => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            BattleFairyScriptAction::GetFetchPower { player_name }
        }
        SCRIPT_FUNCTION_SET_BATTLE_FAIRY_ATTRIBUTE => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            let (attribute, value) = (integer(1), integer(2));
            if attribute == SCRIPT_INT_PARAMETER_ERROR || value == SCRIPT_INT_PARAMETER_ERROR {
                return Some(0);
            }
            BattleFairyScriptAction::SetAttribute {
                player_name,
                attribute,
                value,
            }
        }
        SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SKILL => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            let position = integer(1);
            if !(3..=5).contains(&position) {
                return Some(0);
            }
            BattleFairyScriptAction::ResetSkill {
                player_name,
                position,
            }
        }
        SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SPECIAL_SKILL => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            BattleFairyScriptAction::ResetSkill {
                player_name,
                position: 6,
            }
        }
        SCRIPT_FUNCTION_REVIVE_BATTLE_FAIRY => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            if script_player_id.is_none() {
                return Some(0);
            }
            BattleFairyScriptAction::Revive { player_name }
        }
        SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_ID
        | SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_LEVEL => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            let position = integer(1);
            let valid = if function_id == SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_ID {
                (3..=6).contains(&position)
            } else {
                (0..=6).contains(&position)
            };
            if !valid {
                return Some(0);
            }
            BattleFairyScriptAction::GetSkillValue {
                player_name,
                position,
                value_id: if function_id == SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_ID {
                    2
                } else {
                    1
                },
            }
        }
        SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_EXPERIENCE => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            let experience = integer(1);
            if experience <= 0 || experience == SCRIPT_INT_PARAMETER_ERROR {
                return Some(0);
            }
            BattleFairyScriptAction::AddExperience {
                player_name,
                experience,
            }
        }
        SCRIPT_FUNCTION_GET_BATTLE_FAIRY_ATTRIBUTE => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            let attribute = integer(1);
            if attribute == SCRIPT_INT_PARAMETER_ERROR {
                return Some(0);
            }
            BattleFairyScriptAction::GetAttribute {
                player_name,
                attribute,
            }
        }
        SCRIPT_FUNCTION_RECREATE_BATTLE_FAIRY_ATTRIBUTES => {
            let Some(player_name) = string(0) else {
                return Some(0);
            };
            let (mode, minimum, maximum) = (integer(1), integer(2), integer(3));
            if script_player_id.is_none()
                || !matches!(mode, 0 | 1)
                || minimum == SCRIPT_INT_PARAMETER_ERROR
                || maximum == SCRIPT_INT_PARAMETER_ERROR
            {
                return Some(0);
            }
            BattleFairyScriptAction::RecreateAttributes {
                player_name,
                mode,
                minimum,
                maximum,
            }
        }
        _ => return None,
    };
    Some(game.run_battle_fairy_script_action(script_player_id, action, runtime))
}

fn run_fairy_script_function<Runtime: ScriptFunctionRuntime>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    function_id: i32,
    evaluated_experience: Option<i32>,
) -> Option<i32> {
    if function_id != SCRIPT_FUNCTION_FAIRY_EXP_UP {
        return None;
    }
    let Some(player_id) = script_player_id else {
        return Some(0);
    };
    let experience = evaluated_experience.unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
    if experience == SCRIPT_INT_PARAMETER_ERROR {
        return Some(-1);
    }
    if experience <= 0 {
        return Some(0);
    }
    Some(i32::from(game.fairy_exp_up_selected_goods(
        player_id,
        experience as u32,
        runtime,
    )))
}

enum GoodsItemScriptFunctionOutcome {
    DifferentFunction,
    Invalid,
    Handled(i32),
}

fn script_used_goods(game: &CGame, player_id: i32, goods_id: CGuid) -> Option<&CGoods> {
    game.find_player(player_id)
        .and_then(|player| player.packet().base().find(goods_id))
}

fn script_selected_goods(game: &CGame, player_id: i32) -> Option<&CGoods> {
    let player = game.find_player(player_id)?;
    let goods_id = player.enhancement_selected_goods_id()?;
    player.get_goods_by_id(goods_id)
}

fn run_goods_item_script_function<Runtime: ScriptFunctionRuntime>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    used_item_id: Option<CGuid>,
    function_id: i32,
    argument_count: usize,
    integer_arguments: [Option<i32>; 2],
) -> GoodsItemScriptFunctionOutcome {
    let used_identity = || script_player_id.zip(used_item_id);
    match function_id {
        SCRIPT_FUNCTION_DELETE_USED_GOODS => {
            let requested = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            if argument_count != 1 || requested <= 0 || requested == SCRIPT_INT_PARAMETER_ERROR {
                return GoodsItemScriptFunctionOutcome::Handled(0);
            }
            let Some((player_id, goods_id)) = used_identity() else {
                return GoodsItemScriptFunctionOutcome::Handled(0);
            };
            if script_used_goods(game, player_id, goods_id).is_none() {
                return GoodsItemScriptFunctionOutcome::Handled(0);
            }
            let Some(consumption) = game
                .find_player_mut(player_id)
                .and_then(|player| player.remove_packet_goods_by_id(goods_id, requested as u32))
            else {
                return GoodsItemScriptFunctionOutcome::Handled(0);
            };
            let removed = consumption
                .previous_amount
                .wrapping_sub(consumption.remaining_amount);
            let _ = game.send_player_packet_consumption(&consumption);
            GoodsItemScriptFunctionOutcome::Handled(removed as i32)
        }
        SCRIPT_FUNCTION_CHECK_USED_GOODS => {
            if argument_count != 0 {
                return GoodsItemScriptFunctionOutcome::Invalid;
            }
            let amount = used_identity()
                .and_then(|(player_id, goods_id)| script_used_goods(game, player_id, goods_id))
                .map_or(0, |goods| goods.amount() as i32);
            GoodsItemScriptFunctionOutcome::Handled(amount)
        }
        SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_1 | SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_2 => {
            if argument_count != 1 {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            }
            let Some((player_id, goods_id)) = used_identity() else {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            };
            let Some(goods) = script_used_goods(game, player_id, goods_id) else {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            };
            let property = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let value_id = if function_id == SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_1 {
                1
            } else {
                2
            };
            let value = goods.addon_property_value(game.goods_factory(), property, value_id);
            GoodsItemScriptFunctionOutcome::Handled(
                if value != 0 || goods.query_attribute(property) {
                    value
                } else {
                    -1
                },
            )
        }
        SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_1 | SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_2 => {
            if argument_count != 2 {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            }
            let Some((player_id, goods_id)) = used_identity() else {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            };
            let property = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let modifier = integer_arguments[1].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            let value_id = if function_id == SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_1 {
                1
            } else {
                2
            };
            let update = game.find_player_mut(player_id).and_then(|player| {
                let goods = player.packet_mut().base_mut().find_mut(goods_id)?;
                goods
                    .set_addon_property_modifier_core(property, value_id, modifier)
                    .then(|| (goods.identity(), runtime.encode_goods_for_old_client(goods)))
            });
            if let Some((goods, payload)) = update {
                send_script_goods_update(game, player_id, goods, &payload);
                GoodsItemScriptFunctionOutcome::Handled(1)
            } else {
                GoodsItemScriptFunctionOutcome::Handled(0)
            }
        }
        SCRIPT_FUNCTION_GET_CURRENT_DURABILITY | SCRIPT_FUNCTION_GET_SELECTED_DURABILITY => {
            let Some(player_id) = script_player_id else {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            };
            let goods = if function_id == SCRIPT_FUNCTION_GET_SELECTED_DURABILITY {
                script_selected_goods(game, player_id)
            } else {
                used_item_id.and_then(|goods_id| script_used_goods(game, player_id, goods_id))
            };
            GoodsItemScriptFunctionOutcome::Handled(
                goods.map_or(-1, |goods| goods.current_durability()),
            )
        }
        SCRIPT_FUNCTION_SET_CURRENT_DURABILITY | SCRIPT_FUNCTION_SET_SELECTED_DURABILITY => {
            let requested = integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR);
            if requested == SCRIPT_INT_PARAMETER_ERROR {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            }
            let Some(player_id) = script_player_id else {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            };
            let goods_id = if function_id == SCRIPT_FUNCTION_SET_SELECTED_DURABILITY {
                game.find_player(player_id)
                    .and_then(|player| player.enhancement_selected_goods_id())
            } else {
                used_item_id
                    .filter(|goods_id| script_used_goods(game, player_id, *goods_id).is_some())
            };
            let Some(goods_id) = goods_id else {
                return GoodsItemScriptFunctionOutcome::Handled(-1);
            };
            let (updated, update) = game
                .find_player_mut(player_id)
                .and_then(|player| player.get_goods_by_id_mut(goods_id))
                .map_or((-1, None), |goods| {
                    let updated = goods.set_current_durability(requested);
                    let update = (updated != -1)
                        .then(|| (goods.identity(), runtime.encode_goods_for_old_client(goods)));
                    (updated, update)
                });
            if let Some((goods, payload)) = update {
                send_script_goods_update(game, player_id, goods, &payload);
            }
            GoodsItemScriptFunctionOutcome::Handled(updated)
        }
        _ => GoodsItemScriptFunctionOutcome::DifferentFunction,
    }
}

fn send_script_goods_update(game: &CGame, player_id: i32, goods: ShapeIdentity, payload: &[u8]) {
    let mut message = CMessage::new(0x0b_f918);
    message.add_long(player_id);
    message.base_mut().add_guid(goods.ex_id);
    message.add_ulong(payload.len() as u32);
    message.base_mut().add(payload);
    let _ = message.send_to_player(game.net_server(), player_id);
}

/// Единый reached tail `CScript::RunFunction`: selector уже разрешён через
/// загруженный FunctionList, а аргументы вычислены тем же экземпляром CScript.
/// Порядок family-вызовов не наблюдаем сценарием, потому что каждый owner
/// обязан вернуть `DifferentFunction` до любых side effects для чужого ID.
pub(crate) fn dispatch_script_function<Runtime: ScriptFunctionRuntime>(
    game: &mut CGame,
    runtime: &mut Runtime,
    script_player_id: Option<i32>,
    script_npc_id: Option<i32>,
    script_region_id: Option<i32>,
    used_item_id: Option<CGuid>,
    function_id: i32,
    argument_count: usize,
    integer_arguments: [Option<i32>; 4],
    string_arguments: [Option<&[u8]>; 2],
) -> ScriptFunctionDispatchOutcome {
    match function_id {
        SCRIPT_FUNCTION_OPEN_PRECIOUS_BOX => {
            let legacy_return = script_player_id.map_or(0, |player_id| {
                game.open_precious_box(player_id, string_arguments[0].unwrap_or_default())
            });
            return ScriptFunctionDispatchOutcome::Handled { legacy_return };
        }
        SCRIPT_FUNCTION_GET_PRECIOUS_ITEM => {
            let legacy_return = script_player_id.map_or(-1, |player_id| {
                game.get_precious_box_item(
                    player_id,
                    integer_arguments[0].unwrap_or(SCRIPT_INT_PARAMETER_ERROR),
                    runtime,
                )
            });
            return ScriptFunctionDispatchOutcome::Handled { legacy_return };
        }
        SCRIPT_FUNCTION_CLOSE_PRECIOUS_BOX => {
            if let Some(player_id) = script_player_id {
                game.close_precious_box(player_id);
            }
            return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
        }
        _ => {}
    }
    macro_rules! handled {
        ($outcome:expr, $pattern:path) => {
            match $outcome {
                $pattern { legacy_return, .. } => {
                    return ScriptFunctionDispatchOutcome::Handled { legacy_return }
                }
                _ => {}
            }
        };
    }

    match run_goods_item_script_function(
        game,
        runtime,
        script_player_id,
        used_item_id,
        function_id,
        argument_count,
        [integer_arguments[0], integer_arguments[1]],
    ) {
        GoodsItemScriptFunctionOutcome::Handled(legacy_return) => {
            return ScriptFunctionDispatchOutcome::Handled { legacy_return };
        }
        GoodsItemScriptFunctionOutcome::Invalid => return ScriptFunctionDispatchOutcome::Invalid,
        GoodsItemScriptFunctionOutcome::DifferentFunction => {}
    }

    handled!(
        run_country_war_action_script_function(
            game,
            runtime,
            script_player_id,
            script_npc_id,
            script_region_id,
            function_id,
            [integer_arguments[0], integer_arguments[1]],
        ),
        CountryWarActionScriptFunctionOutcome::Handled
    );
    handled!(
        run_nation_war_script_function(
            game,
            runtime,
            script_player_id,
            function_id,
            [integer_arguments[0], integer_arguments[1]],
        ),
        NationWarScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_war_query_script_function(
            game,
            script_player_id,
            script_npc_id,
            function_id,
            [
                integer_arguments[0],
                integer_arguments[1],
                integer_arguments[2]
            ],
        ),
        CountryWarQueryScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_war_declaration_script_function(
            game,
            script_player_id,
            script_npc_id,
            function_id,
            integer_arguments[0],
        ),
        CountryWarDeclarationScriptFunctionOutcome::Handled
    );

    if let Some(legacy_return) = run_fairy_script_function(
        game,
        runtime,
        script_player_id,
        function_id,
        integer_arguments[0],
    ) {
        return ScriptFunctionDispatchOutcome::Handled { legacy_return };
    }

    if let Some(legacy_return) = run_battle_fairy_script_function(
        game,
        runtime,
        script_player_id,
        function_id,
        integer_arguments,
        string_arguments,
    ) {
        return ScriptFunctionDispatchOutcome::Handled { legacy_return };
    }
    handled!(
        run_country_scalar_query_script_function(
            game,
            script_player_id,
            function_id,
            integer_arguments[0],
        ),
        CountryScalarQueryScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_identity_script_function(
            game,
            script_player_id,
            function_id,
            integer_arguments[0],
            integer_arguments[1],
        ),
        CountryIdentityScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_control_point_script_function(
            game,
            function_id,
            integer_arguments[0],
            integer_arguments[1],
        ),
        CountryControlPointScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_exile_time_script_function(
            game,
            script_player_id,
            function_id,
            integer_arguments[0],
            runtime,
        ),
        CountryExileTimeScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_quest_switch_script_function(
            game,
            script_player_id,
            function_id,
            integer_arguments[0],
            integer_arguments[1],
            integer_arguments[2],
        ),
        CountryQuestSwitchScriptFunctionOutcome::Handled
    );
    handled!(
        run_country_scalar_script_function(
            game,
            function_id,
            integer_arguments[0],
            integer_arguments[1],
        ),
        CountryScalarScriptFunctionOutcome::Handled
    );

    if let EquipmentSessionScriptFunctionOutcome::Opened(_) = run_equipment_session_script_function(
        game,
        script_player_id.unwrap_or_default(),
        function_id,
        runtime,
    ) {
        return ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 };
    }
    match run_equipment_da_kong_script_function(
        game,
        script_player_id.unwrap_or_default(),
        function_id,
        string_arguments[0],
        runtime,
    ) {
        EquipmentDaKongScriptFunctionOutcome::DifferentFunction => {
            ScriptFunctionDispatchOutcome::DifferentFunction
        }
        EquipmentDaKongScriptFunctionOutcome::HandledWithoutCall
        | EquipmentDaKongScriptFunctionOutcome::Refreshed(_) => {
            ScriptFunctionDispatchOutcome::Handled { legacy_return: 0 }
        }
    }
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
