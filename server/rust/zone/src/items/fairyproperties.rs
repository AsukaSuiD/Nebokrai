//! Свойства обычной феи исторического GameServer: constructor, `ExpUp` и
//! `LevelUp` с main-ability формулами поверх addon storage `CGoods`.
//!
//! Упорядоченный журнал эффектов роста параметризован trait-швом
//! [`FairyGrowEffectSink`]: report [`FairyExpReport`] и накопление в
//! `exp_up/level_up` получают generic `Effects` вместо конкретного журнала;
//! конверт в прежний журнал и его alias остаются у старого пакета в точке
//! доставки эффектов.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/goods/fairyproperties.cpp`. Constructor и
//! `ExpUp/LevelUp` перенесены целиком: wrapping experience, границы egg/ripe
//! level, числовой приоритет result, четыре main-ability формулы и
//! упорядоченный эффект журнала роста. Его точное World-сообщение `0x60210`
//! исполняет канонический `CGame` после завершения изменения.
//!
//! Legacy `m_plExp` был nullable указателем внутрь addon storage `CGoods`.
//! Rust хранит linked value как `Option<i32>` и возвращает typed block только
//! когда exact loop действительно попытался бы разыменовать null; adapter
//! загрузки/сохранения товара остаётся владельцем синхронизации. Таблица опыта,
//! globe setup и log setup передаются явно. `LevelUp` масштабирует base и
//! growing rate double-константой `0.0001`, оставляет upgrade rate исходным
//! `float`, а FISTP усекает результат к нулю в `long long`; в свойство
//! прибавляются его младшие 32 бита.

pub const FAIRY_GROW_WORLD_MESSAGE_TYPE: u32 = 0x60210;

const FAIRY_MAIN_STRENGTH: u32 = 0x75;
const FAIRY_MAIN_AGILITY: u32 = 0x76;
const FAIRY_MAIN_WAKAN: u32 = 0x77;
const FAIRY_MAIN_HP: u32 = 0x78;
const MAX_LEGACY_GUID_BYTES: usize = 63;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum FairyExpUpResult {
    #[default]
    None = 0,
    ExpUp = 1,
    LevelUp = 2,
    ChangeState = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FairyExpBlock {
    GuidExceedsLegacyBuffer { length: usize },
    ExperienceUnlinked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FairyGrowLog {
    pub player_id: i32,
    pub log_value: i32,
    pub fairy_guid: Vec<u8>,
    pub fairy_name: Vec<u8>,
    pub level: u32,
}

/// Шов переноса: упорядоченный приёмник эффектов роста феи прежнего уровня.
/// Реализация прежнего владельца — `GameEffectJournal` старого пакета
/// (конверт `FairyGrowLog → GameEffect` остаётся у него); тела накопления
/// обращаются через эту typed-границу с исходным именем операции.
pub trait FairyGrowEffectSink: Default {
    fn push(&mut self, log: FairyGrowLog);
}

#[derive(Clone, Copy, Debug)]
pub struct FairyExpRuntime<'a> {
    pub player_id: i32,
    pub fairy_guid: &'a [u8],
    pub fairy_name: &'a [u8],
    pub log_value: i32,
    pub suppress_grow_log: bool,
    pub grow_log_enabled: bool,
    pub egg_max_level: u32,
    pub upgrade_rate: f32,
}

#[must_use = "результат содержит остаток опыта и упорядоченные World-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FairyExpReport<Effects> {
    pub result: FairyExpUpResult,
    pub remaining_experience: u32,
    pub effects: Effects,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CFairyProperties {
    experience: Option<i32>,
    pub max_exp: u32,
    pub fairy_state: u32,
    pub equip_level: u32,
    pub level: u32,
    pub ripe_min_level: u32,
    pub ripe_max_level: u32,
    pub combinated_times: u32,
    pub max_combinated_times: u32,
    pub main_ability: u32,
    pub growing_rate: u32,
    pub strength: u32,
    pub agility: u32,
    pub wakan: u32,
    pub hp: u32,
    pub base_strength: u32,
    pub base_agility: u32,
    pub base_wakan: u32,
    pub base_hp: u32,
    pub egg_id: u32,
    pub young_id: u32,
    pub ripe_id: u32,
    pub hatch_start_time: u32,
}

impl CFairyProperties {
    pub const fn new() -> Self {
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

    pub const fn experience(&self) -> Option<i32> {
        self.experience
    }

    pub const fn link_experience(&mut self, experience: i32) {
        self.experience = Some(experience);
    }

    pub const fn unlink_experience(&mut self) {
        self.experience = None;
    }

    pub fn exp_up<Threshold, Effects>(
        &mut self,
        experience: &mut u32,
        runtime: FairyExpRuntime<'_>,
        mut threshold_for_level: Threshold,
    ) -> Result<FairyExpReport<Effects>, FairyExpBlock>
    where
        Threshold: FnMut(u32, u32) -> u32,
        Effects: FairyGrowEffectSink,
    {
        let fairy_guid = visible_c_bytes(runtime.fairy_guid);
        if MAX_LEGACY_GUID_BYTES < fairy_guid.len() {
            return Err(FairyExpBlock::GuidExceedsLegacyBuffer {
                length: fairy_guid.len(),
            });
        }
        let fairy_name = visible_c_bytes(runtime.fairy_name);
        let mut result = FairyExpUpResult::None;
        let mut effects = Effects::default();

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
                &mut effects,
            );
            result = result.max(level_result);
            self.experience = Some(0);
        }

        Ok(FairyExpReport {
            result,
            remaining_experience: *experience,
            effects,
        })
    }

    fn level_up<Threshold, Effects>(
        &mut self,
        runtime: FairyExpRuntime<'_>,
        fairy_guid: &[u8],
        fairy_name: &[u8],
        threshold_for_level: &mut Threshold,
        effects: &mut Effects,
    ) -> FairyExpUpResult
    where
        Threshold: FnMut(u32, u32) -> u32,
        Effects: FairyGrowEffectSink,
    {
        self.level = self.level.wrapping_add(1);
        self.max_exp = threshold_for_level(self.equip_level, self.level);
        if !runtime.suppress_grow_log && runtime.grow_log_enabled {
            effects.push(FairyGrowLog {
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
    let growth = f64::from(growing_rate) * 0.0001_f64;
    let mut value = f64::from(base) * 0.0001_f64;
    if !main {
        value *= f64::from(upgrade_rate);
    }
    value *= growth;
    (value.trunc() as i64 as i32) as u32
}

fn visible_c_bytes(bytes: &[u8]) -> Vec<u8> {
    bytes[..bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len())]
        .to_vec()
}
