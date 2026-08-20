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
//! Другие opcodes helpers не интерпретируют.

use std::error::Error;
use std::fmt;

use super::super::organizingsystem::attackcitysys::{
    AttackCityDecodeError, AttackCityFactionUpdateContext, AttackCityPhaseContext, CAttackCitySys,
};
use super::super::organizingsystem::villagewarsys::{
    CVillageWarSys, VillageWarDecodeError, VillageWarFactionUpdateContext, VillageWarPhaseContext,
};

pub(crate) trait WarFactionUpdateContext {
    type Region: Copy;

    /// Ищет только non-null entry в `CGame::s_mapRegion`.
    fn find_server_region(&mut self, region_id: i32) -> Option<Self::Region>;

    fn update_contend_player(&mut self, region: Self::Region);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WarFactionUpdateDispatchError {
    AttackCity(AttackCityDecodeError),
    Village(VillageWarDecodeError),
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
) -> Result<bool, WarFactionUpdateDispatchError> {
    match opcode {
        0x7fe35 => {
            let mut adapter = AttackCityContextAdapter(context);
            let _ = attack_city_sys
                .update_apply_war_factions(payload, cursor, &mut adapter)
                .map_err(WarFactionUpdateDispatchError::AttackCity)?;
            Ok(true)
        }
        0x7fe36 => {
            let mut adapter = VillageWarContextAdapter(context);
            let _ = village_war_sys
                .update_apply_war_factions(payload, cursor, &mut adapter)
                .map_err(WarFactionUpdateDispatchError::Village)?;
            Ok(true)
        }
        _ => Ok(false),
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
) -> Result<bool, WarPhaseDispatchError>
where
    Context: AttackCityPhaseContext + VillageWarPhaseContext,
{
    if !matches!(opcode, 0x7fe1f..=0x7fe25 | 0x7fe2f..=0x7fe33) {
        return Ok(false);
    }

    let war_number = read_phase_war_number(payload, cursor)?;
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
    Ok(true)
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

struct AttackCityContextAdapter<'a, Context>(&'a mut Context);

impl<Context: WarFactionUpdateContext> AttackCityFactionUpdateContext
    for AttackCityContextAdapter<'_, Context>
{
    type Region = Context::Region;

    fn find_server_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.0.find_server_region(region_id)
    }

    fn update_contend_player(&mut self, region: Self::Region) {
        self.0.update_contend_player(region);
    }
}

struct VillageWarContextAdapter<'a, Context>(&'a mut Context);

impl<Context: WarFactionUpdateContext> VillageWarFactionUpdateContext
    for VillageWarContextAdapter<'_, Context>
{
    type Region = Context::Region;

    fn find_server_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.0.find_server_region(region_id)
    }

    fn update_contend_player(&mut self, region: Self::Region) {
        self.0.update_contend_player(region);
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
