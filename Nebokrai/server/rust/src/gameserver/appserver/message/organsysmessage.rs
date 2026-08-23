//! Владелец GameServer dispatcher-а organizing messages `OnOrgasysMessage`.
//!
//! Весь dispatcher RVA `0x000895A0` остаётся `UNKNOWN` (исследовательский декомпилят хранится локально), кроме фазовых
//! cases AttackCity `0x7FE1F..0x7FE25`, Village `0x7FE2F..0x7FE33` и faction
//! update `0x7FE35/0x7FE36` со статусом `IMPLEMENTED`. Точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`; исходник
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\organsysmessage.cpp`.
//!
//! Каждый фазовый case читает ровно один signed war ID и передаёт его своему
//! owner-у. Faction-update cases передают текущие payload/cursor соответствующему
//! `UpdateApplyWarFacs` и игнорируют legacy bool, как исходный switch. Известный
//! opcode считается обработанным даже при отсутствующем schedule; safe
//! short-buffer возвращается локальной ошибкой без придуманного UB-эффекта.
//! Другие opcodes helpers не интерпретируют. Достигнутая war family проходит
//! живой FIFO `CGame`: расписания остаются owned, local-before-proxy lookup
//! мутирует concrete City/Village/base owners, а message/player/log effects
//! исполняет тот же runtime-контекст, который обслуживает MainLoop.

use std::error::Error;
use std::fmt;

use super::super::organizingsystem::attackcitysys::{
    AttackCityDecodeError, AttackCityMembershipBlock, AttackCityPhaseContext, CAttackCitySys,
};
use super::super::organizingsystem::villagewarsys::{
    CVillageWarSys, VillageWarDecodeError, VillageWarPhaseContext,
};
use super::super::servercityregion::CityRegionContext;
use super::super::servervillageregion::VillageRegionContext;
use super::super::serverwarregion::WarRegionContext;
use crate::gameserver::gameserver::game::{CGame, GameWarRegionHandle, ServerRegionOwner};
use crate::nets::netserver::message::CMessage;

pub(crate) trait GameOrganizingWarRuntime: CityRegionContext + VillageRegionContext {
    /// Публикует region-localized `0xBF806(..., GS0127(region name))`.
    fn send_village_clear_player_notice(&mut self, region_id: i32, region_name: &[u8]);
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameOrganizingWarMessageReport {
    FactionUpdate(WarFactionUpdateDispatchReport),
    Phase(WarPhaseDispatchReport),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameOrganizingWarMessageError {
    FactionUpdate(WarFactionUpdateDispatchError),
    Phase(WarPhaseDispatchError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WarPhaseDispatchError {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
}

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

/// Подключает всю достигнутую OrganSys war family к живому `CGame` owner-у.
pub(crate) fn dispatch_game_organizing_war_message<Runtime: GameOrganizingWarRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<GameOrganizingWarMessageReport, GameOrganizingWarMessageError>> {
    let opcode = message.message_type() as u32;
    if !matches!(
        opcode,
        0x7fe1f..=0x7fe25 | 0x7fe2f..=0x7fe33 | 0x7fe35 | 0x7fe36
    ) {
        return None;
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
            .map(GameOrganizingWarMessageReport::FactionUpdate)
            .map_err(GameOrganizingWarMessageError::FactionUpdate)
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
            .map(GameOrganizingWarMessageReport::Phase)
            .map_err(GameOrganizingWarMessageError::Phase)
        }
    };
    game.restore_war_startup_owners(owners);
    Some(result)
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
        let Some(name) = self
            .game
            .find_region(region_id)
            .map(|region| region.name().to_vec())
        else {
            return;
        };
        self.runtime
            .send_village_clear_player_notice(region_id, &name);
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
