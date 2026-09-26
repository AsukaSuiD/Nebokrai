//! GameServer-владелец nation war-региона `ServerNationRegion`.
//!
//! Скалярный state, данные и правила перенесены в Zone
//! `regions/servernationregion` (волна Z-M-X, семья регионов
//! country+nation+city + гейты); здесь hub-обёртка `ServerNationRegion`
//! поверх `CServerWarRegion` с прежними сигнатурами, re-export семейства для
//! старого пакета и evidence-блок. Все методы ниже — тонкие делегации на
//! Zone-агрегат `inner`; inherited war decoder остаётся здесь как вызов
//! `self.war`.
//!
//! Метаданные исследования оригинала; сами по себе не доказывают совместимость.
//! Декомпилятор: Ghidra 12.1.2
//! Полный декомпилят хранится локально и не входит в распространяемый код.
//!
//! Startup inheritance подтверждён constructor-ом: `ServerNationRegion`
//! начинается с `CServerWarRegion` и не имеет собственного wire decoder-а.
//! Его inherited decoder действительно идёт через War -> ServerRegion, но
//! selector `0x0E` получает RT_NATION от обычного `CWorldRegion` без war-tail.
//! Исходный Game читает tail за logical end своего `CMessage`; startup dispatcher
//! Rust поэтому использует только доказанный `CServerRegion` prefix. Граница
//! и причина описаны в ADR-0008.
//! FourNation startup дополнительно материализует подтверждённый
//! `GetReliveRect`, а timing-chain владеет exact 16-byte
//! `_tagPlayerWarTime`, wrapping `timeGetTime` arithmetic и x87-truncated
//! morale→exploit. Линейный owned `Vec` заменяет MSVC `stdext::hash_map`:
//! lookup-семантика совпадает, а недоказанный bucket-order не выдаётся
//! за gameplay-контракт. First-hit guard state, morale mutation и одноразовый
//! YuYingShi gate также принадлежат этому owner-у; создание NPC и сетевые
//! side effects выполняет достигнутый `CGame` caller. Собственный ordered
//! contend-list сохраняет порядок, damage/AI timer arithmetic и захват Алтаря.
//! CancelContendByPlayerID удаляет все записи игрока, после чего caller
//! безусловно сбрасывает флаг захвата и время; NULL player даёт false.
//! Предшествующий AI pass строго связывает четыре смерти stone
//! guards со смертью адмирала и отдаёт ordered magic-stone replacements;
//! player flags, сообщения и concrete NPC/monster lifetime остаются у caller-а.
//! Полный `ServerNationRegion::AI` вызывается из реального `CGame::AI` и
//! объединяет base pass, magic-stone replacements и altar contender tail.
//! Virtual `GetDiedStateTime` замкнут тем же `CGame` death caller-ом через
//! signed GlobeSetup field: wrapping seconds→milliseconds и деление пополам.
//! PDB дополнительно подтверждает пять `m_pShenShou` и пять
//! `m_bShenShouDie`. Exact constructor, `OnWarDeclare` и `OnRefreshRegion`
//! обнуляют death flags, тогда как `OnWarEnd` намеренно их не трогает; этот
//! достигнутый lifecycle хранится у owner-а. Pointer/spawn и death-writer
//! нигде в достигнутом графе не вызываются и не выносятся в process runtime.

pub(crate) use nebokrai_zone::regions::servernationregion::*;

use super::organizingsystem::fournationwarsys::FourNationRect;
use super::serverregion::ServerRegionDecodeError;
use super::skills::skillfactory::CSkillFactory;
use crate::setup::monsterlist::MonsterRegistry;
use super::serverwarregion::{CServerWarRegion, WarRegionDecodeContext, WarRegionDecodeError};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ServerNationRegion {
    pub(crate) war: CServerWarRegion,
    inner: ServerNationRegionState,
}

impl Default for ServerNationRegion {
    fn default() -> Self {
        Self {
            war: CServerWarRegion::default(),
            inner: ServerNationRegionState::default(),
        }
    }
}

impl ServerNationRegion {
    pub(crate) fn decord_from_byte_array<Context: WarRegionDecodeContext>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        area_width: i32,
        area_height: i32,
        monster_registry: &MonsterRegistry,
        skill_factory: &CSkillFactory,
        context: &mut Context,
    ) -> Result<bool, WarRegionDecodeError<ServerRegionDecodeError>> {
        self.war
            .decord_from_byte_array(
                source,
                cursor,
                include_child,
                area_width,
                area_height,
                monster_registry,
                skill_factory,
                context,
            )
    }

    /// Exact `GetReliveRect` копирует все пять country rectangles в owner.
    pub(crate) const fn set_relive_rects(&mut self, rects: [FourNationRect; 5]) {
        self.inner.set_relive_rects(rects);
    }

    pub(crate) const fn relive_rects(&self) -> &[FourNationRect; 5] {
        self.inner.relive_rects()
    }

    pub(crate) fn set_country_names(&mut self, country_names: [Vec<u8>; 5]) {
        self.inner.set_country_names(country_names);
    }

    pub(crate) fn country_name(&self, country: u8) -> &[u8] {
        self.inner.country_name(country)
    }

    pub(crate) const fn morale(&self) -> &[i32; 5] {
        self.inner.morale()
    }

    pub(crate) const fn nation_failed(&self) -> &[bool; 5] {
        self.inner.nation_failed()
    }

    /// Exact `IsNationFail`: только страны `1..4` индексируют flag-array.
    pub(crate) const fn is_nation_fail(&self, country: i32) -> bool {
        self.inner.is_nation_fail(country)
    }

    pub(crate) fn treasure_box_count(&self, country: u8) -> i32 {
        self.inner.treasure_box_count(country)
    }

    /// Region battle owner публикует текущую morale без clamp:
    /// исходные combat callbacks пишут signed `long` напрямую.
    pub(crate) const fn set_country_morale(&mut self, country: usize, morale: i32) -> bool {
        self.inner.set_country_morale(country, morale)
    }

    pub(crate) const fn set_nation_failed(&mut self, country: usize, failed: bool) -> bool {
        self.inner.set_nation_failed(country, failed)
    }

    /// Exact materialized subset `OnWarDeclare`: regional timing and combat
    /// counters start empty; signup counts сам owner в этой функции не читает.
    pub(crate) fn reset_for_war_declare(&mut self) {
        self.inner.reset_for_war_declare();
    }

    /// Exact materialized prefix `OnRefreshRegion`: morale всех five slots
    /// становится 1000, failed flags и regional timing очищаются.
    pub(crate) fn reset_for_region_refresh(&mut self) {
        self.inner.reset_for_region_refresh();
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
        self.inner
            .start_player_timing(player_id, country, country_figure, now_ms);
    }

    pub(crate) fn has_player_timing(&self, player_id: i32) -> bool {
        self.inner.has_player_timing(player_id)
    }

    /// Exact `OnPlayerTimeingFinish`: missing/inactive records не меняются;
    /// death penalty добавляется после elapsed с `DWORD` wrapping.
    pub(crate) fn finish_player_timing(
        &mut self,
        player_id: i32,
        died: bool,
        now_ms: impl FnOnce() -> u32,
    ) -> Option<NationPlayerWarTime> {
        self.inner.finish_player_timing(player_id, died, now_ms)
    }

    /// Exact `ConvertMoraleToExploitForEachPlayer` state pass: каждый active
    /// record сам читает `timeGetTime`, затем owner-map очищается.
    pub(crate) fn take_player_war_awards(
        &mut self,
        now_ms: impl FnMut() -> u32,
    ) -> Vec<NationPlayerWarAward> {
        self.inner.take_player_war_awards(now_ms)
    }

    /// Materialized subset final reset-loop `OnWarEnd`: все пять slots,
    /// включая unused index 0, обнуляются после award pass.
    pub(crate) fn reset_materialized_war_state(&mut self) {
        self.inner.reset_materialized_war_state();
    }

    pub(crate) fn take_stone_guard_results(&mut self) -> [u32; 5] {
        self.inner.take_stone_guard_results()
    }

    /// Exact prefix `AI`: страны обходятся `1..=4`, gate требует именно
    /// `stoneGuardDie == 4 && admiralDied`, после вызова replacement счётчик
    /// guards обнуляется независимо от того, найден ли исходный NPC.
    pub(crate) fn take_due_magic_stone_transitions(&mut self) -> Vec<u8> {
        self.inner.take_due_magic_stone_transitions()
    }

    pub(crate) fn apply_monster_morale(
        &mut self,
        target: NationMoraleTarget,
        defender_country: u8,
        attacker_country: u8,
    ) -> Option<NationMoraleMutation> {
        self.inner
            .apply_monster_morale(target, defender_country, attacker_country)
    }

    /// Exact `OnCarriageBackTown`: slot `0` формально допустим, failed nation
    /// не меняется, а после третьей коробки каждый следующий возврат всё равно
    /// добавляет 400 morale при сохранении счётчика `3`.
    pub(crate) fn carriage_back_town(&mut self, country: i32) -> NationCarriageReturnOutcome {
        self.inner.carriage_back_town(country)
    }

    pub(crate) const fn flag_belong_to_id(&self) -> i32 {
        self.inner.flag_belong_to_id()
    }

    pub(crate) fn contender_for_country(&self, country: i32) -> Option<i32> {
        self.inner.contender_for_country(country)
    }

    pub(crate) fn contender_player_ids(&self) -> Vec<i32> {
        self.inner.contender_player_ids()
    }

    /// Список обрабатывается полностью до player-state и `0xBFF29(0)`.
    /// Как у War, эти безусловные действия выполняет runtime после mutation.
    pub(crate) fn remove_contenders_for_player(&mut self, player_id: i32) {
        self.inner.remove_contenders_for_player(player_id);
    }

    pub(crate) fn add_contend(
        &mut self,
        player_id: i32,
        country_id: i32,
        max_time: i32,
        start_time_ms: u32,
    ) -> bool {
        self.inner
            .add_contend(player_id, country_id, max_time, start_time_ms)
    }

    pub(crate) fn damage_contender(
        &mut self,
        player_id: i32,
        damage: i32,
        maximum_health: u32,
        damage_time_factor: f32,
    ) -> Result<Option<NationContendDamageMutation>, NationContendArithmeticBlock> {
        self.inner
            .damage_contender(player_id, damage, maximum_health, damage_time_factor)
    }

    pub(crate) fn advance_contenders(
        &mut self,
        now_ms: u32,
    ) -> Result<Option<NationContendAdvance>, NationContendArithmeticBlock> {
        self.inner.advance_contenders(now_ms)
    }

    pub(crate) fn capture_contend_symbol(
        &mut self,
        country: u8,
    ) -> Option<NationContendCaptureMutation> {
        self.inner.capture_contend_symbol(country)
    }

    pub(crate) fn clear_contenders(&mut self) {
        self.inner.clear_contenders();
    }

    /// Exact `OnMonsterDamage` first-hit pass. Проверка четырёх stone needles
    /// сохраняет исходный порядок и странность EXE: gate выбирается по needle,
    /// а установленный флаг — по race атакованного monster-а.
    pub(crate) fn register_monster_first_hit(
        &mut self,
        monster_original_name: &[u8],
        defender_country: u8,
        attacker_country: u8,
        string_by_id: impl FnMut(&[u8]) -> Vec<u8>,
    ) -> Option<NationMonsterDamageNotice> {
        self.inner.register_monster_first_hit(
            monster_original_name,
            defender_country,
            attacker_country,
            string_by_id,
        )
    }

    pub(crate) fn mark_yu_ying_shi_due_to_morale(&mut self, country: u8) -> bool {
        self.inner.mark_yu_ying_shi_due_to_morale(country)
    }

    pub(crate) fn mark_yu_ying_shi_due_to_admiral(&mut self, country: u8) -> bool {
        self.inner.mark_yu_ying_shi_due_to_admiral(country)
    }
}
