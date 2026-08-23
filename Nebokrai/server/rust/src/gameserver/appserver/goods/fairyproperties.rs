//! Свойства обычной феи исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/goods/fairyproperties.cpp`. Constructor и
//! `ExpUp/LevelUp` перенесены целиком: wrapping experience, границы egg/ripe
//! level, числовой приоритет result, четыре main-ability формулы на `f32` и
//! world-log `0x60210`.
//!
//! Legacy `m_plExp` был nullable указателем внутрь addon storage `CGoods`.
//! Rust хранит linked value как `Option<i32>` и возвращает typed block только
//! когда exact loop действительно попытался бы разыменовать null; adapter
//! загрузки/сохранения товара остаётся владельцем синхронизации. Таблица опыта,
//! globe setup и log setup передаются явно. MSVC `ROUND -> long long -> int`
//! заменён `f32::round -> i64 -> i32`, включая truncation младших 32 бит.

pub(crate) const FAIRY_GROW_WORLD_MESSAGE_TYPE: u32 = 0x60210;

const FAIRY_MAIN_STRENGTH: u32 = 0x75;
const FAIRY_MAIN_AGILITY: u32 = 0x76;
const FAIRY_MAIN_WAKAN: u32 = 0x77;
const FAIRY_MAIN_HP: u32 = 0x78;
const MAX_LEGACY_GUID_BYTES: usize = 63;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum FairyExpUpResult {
    #[default]
    None = 0,
    ExpUp = 1,
    LevelUp = 2,
    ChangeState = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FairyExpBlock {
    GuidExceedsLegacyBuffer { length: usize },
    ExperienceUnlinked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyGrowLog {
    pub(crate) player_id: i32,
    pub(crate) log_value: i32,
    pub(crate) fairy_guid: Vec<u8>,
    pub(crate) fairy_name: Vec<u8>,
    pub(crate) level: u32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FairyExpRuntime<'a> {
    pub(crate) player_id: i32,
    pub(crate) fairy_guid: &'a [u8],
    pub(crate) fairy_name: &'a [u8],
    pub(crate) log_value: i32,
    pub(crate) suppress_grow_log: bool,
    pub(crate) grow_log_enabled: bool,
    pub(crate) egg_max_level: u32,
    pub(crate) upgrade_rate: f32,
}

#[must_use = "report содержит remaining experience и ordered world-log effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FairyExpReport {
    pub(crate) result: FairyExpUpResult,
    pub(crate) remaining_experience: u32,
    pub(crate) grow_logs: Vec<FairyGrowLog>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CFairyProperties {
    experience: Option<i32>,
    pub(crate) max_exp: u32,
    pub(crate) fairy_state: u32,
    pub(crate) equip_level: u32,
    pub(crate) level: u32,
    pub(crate) ripe_min_level: u32,
    pub(crate) ripe_max_level: u32,
    pub(crate) combinated_times: u32,
    pub(crate) max_combinated_times: u32,
    pub(crate) main_ability: u32,
    pub(crate) growing_rate: u32,
    pub(crate) strength: u32,
    pub(crate) agility: u32,
    pub(crate) wakan: u32,
    pub(crate) hp: u32,
    pub(crate) base_strength: u32,
    pub(crate) base_agility: u32,
    pub(crate) base_wakan: u32,
    pub(crate) base_hp: u32,
    pub(crate) egg_id: u32,
    pub(crate) young_id: u32,
    pub(crate) ripe_id: u32,
    pub(crate) hatch_start_time: u32,
}

impl CFairyProperties {
    pub(crate) const fn new() -> Self {
        Self {
            experience: None,
            max_exp: 0,
            fairy_state: 0,
            equip_level: 0,
            level: 0,
            ripe_min_level: 0,
            ripe_max_level: 0,
            combinated_times: 0,
            max_combinated_times: 0,
            main_ability: 0,
            growing_rate: 0,
            strength: 0,
            agility: 0,
            wakan: 0,
            hp: 0,
            base_strength: 0,
            base_agility: 0,
            base_wakan: 0,
            base_hp: 0,
            egg_id: 0,
            young_id: 0,
            ripe_id: 0,
            hatch_start_time: 0,
        }
    }

    pub(crate) const fn experience(&self) -> Option<i32> {
        self.experience
    }

    pub(crate) const fn link_experience(&mut self, experience: i32) {
        self.experience = Some(experience);
    }

    pub(crate) const fn unlink_experience(&mut self) {
        self.experience = None;
    }

    pub(crate) fn exp_up<Threshold>(
        &mut self,
        experience: &mut u32,
        runtime: FairyExpRuntime<'_>,
        mut threshold_for_level: Threshold,
    ) -> Result<FairyExpReport, FairyExpBlock>
    where
        Threshold: FnMut(u32, u32) -> u32,
    {
        let fairy_guid = visible_c_bytes(runtime.fairy_guid);
        if MAX_LEGACY_GUID_BYTES < fairy_guid.len() {
            return Err(FairyExpBlock::GuidExceedsLegacyBuffer {
                length: fairy_guid.len(),
            });
        }
        let fairy_name = visible_c_bytes(runtime.fairy_name);
        let mut result = FairyExpUpResult::None;
        let mut grow_logs = Vec::new();

        while *experience != 0
            && (self.fairy_state != 0 || self.level < runtime.egg_max_level)
            && self.level < self.ripe_max_level
        {
            let Some(current_experience) = self.experience else {
                return Err(FairyExpBlock::ExperienceUnlinked);
            };
            let accumulated = (current_experience as u32).wrapping_add(*experience);
            self.experience = Some(accumulated as i32);
            if accumulated < self.max_exp {
                *experience = 0;
                result = result.max(FairyExpUpResult::ExpUp);
                continue;
            }

            *experience = accumulated.wrapping_sub(self.max_exp);
            let level_result = self.level_up(
                runtime,
                &fairy_guid,
                &fairy_name,
                &mut threshold_for_level,
                &mut grow_logs,
            );
            result = result.max(level_result);
            self.experience = Some(0);
        }

        Ok(FairyExpReport {
            result,
            remaining_experience: *experience,
            grow_logs,
        })
    }

    fn level_up<Threshold>(
        &mut self,
        runtime: FairyExpRuntime<'_>,
        fairy_guid: &[u8],
        fairy_name: &[u8],
        threshold_for_level: &mut Threshold,
        grow_logs: &mut Vec<FairyGrowLog>,
    ) -> FairyExpUpResult
    where
        Threshold: FnMut(u32, u32) -> u32,
    {
        self.level = self.level.wrapping_add(1);
        self.max_exp = threshold_for_level(self.equip_level, self.level);
        if !runtime.suppress_grow_log && runtime.grow_log_enabled {
            grow_logs.push(FairyGrowLog {
                player_id: runtime.player_id,
                log_value: runtime.log_value,
                fairy_guid: fairy_guid.to_vec(),
                fairy_name: fairy_name.to_vec(),
                level: self.level,
            });
        }

        let result = if self.level == self.ripe_min_level {
            FairyExpUpResult::ChangeState
        } else {
            FairyExpUpResult::LevelUp
        };
        if self.fairy_state == 0 {
            return result;
        }
        let main = match self.main_ability {
            FAIRY_MAIN_STRENGTH => 0,
            FAIRY_MAIN_AGILITY => 1,
            FAIRY_MAIN_WAKAN => 2,
            FAIRY_MAIN_HP => 3,
            _ => return result,
        };

        self.strength = self.strength.wrapping_add(growth_delta(
            self.base_strength,
            self.growing_rate,
            runtime.upgrade_rate,
            main == 0,
        ));
        self.agility = self.agility.wrapping_add(growth_delta(
            self.base_agility,
            self.growing_rate,
            runtime.upgrade_rate,
            main == 1,
        ));
        self.wakan = self.wakan.wrapping_add(growth_delta(
            self.base_wakan,
            self.growing_rate,
            runtime.upgrade_rate,
            main == 2,
        ));
        self.hp = self.hp.wrapping_add(growth_delta(
            self.base_hp,
            self.growing_rate,
            runtime.upgrade_rate,
            main == 3,
        ));
        result
    }
}

fn growth_delta(base: u32, growing_rate: u32, upgrade_rate: f32, main: bool) -> u32 {
    let growth = growing_rate as f32 * 0.0001_f32;
    let mut value = base as f32 * 0.0001_f32;
    if !main {
        value *= upgrade_rate;
    }
    value *= growth;
    (value.round() as i64 as i32) as u32
}

fn visible_c_bytes(bytes: &[u8]) -> Vec<u8> {
    bytes[..bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len())]
        .to_vec()
}
