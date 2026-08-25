//! Владелец GameServer dispatcher-а organizing messages `OnOrgasysMessage`.
//!
//! Весь dispatcher RVA `0x000895A0` остаётся `UNKNOWN` (исследовательский декомпилят хранится локально), кроме фазовых
//! cases faction lifecycle `0x90101/05/06/1A/1B` и
//! `0x7FE01/06/07/18/19/1E`,
//! AttackCity `0x7FE1F..0x7FE25`, Village `0x7FE2F..0x7FE33`, faction
//! update `0x7FE35/0x7FE36`, player-quest route `0x7FE38/0x7FE39`,
//! FourNation `0x7FE3C..0x7FE45` и control tail `0x7FE46..0x7FE4A` со статусом
//! `IMPLEMENTED`. Точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`; исходник
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\organsysmessage.cpp`.
//! Lifecycle cases проверяют session/password/player correlation, передают
//! create snapshot World, ретегируют list page в `0xBFF07` и замыкают
//! application `0x60108`; универсальный legacy session manager заменён
//! минимальным owned state в `CGame` с теми же timeout и operator-флагами.
//! Upgrade charge `0x7FE1E` списывает World-authoritative money/goods через
//! canonical wallet и packet-container effects живого игрока.
//! Declare-war pages ретегируются в `0xBFF19`, selection несёт snapshot в
//! `0x6011F`, а terminal `0x7FE19` публикует клиенту `0xBFF31` после debit.
//! World city-gate authorization `0x7FE2A` возвращается в concrete city owner,
//! обновляет gate/build state и отправляет исходные `GS0042/GS0043` notices.
//! Принятые World заявки деревенской/городской войны `0x7FE34/0x7FE37`
//! возвращаются в live player wallet, сохраняют signed wrapping/clamp старого
//! `SetMoney` и публикуют container change клиенту; отклонённая заявка ответа
//! не создаёт.
//! `0x7FE06` декодирует полный organizing wire до owned-region tail, обновляет
//! faction/master/name/union identity live player и только затем ретегирует
//! `0xBFF06`; name/union входят в последующий war-contender lifecycle.
//!
//! Каждый фазовый case читает ровно один signed war ID и передаёт его своему
//! owner-у. Faction-update cases передают текущие payload/cursor соответствующему
//! `UpdateApplyWarFacs` и игнорируют legacy bool, как исходный switch. Известный
//! opcode считается обработанным даже при отсутствующем schedule; safe
//! short-buffer возвращается локальной ошибкой без придуманного UB-эффекта.
//! Control tail сохраняет FourNation player-war-time; exploit идёт в exact
//! порядке `0xBF80C` → clamped player state → `UpdateProperty` → локализованный
//! `0xBF806(GS1177)`. Затем он ограничивает казну опубликованным CountryParam
//! maximum и отправляет World `0x60314`, сохраняет FourNation morale либо
//! перепаковывает входной router response в адресный `0xBFF36`. Другие opcodes
//! helpers не интерпретируют. FourNation сохраняет отсутствующий в exact
//! switch case `0x7FE42` как no-op, а result request отправляет World ровно
//! пять counters сообщением `0x60319`. Достигнутая war family проходит
//! живой FIFO `CGame`: расписания остаются owned, local-before-proxy lookup
//! мутирует concrete City/Village/base owners, а message/player/log effects
//! исполняет тот же runtime-контекст, который обслуживает MainLoop. Nation
//! relive замыкает owned business/session state, canonical player
//! `0xBF603(type=400,id,...)` и spatial mutation до `bChMap0` log. Nation
//! clear больше не делегируется opaque callback-у: area-ordered monster pass,
//! delete-state/list и четыре удаления `GS1120` исполняются здесь с exact
//! `0xBF504` around-result до каждой mutation.

use std::error::Error;
use std::ffi::CString;
use std::fmt;

use super::super::organizingsystem::attackcitysys::{
    AttackCityDecodeError, AttackCityMembershipBlock, AttackCityPhaseContext, CAttackCitySys,
};
use super::super::organizingsystem::fournationwarsys::{
    FourNationPhaseContext, FourNationRegionRuntime,
};
use super::super::organizingsystem::villagewarsys::{
    CVillageWarSys, VillageWarDecodeError, VillageWarPhaseContext,
};
use super::super::region::{RegionCellAccessBlock, RegionRandomContext};
use super::super::servercityregion::CityRegionContext;
use super::super::serverregion::{CServerRegion, RegionMembershipBlock};
use super::super::servervillageregion::VillageRegionContext;
use super::super::serverwarregion::WarRegionContext;
use super::super::shape::{CShape, ShapeCoordinateBlock, ShapeIdentity};
use crate::gameserver::appserver::player::{CPlayer, PlayerExploitMutationReport};
use crate::gameserver::gameserver::game::{
    CGame, GameWarRegionHandle, OldClientGoodsCodec, ScriptRegionChangeContext, ServerRegionOwner,
    colored_player_notice_message, format_legacy_text_fields,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};

pub(crate) trait GameOrganizingWarRuntime:
    CityRegionContext
    + VillageRegionContext
    + FourNationRegionRuntime
    + RegionRandomContext
    + ScriptRegionChangeContext
{
    /// Исполняет virtual `CPlayer::UpdateProperty` после FourNation exploit.
    fn update_player_property(&mut self, player: &mut CPlayer);

    fn on_four_nation_relive_block(&mut self, player_id: i32, block: FourNationReliveBlock);

    /// Публикует exact `0xBF504(type,id,0)` вокруг monster/NPC до mutation.
    fn send_four_nation_clear_around(
        &mut self,
        message: &CMessage,
        region: &CServerRegion,
        origin: &CShape,
        game: &CGame,
    ) -> i32;

    /// Возвращает первый совпавший ID в текущем observable traversal старого
    /// `stdext::hash_map`; повторный вызов после removal видит новый head.
    fn find_four_nation_clear_npc_id(&mut self, region: &CServerRegion, name: &[u8])
    -> Option<i32>;

    fn on_four_nation_clear_block(&mut self, block: FourNationClearBlock);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FourNationReliveBlock {
    CountryOutsideRectangles { country: u8 },
    RandomPosition(RegionCellAccessBlock),
    Coordinate(crate::gameserver::appserver::shape::ShapeCoordinateBlock),
    Position(RegionMembershipBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FourNationClearBlock {
    MonsterCoordinate {
        monster_id: i32,
        block: ShapeCoordinateBlock,
    },
    NpcRemoval {
        npc_id: i32,
        block: RegionMembershipBlock,
    },
    NpcTraversalMismatch {
        npc_id: i32,
    },
}

pub(crate) trait WarFactionUpdateContext {
    fn update_attack_city_contend_player(&mut self, region_id: i32, schedules: &CAttackCitySys);
    fn update_village_contend_player(&mut self, region_id: i32, schedules: &CVillageWarSys);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WarFactionUpdateDispatchError {
    AttackCity(AttackCityDecodeError),
    Village(VillageWarDecodeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WarFactionUpdateDispatchReport {
    pub(crate) opcode: u32,
    pub(crate) schedule_found: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WarPhaseDispatchReport {
    pub(crate) opcode: u32,
    pub(crate) war_number: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FourNationPhaseDispatchReport {
    pub(crate) opcode: u32,
    pub(crate) war_number: i32,
    pub(crate) schedule_found: bool,
    pub(crate) results: Option<[u32; 5]>,
    pub(crate) delivery: Option<Result<i32, SendMessageError>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingControlDispatchReport {
    FourNationExploit(FourNationExploitDispatchReport),
    FourNationWarTime {
        player_id: i32,
        time_ms: u32,
        previous_time_ms: Option<u32>,
    },
    CountryTreasury {
        country_id: u8,
        requested: i32,
        country_found: bool,
        applied: Option<i32>,
        delivery: Option<Result<i32, SendMessageError>>,
    },
    FourNationMorale {
        morale: i32,
    },
    RegionRouter {
        player_id: i32,
        player_found: bool,
        delivery: Option<i32>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FourNationExploitDispatchReport {
    pub(crate) player_id: i32,
    pub(crate) increment: u32,
    pub(crate) player_found: bool,
    pub(crate) advertised_exploit: Option<u32>,
    pub(crate) mutation: Option<PlayerExploitMutationReport>,
    pub(crate) property_delivery: Option<i32>,
    pub(crate) property_update_called: bool,
    pub(crate) notice_text: Option<Vec<u8>>,
    pub(crate) notice_delivery: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerQuestCommandKind {
    Add,
    Remove,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GamePlayerQuestCommandReport {
    pub(crate) opcode: u32,
    pub(crate) kind: GamePlayerQuestCommandKind,
    pub(crate) player_id: i32,
    pub(crate) quest_id: u16,
    pub(crate) player_found: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerQuestCommandError {
    MissingPlayerId,
    MissingQuestId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameOrganizingMessageReport {
    FactionLifecycle(FactionLifecycleDispatchReport),
    CityGate(CityGateDispatchReport),
    VillageApplication(WarApplicationResponseReport),
    CityApplication(WarApplicationResponseReport),
    FactionUpdate(WarFactionUpdateDispatchReport),
    Phase(WarPhaseDispatchReport),
    FourNationPhase(FourNationPhaseDispatchReport),
    Control(OrganizingControlDispatchReport),
    PlayerQuest(GamePlayerQuestCommandReport),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameOrganizingMessageError {
    FactionLifecycle(FactionLifecycleDispatchError),
    CityGate(FactionLifecycleDispatchError),
    VillageApplication(FactionLifecycleDispatchError),
    CityApplication(FactionLifecycleDispatchError),
    FactionUpdate(WarFactionUpdateDispatchError),
    Phase(WarPhaseDispatchError),
    Control(OrganizingControlDispatchError),
    PlayerQuest(GamePlayerQuestCommandError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CityGateDispatchReport {
    pub(crate) player_id: i32,
    pub(crate) region_id: i32,
    pub(crate) gate_id: i32,
    pub(crate) operation: i32,
    pub(crate) operated: bool,
    pub(crate) notice_delivery: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WarApplicationResponseReport {
    pub(crate) player_id: i32,
    pub(crate) fee: i32,
    pub(crate) player_found: bool,
    pub(crate) previous_money: Option<u32>,
    pub(crate) resulting_money: Option<u32>,
    pub(crate) deliveries: Vec<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionLifecycleDispatchReport {
    pub(crate) opcode: u32,
    pub(crate) player_id: i32,
    pub(crate) correlated: bool,
    pub(crate) delivery: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionLifecycleDispatchError {
    UnexpectedEnd { field: &'static str },
    MissingPlayer,
    InvalidPayload,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingControlDispatchError {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    CountryTreasuryLimitMissing {
        country_id: u8,
    },
    CountryExploitLimitMissing {
        player_id: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WarPhaseDispatchError {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
}

impl fmt::Display for GamePlayerQuestCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPlayerId => formatter.write_str("quest command не содержит player ID"),
            Self::MissingQuestId => formatter.write_str("quest command не содержит quest ID"),
        }
    }
}

impl Error for GamePlayerQuestCommandError {}

impl fmt::Display for WarPhaseDispatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "war phase ID с offset {offset} требует {needed} байт, доступно {available}"
            ),
        }
    }
}

impl Error for WarPhaseDispatchError {}

impl fmt::Display for OrganizingControlDispatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "OrganSys control {field} с offset {offset} требует {needed} байт, доступно {available}"
            ),
            Self::CountryTreasuryLimitMissing { country_id } => write!(
                formatter,
                "для country {country_id} не опубликован _max_country_treasury"
            ),
            Self::CountryExploitLimitMissing { player_id } => write!(
                formatter,
                "для FourNation exploit игрока {player_id} не опубликован _max_exploit"
            ),
        }
    }
}

impl Error for OrganizingControlDispatchError {}

impl fmt::Display for WarFactionUpdateDispatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AttackCity(error) => write!(formatter, "AttackCity faction update: {error}"),
            Self::Village(error) => write!(formatter, "Village faction update: {error}"),
        }
    }
}

impl Error for WarFactionUpdateDispatchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AttackCity(error) => Some(error),
            Self::Village(error) => Some(error),
        }
    }
}

/// Обрабатывает только доказанные faction-update opcodes `0x7FE35/0x7FE36`.
pub(crate) fn dispatch_war_faction_update<Context: WarFactionUpdateContext>(
    opcode: u32,
    payload: &[u8],
    cursor: &mut usize,
    attack_city_sys: &mut CAttackCitySys,
    village_war_sys: &mut CVillageWarSys,
    context: &mut Context,
) -> Option<Result<WarFactionUpdateDispatchReport, WarFactionUpdateDispatchError>> {
    match opcode {
        0x7fe35 => {
            let region_id = match attack_city_sys.update_apply_war_factions(payload, cursor) {
                Ok(region_id) => region_id,
                Err(error) => {
                    return Some(Err(WarFactionUpdateDispatchError::AttackCity(error)));
                }
            };
            if let Some(region_id) = region_id {
                context.update_attack_city_contend_player(region_id, attack_city_sys);
            }
            Some(Ok(WarFactionUpdateDispatchReport {
                opcode,
                schedule_found: region_id.is_some(),
            }))
        }
        0x7fe36 => {
            let region_id = match village_war_sys.update_apply_war_factions(payload, cursor) {
                Ok(region_id) => region_id,
                Err(error) => {
                    return Some(Err(WarFactionUpdateDispatchError::Village(error)));
                }
            };
            if let Some(region_id) = region_id {
                context.update_village_contend_player(region_id, village_war_sys);
            }
            Some(Ok(WarFactionUpdateDispatchReport {
                opcode,
                schedule_found: region_id.is_some(),
            }))
        }
        _ => None,
    }
}

/// Обрабатывает только доказанные фазовые opcodes городских и деревенских войн.
pub(crate) fn dispatch_war_phase<Context>(
    opcode: u32,
    payload: &[u8],
    cursor: &mut usize,
    attack_city_sys: &mut CAttackCitySys,
    village_war_sys: &mut CVillageWarSys,
    context: &mut Context,
) -> Option<Result<WarPhaseDispatchReport, WarPhaseDispatchError>>
where
    Context: AttackCityPhaseContext + VillageWarPhaseContext,
{
    if !matches!(opcode, 0x7fe1f..=0x7fe25 | 0x7fe2f..=0x7fe33) {
        return None;
    }

    let war_number = match read_phase_war_number(payload, cursor) {
        Ok(war_number) => war_number,
        Err(error) => return Some(Err(error)),
    };
    match opcode {
        0x7fe1f => attack_city_sys.on_declar_war(war_number, context),
        0x7fe20 => attack_city_sys.on_attack_city_start(war_number, context),
        0x7fe21 => attack_city_sys.on_attack_city_time_out(war_number, context),
        0x7fe22 => attack_city_sys.on_attack_city_end(war_number, context),
        0x7fe23 => attack_city_sys.on_mass(war_number, context),
        0x7fe24 => attack_city_sys.on_clear_other_player(war_number, context),
        0x7fe25 => attack_city_sys.on_refresh_region(war_number, context),
        0x7fe2f => village_war_sys.on_delcare_war(war_number, context),
        0x7fe30 => village_war_sys.on_attack_village_start(war_number, context),
        0x7fe31 => village_war_sys.on_attack_village_out_time(war_number, context),
        0x7fe32 => village_war_sys.on_attack_village_end(war_number, context),
        0x7fe33 => village_war_sys.on_clear_player(war_number, context),
        _ => unreachable!("opcode отфильтрован перед чтением payload"),
    }
    Some(Ok(WarPhaseDispatchReport { opcode, war_number }))
}

fn dispatch_game_player_quest_command(
    opcode: u32,
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<GamePlayerQuestCommandReport, GamePlayerQuestCommandError> {
    let player_id = message
        .base_mut()
        .get_long()
        .ok_or(GamePlayerQuestCommandError::MissingPlayerId)?;
    let quest_id = message
        .base_mut()
        .get_short()
        .ok_or(GamePlayerQuestCommandError::MissingQuestId)? as u16;
    let kind = if opcode == 0x7fe38 {
        GamePlayerQuestCommandKind::Add
    } else {
        GamePlayerQuestCommandKind::Remove
    };
    let player_found = game.find_player(player_id).is_some();
    if player_found {
        match kind {
            GamePlayerQuestCommandKind::Add => game.add_script_player_quest(player_id, quest_id),
            GamePlayerQuestCommandKind::Remove => {
                game.remove_script_player_quest(player_id, quest_id)
            }
        }
    }
    Ok(GamePlayerQuestCommandReport {
        opcode,
        kind,
        player_id,
        quest_id,
        player_found,
    })
}

/// Подключает всю достигнутую OrganSys family к живому `CGame` owner-у.
pub(crate) fn dispatch_game_organizing_message<
    Runtime: GameOrganizingWarRuntime + ScriptRegionChangeContext + OldClientGoodsCodec,
>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<GameOrganizingMessageReport, GameOrganizingMessageError>> {
    let opcode = message.message_type() as u32;
    if !matches!(
        opcode,
        0x90101
            | 0x90105
            | 0x90106
            | 0x9011a
            | 0x9011b
            | 0x7fe01
            | 0x7fe06
            | 0x7fe07
            | 0x7fe18
            | 0x7fe19
            | 0x7fe1e
            | 0x7fe2a
            | 0x7fe1f..=0x7fe25
            | 0x7fe2f..=0x7fe33
            | 0x7fe34
            | 0x7fe35
            | 0x7fe36
            | 0x7fe37
            | 0x7fe38
            | 0x7fe39
            | 0x7fe3c..=0x7fe3f
            | 0x7fe41
            | 0x7fe43..=0x7fe45
            | 0x7fe46..=0x7fe4a
    ) {
        return None;
    }

    if matches!(opcode, 0x7fe38 | 0x7fe39) {
        return Some(
            dispatch_game_player_quest_command(opcode, message, game)
                .map(GameOrganizingMessageReport::PlayerQuest)
                .map_err(GameOrganizingMessageError::PlayerQuest),
        );
    }

    if opcode == 0x7fe2a {
        return Some(
            dispatch_city_gate_response(message, game, runtime)
                .map(GameOrganizingMessageReport::CityGate)
                .map_err(GameOrganizingMessageError::CityGate),
        );
    }

    if opcode == 0x7fe34 {
        return Some(
            dispatch_village_war_application_response(message, game, runtime)
                .map(GameOrganizingMessageReport::VillageApplication)
                .map_err(GameOrganizingMessageError::VillageApplication),
        );
    }

    if opcode == 0x7fe37 {
        return Some(
            dispatch_war_application_response(message, game, runtime)
                .map(GameOrganizingMessageReport::CityApplication)
                .map_err(GameOrganizingMessageError::CityApplication),
        );
    }

    if matches!(
        opcode,
        0x90101
            | 0x90105
            | 0x90106
            | 0x9011a
            | 0x9011b
            | 0x7fe01
            | 0x7fe06
            | 0x7fe07
            | 0x7fe18
            | 0x7fe19
            | 0x7fe1e
    ) {
        return Some(
            dispatch_faction_lifecycle_message(opcode, message, game, runtime)
                .map(GameOrganizingMessageReport::FactionLifecycle)
                .map_err(GameOrganizingMessageError::FactionLifecycle),
        );
    }

    if matches!(opcode, 0x7fe46..=0x7fe4a) {
        return Some(
            dispatch_organizing_control_message(opcode, message, game, runtime)
                .map(GameOrganizingMessageReport::Control)
                .map_err(GameOrganizingMessageError::Control),
        );
    }

    if matches!(
        opcode,
        0x7fe3c..=0x7fe3f | 0x7fe41 | 0x7fe43..=0x7fe45
    ) {
        return Some(
            dispatch_four_nation_phase_message(opcode, message, game, runtime)
                .map(GameOrganizingMessageReport::FourNationPhase)
                .map_err(GameOrganizingMessageError::Phase),
        );
    }

    let mut owners = game.take_war_startup_owners();
    let result = {
        let mut context = GameOrganizingWarContext { game, runtime };
        let (payload, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        if matches!(opcode, 0x7fe35 | 0x7fe36) {
            dispatch_war_faction_update(
                opcode,
                payload,
                cursor,
                &mut owners.attack_city,
                &mut owners.village,
                &mut context,
            )
            .expect("faction opcode проверен перед dispatcher-ом")
            .map(GameOrganizingMessageReport::FactionUpdate)
            .map_err(GameOrganizingMessageError::FactionUpdate)
        } else {
            dispatch_war_phase(
                opcode,
                payload,
                cursor,
                &mut owners.attack_city,
                &mut owners.village,
                &mut context,
            )
            .expect("phase opcode проверен перед dispatcher-ом")
            .map(GameOrganizingMessageReport::Phase)
            .map_err(GameOrganizingMessageError::Phase)
        }
    };
    game.restore_war_startup_owners(owners);
    Some(result)
}

fn dispatch_village_war_application_response<Runtime: OldClientGoodsCodec>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Result<WarApplicationResponseReport, FactionLifecycleDispatchError> {
    dispatch_war_application_response(message, game, runtime)
}

fn dispatch_war_application_response<Runtime: OldClientGoodsCodec>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Result<WarApplicationResponseReport, FactionLifecycleDispatchError> {
    let player_id = message
        .base_mut()
        .get_long()
        .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field: "player ID" })?;
    let fee =
        message
            .base_mut()
            .get_long()
            .ok_or(FactionLifecycleDispatchError::UnexpectedEnd {
                field: "village application fee",
            })?;
    if !message.base_mut().unread_bytes().is_empty() {
        return Err(FactionLifecycleDispatchError::InvalidPayload);
    }
    let money = game.apply_war_application_money(player_id, fee, runtime);
    let (previous_money, resulting_money, deliveries) = money
        .map(|(previous, resulting, deliveries)| (Some(previous), Some(resulting), deliveries))
        .unwrap_or((None, None, Vec::new()));
    Ok(WarApplicationResponseReport {
        player_id,
        fee,
        player_found: previous_money.is_some(),
        previous_money,
        resulting_money,
        deliveries,
    })
}

fn dispatch_city_gate_response<Runtime: GameOrganizingWarRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Result<CityGateDispatchReport, FactionLifecycleDispatchError> {
    let player_id = message
        .base_mut()
        .get_long()
        .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field: "player ID" })?;
    let region_id = message
        .base_mut()
        .get_long()
        .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field: "region ID" })?;
    let gate_id = message
        .base_mut()
        .get_long()
        .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field: "gate ID" })?;
    let operation = message
        .base_mut()
        .get_long()
        .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field: "operation" })?;
    if !message.base_mut().unread_bytes().is_empty() {
        return Err(FactionLifecycleDispatchError::InvalidPayload);
    }
    let operated = game.operate_script_city_gate(region_id, gate_id, operation, runtime);
    let state = game.script_city_gate_state(region_id, gate_id);
    let notice_id = match (operation, state) {
        (0, 0) => Some(b"GS0042".as_slice()),
        (1, 1) => Some(b"GS0043".as_slice()),
        _ => None,
    };
    let notice_delivery = notice_id.map(|notice_id| {
        colored_player_notice_message(0xffff_ffff, 0xffff_0000, game.get_string_by_id(notice_id))
            .send_to_player(game.net_server(), player_id)
    });
    Ok(CityGateDispatchReport {
        player_id,
        region_id,
        gate_id,
        operation,
        operated,
        notice_delivery,
    })
}

fn dispatch_faction_lifecycle_message<Runtime: ScriptRegionChangeContext>(
    opcode: u32,
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Result<FactionLifecycleDispatchReport, FactionLifecycleDispatchError> {
    let read_i64 = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long64()
            .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field })
    };
    let read_i32 = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long()
            .ok_or(FactionLifecycleDispatchError::UnexpectedEnd { field })
    };
    match opcode {
        0x90101 => {
            message.resolve_player_context(game);
            let player_id = message
                .player_id()
                .ok_or(FactionLifecycleDispatchError::MissingPlayer)?;
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            let accepted = read_i32(message, "accepted")?;
            let faction_name = (accepted != 0).then(|| message.base_mut().get_str_bytes(20));
            let faction_name = match faction_name {
                Some(Some(name)) if !name.is_empty() => Some(name),
                Some(_) => return Err(FactionLifecycleDispatchError::InvalidPayload),
                None => None,
            };
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated = game.submit_script_faction_creation(
                player_id,
                session_id,
                password,
                faction_name.as_deref(),
                runtime,
            );
            Ok(FactionLifecycleDispatchReport {
                opcode,
                player_id,
                correlated,
                delivery: None,
            })
        }
        0x7fe01 => {
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            let player_id = read_i32(message, "player ID")?;
            let result = read_i32(message, "result")?;
            if !matches!(result, 0 | 1) || !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated =
                game.finish_script_faction_creation(session_id, password, player_id, result);
            Ok(FactionLifecycleDispatchReport {
                opcode,
                player_id,
                correlated,
                delivery: None,
            })
        }
        0x7fe06 => {
            let player_id = read_i32(message, "player ID")?;
            let faction_id = read_i32(message, "faction ID")?;
            let mut faction_level = 0;
            let mut faction_master_id = 0;
            let mut faction_name = Vec::new();
            let mut union_id = 0;
            if faction_id > 0 {
                let _logo_id = read_i32(message, "faction logo ID")?;
                faction_level = message.base_mut().get_word().ok_or(
                    FactionLifecycleDispatchError::UnexpectedEnd {
                        field: "faction level",
                    },
                )?;
                let _experience = read_i32(message, "faction experience")?;
                let _force = read_i32(message, "faction force")?;
                let _contribute = read_i32(message, "faction contribute")?;
                faction_name = message
                    .base_mut()
                    .get_str_bytes(0x100)
                    .ok_or(FactionLifecycleDispatchError::InvalidPayload)?;
                let _title = message
                    .base_mut()
                    .get_str_bytes(0x100)
                    .ok_or(FactionLifecycleDispatchError::InvalidPayload)?;
                faction_master_id = read_i32(message, "faction master ID")?;
                union_id = read_i32(message, "union ID")?;
                let _union_master_id = read_i32(message, "union master ID")?;
                for field in ["enemy factions", "city-war enemy factions"] {
                    let count = read_i32(message, field)?;
                    if count < 0 {
                        return Err(FactionLifecycleDispatchError::InvalidPayload);
                    }
                    for _ in 0..count {
                        let _ = read_i32(message, field)?;
                    }
                }
                let owned_count = read_i32(message, "owned regions")?;
                if owned_count < 0 {
                    return Err(FactionLifecycleDispatchError::InvalidPayload);
                }
                for _ in 0..owned_count {
                    let _region_id = read_i32(message, "owned region ID")?;
                    let _war_type = message.base_mut().get_word().ok_or(
                        FactionLifecycleDispatchError::UnexpectedEnd {
                            field: "owned region war type",
                        },
                    )?;
                    let _reserved = message.base_mut().get_word().ok_or(
                        FactionLifecycleDispatchError::UnexpectedEnd {
                            field: "owned region reserved",
                        },
                    )?;
                }
            }
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated = if let Some(player) = game.find_player_mut(player_id) {
                player.restore_faction_identity(
                    faction_id,
                    faction_level,
                    faction_master_id,
                    &faction_name,
                    union_id,
                );
                true
            } else {
                false
            };
            let delivery = correlated.then(|| {
                message.set_message_type(0x000b_ff06);
                let _ = game.send_player_shape_around(player_id, None, message);
                message.send_to_player(game.net_server(), player_id)
            });
            Ok(FactionLifecycleDispatchReport {
                opcode,
                player_id,
                correlated,
                delivery,
            })
        }
        0x90105 => {
            message.resolve_player_context(game);
            let player_id = message
                .player_id()
                .ok_or(FactionLifecycleDispatchError::MissingPlayer)?;
            let session_id = read_i64(message, "session ID")?;
            // Exact `0x90105` advances past password, but `OnDo` correlates
            // continuation only by managed session ID and current player.
            let _password = read_i32(message, "password")?;
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated = game.continue_script_faction_application(player_id, session_id);
            Ok(FactionLifecycleDispatchReport {
                opcode,
                player_id,
                correlated,
                delivery: None,
            })
        }
        0x90106 => {
            message.resolve_player_context(game);
            let player_id = message
                .player_id()
                .ok_or(FactionLifecycleDispatchError::MissingPlayer)?;
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            let discarded = read_i32(message, "discarded faction ID")?;
            let accepted = read_i32(message, "accepted")?;
            let faction_name = message
                .base_mut()
                .get_str_bytes(20)
                .ok_or(FactionLifecycleDispatchError::InvalidPayload)?;
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated = game.select_script_faction_application(
                player_id,
                session_id,
                password,
                discarded,
                accepted,
                &faction_name,
            );
            Ok(FactionLifecycleDispatchReport {
                opcode,
                player_id,
                correlated,
                delivery: None,
            })
        }
        0x7fe07 => {
            let player_id = read_i32(message, "player ID")?;
            let total = read_i32(message, "total factions")?;
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            if total < 0 {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            if total == 0 {
                if !message.base_mut().unread_bytes().is_empty() {
                    return Err(FactionLifecycleDispatchError::InvalidPayload);
                }
                let correlated =
                    game.finish_empty_script_faction_application(player_id, session_id, password);
                return Ok(FactionLifecycleDispatchReport {
                    opcode,
                    player_id,
                    correlated,
                    delivery: None,
                });
            }
            let correlated =
                game.script_faction_application_is_active(player_id, session_id, password);
            let delivery = correlated.then(|| {
                message.set_message_type(0x000b_ff07);
                message.send_to_player(game.net_server(), player_id)
            });
            Ok(FactionLifecycleDispatchReport {
                opcode,
                player_id,
                correlated,
                delivery,
            })
        }
        0x7fe1e => {
            let player_id = read_i32(message, "player ID")?;
            let money = read_i32(message, "money")?;
            let goods_name = message
                .base_mut()
                .get_str_bytes(100)
                .ok_or(FactionLifecycleDispatchError::InvalidPayload)?;
            if money < 0 || !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated =
                game.apply_script_faction_upgrade_debit(player_id, money as u32, &goods_name);
            Ok(FactionLifecycleDispatchReport {
                opcode,
                player_id,
                correlated,
                delivery: None,
            })
        }
        0x9011a => {
            message.resolve_player_context(game);
            let player_id = message
                .player_id()
                .ok_or(FactionLifecycleDispatchError::MissingPlayer)?;
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated = game.continue_script_faction_war_page(player_id, session_id, password);
            Ok(FactionLifecycleDispatchReport {
                opcode,
                player_id,
                correlated,
                delivery: None,
            })
        }
        0x9011b => {
            message.resolve_player_context(game);
            let player_id = message
                .player_id()
                .ok_or(FactionLifecycleDispatchError::MissingPlayer)?;
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            let target_faction_id = read_i32(message, "target faction ID")?;
            let war_type = if target_faction_id > 0 {
                read_i32(message, "war type")?
            } else {
                0
            };
            if !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let correlated = if target_faction_id > 0 {
                game.select_script_faction_war_target(
                    player_id,
                    session_id,
                    password,
                    target_faction_id,
                    war_type,
                    runtime,
                )
            } else {
                game.close_script_faction_war_declaration(player_id, session_id, password)
            };
            Ok(FactionLifecycleDispatchReport {
                opcode,
                player_id,
                correlated,
                delivery: None,
            })
        }
        0x7fe18 => {
            let player_id = read_i32(message, "player ID")?;
            let total = read_i32(message, "total factions")?;
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            if total < 0 {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            if total == 0 {
                if !message.base_mut().unread_bytes().is_empty() {
                    return Err(FactionLifecycleDispatchError::InvalidPayload);
                }
                let correlated =
                    game.close_script_faction_war_declaration(player_id, session_id, password);
                return Ok(FactionLifecycleDispatchReport {
                    opcode,
                    player_id,
                    correlated,
                    delivery: None,
                });
            }
            let correlated = game.script_faction_war_is_active(player_id, session_id, password);
            let delivery = correlated.then(|| {
                message.set_message_type(0x000b_ff19);
                message.send_to_player(game.net_server(), player_id)
            });
            Ok(FactionLifecycleDispatchReport {
                opcode,
                player_id,
                correlated,
                delivery,
            })
        }
        0x7fe19 => {
            let session_id = read_i64(message, "session ID")?;
            let password = read_i32(message, "password")?;
            let player_id = read_i32(message, "player ID")?;
            let money = read_i32(message, "money")?;
            if money < 0 || !message.base_mut().unread_bytes().is_empty() {
                return Err(FactionLifecycleDispatchError::InvalidPayload);
            }
            let delivery = game.finish_script_faction_war_result(
                player_id,
                session_id,
                password,
                money as u32,
            );
            Ok(FactionLifecycleDispatchReport {
                opcode,
                player_id,
                correlated: delivery.is_some(),
                delivery,
            })
        }
        _ => unreachable!("faction lifecycle opcode проверен caller-ом"),
    }
}

fn dispatch_four_nation_phase_message<Runtime: GameOrganizingWarRuntime>(
    opcode: u32,
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Result<FourNationPhaseDispatchReport, WarPhaseDispatchError> {
    let (payload, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
    let war_number = read_phase_war_number(payload, cursor)?;
    let schedule_exists = game
        .four_nation_war_sys()
        .setups()
        .get(war_number as usize)
        .is_some();
    let sign_up_counts = if opcode == 0x7fe3e && schedule_exists {
        let mut values = [0; 5];
        for value in &mut values {
            *value = read_phase_war_number(payload, cursor)?;
        }
        Some(values)
    } else {
        None
    };
    let mut owners = game.take_war_startup_owners();
    let (schedule_found, results) = {
        let mut context = GameOrganizingWarContext { game, runtime };
        match opcode {
            0x7fe3c => (
                owners.four_nation.on_war_start(war_number, &mut context),
                None,
            ),
            0x7fe3d => (
                owners
                    .four_nation
                    .on_sign_up_war_start(war_number, &mut context),
                None,
            ),
            0x7fe3e => (
                if let Some(sign_up_counts) = sign_up_counts {
                    owners
                        .four_nation
                        .on_sign_up_war_end(war_number, sign_up_counts, &mut context)
                } else {
                    false
                },
                None,
            ),
            0x7fe3f => (
                owners.four_nation.on_enter_start(war_number, &mut context),
                None,
            ),
            0x7fe41 => (
                owners
                    .four_nation
                    .on_refresh_region(war_number, &mut context),
                None,
            ),
            0x7fe43 => (
                owners.four_nation.on_war_end(war_number, &mut context),
                None,
            ),
            0x7fe44 => (
                owners.four_nation.on_clear_war(war_number, &mut context),
                None,
            ),
            0x7fe45 => {
                let results = owners
                    .four_nation
                    .take_war_results(war_number, &mut context);
                (
                    owners
                        .four_nation
                        .setups()
                        .get(war_number as usize)
                        .is_some(),
                    results,
                )
            }
            _ => unreachable!("FourNation phase opcode проверен outer dispatcher-ом"),
        }
    };
    game.restore_war_startup_owners(owners);

    let delivery = results.map(|values| {
        let mut response = CMessage::new(0x60319);
        for value in values {
            response.base_mut().add_ulong(value);
        }
        response.send(game, false)
    });
    Ok(FourNationPhaseDispatchReport {
        opcode,
        war_number,
        schedule_found,
        results,
        delivery,
    })
}

fn dispatch_organizing_control_message<Runtime: GameOrganizingWarRuntime>(
    opcode: u32,
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Result<OrganizingControlDispatchReport, OrganizingControlDispatchError> {
    match opcode {
        0x7fe46 => {
            let player_id = read_control_i32(message, "exploit player ID")?;
            let increment = read_control_i32(message, "exploit increment")? as u32;
            let Some(previous_exploit) = game
                .find_player(player_id)
                .map(|player| player.base_properties().exploit)
            else {
                return Ok(OrganizingControlDispatchReport::FourNationExploit(
                    FourNationExploitDispatchReport {
                        player_id,
                        increment,
                        player_found: false,
                        advertised_exploit: None,
                        mutation: None,
                        property_delivery: None,
                        property_update_called: false,
                        notice_text: None,
                        notice_delivery: None,
                    },
                ));
            };
            let maximum = game
                .country_param()
                .max_exploit()
                .ok_or(OrganizingControlDispatchError::CountryExploitLimitMissing { player_id })?;
            let advertised_exploit = previous_exploit.wrapping_add(increment);
            let mut property = CMessage::new(0xbf80c);
            property.base_mut().add_long(player_id);
            property.base_mut().add_long(player_id);
            property.base_mut().add_str(Some(c"dwExploit"));
            property.base_mut().add_ulong(advertised_exploit);
            let property_delivery = property.send_to_player(game.net_server(), player_id);
            let mutation = {
                let player = game
                    .find_player_mut(player_id)
                    .expect("player проверен до exact exploit mutation");
                let mutation = player.set_exploit(advertised_exploit, maximum);
                runtime.update_player_property(player);
                mutation
            };

            let notice_text = Some(format_four_nation_exploit_notice(
                game.get_string_by_id(b"GS1177"),
                increment as i32,
            ));
            let notice_delivery = notice_text.as_ref().map(|text| {
                let text = CString::new(text.as_slice())
                    .expect("legacy string prefix и decimal не содержат NUL");
                let mut notice = CMessage::new(0xbf806);
                notice.base_mut().add_ulong(u32::MAX);
                notice.base_mut().add_ulong(0xffff_0000);
                notice.base_mut().add_str(Some(&text));
                notice.send_to_player(game.net_server(), player_id)
            });
            Ok(OrganizingControlDispatchReport::FourNationExploit(
                FourNationExploitDispatchReport {
                    player_id,
                    increment,
                    player_found: true,
                    advertised_exploit: Some(advertised_exploit),
                    mutation: Some(mutation),
                    property_delivery: Some(property_delivery),
                    property_update_called: true,
                    notice_text,
                    notice_delivery,
                },
            ))
        }
        0x7fe47 => {
            let player_id = read_control_i32(message, "war-time player ID")?;
            let time_ms = read_control_i32(message, "player war time")? as u32;
            let previous_time_ms = game
                .four_nation_war_sys_mut()
                .set_one_player_war_time(player_id, time_ms);
            Ok(OrganizingControlDispatchReport::FourNationWarTime {
                player_id,
                time_ms,
                previous_time_ms,
            })
        }
        0x7fe48 => {
            let country_id = read_control_i32(message, "country ID")? as u8;
            let requested = read_control_i32(message, "country treasury")?;
            if game.country_handler().country(country_id).is_none() {
                return Ok(OrganizingControlDispatchReport::CountryTreasury {
                    country_id,
                    requested,
                    country_found: false,
                    applied: None,
                    delivery: None,
                });
            }
            let maximum = game.country_param().max_country_treasury().ok_or(
                OrganizingControlDispatchError::CountryTreasuryLimitMissing { country_id },
            )?;
            let applied = if requested < 0 {
                0
            } else {
                requested.min(maximum)
            };
            let update = game
                .country_handler_mut()
                .country_mut(country_id)
                .expect("country проверен до exact treasury mutation")
                .set_country_treasury(applied);
            let delivery = Some(update.send(game, false));
            Ok(OrganizingControlDispatchReport::CountryTreasury {
                country_id,
                requested,
                country_found: true,
                applied: Some(applied),
                delivery,
            })
        }
        0x7fe49 => {
            let morale = read_control_i32(message, "FourNation morale")?;
            game.four_nation_war_sys_mut().set_morale(morale);
            Ok(OrganizingControlDispatchReport::FourNationMorale { morale })
        }
        0x7fe4a => {
            let player_id = read_control_i32(message, "router player ID")?;
            let player_found = game.find_player(player_id).is_some();
            let delivery = player_found.then(|| {
                message.set_message_type(0xbff36);
                message.base_mut().update();
                message.send_to_player(game.net_server(), player_id)
            });
            Ok(OrganizingControlDispatchReport::RegionRouter {
                player_id,
                player_found,
                delivery,
            })
        }
        _ => unreachable!("control opcode отфильтрован перед dispatcher-ом"),
    }
}

fn format_four_nation_exploit_notice(template: &[u8], increment: i32) -> Vec<u8> {
    let template = legacy_c_string_prefix(template);
    let Some(marker) = template.windows(2).position(|window| window == b"%d") else {
        return template[..template.len().min(0xff)].to_vec();
    };
    let value = increment.to_string();
    let mut result = Vec::with_capacity(template.len().saturating_add(value.len()));
    result.extend_from_slice(&template[..marker]);
    result.extend_from_slice(value.as_bytes());
    result.extend_from_slice(&template[marker + 2..]);
    result.truncate(0xff);
    result
}

fn format_legacy_integer_fields(
    template: &[u8],
    values: &[String],
    maximum_bytes: usize,
) -> Vec<u8> {
    let template = legacy_c_string_prefix(template);
    let mut result = Vec::with_capacity(template.len());
    let mut offset = 0;
    let mut value_index = 0;
    while offset < template.len() {
        if template[offset] != b'%' || offset + 1 >= template.len() {
            result.push(template[offset]);
            offset += 1;
            continue;
        }
        if template[offset + 1] == b'%' {
            result.push(b'%');
            offset += 2;
            continue;
        }
        let conversion_length = if template[offset + 1] == b'l'
            && template
                .get(offset + 2)
                .is_some_and(|byte| matches!(byte, b'd' | b'i' | b'u'))
        {
            3
        } else if matches!(template[offset + 1], b'd' | b'i' | b'u') {
            2
        } else {
            result.push(template[offset]);
            offset += 1;
            continue;
        };
        let Some(value) = values.get(value_index) else {
            result.extend_from_slice(&template[offset..offset + conversion_length]);
            offset += conversion_length;
            continue;
        };
        result.extend_from_slice(value.as_bytes());
        value_index += 1;
        offset += conversion_length;
    }
    result.truncate(maximum_bytes);
    result
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let length = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..length]
}

fn read_control_i32(
    message: &mut CMessage,
    field: &'static str,
) -> Result<i32, OrganizingControlDispatchError> {
    let base = message.base_mut();
    let offset = base.cursor();
    let available = base.as_wire_bytes().len().saturating_sub(offset);
    base.get_long()
        .ok_or(OrganizingControlDispatchError::UnexpectedEnd {
            field,
            offset,
            needed: 4,
            available,
        })
}

fn read_phase_war_number(payload: &[u8], cursor: &mut usize) -> Result<i32, WarPhaseDispatchError> {
    let offset = *cursor;
    let available = payload.len().saturating_sub(offset);
    let Some(bytes) = payload.get(offset..offset.saturating_add(4)) else {
        return Err(WarPhaseDispatchError::UnexpectedEnd {
            offset,
            needed: 4,
            available,
        });
    };
    *cursor += 4;
    Ok(i32::from_le_bytes(
        bytes.try_into().expect("slice имеет 4 байта"),
    ))
}

struct GameOrganizingWarContext<'a, Runtime> {
    game: &'a mut CGame,
    runtime: &'a mut Runtime,
}

enum ContendSchedule<'a> {
    AttackCity(&'a CAttackCitySys),
    Village(&'a CVillageWarSys),
}

enum ContendProjectionError<RuntimeError> {
    Schedule(AttackCityMembershipBlock),
    Runtime(RuntimeError),
}

struct ContendProjectionContext<'a, Runtime> {
    runtime: &'a mut Runtime,
    schedule: ContendSchedule<'a>,
    war_number: i32,
}

impl<Runtime: WarRegionContext> WarRegionContext for ContendProjectionContext<'_, Runtime> {
    type MembershipError = ContendProjectionError<Runtime::MembershipError>;

    fn player_faction_id(&mut self, player_id: i32) -> Option<i32> {
        self.runtime.player_faction_id(player_id)
    }

    fn is_apply_war_faction(&mut self, faction_id: i32) -> Result<bool, Self::MembershipError> {
        match self.schedule {
            ContendSchedule::AttackCity(schedule) => schedule
                .is_already_declar_for_war(self.war_number, faction_id)
                .map_err(ContendProjectionError::Schedule),
            ContendSchedule::Village(schedule) => {
                Ok(schedule.is_already_declar_for_war(self.war_number, faction_id))
            }
        }
    }

    fn send_contend_time(&mut self, player_id: i32, time: i32) {
        self.runtime.send_contend_time(player_id, time);
    }

    fn set_global_player_contend_state(&mut self, player_id: i32, state: bool) {
        self.runtime
            .set_global_player_contend_state(player_id, state);
    }

    fn set_region_player_contend_state(&mut self, region_id: i32, player_id: i32, state: bool) {
        self.runtime
            .set_region_player_contend_state(region_id, player_id, state);
    }
}

impl<Runtime: GameOrganizingWarRuntime> GameOrganizingWarContext<'_, Runtime> {
    fn lookup_region_then_proxy(&self, region_id: i32) -> Option<GameWarRegionHandle> {
        if self.game.find_region(region_id).is_some() {
            Some(GameWarRegionHandle::Local(region_id))
        } else {
            self.game
                .find_proxy_region(region_id)
                .map(|_| GameWarRegionHandle::Proxy(region_id))
        }
    }

    fn lookup_server_region(&self, region_id: i32) -> Option<GameWarRegionHandle> {
        self.game
            .find_region(region_id)
            .map(|_| GameWarRegionHandle::Local(region_id))
    }

    fn update_contenders(&mut self, region_id: i32, schedule: ContendSchedule<'_>) {
        let Some(region) = self.game.find_region_mut(region_id) else {
            return;
        };
        let war = match region {
            ServerRegionOwner::Village(region) => &mut region.war,
            ServerRegionOwner::City(region) => &mut region.war,
            ServerRegionOwner::Nation(region) => &mut region.war,
            ServerRegionOwner::GodsBattle(region) => &mut region.war,
            ServerRegionOwner::Base(_) | ServerRegionOwner::Country(_) => return,
        };
        let mut context = ContendProjectionContext {
            runtime: self.runtime,
            schedule,
            war_number: war.base.get_war_number(),
        };
        let _ = war.update_contend_player(&mut context);
    }

    fn kick_out_four_nation_players(
        &mut self,
        region: &mut super::super::servernationregion::ServerNationRegion,
    ) {
        let player_ids = region.war.base.registered_player_ids();
        let (area_width, area_height) = self.game.area_dimensions();
        for player_id in player_ids {
            let Some((country, eligible)) = self
                .game
                .find_player(player_id)
                .map(|player| (player.country(), player.can_start_nation_war_timing()))
            else {
                continue;
            };
            if !eligible {
                continue;
            }
            let Some(rect) = region.relive_rects().get(usize::from(country)).copied() else {
                self.runtime.on_four_nation_relive_block(
                    player_id,
                    FourNationReliveBlock::CountryOutsideRectangles { country },
                );
                continue;
            };
            let destination = match region.war.base.region.get_random_pos_in_range(
                rect.left,
                rect.top,
                rect.right.wrapping_sub(rect.left),
                rect.bottom.wrapping_sub(rect.top),
                self.runtime,
            ) {
                Ok(destination) => destination,
                Err(block) => {
                    self.runtime.on_four_nation_relive_block(
                        player_id,
                        FourNationReliveBlock::RandomPosition(block),
                    );
                    continue;
                }
            };

            let direction = self
                .game
                .find_player(player_id)
                .expect("Nation m_vPlayers ID проверен до random position")
                .shape()
                .get_direction();
            let _business = self.game.finish_player_business(player_id);
            {
                let player = self
                    .game
                    .find_player_mut(player_id)
                    .expect("end_business не удаляет Nation player owner");
                player.prepare_nation_relive();
            }
            let previous = {
                let player = self
                    .game
                    .find_player(player_id)
                    .expect("end_business не удаляет player owner");
                let x = match player.shape().get_tile_x() {
                    Ok(x) => x,
                    Err(block) => {
                        self.runtime.on_four_nation_relive_block(
                            player_id,
                            FourNationReliveBlock::Coordinate(block),
                        );
                        continue;
                    }
                };
                let y = match player.shape().get_tile_y() {
                    Ok(y) => y,
                    Err(block) => {
                        self.runtime.on_four_nation_relive_block(
                            player_id,
                            FourNationReliveBlock::Coordinate(block),
                        );
                        continue;
                    }
                };
                (x, y)
            };

            if previous != (destination.x, destination.y) {
                let mut movement = CMessage::new(0xbf603);
                movement.base_mut().add_long(400);
                movement.base_mut().add_long(player_id);
                movement.base_mut().add_long(destination.x);
                movement.base_mut().add_long(destination.y);
                movement.base_mut().add_long(0);
                let player = self
                    .game
                    .find_player(player_id)
                    .expect("Nation relive player остаётся live до around send");
                if let Err(block) = self.runtime.send_nation_player_around(
                    &region.war.base,
                    player.shape(),
                    None,
                    &movement,
                ) {
                    self.runtime.on_four_nation_relive_block(
                        player_id,
                        FourNationReliveBlock::Coordinate(block),
                    );
                }

                let facts = player.nation_relive_position_facts(area_width, area_height);
                let result = {
                    let player = self
                        .game
                        .find_player_mut(player_id)
                        .expect("around send не удаляет player owner");
                    region.war.base.set_move_shape_tile_position(
                        player.nation_relive_shape_mut(),
                        destination.x,
                        destination.y,
                        facts,
                    )
                };
                if let Err(block) = result {
                    self.runtime.on_four_nation_relive_block(
                        player_id,
                        FourNationReliveBlock::Position(block),
                    );
                    continue;
                }
            }

            let (wallet_gold, bank_gold) = {
                let player = self
                    .game
                    .find_player_mut(player_id)
                    .expect("Nation relive player остаётся live до direction/log");
                player.nation_relive_shape_mut().set_direction(direction);
                (player.money(), player.depot_money())
            };
            let _change_log = self.game.send_player_change_region_log(
                0,
                player_id,
                wallet_gold,
                bank_gold,
                region.war.base.id,
                previous.0,
                previous.1,
                region.war.base.id,
                destination.x,
                destination.y,
            );
        }
    }

    /// Exact `ServerNationRegion::OnClearWar`: active/sleeping/pet/carriage
    /// traversal сохраняет area storage order, подходящие monsters получают
    /// disappear packet и state `1`, sleeping state `1` дополнительно
    /// попадают в delete-list, затем удаляются до четырёх `GS1120` NPC.
    fn clear_four_nation_war(
        &mut self,
        region: &mut super::super::servernationregion::ServerNationRegion,
    ) {
        let width = region.war.base.region.width;
        let height = region.war.base.region.height;
        for monster_id in region.war.base.area_monster_ids() {
            let Some(monster) = region.war.base.find_monster_by_id(monster_id) else {
                continue;
            };
            if !monster.can_clear_from_nation_war() {
                continue;
            }
            let shape = monster.move_shape().shape();
            let tile_x = match shape.get_tile_x() {
                Ok(tile_x) => tile_x,
                Err(block) => {
                    self.runtime.on_four_nation_clear_block(
                        FourNationClearBlock::MonsterCoordinate { monster_id, block },
                    );
                    continue;
                }
            };
            let tile_y = match shape.get_tile_y() {
                Ok(tile_y) => tile_y,
                Err(block) => {
                    self.runtime.on_four_nation_clear_block(
                        FourNationClearBlock::MonsterCoordinate { monster_id, block },
                    );
                    continue;
                }
            };
            if tile_x < 0 || tile_x >= width || tile_y < 0 || tile_y >= height {
                continue;
            }

            let identity = shape.identity();
            let mut removal = CMessage::new(0xbf504);
            removal.base_mut().add_long(identity.object_type);
            removal.base_mut().add_long(identity.id);
            removal.base_mut().add_long(0);
            let _delivery = self.runtime.send_four_nation_clear_around(
                &removal,
                &region.war.base,
                shape,
                self.game,
            );
            region
                .war
                .base
                .find_monster_by_id_mut(monster_id)
                .expect("around send не удаляет Nation monster owner")
                .stage_for_delete();
        }

        for monster_id in region.war.base.sleeping_monster_ids() {
            let Some(monster) = region.war.base.find_monster_by_id(monster_id) else {
                continue;
            };
            if monster.staged_for_delete() {
                region.war.base.stage_delete_shape(ShapeIdentity {
                    object_type: 600,
                    id: monster_id,
                    ex_id: crate::public::guid::CGuid::GUID_INVALID,
                });
            }
        }

        let npc_name = self.game.get_string_by_id(b"GS1120").to_vec();
        for _ in 0..4 {
            let Some(npc_id) = self
                .runtime
                .find_four_nation_clear_npc_id(&region.war.base, &npc_name)
            else {
                continue;
            };
            let Some(npc) = region.war.base.find_npc_by_id(npc_id).filter(|npc| {
                npc.move_shape().shape().base_object().get_name() == npc_name.as_slice()
            }) else {
                self.runtime.on_four_nation_clear_block(
                    FourNationClearBlock::NpcTraversalMismatch { npc_id },
                );
                continue;
            };
            let shape = npc.move_shape().shape();
            let identity = shape.identity();
            let mut removal = CMessage::new(0xbf504);
            removal.base_mut().add_long(identity.object_type);
            removal.base_mut().add_long(identity.id);
            removal.base_mut().add_long(0);
            let _delivery = self.runtime.send_four_nation_clear_around(
                &removal,
                &region.war.base,
                shape,
                self.game,
            );
            if let Err(block) = region.war.base.remove_owned_npc_by_id(identity.id) {
                self.runtime
                    .on_four_nation_clear_block(FourNationClearBlock::NpcRemoval {
                        npc_id: identity.id,
                        block,
                    });
            }
        }
    }

    fn on_war_declare(&mut self, region: GameWarRegionHandle, war_number: i32) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                let Some(region) = self.game.find_region_mut(region_id) else {
                    return;
                };
                match region {
                    ServerRegionOwner::Village(region) => {
                        region.on_war_declare(war_number, self.runtime)
                    }
                    ServerRegionOwner::City(region) => {
                        region.on_war_declare(war_number, self.runtime)
                    }
                    ServerRegionOwner::Nation(region) => region.war.on_war_declare(war_number),
                    ServerRegionOwner::GodsBattle(region) => region.war.on_war_declare(war_number),
                    ServerRegionOwner::Base(region) => region.on_war_declare(war_number),
                    ServerRegionOwner::Country(region) => region.base.on_war_declare(war_number),
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.game.find_proxy_region_mut(region_id) {
                    region.on_war_declare(war_number);
                }
            }
        }
    }

    fn on_war_start(&mut self, region: GameWarRegionHandle, war_number: i32) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                let Some(region) = self.game.find_region_mut(region_id) else {
                    return;
                };
                match region {
                    ServerRegionOwner::Village(region) => {
                        region.on_war_start(war_number, self.runtime)
                    }
                    ServerRegionOwner::City(region) => {
                        region.on_war_start(war_number, self.runtime)
                    }
                    region => region.base_mut().on_war_start(war_number),
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.game.find_proxy_region_mut(region_id) {
                    region.on_war_start();
                }
            }
        }
    }

    fn on_war_time_out(&mut self, region: GameWarRegionHandle, war_number: i32) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        let Some(region) = self.game.find_region_mut(region_id) else {
            return;
        };
        match region {
            ServerRegionOwner::Village(region) => region.on_war_time_out(war_number, self.runtime),
            ServerRegionOwner::City(region) => region.on_war_time_out(war_number, self.runtime),
            region => region.base_mut().on_war_time_out(war_number),
        }
    }

    fn on_war_end(&mut self, region: GameWarRegionHandle, war_number: i32) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                let Some(region) = self.game.find_region_mut(region_id) else {
                    return;
                };
                match region {
                    ServerRegionOwner::Village(region) => {
                        region.on_war_end(war_number, self.runtime)
                    }
                    ServerRegionOwner::City(region) => region.on_war_end(war_number, self.runtime),
                    ServerRegionOwner::Nation(region) => region.war.on_war_end(war_number),
                    ServerRegionOwner::GodsBattle(region) => region.war.on_war_end(war_number),
                    region => region.base_mut().on_war_end(war_number),
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.game.find_proxy_region_mut(region_id) {
                    region.on_war_end();
                }
            }
        }
    }

    fn on_war_mass(&mut self, region: GameWarRegionHandle, war_number: i32) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                let Some(region) = self.game.find_region_mut(region_id) else {
                    return;
                };
                match region {
                    ServerRegionOwner::City(region) => region.on_war_mass(war_number, self.runtime),
                    region => region.base_mut().on_war_mass(war_number),
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.game.find_proxy_region_mut(region_id) {
                    region.on_war_mass();
                }
            }
        }
    }
}

impl<Runtime: GameOrganizingWarRuntime> WarFactionUpdateContext
    for GameOrganizingWarContext<'_, Runtime>
{
    fn update_attack_city_contend_player(&mut self, region_id: i32, schedules: &CAttackCitySys) {
        self.update_contenders(region_id, ContendSchedule::AttackCity(schedules));
    }

    fn update_village_contend_player(&mut self, region_id: i32, schedules: &CVillageWarSys) {
        self.update_contenders(region_id, ContendSchedule::Village(schedules));
    }
}

impl<Runtime: GameOrganizingWarRuntime> FourNationPhaseContext
    for GameOrganizingWarContext<'_, Runtime>
{
    type Region = GameWarRegionHandle;

    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region> {
        self.lookup_region_then_proxy(region_id)
    }

    fn find_server_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.lookup_server_region(region_id)
    }

    fn find_server_nation_region(&mut self, region_id: i32) -> Option<Self::Region> {
        matches!(
            self.game.find_region(region_id),
            Some(ServerRegionOwner::Nation(_))
        )
        .then_some(GameWarRegionHandle::Local(region_id))
    }

    fn on_war_declare(&mut self, region: Self::Region, war_number: i32) {
        GameOrganizingWarContext::on_war_declare(self, region, war_number);
    }

    fn on_nation_war_declare(
        &mut self,
        region: Self::Region,
        war_number: i32,
        sign_up_counts: [i32; 5],
    ) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        if let Some(ServerRegionOwner::Nation(region)) = self.game.find_region_mut(region_id) {
            region.war.on_war_declare(war_number);
            region.reset_for_war_declare();
            self.runtime
                .on_four_nation_declare(region, war_number, sign_up_counts);
        }
    }

    fn on_war_mass(&mut self, region: Self::Region, war_number: i32) {
        if let GameWarRegionHandle::Local(region_id) = region
            && let Some(owner) = self.game.take_region_owner(region_id)
        {
            let ServerRegionOwner::Nation(mut region) = owner else {
                self.game.restore_region_owner(owner);
                GameOrganizingWarContext::on_war_mass(
                    self,
                    GameWarRegionHandle::Local(region_id),
                    war_number,
                );
                return;
            };
            region.war.base.on_war_mass(war_number);
            self.kick_out_four_nation_players(&mut region);
            self.game
                .restore_region_owner(ServerRegionOwner::Nation(region));
            return;
        }
        GameOrganizingWarContext::on_war_mass(self, region, war_number);
    }

    fn on_war_start(&mut self, region: Self::Region, war_number: i32) {
        GameOrganizingWarContext::on_war_start(self, region, war_number);
    }

    fn on_refresh_region(&mut self, region: Self::Region, war_number: i32) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        let Some(region) = self.game.find_region_mut(region_id) else {
            return;
        };
        match region {
            ServerRegionOwner::Nation(region) => {
                region.reset_for_region_refresh();
                self.runtime.on_four_nation_refresh(region, war_number)
            }
            region => region.base_mut().on_refresh_region(war_number),
        }
    }

    fn on_war_end(&mut self, region: Self::Region, war_number: i32) {
        if let GameWarRegionHandle::Local(region_id) = region
            && let Some(owner) = self.game.take_region_owner(region_id)
        {
            let ServerRegionOwner::Nation(mut region) = owner else {
                self.game.restore_region_owner(owner);
                GameOrganizingWarContext::on_war_end(
                    self,
                    GameWarRegionHandle::Local(region_id),
                    war_number,
                );
                return;
            };

            self.runtime
                .add_four_nation_region_log(b"ServerNationRegion::OnWarEnd");
            region.war.on_war_end(war_number);

            let end = CMessage::new(0xbf819);
            let _end_delivery = end.send_to_region(Some(&region.war.base), None, self.game);
            self.kick_out_four_nation_players(&mut region);

            let awards = region.take_player_war_awards(|| self.runtime.four_nation_now_millis());
            for award in awards {
                let mut elapsed = CMessage::new(0x6031c);
                elapsed.base_mut().add_long(award.player_id);
                elapsed.base_mut().add_ulong(award.elapsed_time_ms);
                elapsed.base_mut().add_long(award.country);
                let _elapsed_delivery = elapsed.send(self.game, false);

                let log = format_legacy_integer_fields(
                    self.game.get_string_by_id(b"GS1134"),
                    &[
                        award.player_id.to_string(),
                        (award.elapsed_time_ms / 1000).to_string(),
                        award.exploit.to_string(),
                    ],
                    0x7f,
                );
                self.runtime.add_four_nation_region_log(&log);

                let Some(previous_exploit) = self
                    .game
                    .find_player(award.player_id)
                    .map(|player| player.base_properties().exploit)
                else {
                    let mut offline = CMessage::new(0x6031a);
                    offline.base_mut().add_long(award.player_id);
                    offline.base_mut().add_ulong(award.exploit);
                    let _offline_delivery = offline.send(self.game, false);
                    continue;
                };

                let advertised_exploit = previous_exploit.wrapping_add(award.exploit);
                let mut property = CMessage::new(0xbf80c);
                property.base_mut().add_long(award.player_id);
                property.base_mut().add_long(award.player_id);
                property.base_mut().add_str(Some(c"dwExploit"));
                property.base_mut().add_ulong(advertised_exploit);
                let _property_delivery =
                    property.send_to_player(self.game.net_server(), award.player_id);
                {
                    let player = self
                        .game
                        .find_player_mut(award.player_id)
                        .expect("online award player проверен до mutation");
                    let _mutation = player.set_exploit_property_value(advertised_exploit);
                    self.runtime.update_player_property(player);
                }

                let notice = format_legacy_integer_fields(
                    self.game.get_string_by_id(b"GS1135"),
                    &[award.exploit.to_string()],
                    0xff,
                );
                let notice = CString::new(notice)
                    .expect("localized FourNation exploit notice обрезан до NUL");
                let mut message = CMessage::new(0xbf806);
                message.base_mut().add_ulong(u32::MAX);
                message.base_mut().add_ulong(0xffff_0000);
                message.base_mut().add_str(Some(&notice));
                let _notice_delivery =
                    message.send_to_player(self.game.net_server(), award.player_id);
            }

            self.runtime
                .reset_four_nation_region_combat_state(&mut region, war_number);
            region.reset_materialized_war_state();
            self.game
                .restore_region_owner(ServerRegionOwner::Nation(region));
            return;
        }
        GameOrganizingWarContext::on_war_end(self, region, war_number);
    }

    fn on_clear_war(&mut self, region: Self::Region, _war_number: i32) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        let Some(owner) = self.game.take_region_owner(region_id) else {
            return;
        };
        let ServerRegionOwner::Nation(mut region) = owner else {
            self.game.restore_region_owner(owner);
            return;
        };
        self.clear_four_nation_war(&mut region);
        self.game
            .restore_region_owner(ServerRegionOwner::Nation(region));
    }

    fn take_war_results(&mut self, region: Self::Region) -> [u32; 5] {
        let GameWarRegionHandle::Local(region_id) = region else {
            return [0; 5];
        };
        let Some(ServerRegionOwner::Nation(region)) = self.game.find_region_mut(region_id) else {
            return [0; 5];
        };
        region.take_stone_guard_results()
    }

    fn add_war_end_log(&mut self, war_number: i32) {
        self.runtime.add_four_nation_war_end_log(war_number);
    }
}

impl<Runtime: GameOrganizingWarRuntime> AttackCityPhaseContext
    for GameOrganizingWarContext<'_, Runtime>
{
    type Region = GameWarRegionHandle;

    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region> {
        self.lookup_region_then_proxy(region_id)
    }

    fn find_server_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.lookup_server_region(region_id)
    }

    fn on_war_declare(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_declare(region, war_number);
    }

    fn on_war_start(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_start(region, war_number);
    }

    fn on_war_time_out(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_time_out(region, war_number);
    }

    fn on_war_end(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_end(region, war_number);
    }

    fn on_war_mass(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_mass(region, war_number);
    }

    fn on_clear_other_player(&mut self, region: Self::Region, war_number: i32) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        if let Some(ServerRegionOwner::City(region)) = self.game.find_region_mut(region_id) {
            region.on_clear_other_player(war_number, self.runtime);
        }
    }

    fn on_refresh_region(&mut self, region: Self::Region, war_number: i32) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        if let Some(ServerRegionOwner::City(region)) = self.game.find_region_mut(region_id) {
            region.on_refresh_region(war_number, self.runtime);
        }
    }
}

impl<Runtime: GameOrganizingWarRuntime> VillageWarPhaseContext
    for GameOrganizingWarContext<'_, Runtime>
{
    type Region = GameWarRegionHandle;

    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region> {
        self.lookup_region_then_proxy(region_id)
    }

    fn find_server_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.lookup_server_region(region_id)
    }

    fn owned_city_faction(&mut self, region: Self::Region) -> i32 {
        match region {
            GameWarRegionHandle::Local(region_id) => self
                .game
                .find_region(region_id)
                .map(|region| region.base().owned_city_faction())
                .unwrap_or(0),
            GameWarRegionHandle::Proxy(region_id) => self
                .game
                .find_proxy_region(region_id)
                .map(|region| region.owned_city_org().0)
                .unwrap_or(0),
        }
    }

    fn owned_city_union(&mut self, region: Self::Region) -> i32 {
        match region {
            GameWarRegionHandle::Local(region_id) => self
                .game
                .find_region(region_id)
                .map(|region| region.base().owned_city_union())
                .unwrap_or(0),
            GameWarRegionHandle::Proxy(region_id) => self
                .game
                .find_proxy_region(region_id)
                .map(|region| region.owned_city_org().1)
                .unwrap_or(0),
        }
    }

    fn set_owned_city_org(&mut self, region: Self::Region, faction_id: i32, union_id: i32) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                if let Some(region) = self.game.find_region_mut(region_id) {
                    region.base_mut().set_owned_city_org(faction_id, union_id);
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.game.find_proxy_region_mut(region_id) {
                    region.set_owned_city_org(faction_id, union_id);
                }
            }
        }
    }

    fn region_country(&self, region: Self::Region) -> u8 {
        match region {
            GameWarRegionHandle::Local(region_id) => self
                .game
                .find_region(region_id)
                .map(|region| region.base().country)
                .unwrap_or(0),
            GameWarRegionHandle::Proxy(region_id) => self
                .game
                .find_proxy_region(region_id)
                .map(|region| region.country())
                .unwrap_or(0),
        }
    }

    fn set_region_country(&mut self, region: Self::Region, country: u8) {
        match region {
            GameWarRegionHandle::Local(region_id) => {
                if let Some(region) = self.game.find_region_mut(region_id) {
                    region.base_mut().country = country;
                }
            }
            GameWarRegionHandle::Proxy(region_id) => {
                if let Some(region) = self.game.find_proxy_region_mut(region_id) {
                    region.set_country(country);
                }
            }
        }
    }

    fn on_war_declare(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_declare(region, war_number);
    }

    fn on_war_start(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_start(region, war_number);
    }

    fn on_war_time_out(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_time_out(region, war_number);
    }

    fn on_war_end(&mut self, region: Self::Region, war_number: i32) {
        self.on_war_end(region, war_number);
    }

    fn send_clear_player_notice(&mut self, region: Self::Region) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        let Some(owner) = self.game.take_region_owner(region_id) else {
            return;
        };
        let name = owner.name().to_vec();
        let text =
            format_legacy_text_fields(self.game.get_string_by_id(b"GS0127"), &[&name], 0x3ff);
        let _delivery = colored_player_notice_message(0xffda_edfe, 0, &text).send_to_region(
            Some(owner.base()),
            None,
            self.game,
        );
        self.game.restore_region_owner(owner);
    }

    fn start_clear_player_out(&mut self, region: Self::Region, delay_ms: i32) {
        let GameWarRegionHandle::Local(region_id) = region else {
            return;
        };
        let now_ms = VillageRegionContext::now_millis(self.runtime);
        if let Some(region) = self.game.find_region_mut(region_id) {
            region
                .base_mut()
                .start_clear_player_out_at(delay_ms, now_ms);
        }
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\organsysmessage.cpp

// ============================================================================
// FUNCTION: OnOrgasysMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\organsysmessage.cpp:28
// RVA: 0x000895A0
// ADDRESS: 004895a0
// PROTOTYPE: void __cdecl OnOrgasysMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049094e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\organsysmessage.cpp
// RVA: 0x0009094E
// ADDRESS: 0049094e
// PROTOTYPE: undefined Catch@0049094e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
