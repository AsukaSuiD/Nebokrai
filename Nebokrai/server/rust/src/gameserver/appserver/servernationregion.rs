//! Метаданные исследования оригинала; сами по себе не доказывают совместимость.
//! Декомпилятор: Ghidra 12.1.2
//! Полный декомпилят хранится локально и не входит в распространяемый код.
//!
//! Startup inheritance подтверждён constructor-ом: `ServerNationRegion`
//! начинается с `CServerWarRegion` и не имеет собственного wire decoder-а.
//! Поэтому typed startup owner делегирует exact War -> ServerRegion chain.
//! FourNation startup дополнительно материализует подтверждённый
//! `GetReliveRect`, а timing-chain владеет exact 16-byte
//! `_tagPlayerWarTime`, wrapping `timeGetTime` arithmetic и x87-truncated
//! morale→exploit. Линейный owned `Vec` заменяет MSVC `stdext::hash_map`:
//! lookup-семантика совпадает, а недоказанный bucket-order не выдаётся
//! за gameplay-контракт. First-hit guard state, morale mutation и одноразовый
//! YuYingShi gate также принадлежат этому owner-у; создание NPC и сетевые
//! side effects выполняет достигнутый `CGame` caller. Собственный ordered
//! contend-list сохраняет early-remove quirk, damage/AI timer arithmetic и
//! захват Алтаря. Предшествующий AI pass строго связывает четыре смерти stone
//! guards со смертью адмирала и отдаёт ordered magic-stone replacements;
//! player flags, сообщения и concrete NPC/monster lifetime остаются у caller-а.

use super::organizingsystem::fournationwarsys::FourNationRect;
use super::serverregion::ServerRegionDecodeError;
use super::serverwarregion::{CServerWarRegion, WarRegionDecodeContext, WarRegionDecodeError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ServerNationRegion {
    pub(crate) war: CServerWarRegion,
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
    stone_guard_died: [u32; 5],
    yu_ying_shi_added: [bool; 5],
    da_jiang_jun_died: [bool; 5],
    player_war_times: Vec<NationPlayerWarTime>,
}

impl Default for ServerNationRegion {
    fn default() -> Self {
        Self {
            war: CServerWarRegion::default(),
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
            stone_guard_died: [0; 5],
            yu_ying_shi_added: [false; 5],
            da_jiang_jun_died: [false; 5],
            player_war_times: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NationMoraleTarget {
    StoneGuard,
    Guard,
    Admiral,
    MagicStone,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NationMonsterDamageNotice {
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
pub(crate) struct NationMoraleMutation {
    pub(crate) defender_country: u8,
    pub(crate) attacker_country: u8,
    pub(crate) target: NationMoraleTarget,
    pub(crate) defender_morale: i32,
    pub(crate) attacker_morale: i32,
    pub(crate) lost_morale: i32,
    pub(crate) first_guard_notice: bool,
    pub(crate) check_morale_spawn: bool,
    pub(crate) check_admiral_spawn: bool,
    pub(crate) nation_failed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NationCarriageReturnMutation {
    pub(crate) country: u8,
    pub(crate) treasure_boxes: i32,
    pub(crate) morale: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NationCarriageReturnOutcome {
    CountryOutsideNation,
    NationFailed,
    Applied(NationCarriageReturnMutation),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct NationContend {
    pub(crate) player_id: i32,
    pub(crate) country_id: i32,
    pub(crate) current_time: i32,
    pub(crate) max_time: i32,
    pub(crate) start_time_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NationContendCancelOutcome {
    Removed { legacy_return: Option<bool> },
    MissingReset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NationContendDamageMutation {
    pub(crate) player_id: i32,
    pub(crate) current_time: i32,
    pub(crate) percentage: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NationContendArithmeticBlock {
    pub(crate) current_time: i32,
    pub(crate) max_time: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct NationContendAdvance {
    pub(crate) progress: Vec<(i32, i32)>,
    pub(crate) completed: Option<NationContend>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NationContendCaptureMutation {
    pub(crate) country: u8,
    pub(crate) morale: i32,
    pub(crate) cancelled_player_ids: Vec<i32>,
}

pub(crate) fn classify_nation_morale_target(
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
pub(crate) struct NationPlayerWarTime {
    pub(crate) player_id: i32,
    pub(crate) country: i32,
    pub(crate) country_figure: i32,
    pub(crate) start_time_ms: u32,
    pub(crate) elapsed_time_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NationPlayerWarAward {
    pub(crate) player_id: i32,
    pub(crate) country: i32,
    pub(crate) elapsed_time_ms: u32,
    pub(crate) exploit: u32,
}

impl ServerNationRegion {
    pub(crate) fn decord_from_byte_array<Context: WarRegionDecodeContext>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        context: &mut Context,
    ) -> Result<bool, WarRegionDecodeError<ServerRegionDecodeError<Context::RuntimeError>>> {
        self.war
            .decord_from_byte_array(source, cursor, include_child, context)
    }

    /// Exact `GetReliveRect` копирует все пять country rectangles в owner.
    pub(crate) const fn set_relive_rects(&mut self, rects: [FourNationRect; 5]) {
        self.relive_rects = rects;
    }

    pub(crate) const fn relive_rects(&self) -> &[FourNationRect; 5] {
        &self.relive_rects
    }

    pub(crate) fn set_country_names(&mut self, country_names: [Vec<u8>; 5]) {
        self.country_names = country_names;
    }

    pub(crate) fn country_name(&self, country: u8) -> &[u8] {
        self.country_names
            .get(usize::from(country))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub(crate) const fn morale(&self) -> &[i32; 5] {
        &self.morale
    }

    pub(crate) const fn nation_failed(&self) -> &[bool; 5] {
        &self.nation_failed
    }

    pub(crate) fn treasure_box_count(&self, country: u8) -> i32 {
        self.treasure_boxes
            .get(usize::from(country))
            .copied()
            .unwrap_or_default()
    }

    /// Region battle owner публикует текущую morale без clamp:
    /// исходные combat callbacks пишут signed `long` напрямую.
    pub(crate) const fn set_country_morale(&mut self, country: usize, morale: i32) -> bool {
        if country >= self.morale.len() {
            return false;
        }
        self.morale[country] = morale;
        true
    }

    pub(crate) const fn set_nation_failed(&mut self, country: usize, failed: bool) -> bool {
        if country >= self.nation_failed.len() {
            return false;
        }
        self.nation_failed[country] = failed;
        true
    }

    /// Exact materialized subset `OnWarDeclare`: regional timing and combat
    /// counters start empty; signup counts сам owner в этой функции не читает.
    pub(crate) fn reset_for_war_declare(&mut self) {
        self.flag_belong_to_id = 0;
        self.player_war_times.clear();
        self.lost_morale.fill(0);
        self.nation_failed.fill(false);
        self.treasure_boxes.fill(0);
        self.magic_stone_attacked.fill(false);
        self.jin_wei_jun_attacked.fill(false);
        self.guard_attacked.fill(false);
        self.guard_first_die.fill(false);
        self.stone_guard_died.fill(0);
        self.yu_ying_shi_added.fill(false);
        self.da_jiang_jun_died.fill(false);
    }

    /// Exact materialized prefix `OnRefreshRegion`: morale всех five slots
    /// становится 1000, failed flags и regional timing очищаются.
    pub(crate) fn reset_for_region_refresh(&mut self) {
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
        self.stone_guard_died.fill(0);
        self.yu_ying_shi_added.fill(false);
        self.da_jiang_jun_died.fill(false);
    }

    /// Exact `OnPlayerTimgingStart`: repeated start меняет только clock,
    /// не переснимая country/identity и не обнуляя elapsed.
    pub(crate) fn start_player_timing(
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

    pub(crate) fn has_player_timing(&self, player_id: i32) -> bool {
        self.player_war_times
            .iter()
            .any(|record| record.player_id == player_id)
    }

    /// Exact `OnPlayerTimeingFinish`: missing/inactive records не меняются;
    /// death penalty добавляется после elapsed с `DWORD` wrapping.
    pub(crate) fn finish_player_timing(
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
    pub(crate) fn take_player_war_awards(
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
    pub(crate) fn reset_materialized_war_state(&mut self) {
        self.flag_belong_to_id = 0;
        self.lost_morale.fill(0);
        self.nation_failed.fill(false);
        self.magic_stone_attacked.fill(false);
        self.jin_wei_jun_attacked.fill(false);
        self.guard_attacked.fill(false);
        self.guard_first_die.fill(false);
        self.stone_guard_died.fill(0);
        self.yu_ying_shi_added.fill(false);
        self.da_jiang_jun_died.fill(false);
    }

    pub(crate) fn take_stone_guard_results(&mut self) -> [u32; 5] {
        std::mem::take(&mut self.stone_guard_died)
    }

    /// Exact prefix `AI`: страны обходятся `1..=4`, gate требует именно
    /// `stoneGuardDie == 4 && admiralDied`, после вызова replacement счётчик
    /// guards обнуляется независимо от того, найден ли исходный NPC.
    pub(crate) fn take_due_magic_stone_transitions(&mut self) -> Vec<u8> {
        let mut countries = Vec::new();
        for country in 1..=4 {
            if self.stone_guard_died[country] == 4 && self.da_jiang_jun_died[country] {
                countries.push(country as u8);
                self.stone_guard_died[country] = 0;
            }
        }
        countries
    }

    pub(crate) fn apply_monster_morale(
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
    pub(crate) fn carriage_back_town(&mut self, country: i32) -> NationCarriageReturnOutcome {
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

    pub(crate) const fn flag_belong_to_id(&self) -> i32 {
        self.flag_belong_to_id
    }

    pub(crate) fn contender_for_country(&self, country: i32) -> Option<i32> {
        self.contenders
            .iter()
            .find(|contender| contender.country_id == country)
            .map(|contender| contender.player_id)
    }

    pub(crate) fn contender_player_ids(&self) -> Vec<i32> {
        self.contenders
            .iter()
            .map(|contender| contender.player_id)
            .collect()
    }

    /// Найденная запись в EXE удаляется ранним return до player flag и
    /// `0xBFF29(0)`; возвращаемый машинный `AL` в этой ветви не определён.
    pub(crate) fn cancel_contend_by_player_id(
        &mut self,
        player_id: i32,
    ) -> NationContendCancelOutcome {
        if let Some(index) = self
            .contenders
            .iter()
            .position(|contender| contender.player_id == player_id)
        {
            self.contenders.remove(index);
            NationContendCancelOutcome::Removed {
                legacy_return: None,
            }
        } else {
            NationContendCancelOutcome::MissingReset
        }
    }

    pub(crate) fn add_contend(
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

    pub(crate) fn damage_contender(
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

    pub(crate) fn advance_contenders(
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

    pub(crate) fn capture_contend_symbol(
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

    pub(crate) fn clear_contenders(&mut self) {
        self.contenders.clear();
    }

    /// Exact `OnMonsterDamage` first-hit pass. Проверка четырёх stone needles
    /// сохраняет исходный порядок и странность EXE: gate выбирается по needle,
    /// а установленный флаг — по race атакованного monster-а.
    pub(crate) fn register_monster_first_hit(
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

    pub(crate) fn mark_yu_ying_shi_due_to_morale(&mut self, country: u8) -> bool {
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

    pub(crate) fn mark_yu_ying_shi_due_to_admiral(&mut self, country: u8) -> bool {
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
pub(crate) fn convert_morale_to_exploit(
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

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp

// ============================================================================
// FUNCTION: CArea::`vector_deleting_destructor'
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007BB00
// ADDRESS: 0047bb00
// PROTOTYPE: void * __thiscall `vector_deleting_destructor'(uint param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagNpc::tagNpc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007C550
// ADDRESS: 0047c550
// PROTOTYPE: undefined __thiscall tagNpc(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagNpc::~tagNpc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007C570
// ADDRESS: 0047c570
// PROTOTYPE: void __thiscall ~tagNpc(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagMonsterList::~tagMonsterList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007C5C0
// ADDRESS: 0047c5c0
// PROTOTYPE: void __thiscall ~tagMonsterList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagMonsterList::tagMonsterList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007CA60
// ADDRESS: 0047ca60
// PROTOTYPE: undefined __thiscall tagMonsterList(tagMonsterList * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagNpc::tagNpc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007CB00
// ADDRESS: 0047cb00
// PROTOTYPE: undefined __thiscall tagNpc(tagNpc * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047d606
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007D606
// ADDRESS: 0047d606
// PROTOTYPE: undefined Catch@0047d606()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047de0e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007DE0E
// ADDRESS: 0047de0e
// PROTOTYPE: undefined Catch@0047de0e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047dfa1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007DFA1
// ADDRESS: 0047dfa1
// PROTOTYPE: undefined Catch@0047dfa1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047e3c9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007E3C9
// ADDRESS: 0047e3c9
// PROTOTYPE: undefined Catch@0047e3c9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047e546
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007E546
// ADDRESS: 0047e546
// PROTOTYPE: undefined Catch@0047e546()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047e635
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007E635
// ADDRESS: 0047e635
// PROTOTYPE: undefined Catch@0047e635()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047e6d8
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007E6D8
// ADDRESS: 0047e6d8
// PROTOTYPE: undefined Catch@0047e6d8()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047ea26
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007EA26
// ADDRESS: 0047ea26
// PROTOTYPE: undefined Catch@0047ea26()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047fd99
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007FD99
// ADDRESS: 0047fd99
// PROTOTYPE: undefined Catch@0047fd99()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047ffe2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007FFE2
// ADDRESS: 0047ffe2
// PROTOTYPE: undefined Catch@0047ffe2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00480071
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00080071
// ADDRESS: 00480071
// PROTOTYPE: undefined Catch@00480071()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004801be
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x000801BE
// ADDRESS: 004801be
// PROTOTYPE: undefined Catch@004801be()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00482114
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00082114
// ADDRESS: 00482114
// PROTOTYPE: undefined Catch@00482114()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00482190
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00082190
// ADDRESS: 00482190
// PROTOTYPE: undefined Catch@00482190()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagMonster::tagMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00082F70
// ADDRESS: 00482f70
// PROTOTYPE: undefined __thiscall tagMonster(tagMonster * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagMonster::~tagMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00083260
// ADDRESS: 00483260
// PROTOTYPE: void __thiscall ~tagMonster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00483a84
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00083A84
// ADDRESS: 00483a84
// PROTOTYPE: undefined Catch@00483a84()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00483b35
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00083B35
// ADDRESS: 00483b35
// PROTOTYPE: undefined Catch@00483b35()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00483c74
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00083C74
// ADDRESS: 00483c74
// PROTOTYPE: undefined Catch@00483c74()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00483d7e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00083D7E
// ADDRESS: 00483d7e
// PROTOTYPE: undefined Catch@00483d7e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagWeatherTime::~tagWeatherTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00083F30
// ADDRESS: 00483f30
// PROTOTYPE: void __thiscall ~tagWeatherTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004844de
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x000844DE
// ADDRESS: 004844de
// PROTOTYPE: undefined Catch@004844de()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00484db0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00084DB0
// ADDRESS: 00484db0
// PROTOTYPE: undefined Catch@00484db0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00485014
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00085014
// ADDRESS: 00485014
// PROTOTYPE: undefined Catch@00485014()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004850c5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x000850C5
// ADDRESS: 004850c5
// PROTOTYPE: undefined Catch@004850c5()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::GetReliveRect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:892
// RVA: 0x000F1150
// ADDRESS: 004f1150
// PROTOTYPE: void __thiscall GetReliveRect(tagRECT * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnWarStart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:904
// RVA: 0x000F1220
// ADDRESS: 004f1220
// PROTOTYPE: void __thiscall OnWarStart(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::ConvertMoraleToExploit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1815
// RVA: 0x000F1230
// ADDRESS: 004f1230
// PROTOTYPE: ulong __thiscall ConvertMoraleToExploit(ulong param_1, int param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::IsNationFail
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:2267
// RVA: 0x000F15B0
// ADDRESS: 004f15b0
// PROTOTYPE: bool __thiscall IsNationFail(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::GetDiedStateTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:2347
// RVA: 0x000F15D0
// ADDRESS: 004f15d0
// PROTOTYPE: long __thiscall GetDiedStateTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::GetIsPlayerIsContendSymbol
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1534
// RVA: 0x000F1610
// ADDRESS: 004f1610
// PROTOTYPE: bool __thiscall GetIsPlayerIsContendSymbol(long param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::CancelContendByPlayerID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1549
// RVA: 0x000F1640
// ADDRESS: 004f1640
// PROTOTYPE: bool __thiscall CancelContendByPlayerID(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnPlayerDamage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1630
// RVA: 0x000F16C0
// ADDRESS: 004f16c0
// PROTOTYPE: void __thiscall OnPlayerDamage(CPlayer * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnPlayerTimeingFinish
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1733
// RVA: 0x000F1800
// ADDRESS: 004f1800
// PROTOTYPE: void __thiscall OnPlayerTimeingFinish(CPlayer * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::SendNotifyWhenFisrtGuardOfCountryDie
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1095
// RVA: 0x000F18E0
// ADDRESS: 004f18e0
// PROTOTYPE: void __thiscall SendNotifyWhenFisrtGuardOfCountryDie(int param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnCarriageBackTown
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1442
// RVA: 0x000F1A70
// ADDRESS: 004f1a70
// PROTOTYPE: void __thiscall OnCarriageBackTown(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::CheckAdd_YUYINGSHI_DueToAdmiralDie
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1948
// RVA: 0x000F1DA0
// ADDRESS: 004f1da0
// PROTOTYPE: void __thiscall CheckAdd_YUYINGSHI_DueToAdmiralDie(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::CheckAdd_YUYINGSHI_DueToMoraleChange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:2052
// RVA: 0x000F23E0
// ADDRESS: 004f23e0
// PROTOTYPE: void __thiscall CheckAdd_YUYINGSHI_DueToMoraleChange(int param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::DelNpcMagicStoneAndAddMonsterMagicStone
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// IMPLEMENTED_SUBCHAIN: полный localized NPC lookup, два around-пакета,
// owned removal и fixed-position AddMonster материализованы в `CGame` выше.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:2274
// RVA: 0x000F3030
// ADDRESS: 004f3030
// PROTOTYPE: void __thiscall DelNpcMagicStoneAndAddMonsterMagicStone(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::~ServerNationRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:42
// RVA: 0x000F3900
// ADDRESS: 004f3900
// PROTOTYPE: void __thiscall ~ServerNationRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OpMorale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1113
// RVA: 0x000F3960
// ADDRESS: 004f3960
// PROTOTYPE: bool __thiscall OpMorale(CMonster * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnMonsterDie
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1381
// RVA: 0x000F4720
// ADDRESS: 004f4720
// PROTOTYPE: void __thiscall OnMonsterDie(CMonster * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnClearWar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// IMPLEMENTED_SUBCHAIN: полный area monster/delete-state pass и четыре
// `GS1120` NPC removal материализованы фазовым caller-ом в organsysmessage.rs.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:948
// RVA: 0x000F4E60
// ADDRESS: 004f4e60
// PROTOTYPE: void __thiscall OnClearWar(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::AddContend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1588
// RVA: 0x000F5550
// ADDRESS: 004f5550
// PROTOTYPE: void __thiscall AddContend(CPlayer * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::ServerNationRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:14
// RVA: 0x000F5810
// ADDRESS: 004f5810
// PROTOTYPE: undefined __thiscall ServerNationRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnWarDeclare
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:388
// RVA: 0x000F5920
// ADDRESS: 004f5920
// PROTOTYPE: void __thiscall OnWarDeclare(long param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnRefreshRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:424
// RVA: 0x000F5990
// ADDRESS: 004f5990
// PROTOTYPE: void __thiscall OnRefreshRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::KickOutAllPlayerToReturnPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1067
// RVA: 0x000F62E0
// ADDRESS: 004f62e0
// PROTOTYPE: void __thiscall KickOutAllPlayerToReturnPoint(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnMonsterDamage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1309
// RVA: 0x000F63E0
// ADDRESS: 004f63e0
// PROTOTYPE: void __thiscall OnMonsterDamage(CMonster * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnEnterContend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1492
// RVA: 0x000F6B40
// ADDRESS: 004f6b40
// PROTOTYPE: void __thiscall OnEnterContend(CPlayer * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::CancelContendToAllPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1574
// RVA: 0x000F6D70
// ADDRESS: 004f6d70
// PROTOTYPE: void __thiscall CancelContendToAllPlayer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::ConvertMoraleToExploitForEachPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1751
// RVA: 0x000F6DF0
// ADDRESS: 004f6df0
// PROTOTYPE: void __thiscall ConvertMoraleToExploitForEachPlayer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnContendTimeOver
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:219
// RVA: 0x000F7420
// ADDRESS: 004f7420
// PROTOTYPE: void __thiscall OnContendTimeOver(_tagNationContend * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnWarMass
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:412
// RVA: 0x000F7DB0
// ADDRESS: 004f7db0
// PROTOTYPE: void __thiscall OnWarMass(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnWarEnd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:910
// RVA: 0x000F7DD0
// ADDRESS: 004f7dd0
// PROTOTYPE: void __thiscall OnWarEnd(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnPlayerTimgingStart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1662
// RVA: 0x000F7EA0
// ADDRESS: 004f7ea0
// PROTOTYPE: void __thiscall OnPlayerTimgingStart(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// IMPLEMENTED_SUBCHAIN: base-AI callback, ordered magic-stone gates и полный
// contend timer/completion pass материализованы в `CGame::nation_contend_ai`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:156
// RVA: 0x000F8290
// ADDRESS: 004f8290
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
