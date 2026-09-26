//! Данные и скалярные правила nation war-региона `ServerNationRegion`
//! исторического GameServer, перенесённые в Zone `regions/` волной Z-M-X
//! (семья регионов country+nation+city + гейты). Исходный владелец —
//! `appserver/servernationregion.h/.cpp`. Переходный агрегат `ServerNationRegion`
//! остаётся в старом пакете: хранит hub `CServerWarRegion` и делегирует этому
//! агрегату весь чистый state и его скалярные операции без изменения
//! сигнатур; inherited war decoder и evidence-блок остаются у старого пакета.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2). Статусы достигнутых тел (`First-hit guard state`, morale mutation,
//! `x87`-цепочка morale→exploit `0xF1230`, contend timing/damage arithmetic,
//! exact constructor/`OnWarDeclare`/`OnRefreshRegion` lifecycle) сохранены из
//! шапки старого владельца без повышения; startup `RT_NATION` selector
//! относится к границе inherited decoder-а старого пакета (ADR-0008) и не
//! читается этим модулем.
//!
//! Линейный owned `Vec` заменяет MSVC `stdext::hash_map` и `m_listContend`:
//! lookup-семантика и порядок сохранены, недоказанный bucket-order hash-обхода
//! не выдаётся за gameplay-контракт. `DWORD`-время и signed умножения
//! сохраняют wrapping. `GetDiedStateTime` (seconds→milliseconds ×0.5 через
//! signed GlobeSetup field), creation NPC и сетевые side effects исполнения
//! остаются у вызывающего `CGame` старого пакета.

use crate::activities::fournationwarsys::FourNationRect;

/// Скалярный state nation war-региона без hub `CServerWarRegion`:
/// country names, принадлежность флага, приоритизованный contend-spisок,
/// relive rectangles, morale/failed-флаги, treasure-counters, first-hit
/// и die-флаги монстров и player-timing запись `_tagPlayerWarTime`.
/// Exact constructor/`OnWarDeclare`/`OnRefreshRegion` lifecycle фиксирует
/// пять `m_bShenShouDie`; `OnWarEnd` намеренно их не трогает.
#[derive(Debug, Eq, PartialEq)]
pub struct ServerNationRegionState {
    country_names: [Vec<u8>; 5],
    flag_belong_to_id: i32,
    contenders: Vec<NationContend>,
    relive_rects: [FourNationRect; 5],
    morale: [i32; 5],
    lost_morale: [i32; 5],
    nation_failed: [bool; 5],
    treasure_boxes: [i32; 5],
    magic_stone_attacked: [bool; 5],
    jin_wei_jun_attacked: [bool; 5],
    guard_attacked: [bool; 5],
    guard_first_die: [bool; 5],
    _shen_shou_died: [bool; 5],
    stone_guard_died: [u32; 5],
    yu_ying_shi_added: [bool; 5],
    da_jiang_jun_died: [bool; 5],
    player_war_times: Vec<NationPlayerWarTime>,
}

impl Default for ServerNationRegionState {
    fn default() -> Self {
        Self {
            country_names: std::array::from_fn(|_| Vec::new()),
            flag_belong_to_id: 0,
            contenders: Vec::new(),
            relive_rects: [FourNationRect::default(); 5],
            morale: [1000; 5],
            lost_morale: [0; 5],
            nation_failed: [false; 5],
            treasure_boxes: [0; 5],
            magic_stone_attacked: [false; 5],
            jin_wei_jun_attacked: [false; 5],
            guard_attacked: [false; 5],
            guard_first_die: [false; 5],
            _shen_shou_died: [false; 5],
            stone_guard_died: [0; 5],
            yu_ying_shi_added: [false; 5],
            da_jiang_jun_died: [false; 5],
            player_war_times: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NationMoraleTarget {
    StoneGuard,
    Guard,
    Admiral,
    MagicStone,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NationMonsterDamageNotice {
    StoneGuard {
        defender_country: u8,
        attacker_country: u8,
    },
    JinWeiJun {
        defender_country: u8,
        attacker_country: u8,
    },
    MagicStone {
        defender_country: u8,
        attacker_country: u8,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NationMoraleMutation {
    pub defender_country: u8,
    pub attacker_country: u8,
    pub target: NationMoraleTarget,
    pub defender_morale: i32,
    pub attacker_morale: i32,
    pub lost_morale: i32,
    pub first_guard_notice: bool,
    pub check_morale_spawn: bool,
    pub check_admiral_spawn: bool,
    pub nation_failed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NationCarriageReturnMutation {
    pub country: u8,
    pub treasure_boxes: i32,
    pub morale: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NationCarriageReturnOutcome {
    CountryOutsideNation,
    NationFailed,
    Applied(NationCarriageReturnMutation),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NationContend {
    pub player_id: i32,
    pub country_id: i32,
    pub current_time: i32,
    pub max_time: i32,
    pub start_time_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NationContendDamageMutation {
    pub player_id: i32,
    pub current_time: i32,
    pub percentage: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NationContendArithmeticBlock {
    pub current_time: i32,
    pub max_time: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NationContendAdvance {
    pub progress: Vec<(i32, i32)>,
    pub completed: Option<NationContend>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NationContendCaptureMutation {
    pub country: u8,
    pub morale: i32,
    pub cancelled_player_ids: Vec<i32>,
}

/// Exact string-table classificator: grouped `GS` string IDs отображают
/// original monster name на morale-target; порядок проверки групп исходный.
pub fn classify_nation_morale_target(
    monster_original_name: &[u8],
    mut string_by_id: impl FnMut(&[u8]) -> Vec<u8>,
) -> Option<NationMoraleTarget> {
    const GROUPS: &[(NationMoraleTarget, &[&[u8]])] = &[
        (
            NationMoraleTarget::StoneGuard,
            &[b"GS1088", b"GS1096", b"GS1104", b"GS1112"],
        ),
        (
            NationMoraleTarget::Guard,
            &[
                b"GS1095", b"GS1103", b"GS1111", b"GS1119", b"GS1092", b"GS1100", b"GS1108",
                b"GS1116", b"GS1094", b"GS1102", b"GS1110", b"GS1118",
            ],
        ),
        (
            NationMoraleTarget::Admiral,
            &[b"GS1091", b"GS1099", b"GS1107", b"GS1115"],
        ),
        (
            NationMoraleTarget::MagicStone,
            &[b"GS1142", b"GS1139", b"GS1140", b"GS1141"],
        ),
    ];
    GROUPS.iter().find_map(|(target, ids)| {
        ids.iter()
            .any(|id| string_by_id(id) == monster_original_name)
            .then_some(*target)
    })
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NationPlayerWarTime {
    pub player_id: i32,
    pub country: i32,
    pub country_figure: i32,
    pub start_time_ms: u32,
    pub elapsed_time_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NationPlayerWarAward {
    pub player_id: i32,
    pub country: i32,
    pub elapsed_time_ms: u32,
    pub exploit: u32,
}

impl ServerNationRegionState {
    /// Exact `GetReliveRect` копирует все пять country rectangles в owner.
    pub const fn set_relive_rects(&mut self, rects: [FourNationRect; 5]) {
        self.relive_rects = rects;
    }

    pub const fn relive_rects(&self) -> &[FourNationRect; 5] {
        &self.relive_rects
    }

    pub fn set_country_names(&mut self, country_names: [Vec<u8>; 5]) {
        self.country_names = country_names;
    }

    pub fn country_name(&self, country: u8) -> &[u8] {
        self.country_names
            .get(usize::from(country))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub const fn morale(&self) -> &[i32; 5] {
        &self.morale
    }

    pub const fn nation_failed(&self) -> &[bool; 5] {
        &self.nation_failed
    }

    /// Exact `IsNationFail`: только страны `1..4` индексируют flag-array.
    pub const fn is_nation_fail(&self, country: i32) -> bool {
        if country <= 0 || country >= 5 {
            return false;
        }
        self.nation_failed[country as usize]
    }

    pub fn treasure_box_count(&self, country: u8) -> i32 {
        self.treasure_boxes
            .get(usize::from(country))
            .copied()
            .unwrap_or_default()
    }

    /// Region battle owner публикует текущую morale без clamp:
    /// исходные combat callbacks пишут signed `long` напрямую.
    pub const fn set_country_morale(&mut self, country: usize, morale: i32) -> bool {
        if country >= self.morale.len() {
            return false;
        }
        self.morale[country] = morale;
        true
    }

    pub const fn set_nation_failed(&mut self, country: usize, failed: bool) -> bool {
        if country >= self.nation_failed.len() {
            return false;
        }
        self.nation_failed[country] = failed;
        true
    }

    /// Exact materialized subset `OnWarDeclare`: regional timing and combat
    /// counters start empty; signup counts сам owner в этой функции не читает.
    pub fn reset_for_war_declare(&mut self) {
        self.flag_belong_to_id = 0;
        self.player_war_times.clear();
        self.lost_morale.fill(0);
        self.nation_failed.fill(false);
        self.treasure_boxes.fill(0);
        self.magic_stone_attacked.fill(false);
        self.jin_wei_jun_attacked.fill(false);
        self.guard_attacked.fill(false);
        self.guard_first_die.fill(false);
        self._shen_shou_died.fill(false);
        self.stone_guard_died.fill(0);
        self.yu_ying_shi_added.fill(false);
        self.da_jiang_jun_died.fill(false);
    }

    /// Exact materialized prefix `OnRefreshRegion`: morale всех five slots
    /// становится 1000, failed flags и regional timing очищаются.
    pub fn reset_for_region_refresh(&mut self) {
        self.flag_belong_to_id = 0;
        self.player_war_times.clear();
        self.morale.fill(1000);
        self.lost_morale.fill(0);
        self.nation_failed.fill(false);
        self.treasure_boxes.fill(0);
        self.magic_stone_attacked.fill(false);
        self.jin_wei_jun_attacked.fill(false);
        self.guard_attacked.fill(false);
        self.guard_first_die.fill(false);
        self._shen_shou_died.fill(false);
        self.stone_guard_died.fill(0);
        self.yu_ying_shi_added.fill(false);
        self.da_jiang_jun_died.fill(false);
    }

    /// Exact `OnPlayerTimgingStart`: repeated start меняет только clock,
    /// не переснимая country/identity и не обнуляя elapsed.
    pub fn start_player_timing(
        &mut self,
        player_id: i32,
        country: i32,
        country_figure: i32,
        now_ms: u32,
    ) {
        if let Some(record) = self
            .player_war_times
            .iter_mut()
            .find(|record| record.player_id == player_id)
        {
            record.start_time_ms = now_ms;
            return;
        }
        self.player_war_times.push(NationPlayerWarTime {
            player_id,
            country,
            country_figure,
            start_time_ms: now_ms,
            elapsed_time_ms: 0,
        });
    }

    pub fn has_player_timing(&self, player_id: i32) -> bool {
        self.player_war_times
            .iter()
            .any(|record| record.player_id == player_id)
    }

    /// Exact `OnPlayerTimeingFinish`: missing/inactive records не меняются;
    /// death penalty добавляется после elapsed с `DWORD` wrapping.
    pub fn finish_player_timing(
        &mut self,
        player_id: i32,
        died: bool,
        now_ms: impl FnOnce() -> u32,
    ) -> Option<NationPlayerWarTime> {
        let record = self
            .player_war_times
            .iter_mut()
            .find(|record| record.player_id == player_id && record.start_time_ms != 0)?;
        let now_ms = now_ms();
        record.elapsed_time_ms = record
            .elapsed_time_ms
            .wrapping_add(now_ms.wrapping_sub(record.start_time_ms));
        record.start_time_ms = 0;
        if died {
            record.elapsed_time_ms = record.elapsed_time_ms.wrapping_add(30_000);
        }
        Some(*record)
    }

    /// Exact `ConvertMoraleToExploitForEachPlayer` state pass: каждый active
    /// record сам читает `timeGetTime`, затем owner-map очищается.
    pub fn take_player_war_awards(
        &mut self,
        mut now_ms: impl FnMut() -> u32,
    ) -> Vec<NationPlayerWarAward> {
        let mut awards = Vec::with_capacity(self.player_war_times.len());
        for mut record in self.player_war_times.drain(..) {
            if record.start_time_ms != 0 {
                let now_ms = now_ms();
                record.elapsed_time_ms = record
                    .elapsed_time_ms
                    .wrapping_add(now_ms.wrapping_sub(record.start_time_ms));
                record.start_time_ms = 0;
            }
            let morale = self
                .morale
                .get(record.country as usize)
                .copied()
                .unwrap_or_default();
            awards.push(NationPlayerWarAward {
                player_id: record.player_id,
                country: record.country,
                elapsed_time_ms: record.elapsed_time_ms,
                exploit: convert_morale_to_exploit(
                    record.elapsed_time_ms,
                    morale,
                    record.country_figure,
                ),
            });
        }
        awards
    }

    /// Materialized subset final reset-loop `OnWarEnd`: все пять slots,
    /// включая unused index 0, обнуляются после award pass.
    pub fn reset_materialized_war_state(&mut self) {
        self.flag_belong_to_id = 0;
        self.lost_morale.fill(0);
        self.nation_failed.fill(false);
        self.treasure_boxes.fill(0);
        self.magic_stone_attacked.fill(false);
        self.jin_wei_jun_attacked.fill(false);
        self.guard_attacked.fill(false);
        self.guard_first_die.fill(false);
        self._shen_shou_died.fill(false);
        self.stone_guard_died.fill(0);
        self.yu_ying_shi_added.fill(false);
        self.da_jiang_jun_died.fill(false);
    }

    pub fn take_stone_guard_results(&mut self) -> [u32; 5] {
        std::mem::take(&mut self.stone_guard_died)
    }

    /// Exact prefix `AI`: страны обходятся `1..=4`, gate требует именно
    /// `stoneGuardDie == 4 && admiralDied`, после вызова replacement счётчик
    /// guards обнуляется независимо от того, найден ли исходный NPC.
    pub fn take_due_magic_stone_transitions(&mut self) -> Vec<u8> {
        let mut countries = Vec::new();
        for country in 1..=4 {
            if self.stone_guard_died[country] == 4 && self.da_jiang_jun_died[country] {
                countries.push(country as u8);
                self.stone_guard_died[country] = 0;
            }
        }
        countries
    }

    pub fn apply_monster_morale(
        &mut self,
        target: NationMoraleTarget,
        defender_country: u8,
        attacker_country: u8,
    ) -> Option<NationMoraleMutation> {
        let defender = usize::from(defender_country);
        let attacker = usize::from(attacker_country);
        if !(1..=4).contains(&defender) || !(1..=4).contains(&attacker) {
            return None;
        }
        let delta = match target {
            NationMoraleTarget::StoneGuard => 50,
            NationMoraleTarget::Guard => 25,
            NationMoraleTarget::Admiral => 100,
            NationMoraleTarget::MagicStone => 300,
        };
        if self.morale[defender] > delta + 99 {
            self.morale[defender] = self.morale[defender].wrapping_sub(delta);
        }
        if self.morale[defender] < 100 {
            self.morale[defender] = 100;
        }
        let attacker_receives_morale = !self.nation_failed[attacker] && attacker != defender;
        let first_guard_notice = attacker_receives_morale && !self.guard_first_die[defender];
        if attacker_receives_morale {
            self.morale[attacker] = self.morale[attacker].wrapping_add(delta);
            self.lost_morale[defender] = self.lost_morale[defender].wrapping_add(delta);
            if first_guard_notice {
                self.guard_first_die[defender] = true;
            }
        }
        if target == NationMoraleTarget::StoneGuard {
            self.stone_guard_died[defender] = self.stone_guard_died[defender].wrapping_add(1);
        }
        if target == NationMoraleTarget::Admiral {
            self.da_jiang_jun_died[defender] = true;
        }
        let nation_failed = target == NationMoraleTarget::MagicStone;
        if nation_failed {
            self.nation_failed[defender] = true;
            if self.treasure_boxes[defender] != 0 {
                self.morale[defender] = self.morale[defender]
                    .wrapping_sub(self.treasure_boxes[defender].wrapping_mul(400));
                if self.morale[defender] < 100 {
                    self.morale[defender] = 100;
                }
            }
        }
        Some(NationMoraleMutation {
            defender_country,
            attacker_country,
            target,
            defender_morale: self.morale[defender],
            attacker_morale: self.morale[attacker],
            lost_morale: self.lost_morale[defender],
            first_guard_notice,
            check_morale_spawn: target != NationMoraleTarget::MagicStone
                || self.morale[attacker] > 2499,
            check_admiral_spawn: target == NationMoraleTarget::Admiral,
            nation_failed,
        })
    }

    /// Exact `OnCarriageBackTown`: slot `0` формально допустим, failed nation
    /// не меняется, а после третьей коробки каждый следующий возврат всё равно
    /// добавляет 400 morale при сохранении счётчика `3`.
    pub fn carriage_back_town(&mut self, country: i32) -> NationCarriageReturnOutcome {
        let Ok(country) = usize::try_from(country) else {
            return NationCarriageReturnOutcome::CountryOutsideNation;
        };
        if country >= self.treasure_boxes.len() {
            return NationCarriageReturnOutcome::CountryOutsideNation;
        }
        if self.nation_failed[country] {
            return NationCarriageReturnOutcome::NationFailed;
        }
        self.treasure_boxes[country] = self.treasure_boxes[country].wrapping_add(1);
        self.morale[country] = self.morale[country].wrapping_add(400);
        if self.treasure_boxes[country] > 3 {
            self.treasure_boxes[country] = 3;
        }
        NationCarriageReturnOutcome::Applied(NationCarriageReturnMutation {
            country: country as u8,
            treasure_boxes: self.treasure_boxes[country],
            morale: self.morale[country],
        })
    }

    pub const fn flag_belong_to_id(&self) -> i32 {
        self.flag_belong_to_id
    }

    pub fn contender_for_country(&self, country: i32) -> Option<i32> {
        self.contenders
            .iter()
            .find(|contender| contender.country_id == country)
            .map(|contender| contender.player_id)
    }

    pub fn contender_player_ids(&self) -> Vec<i32> {
        self.contenders
            .iter()
            .map(|contender| contender.player_id)
            .collect()
    }

    /// Список обрабатывается полностью до player-state и `0xBFF29(0)`.
    /// Как у War, эти безусловные действия выполняет runtime после mutation.
    pub fn remove_contenders_for_player(&mut self, player_id: i32) {
        self.contenders
            .retain(|contender| contender.player_id != player_id);
    }

    pub fn add_contend(
        &mut self,
        player_id: i32,
        country_id: i32,
        max_time: i32,
        start_time_ms: u32,
    ) -> bool {
        let first_for_country = self.contender_for_country(country_id).is_none();
        self.contenders.push(NationContend {
            player_id,
            country_id,
            current_time: 0,
            max_time,
            start_time_ms,
        });
        first_for_country
    }

    pub fn damage_contender(
        &mut self,
        player_id: i32,
        damage: i32,
        maximum_health: u32,
        damage_time_factor: f32,
    ) -> Result<Option<NationContendDamageMutation>, NationContendArithmeticBlock> {
        if damage <= 0 || maximum_health == 0 {
            return Ok(None);
        }
        let Some(contender) = self
            .contenders
            .iter_mut()
            .find(|contender| contender.player_id == player_id)
        else {
            return Ok(None);
        };
        // EXE сначала сохраняет `damage / maxHP * factor` в binary32, затем
        // умножает exact signed maxTime в x87 и `fistp` с RC=11 (truncate).
        let scaled_ratio = ((f64::from(damage) / f64::from(maximum_health))
            * f64::from(damage_time_factor)) as f32;
        let decrease = x87_i32_times_f32_truncating(contender.max_time, scaled_ratio);
        contender.current_time = contender.current_time.wrapping_sub(decrease).max(0);
        let percentage = contend_percentage(contender.current_time, contender.max_time)?;
        Ok(Some(NationContendDamageMutation {
            player_id,
            current_time: contender.current_time,
            percentage,
        }))
    }

    pub fn advance_contenders(
        &mut self,
        now_ms: u32,
    ) -> Result<Option<NationContendAdvance>, NationContendArithmeticBlock> {
        if self.contenders.is_empty() {
            return Ok(None);
        }
        let mut advance = NationContendAdvance::default();
        for contender in &mut self.contenders {
            let elapsed = now_ms.wrapping_sub(contender.start_time_ms);
            let candidate = (contender.current_time as u32).wrapping_add(elapsed);
            if candidate >= contender.max_time as u32 {
                advance.completed = Some(*contender);
                return Ok(Some(advance));
            }
            if elapsed > 999 {
                contender.current_time = contender.current_time.wrapping_add(elapsed as i32);
                contender.start_time_ms = now_ms;
                let percentage = contend_percentage(contender.current_time, contender.max_time)?;
                advance.progress.push((contender.player_id, percentage));
            }
        }
        Ok(Some(advance))
    }

    pub fn capture_contend_symbol(
        &mut self,
        country: u8,
    ) -> Option<NationContendCaptureMutation> {
        let country_index = usize::from(country);
        if !(1..=4).contains(&country_index) {
            return None;
        }
        self.flag_belong_to_id = i32::from(country);
        self.morale[country_index] = self.morale[country_index].wrapping_add(200);
        Some(NationContendCaptureMutation {
            country,
            morale: self.morale[country_index],
            cancelled_player_ids: self
                .contenders
                .iter()
                .map(|contender| contender.player_id)
                .collect(),
        })
    }

    pub fn clear_contenders(&mut self) {
        self.contenders.clear();
    }

    /// Exact `OnMonsterDamage` first-hit pass. Проверка четырёх stone needles
    /// сохраняет исходный порядок и странность EXE: gate выбирается по needle,
    /// а установленный флаг — по race атакованного monster-а.
    pub fn register_monster_first_hit(
        &mut self,
        monster_original_name: &[u8],
        defender_country: u8,
        attacker_country: u8,
        mut string_by_id: impl FnMut(&[u8]) -> Vec<u8>,
    ) -> Option<NationMonsterDamageNotice> {
        let defender = usize::from(defender_country);
        let attacker = usize::from(attacker_country);
        if !(1..=4).contains(&defender) || !(1..=4).contains(&attacker) || defender == attacker {
            return None;
        }

        for (country, needle) in [b"11000", b"12000", b"13000", b"14000"]
            .into_iter()
            .enumerate()
            .map(|(index, needle)| (index + 1, needle.as_slice()))
        {
            if !self.guard_attacked[country]
                && monster_original_name
                    .windows(needle.len())
                    .any(|window| window == needle)
            {
                self.guard_attacked[defender] = true;
                return Some(NationMonsterDamageNotice::StoneGuard {
                    defender_country,
                    attacker_country,
                });
            }
        }

        for (country, string_id) in [b"GS1095", b"GS1103", b"GS1111", b"GS1119"]
            .into_iter()
            .enumerate()
            .map(|(index, id)| (index + 1, id.as_slice()))
        {
            if !self.jin_wei_jun_attacked[country]
                && string_by_id(string_id) == monster_original_name
            {
                self.jin_wei_jun_attacked[defender] = true;
                return Some(NationMonsterDamageNotice::JinWeiJun {
                    defender_country,
                    attacker_country,
                });
            }
        }

        for (country, string_id) in [b"GS1142", b"GS1139", b"GS1140", b"GS1141"]
            .into_iter()
            .enumerate()
            .map(|(index, id)| (index + 1, id.as_slice()))
        {
            if !self.magic_stone_attacked[country]
                && string_by_id(string_id) == monster_original_name
            {
                self.magic_stone_attacked[defender] = true;
                return Some(NationMonsterDamageNotice::MagicStone {
                    defender_country,
                    attacker_country,
                });
            }
        }
        None
    }

    pub fn mark_yu_ying_shi_due_to_morale(&mut self, country: u8) -> bool {
        let country = usize::from(country);
        if !(1..=4).contains(&country)
            || self.yu_ying_shi_added[country]
            || (self.lost_morale[country] < 300 && self.morale[country] < 2500)
        {
            return false;
        }
        self.yu_ying_shi_added[country] = true;
        true
    }

    pub fn mark_yu_ying_shi_due_to_admiral(&mut self, country: u8) -> bool {
        let country = usize::from(country);
        if !(1..=4).contains(&country) || self.yu_ying_shi_added[country] {
            return false;
        }
        self.yu_ying_shi_added[country] = true;
        true
    }
}

fn contend_percentage(
    current_time: i32,
    max_time: i32,
) -> Result<i32, NationContendArithmeticBlock> {
    if max_time == 0 {
        return Ok(0);
    }
    current_time
        .wrapping_mul(100)
        .checked_div(max_time)
        .ok_or(NationContendArithmeticBlock {
            current_time,
            max_time,
        })
}

fn x87_i32_times_f32_truncating(value: i32, coefficient: f32) -> i32 {
    let bits = coefficient.to_bits();
    let exponent = (bits >> 23) & 0xff;
    let fraction = bits & 0x7f_ffff;
    if exponent == 0xff {
        return i32::MIN;
    }
    if exponent == 0 && fraction == 0 || value == 0 {
        return 0;
    }
    let significand = if exponent == 0 {
        u128::from(fraction)
    } else {
        u128::from((1 << 23) | fraction)
    };
    let binary_exponent = if exponent == 0 {
        -149
    } else {
        exponent as i32 - 150
    };
    let product = u128::from(value.unsigned_abs()) * significand;
    let magnitude = if binary_exponent >= 0 {
        let shift = binary_exponent as u32;
        if shift >= u128::BITS || product > (u128::MAX >> shift) {
            u128::MAX
        } else {
            product << shift
        }
    } else {
        let shift = -binary_exponent as u32;
        if shift >= u128::BITS {
            0
        } else {
            product >> shift
        }
    };
    let negative = value.is_negative() ^ (bits >> 31 != 0);
    let limit = if negative {
        1_u128 << 31
    } else {
        i32::MAX as u128
    };
    if magnitude > limit {
        return i32::MIN;
    }
    if negative {
        (-(magnitude as i64)) as i32
    } else {
        magnitude as i32
    }
}

/// Exact `ConvertMoraleToExploit` RVA `0xF1230`. EXE ставит x87 RC=11
/// перед `fistp`, поэтому для положительной morale нужно truncation,
/// а не Rust `round()`.
pub fn convert_morale_to_exploit(
    elapsed_time_ms: u32,
    morale: i32,
    country_figure: i32,
) -> u32 {
    if elapsed_time_ms == 0 {
        return 0;
    }
    let band = match elapsed_time_ms {
        0..600_000 => 0,
        600_000..1_800_000 => 1,
        1_800_000..2_700_000 => 2,
        2_700_000..3_600_000 => 3,
        _ => 4,
    };
    let multiplier = match country_figure {
        1 => [0.3, 0.75, 1.05, 1.35, 1.5][band],
        2..=8 => [0.24, 0.6, 0.84, 1.08, 1.2][band],
        _ => [0.2, 0.5, 0.7, 0.9, 1.0][band],
    };
    (f64::from(morale) * multiplier).trunc() as i64 as u32
}
