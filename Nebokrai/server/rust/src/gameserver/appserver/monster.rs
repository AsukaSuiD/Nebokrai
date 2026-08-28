//! Достигнутая часть свойств и жизненного цикла `CMonster`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `server/gameserver/appserver/monster.h/.cpp` подтверждают
//! наследование от `CMoveShape`, тип `600`, начальные HP `1`, индекс обновления
//! `-1`, нулевые признаки лидера и десять множителей `1.0`.
//! `CBaseObject::CreateObject(600,id)` записывает ID после конструктора
//! производного класса.
//!
//! Старый `m_pBaseProperty` указывал внутрь общего `CMonsterList` и
//! перепривязывался после селектора `0x02`. Rust хранит точный байтовый ключ
//! исходного имени и разрешает текущий `MonsterProperties` у `CGame`, устраняя
//! висячий указатель и сохраняя обновление свойств. Снимок создания — имя,
//! графика, HP и скорость — остаётся в объекте, как в `AddMonster`.
//! `InitSkills/InitAI`, сериализация и полный автономный ИИ остаются ниже в
//! исходном материале. Достигнутая цепочка базовой атаки хранит канонические
//! HP, защиту первого нападающего, снимок смертельной атаки и цель боевого ИИ;
//! `CGame` координирует урон и смерть, `Nation/GodsBattle`, награду, добычу,
//! сценарий и окончательное удаление из региона. `Defense` и `Died` проходят
//! через каноническую очередь `passive_actions` владельца `CBaseAI`, причём
//! достигнутый `Defense` обрабатывается до активного хода монстра.
//! Специализированный ИИ синего босса с ID `0x67` хранит здесь восемь
//! одноразовых HP-порогов ярости как часть жизненного цикла конкретного
//! монстра; выбор навыка остаётся в `ai/bossblue.rs`.
//! ИИ демона-босса с ID `0x68` аналогично хранит восемь порогов призыва и
//! исходный таймер повторного призыва, инициализируемый вместе с созданием
//! конкретного монстра; выбор принадлежит `ai/bossfiend.rs`.
//!
//! Для обычного монстра со списком навыков `0x2bd`, `0x2d1`, `0x2ef`,
//! `0x197`, `0x191`, `0x198`, `0x199`, `0x19a`, `0x19b`, `0x19c`, `0x19d`,
//! `0x19e`, `0x19f`, `0x1a0`, `0x1a1`, `0x1a2`, `0x1a3`, `0x1a4` и `0x1a5`
//! тот же владелец
//! хранит цель, выбранный по исходным `odds` навык, выполнение и задержку
//! повторного применения. Быстрая атака дополнительно хранит визуальную фазу
//! и первый из двух ударов; `0x19d/0x1a1` используют единое состояние полёта
//! прямого снаряда, `0x1a0/0x1a2` — общий пошаговый путь с разной шириной,
//! `0x1a3` — накапливаемые состояния ярости, а `0x1a4` — длительный линейный
//! путь поражения. `0x1a5` использует тот же пошаговый полёт с собственной
//! начальной позицией и областью уровня 3. Поиск игроков и питомцев,
//! преследование ИИ `0/3` и задержка обходного шага остаются здесь; урон и
//! смерть питомца сохраняют приоритет цели и связь с хозяином. Пассивная либо
//! командная цель питомца доходит через масштабированную атаку до смерти дикого
//! монстра. Расписание питомца хранит ежесекундную проверку хозяина,
//! шестичасовой счётчик возраста и срок одичания, а `CGame` координирует поиск
//! целей активного режима, возврат, уведомления и исчезновение. Случайное
//! перемещение без цели, специальные сторожевые ИИ и списки с ещё не
//! достигнутыми навыками этим не подменяются. Призванный монстр хранит срок в
//! том же владельце и исчезает
//! до обычного хода ИИ, а создание сразу публикует его в `MonsterWorld` без
//! параллельного контейнера. Для достигнутого обычного
//! пути бездействия `CBaseAI` хранит время начала и интервал сна; `CGame`
//! восстанавливает HP и рассылает `OnChangeStates` до возврата ID в список
//! активных объектов области. Владелец повозки хранит режимы следования и
//! ожидания, ежесекундную привязку к хозяину и срок ожидания недействительного
//! хозяина; `CGame` исполняет движение, уведомления, отвязку и пакет удаления.
//! Восстановление питомца при входе и управление клиентом используют
//! принадлежащие владельцу `tagMasterInfo`, признак и ход приручения, раздельные
//! множители опыта и свойств `Globe`, достигнутое повышение уровня от опыта
//! последователя с `0xC0203` и узкое состояние управления питомцем. Асинхронное
//! дерево решений `CPet` этим не подменяется.
//! Достигнутое приручение хранит исходный счётчик попыток в том же владельце:
//! проверка выполняется до увеличения, а установка признака — после него,
//! включая исходную недостижимость успеха на последней разрешённой попытке.

use super::ai::baseai::CBaseAI;
use super::ai::bossblue::BossBlueAiState;
use super::ai::bossfiend::BossFiendAiState;
use super::masterinfo::MasterInfo;
use super::summonedcreature::{SummonedCreatureLifecycle, SummonedCreatureTick};
use super::moveshape::{CMoveShape, MoveShapePositionFacts};
use super::shape::{SHAPE_CHANGE_DELETE, ShapeFigure, ShapeIdentity, ShapeView};
use super::skills::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use super::skills::energybolt::PathProjectileProgress;
use super::skills::bossfiendpenetrate::BossFiendPenetrateProgress;
use super::skills::littlestar::LittleStarProgress;
use super::skills::monsterfastattack::MonsterFastAttackProgress;
use super::skills::monsterprojectile::MonsterProjectileProgress;
use super::skills::spiderweb::SpiderWebProgress;
use super::skills::spidermist::SpiderMistProgress;
use super::skills::summoncreatureskill::SummonCreatureProgress;
use super::skills::yunshenglightning::YunShengLightningProgress;
use crate::setup::monsterlist::MonsterProperties;

const MONSTER_TYPE: i32 = 600;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CMonster {
    move_shape: CMoveShape,
    original_name: Vec<u8>,
    base_property_key: Option<Vec<u8>>,
    script_file: Vec<u8>,
    hit_points: u32,
    live_time: i32,
    refresh_index: i32,
    sign: u16,
    leader_sign: u16,
    leader_distance: u16,
    leader_type: i32,
    leader_id: i32,
    died_remove: bool,
    factors: [u32; 10],
    master_info: MasterInfo,
    tamed: bool,
    tame_attempt_count: u32,
    pet_level: u32,
    pet_experience: u32,
    pet_mode: i32,
    pet_action: i32,
    pet_target: Option<ShapeIdentity>,
    pet_seek_master_ms: u32,
    pet_life_cycle_ms: u32,
    pet_life_cycle_counter: u32,
    pet_invalid_master_ms: u32,
    pet_master_logout: bool,
    carriage_action: i32,
    carriage_invalid_master_ms: u32,
    carriage_seek_master_ms: u32,
    carriage_master_logout: bool,
    first_attack_player_id: i32,
    last_attack_timer_ms: u32,
    killed_by: Option<MonsterKillingAttack>,
    ai_target: Option<ShapeIdentity>,
    base_attack_cast: Option<MonsterBaseAttackCast>,
    fast_attack_progress: Option<MonsterFastAttackProgress>,
    monster_projectile_progress: Option<MonsterProjectileProgress>,
    path_projectile_progress: Option<PathProjectileProgress>,
    boss_fiend_penetrate_progress: Option<BossFiendPenetrateProgress>,
    little_star_progress: Option<LittleStarProgress>,
    spider_web_progress: Option<SpiderWebProgress>,
    spider_mist_progress: Option<SpiderMistProgress>,
    summon_creature_progress: Option<SummonCreatureProgress>,
    yunsheng_lightning_progress: Option<YunShengLightningProgress>,
    summoned_creature: Option<SummonedCreatureLifecycle>,
    last_base_attack_ms: u32,
    base_attack_owned_tick: bool,
    trace_move_delay: Option<MonsterTraceMoveDelay>,
    boss_blue_ai: BossBlueAiState,
    boss_fiend_ai: Option<BossFiendAiState>,
    base_ai: CBaseAI,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MonsterCombatProperties {
    pub(crate) level: u8,
    pub(crate) defense: u32,
    pub(crate) dodge: u32,
    pub(crate) element_resistance: u32,
    pub(crate) soul_resistance: u16,
    pub(crate) attack_avoid: u16,
    pub(crate) element_avoid: u16,
    pub(crate) promotion_magic_attack_factor: Option<u16>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PetAttackProperties {
    pub(crate) minimum_attack: u32,
    pub(crate) maximum_attack: u32,
    pub(crate) attack_interval: u32,
    pub(crate) stop_frame: u32,
    pub(crate) speed_bits: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MonsterKillingAttack {
    pub(crate) attacker_type: i32,
    pub(crate) attacker_id: i32,
    pub(crate) skill_id: u32,
    pub(crate) skill_level: u8,
    pub(crate) critical: bool,
    pub(crate) blast_attack: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PetExperienceUpdate {
    pub(crate) level: u32,
    pub(crate) experience: u32,
    pub(crate) maximum_hp: u32,
    pub(crate) hit_points: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MonsterBaseAttackDispatch {
    pub(crate) target: ShapeIdentity,
    pub(crate) skill_id: u32,
    pub(crate) skill_level: u16,
}

pub(crate) type MonsterBaseAttackCast = SkillExecutionKernel<MonsterBaseAttackDispatch>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MonsterTraceMoveDelay {
    pub(crate) started_at_ms: u32,
    pub(crate) delay_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MonsterWakeMutation {
    pub(crate) hit_points: u32,
    pub(crate) publish_states: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PetLifecycleFacts {
    pub(crate) now_ms: u32,
    pub(crate) wild_time_ms: u32,
    pub(crate) master_present: bool,
    pub(crate) master_close: bool,
    pub(crate) safe_cell: bool,
    pub(crate) reclaimable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PetLifecycleNotice {
    AgeWarning,
    AgeExpired,
    BecameWild,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PetLifecycleOutcome {
    pub(crate) notice: Option<PetLifecycleNotice>,
    pub(crate) reclaim: bool,
    pub(crate) vanish: bool,
}

impl CMonster {
    pub(crate) fn with_constructor_defaults() -> Self {
        let mut move_shape = CMoveShape::default();
        move_shape
            .shape_mut()
            .base_object_mut()
            .set_type(MONSTER_TYPE);
        Self {
            move_shape,
            original_name: Vec::new(),
            base_property_key: None,
            script_file: Vec::new(),
            hit_points: 1,
            live_time: -1,
            refresh_index: -1,
            sign: 0,
            leader_sign: 0,
            leader_distance: 0,
            leader_type: 0,
            leader_id: 0,
            died_remove: false,
            factors: [1.0f32.to_bits(); 10],
            master_info: MasterInfo::default(),
            tamed: false,
            tame_attempt_count: 0,
            pet_level: 0,
            pet_experience: 0,
            pet_mode: 0,
            pet_action: 1,
            pet_target: None,
            pet_seek_master_ms: 0,
            pet_life_cycle_ms: 0,
            pet_life_cycle_counter: 0,
            pet_invalid_master_ms: 0,
            pet_master_logout: false,
            carriage_action: 0,
            carriage_invalid_master_ms: 0,
            carriage_seek_master_ms: 0,
            carriage_master_logout: false,
            first_attack_player_id: 0,
            last_attack_timer_ms: 0,
            killed_by: None,
            ai_target: None,
            base_attack_cast: None,
            fast_attack_progress: None,
            monster_projectile_progress: None,
            path_projectile_progress: None,
            boss_fiend_penetrate_progress: None,
            little_star_progress: None,
            spider_web_progress: None,
            spider_mist_progress: None,
            summon_creature_progress: None,
            yunsheng_lightning_progress: None,
            summoned_creature: None,
            last_base_attack_ms: 0,
            base_attack_owned_tick: false,
            trace_move_delay: None,
            boss_blue_ai: BossBlueAiState::default(),
            boss_fiend_ai: None,
            base_ai: CBaseAI::default(),
        }
    }

    pub(crate) const fn move_shape(&self) -> &CMoveShape {
        &self.move_shape
    }

    pub(crate) const fn move_shape_mut(&mut self) -> &mut CMoveShape {
        &mut self.move_shape
    }

    pub(crate) const fn master_info(&self) -> MasterInfo {
        self.master_info
    }

    pub(crate) const fn set_master_info(&mut self, master_info: MasterInfo) {
        self.master_info = master_info;
    }

    pub(crate) const fn set_summoned_creature_lifecycle(
        &mut self,
        lifecycle: Option<SummonedCreatureLifecycle>,
    ) {
        self.summoned_creature = lifecycle;
    }

    pub(crate) const fn is_summoned_creature(&self) -> bool {
        self.summoned_creature.is_some()
    }

    pub(crate) fn tick_summoned_creature(&self, now_ms: u32) -> Option<SummonedCreatureTick> {
        self.summoned_creature
            .map(|lifecycle| lifecycle.tick(now_ms, CMoveShape::is_died(self.hit_points)))
    }

    pub(crate) const fn set_tamed(&mut self, tamed: bool) {
        self.tamed = tamed;
    }

    pub(crate) const fn is_tamed(&self) -> bool {
        self.tamed
    }

    pub(crate) const fn is_tamable(&self, property: &MonsterProperties) -> bool {
        property.tamable == 1
            && self.tame_attempt_count < property.maximum_tame_attempt_count
    }

    pub(crate) const fn increase_tame_attempt_count(&mut self) {
        self.tame_attempt_count = self.tame_attempt_count.wrapping_add(1);
    }

    pub(crate) fn try_become_tamed(
        &mut self,
        property: &MonsterProperties,
        master: MasterInfo,
        pet_mode: i32,
        factors: Option<[f32; 10]>,
    ) -> bool {
        if property.tamable != 1
            || self.tame_attempt_count >= property.maximum_tame_attempt_count
            || (self.tamed
                && self.master_info.master_type == 400
                && self.master_info.master_id != 0)
        {
            return false;
        }
        self.clear_ai_target();
        self.tamed = true;
        self.master_info = master;
        self.pet_mode = pet_mode;
        if let Some(factors) = factors {
            self.adjust_pet_factors(factors);
        }
        true
    }

    pub(crate) fn is_carriage(&self, property: &MonsterProperties) -> bool {
        !self.tamed && property.tamable == 1 && property.maximum_tame_attempt_count == 0
    }

    pub(crate) const fn carriage_action(&self) -> i32 {
        self.carriage_action
    }

    pub(crate) const fn set_carriage_action(&mut self, action: i32) {
        self.carriage_action = action;
    }

    pub(crate) const fn carriage_invalid_master_ms(&self) -> u32 {
        self.carriage_invalid_master_ms
    }

    pub(crate) const fn set_carriage_invalid_master_ms(&mut self, timestamp_ms: u32) {
        self.carriage_invalid_master_ms = timestamp_ms;
    }

    pub(crate) const fn carriage_master_logout(&self) -> bool {
        self.carriage_master_logout
    }

    pub(crate) const fn set_carriage_master_logout(&mut self, logout: bool) {
        self.carriage_master_logout = logout;
    }

    pub(crate) fn carriage_master_check_due(&mut self, now_ms: u32) -> bool {
        if now_ms.wrapping_sub(self.carriage_seek_master_ms) < 1_000 {
            return false;
        }
        self.carriage_seek_master_ms = now_ms;
        true
    }

    pub(crate) const fn is_owned_pet(&self, player_id: i32) -> bool {
        self.master_info.master_type == 400 && self.master_info.master_id == player_id
    }

    pub(crate) const fn set_pet_progress(&mut self, level: u32, experience: u32) {
        self.pet_level = level;
        self.pet_experience = experience;
    }

    pub(crate) const fn pet_progress(&self) -> (u32, u32) {
        (self.pet_level, self.pet_experience)
    }

    pub(crate) fn increase_pet_experience(
        &mut self,
        experience: u32,
        property: &MonsterProperties,
        experience_factor: f32,
        next_factors: Option<[f32; 10]>,
    ) -> Option<PetExperienceUpdate> {
        self.pet_experience = self.pet_experience.wrapping_add(experience);
        if self.pet_level < 10 {
            let threshold = self.pet_maximum_hp(property) as f32 * experience_factor;
            if self.pet_experience as f32 > threshold && self.pet_level + 1 < 10 {
                self.pet_level += 1;
                self.pet_experience = 0;
                if let Some(factors) = next_factors {
                    self.adjust_pet_factors(factors);
                }
                self.hit_points = self.pet_maximum_hp(property);
            }
        }
        (experience != 0).then(|| PetExperienceUpdate {
            level: self.pet_level,
            experience: self.pet_experience,
            maximum_hp: self.pet_maximum_hp(property),
            hit_points: self.hit_points,
        })
    }

    pub(crate) fn adjust_pet_factors(&mut self, factors: [f32; 10]) {
        self.factors = factors.map(f32::to_bits);
    }

    pub(crate) fn pet_maximum_hp(&self, property: &MonsterProperties) -> u32 {
        (property.maximum_hp as f32 * f32::from_bits(self.factors[6])).round_ties_even() as u32
    }

    pub(crate) const fn set_pet_mode(&mut self, mode: i32) {
        self.pet_mode = mode;
    }

    pub(crate) const fn pet_mode(&self) -> i32 {
        self.pet_mode
    }

    pub(crate) fn set_pet_action(&mut self, action: i32) {
        self.pet_action = action;
        if action != 0 {
            self.pet_target = None;
            self.ai_target = None;
            self.cancel_base_attack_cast();
            self.trace_move_delay = None;
        }
    }

    pub(crate) const fn pet_action(&self) -> i32 {
        self.pet_action
    }

    /// Stateful часть exact `CPet::OnSchedule`; lookup master/region/skill и
    /// observable wire/delete effects остаются у `CGame` caller-а.
    pub(crate) fn tick_pet_lifecycle(&mut self, facts: PetLifecycleFacts) -> PetLifecycleOutcome {
        const SEEK_MASTER_INTERVAL_MS: u32 = 1_000;
        const LIFE_CYCLE_INTERVAL_MS: u32 = 21_600_000;

        let mut outcome = PetLifecycleOutcome::default();
        if !self.tamed {
            return outcome;
        }
        if self.pet_seek_master_ms != 0
            && facts.now_ms.wrapping_sub(self.pet_seek_master_ms) < SEEK_MASTER_INTERVAL_MS
        {
            return outcome;
        }
        if self.pet_life_cycle_ms == 0 {
            self.pet_life_cycle_ms = facts.now_ms;
        }
        if facts.now_ms.wrapping_sub(self.pet_life_cycle_ms) >= LIFE_CYCLE_INTERVAL_MS {
            self.pet_life_cycle_counter = self.pet_life_cycle_counter.wrapping_add(1);
            self.pet_life_cycle_ms = facts.now_ms;
            if self.pet_life_cycle_counter < 4 {
                if facts.master_present {
                    outcome.notice = Some(PetLifecycleNotice::AgeWarning);
                }
            } else {
                if facts.master_present {
                    outcome.notice = Some(PetLifecycleNotice::AgeExpired);
                }
                outcome.vanish = true;
                return outcome;
            }
        }

        self.pet_seek_master_ms = facts.now_ms;
        if !facts.master_present {
            self.pet_master_logout = true;
            if self.pet_invalid_master_ms == 0 {
                self.pet_action = 2;
                if facts.safe_cell {
                    self.clear_ai_target();
                    self.pet_mode = 0;
                } else {
                    if self.pet_mode == 2 {
                        self.clear_ai_target();
                    }
                    self.pet_mode = 1;
                }
                self.pet_seek_master_ms = 0;
                self.pet_invalid_master_ms = facts.now_ms;
            }
        } else {
            if self.pet_master_logout && facts.reclaimable {
                if self.pet_mode == 2 || self.ai_target.is_some() {
                    self.clear_ai_target();
                }
                self.pet_invalid_master_ms = 0;
                self.pet_seek_master_ms = 0;
                self.pet_mode = 1;
                self.pet_action = 1;
                self.pet_master_logout = false;
                outcome.reclaim = true;
            }
            if facts.master_close {
                self.pet_invalid_master_ms = 0;
                return outcome;
            }
            if self.pet_invalid_master_ms == 0 {
                self.pet_invalid_master_ms = facts.now_ms;
            }
        }

        if self.pet_invalid_master_ms != 0
            && facts.now_ms.wrapping_sub(self.pet_invalid_master_ms) >= facts.wild_time_ms
        {
            if facts.master_present {
                outcome.notice = Some(PetLifecycleNotice::BecameWild);
            }
            outcome.vanish = true;
        }
        outcome
    }

    pub(crate) fn set_pet_target(&mut self, target: ShapeIdentity) {
        self.pet_action = 0;
        self.pet_target = Some(target);
        self.ai_target = Some(target);
        self.cancel_base_attack_cast();
        self.trace_move_delay = None;
    }

    pub(crate) fn retarget_passive_pet(&mut self, target: ShapeIdentity) -> bool {
        if !self.tamed || self.pet_mode != 1 || self.ai_target.is_some() {
            return false;
        }
        if self.pet_action == 1 {
            self.pet_action = 0;
        }
        self.ai_target = Some(target);
        true
    }

    pub(crate) fn evanish_pet(&mut self) {
        self.stage_for_delete();
    }

    /// Назначает exact поля, которые `AddMonster` пишет до virtual `Init`.
    pub(crate) fn bind_spawn_property(&mut self, property: &MonsterProperties) {
        let shape = self.move_shape.shape_mut();
        shape.base_object_mut().set_name(&property.name);
        shape
            .base_object_mut()
            .set_graphics_id(property.picture_id as i32);
        self.original_name = property.original_name.clone();
        self.base_property_key = Some(property.original_name.clone());
        self.hit_points = property.maximum_hp;
    }

    /// Скорость исходный spawn назначает только после `Init` и позиции.
    pub(crate) fn set_spawn_speed(&mut self, property: &MonsterProperties) {
        self.move_shape
            .shape_mut()
            .set_speed(property.move_speed as f32);
    }

    pub(crate) fn base_property_key(&self) -> Option<&[u8]> {
        self.base_property_key.as_deref()
    }

    pub(crate) fn original_name(&self) -> &[u8] {
        &self.original_name
    }

    pub(crate) fn script_file(&self) -> &[u8] {
        &self.script_file
    }

    pub(crate) fn display_name(&self) -> &[u8] {
        let name = self.move_shape.shape().base_object().get_name();
        if name.is_empty() {
            &self.original_name
        } else {
            name
        }
    }

    pub(crate) const fn hit_points(&self) -> u32 {
        self.hit_points
    }

    pub(crate) const fn set_hit_points(&mut self, hit_points: u32) {
        self.hit_points = hit_points;
    }

    pub(crate) fn hibernate_ai(&mut self, now_ms: u32) {
        self.base_ai.hibernate(now_ms);
    }

    pub(crate) const fn is_ai_hibernated(&self) -> bool {
        self.base_ai.is_hibernated()
    }

    /// `CMonsterAI::WakeUp` сначала завершает сон, затем восстанавливает
    /// HP с точной DWORD-арифметикой и вызывает `OnChangeStates`, если до
    /// пробуждения HP отличался от максимума. Сам круговой пакет шлёт `CGame`.
    pub(crate) fn wake_ai(
        &mut self,
        now_ms: u32,
        resume_timer_ms: u32,
        property: &MonsterProperties,
    ) -> MonsterWakeMutation {
        let dormancy_interval_ms = self.base_ai.wake_up(now_ms);
        let maximum_hp = if self.tamed {
            self.pet_maximum_hp(property)
        } else {
            property.maximum_hp
        };
        let publish_states = self.hit_points != maximum_hp;
        if publish_states {
            if resume_timer_ms == 0 {
                tracing::warn!(
                    monster_id = self.move_shape.shape().identity().id,
                    "нулевой интервал восстановления монстра не допускает деление"
                );
            } else {
                let recovery_steps = dormancy_interval_ms / resume_timer_ms;
                let recovery_speed = (property.hp_recover_speed as i32).max(1) as u16 as u32;
                self.hit_points = self
                    .hit_points
                    .wrapping_add(recovery_speed.wrapping_mul(recovery_steps))
                    .min(maximum_hp);
            }
        }
        if property.ai == 0x67 {
            self.boss_blue_ai.wake(self.hit_points, maximum_hp);
        }
        MonsterWakeMutation {
            hit_points: self.hit_points,
            publish_states,
        }
    }

    pub(crate) fn boss_blue_ai_mut(&mut self) -> &mut BossBlueAiState {
        &mut self.boss_blue_ai
    }

    pub(crate) fn initialize_special_ai(&mut self, ai_type: u32, now_ms: u32) {
        self.boss_fiend_ai = (ai_type == 0x68).then(|| BossFiendAiState::new(now_ms));
    }

    pub(crate) fn boss_fiend_ai_mut(&mut self) -> Option<&mut BossFiendAiState> {
        self.boss_fiend_ai.as_mut()
    }

    pub(crate) const fn boss_fiend_ai(&self) -> Option<&BossFiendAiState> {
        self.boss_fiend_ai.as_ref()
    }

    pub(crate) fn combat_properties(
        &self,
        property: &MonsterProperties,
    ) -> MonsterCombatProperties {
        let factor = |index: usize| f32::from_bits(self.factors[index]);
        let scaled =
            |value: u32, index: usize| ((value as f32) * factor(index)).round_ties_even() as u32;
        let mut properties = MonsterCombatProperties {
            level: property.level as u8,
            defense: scaled(property.defence, 5),
            dodge: property.dodge,
            element_resistance: scaled(property.element_resistant, 3),
            soul_resistance: property.soul_resistant as u16,
            attack_avoid: property.attack_avoid,
            element_avoid: property.element_avoid,
            promotion_magic_attack_factor: self.move_shape.promotion_magic_attack_factor(),
        };
        for state in self.move_shape.reached_property_states() { if let super::moveshape::ReachedPropertyState::PoisonFog(state) = state { properties = state.apply_to_monster(properties); } }
        properties
    }

    pub(crate) fn pet_attack_properties(
        &self,
        property: &MonsterProperties,
    ) -> PetAttackProperties {
        let factor = |index: usize| f32::from_bits(self.factors[index]);
        let scaled = |value: u32, index: usize| {
            ((value as f32) * factor(index)).round_ties_even().max(0.0) as u32
        };
        let scaled_attack = |value: u32, index: usize| {
            let value = value.max(1);
            let adjusted = scaled(value, index);
            if adjusted == 0 { value } else { adjusted }
        };
        let (minimum_attack, maximum_attack) = self.state_attack_bounds(
            scaled_attack(property.minimum_attack, 1),
            scaled_attack(property.maximum_attack, 0),
        );
        PetAttackProperties {
            minimum_attack,
            maximum_attack,
            attack_interval: scaled(property.attack_speed, 7),
            stop_frame: scaled(property.stop_frame, 9),
            speed_bits: ((self.move_shape.shape().get_speed() * factor(8)).max(0.0)).to_bits(),
        }
    }

    pub(crate) fn state_attack_bounds(
        &self,
        mut minimum: u32,
        mut maximum: u32,
    ) -> (u32, u32) {
        for state in self.move_shape.swordship_states() {
            (minimum, maximum) = state.apply_to_monster(minimum, maximum);
        }
        for state in self.move_shape.battle_fairy_attribute_states() {
            minimum = state.apply_to_monster_attack(minimum);
            maximum = state.apply_to_monster_attack(maximum);
        }
        for state in self.move_shape.fury_states() {
            maximum = state.apply_to_monster_max_attack(maximum);
        }
        for state in self.move_shape.reached_property_states() {
            match state {
                super::moveshape::ReachedPropertyState::Weak(state) => {
                    (minimum, maximum) = state.apply_to_monster(minimum, maximum);
                }
                super::moveshape::ReachedPropertyState::PoisonFog(_) => {}
                super::moveshape::ReachedPropertyState::GodBless(state) => {
                    (minimum, maximum, _) = state.apply_to_monster(minimum, maximum, 0);
                }
                super::moveshape::ReachedPropertyState::Roar(state) => {
                    (minimum, maximum, _) = state.apply_to_monster(minimum, maximum, 0);
                }
            }
        }
        if let Some(state) = self.move_shape.boss_blue_fury_state() {
            minimum = state.apply_to_monster_attack(minimum);
            maximum = state.apply_to_monster_attack(maximum);
        }
        (minimum, maximum)
    }

    pub(crate) fn battle_fairy_element_modify(&self, mut value: i32) -> i32 {
        for state in self.move_shape.reached_property_states() {
            match state {
                super::moveshape::ReachedPropertyState::GodBless(state) => {
                    (_, _, value) = state.apply_to_monster(0, 0, value);
                }
                super::moveshape::ReachedPropertyState::Roar(state) => {
                    (_, _, value) = state.apply_to_monster(0, 0, value);
                }
                super::moveshape::ReachedPropertyState::Weak(_) => {}
                super::moveshape::ReachedPropertyState::PoisonFog(_) => {}
            }
        }
        for state in self.move_shape.battle_fairy_attribute_states() {
            value = state.apply_to_monster_element(value);
        }
        value
    }

    /// Точная завершающая часть владельца защиты `CMonster::OnBeenHurted`.
    /// Уведомление Nation остаётся в `CGame` перед этой мутацией, как в EXE.
    pub(crate) fn register_attacking_player(
        &mut self,
        attacker_player_id: i32,
        now_ms: u32,
        protection_ms: u32,
    ) -> bool {
        if self.first_attack_player_id != 0
            && now_ms.wrapping_sub(self.last_attack_timer_ms) <= protection_ms
        {
            if self.first_attack_player_id != attacker_player_id {
                return false;
            }
            self.last_attack_timer_ms = now_ms;
            return true;
        }
        self.first_attack_player_id = attacker_player_id;
        self.last_attack_timer_ms = now_ms;
        true
    }

    pub(crate) const fn first_attack_player_id(&self) -> i32 {
        self.first_attack_player_id
    }

    pub(crate) const fn refresh_index(&self) -> i32 {
        self.refresh_index
    }

    pub(crate) fn set_killed_by(&mut self, attack: MonsterKillingAttack) {
        self.killed_by = Some(attack);
    }

    pub(crate) const fn killed_by(&self) -> Option<MonsterKillingAttack> {
        self.killed_by
    }

    pub(crate) fn when_been_hurted_by(&mut self, attacker: ShapeIdentity, now_ms: u32) {
        self.when_been_hurted(now_ms);
        if self.ai_target.is_none() && matches!(attacker.object_type, 400 | 600) {
            self.ai_target = Some(attacker);
        }
    }

    /// Общая часть `CBaseAI::WhenBeenHurted` без политики выбора цели
    /// конкретного производного ИИ.
    pub(crate) fn when_been_hurted(&mut self, now_ms: u32) {
        self.base_ai.when_been_hurted(now_ms);
    }

    pub(crate) fn when_pet_been_hurted_by(&mut self, attacker: ShapeIdentity, now_ms: u32) {
        self.base_ai.when_been_hurted(now_ms);
        if self.ai_target.is_none()
            || self
                .ai_target
                .is_some_and(|target| target.object_type != 400 && attacker.object_type == 400)
        {
            if self.pet_action == 1 {
                self.pet_action = 0;
            }
            self.ai_target = Some(attacker);
        }
    }

    pub(crate) fn when_been_killed(&mut self, now_ms: u32) {
        self.base_ai.when_been_killed(now_ms);
        self.ai_target = None;
    }

    pub(crate) fn process_reached_defense_actions(&mut self) -> usize {
        self.base_ai.process_reached_defense_actions()
    }

    pub(crate) const fn ai_target(&self) -> Option<ShapeIdentity> {
        self.ai_target
    }

    pub(crate) fn set_ai_target(&mut self, target: ShapeIdentity) {
        if self.tamed && self.pet_action == 1 {
            self.pet_action = 0;
        }
        self.ai_target = Some(target);
    }

    pub(crate) const fn base_attack_cast(&self) -> Option<MonsterBaseAttackCast> {
        self.base_attack_cast
    }

    pub(crate) fn begin_base_attack_cast(
        &mut self,
        target: ShapeIdentity,
        skill_id: u32,
        skill_level: u16,
        now_ms: u32,
    ) {
        let mut execution = MonsterBaseAttackCast::begin(MonsterBaseAttackDispatch {
            target,
            skill_id,
            skill_level,
        }, now_ms);
        let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        self.base_attack_cast = Some(execution);
    }

    pub(crate) fn begin_fast_attack_progress(&mut self) {
        self.fast_attack_progress = Some(MonsterFastAttackProgress::default());
    }

    pub(crate) const fn fast_attack_progress(&self) -> Option<MonsterFastAttackProgress> {
        self.fast_attack_progress
    }

    pub(crate) fn fast_attack_progress_mut(&mut self) -> Option<&mut MonsterFastAttackProgress> {
        self.fast_attack_progress.as_mut()
    }

    pub(crate) fn begin_monster_projectile_progress(&mut self) {
        self.monster_projectile_progress = Some(MonsterProjectileProgress::default());
    }

    pub(crate) const fn monster_projectile_progress(&self) -> Option<MonsterProjectileProgress> {
        self.monster_projectile_progress
    }

    pub(crate) fn monster_projectile_progress_mut(
        &mut self,
    ) -> Option<&mut MonsterProjectileProgress> {
        self.monster_projectile_progress.as_mut()
    }

    pub(crate) const fn path_projectile_progress(&self) -> Option<&PathProjectileProgress> {
        self.path_projectile_progress.as_ref()
    }

    pub(crate) fn set_path_projectile_progress(&mut self, progress: PathProjectileProgress) {
        self.path_projectile_progress = Some(progress);
    }

    pub(crate) fn boss_fiend_penetrate_progress(&self) -> Option<&BossFiendPenetrateProgress> {
        self.boss_fiend_penetrate_progress.as_ref()
    }

    pub(crate) fn set_boss_fiend_penetrate_progress(
        &mut self,
        progress: BossFiendPenetrateProgress,
    ) {
        self.boss_fiend_penetrate_progress = Some(progress);
    }

    pub(crate) fn little_star_progress(&self) -> Option<&LittleStarProgress> {
        self.little_star_progress.as_ref()
    }

    pub(crate) fn set_little_star_progress(&mut self, progress: LittleStarProgress) {
        self.little_star_progress = Some(progress);
    }

    pub(crate) const fn spider_web_progress(&self) -> Option<SpiderWebProgress> {
        self.spider_web_progress
    }

    pub(crate) const fn set_spider_web_progress(&mut self, progress: SpiderWebProgress) {
        self.spider_web_progress = Some(progress);
    }

    pub(crate) const fn spider_mist_progress(&self) -> Option<SpiderMistProgress> {
        self.spider_mist_progress
    }

    pub(crate) const fn set_spider_mist_progress(&mut self, progress: SpiderMistProgress) {
        self.spider_mist_progress = Some(progress);
    }

    pub(crate) const fn summon_creature_progress(&self) -> Option<SummonCreatureProgress> {
        self.summon_creature_progress
    }

    pub(crate) const fn set_summon_creature_progress(&mut self, progress: SummonCreatureProgress) {
        self.summon_creature_progress = Some(progress);
    }

    pub(crate) const fn yunsheng_lightning_progress(&self) -> Option<YunShengLightningProgress> {
        self.yunsheng_lightning_progress
    }

    pub(crate) const fn set_yunsheng_lightning_progress(
        &mut self,
        progress: YunShengLightningProgress,
    ) {
        self.yunsheng_lightning_progress = Some(progress);
    }

    pub(crate) fn finish_base_attack_cast(&mut self, now_ms: u32) -> Option<MonsterBaseAttackCast> {
        let mut execution = self.base_attack_cast.take()?;
        self.fast_attack_progress = None;
        self.monster_projectile_progress = None;
        self.path_projectile_progress = None;
        self.boss_fiend_penetrate_progress = None;
        self.little_star_progress = None;
        self.spider_web_progress = None;
        self.spider_mist_progress = None;
        self.summon_creature_progress = None;
        self.yunsheng_lightning_progress = None;
        self.move_shape.set_current_skill_id(None);
        let _ = execution.terminate(SkillTermination::Completed);
        self.last_base_attack_ms = now_ms;
        Some(execution)
    }

    pub(crate) fn advance_base_attack_cast(
        &mut self,
        expected: SkillStage,
        next: SkillStage,
    ) -> bool {
        self.base_attack_cast
            .as_mut()
            .is_some_and(|execution| execution.advance(expected, next))
    }

    pub(crate) fn clear_ai_target(&mut self) {
        self.ai_target = None;
        self.cancel_base_attack_cast();
        self.trace_move_delay = None;
        if self.tamed && self.pet_action == 0 {
            self.pet_action = 1;
        }
    }

    pub(crate) fn cancel_base_attack_cast(&mut self) {
        self.fast_attack_progress = None;
        self.monster_projectile_progress = None;
        self.path_projectile_progress = None;
        self.boss_fiend_penetrate_progress = None;
        self.little_star_progress = None;
        self.spider_web_progress = None;
        self.spider_mist_progress = None;
        self.summon_creature_progress = None;
        self.yunsheng_lightning_progress = None;
        if let Some(mut execution) = self.base_attack_cast.take() {
            self.move_shape.set_current_skill_id(None);
            let _ = execution.terminate(SkillTermination::Cancelled);
        }
    }

    pub(crate) const fn last_base_attack_ms(&self) -> u32 {
        self.last_base_attack_ms
    }

    pub(crate) const fn set_base_attack_owned_tick(&mut self, owned: bool) {
        self.base_attack_owned_tick = owned;
    }

    pub(crate) fn take_base_attack_owned_tick(&mut self) -> bool {
        std::mem::take(&mut self.base_attack_owned_tick)
    }

    pub(crate) const fn trace_move_delay(&self) -> Option<MonsterTraceMoveDelay> {
        self.trace_move_delay
    }

    pub(crate) const fn begin_trace_move_delay(&mut self, started_at_ms: u32, delay_ms: u32) {
        self.trace_move_delay = Some(MonsterTraceMoveDelay {
            started_at_ms,
            delay_ms,
        });
    }

    pub(crate) const fn clear_trace_move_delay(&mut self) {
        self.trace_move_delay = None;
    }

    pub(crate) const fn movement_position_facts(
        &self,
        figure: ShapeFigure,
        area_width: i32,
        area_height: i32,
    ) -> MoveShapePositionFacts {
        MoveShapePositionFacts {
            current_hit_points: self.hit_points,
            figure,
            current_area: None,
            area_width,
            area_height,
        }
    }

    /// Guards reached from `CMonster::OnBeenHurted` before Nation first-hit
    /// dispatch: action `ACT_DIED` and health-based death are independent.
    pub(crate) fn can_trigger_nation_damage(&self) -> bool {
        self.move_shape.shape().get_action() != 6 && !CMoveShape::is_died(self.hit_points)
    }

    /// Exact `OnClearWar` predicate использует ту же пару virtual action/HP,
    /// но остаётся отдельным gameplay-контрактом phase cleanup.
    pub(crate) fn can_clear_from_nation_war(&self) -> bool {
        self.move_shape.shape().get_action() != 6 && !CMoveShape::is_died(self.hit_points)
    }

    pub(crate) const fn staged_for_delete(&self) -> bool {
        self.move_shape.shape().change_state() == SHAPE_CHANGE_DELETE
    }

    pub(crate) fn stage_for_delete(&mut self) {
        self.move_shape
            .shape_mut()
            .set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub(crate) fn set_script_file(&mut self, script_file: &[u8]) {
        let prefix_len = script_file
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(script_file.len());
        self.script_file.clear();
        self.script_file
            .extend_from_slice(&script_file[..prefix_len]);
    }

    pub(crate) const fn set_refresh_data(
        &mut self,
        sign: u16,
        leader_sign: u16,
        leader_distance: u16,
        refresh_index: i32,
    ) {
        self.sign = sign;
        self.leader_sign = leader_sign;
        self.leader_distance = leader_distance;
        self.refresh_index = refresh_index;
    }

    pub(crate) fn figure(property: &MonsterProperties) -> ShapeFigure {
        let figure = property.figure as u8;
        ShapeFigure::from_directions([figure; 4])
    }

    pub(crate) fn shape_view(&self, property: &MonsterProperties) -> Option<ShapeView> {
        let shape = self.move_shape.shape();
        Some(ShapeView {
            identity: shape.identity(),
            tile_x: shape.get_tile_x().ok()?,
            tile_y: shape.get_tile_y().ok()?,
            pos_x_bits: shape.get_pos_x().to_bits(),
            pos_y_bits: shape.get_pos_y().to_bits(),
            figure: Self::figure(property),
        })
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h

// ============================================================================
// FUNCTION: CMonster::GetScriptFile
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:249
// RVA: 0x00031410
// ADDRESS: 00431410
// PROTOTYPE: char * __thiscall GetScriptFile(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetScriptFile
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:250
// RVA: 0x00038C70
// ADDRESS: 00438c70
// PROTOTYPE: void __thiscall SetScriptFile(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d0882
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp
// RVA: 0x000D0882
// ADDRESS: 004d0882
// PROTOTYPE: undefined Catch@004d0882()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetMasterInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:107
// RVA: 0x000E63E0
// ADDRESS: 004e63e0
// PROTOTYPE: void __thiscall SetMasterInfo(tagMasterInfo * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetMasterInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:112
// RVA: 0x000E63F0
// ADDRESS: 004e63f0
// PROTOTYPE: tagMasterInfo * __thiscall GetMasterInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::Init
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:117
// RVA: 0x000E6400
// ADDRESS: 004e6400
// PROTOTYPE: void __thiscall Init(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:144
// RVA: 0x000E6420
// ADDRESS: 004e6420
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAttackAvoid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:273
// RVA: 0x000E64F0
// ADDRESS: 004e64f0
// PROTOTYPE: ushort __thiscall GetAttackAvoid(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetElementAvoid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:298
// RVA: 0x000E6520
// ADDRESS: 004e6520
// PROTOTYPE: ushort __thiscall GetElementAvoid(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetAttackAvoid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:282
// RVA: 0x000E6550
// ADDRESS: 004e6550
// PROTOTYPE: void __thiscall SetAttackAvoid(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetMaxHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1389
// RVA: 0x000E65A0
// ADDRESS: 004e65a0
// PROTOTYPE: ulong __thiscall GetMaxHP(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetMinAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1401
// RVA: 0x000E6620
// ADDRESS: 004e6620
// PROTOTYPE: ulong __thiscall GetMinAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetMaxAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1433
// RVA: 0x000E66C0
// ADDRESS: 004e66c0
// PROTOTYPE: ulong __thiscall GetMaxAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetHit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1467
// RVA: 0x000E6760
// ADDRESS: 004e6760
// PROTOTYPE: ushort __thiscall GetHit(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetDef
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1474
// RVA: 0x000E6780
// ADDRESS: 004e6780
// PROTOTYPE: ulong __thiscall GetDef(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetDodge
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1489
// RVA: 0x000E6800
// ADDRESS: 004e6800
// PROTOTYPE: ushort __thiscall GetDodge(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAtcSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1504
// RVA: 0x000E6860
// ADDRESS: 004e6860
// PROTOTYPE: short __thiscall GetAtcSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetElementResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1510
// RVA: 0x000E6880
// ADDRESS: 004e6880
// PROTOTYPE: ulong __thiscall GetElementResistant(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetElementModify
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1535
// RVA: 0x000E6900
// ADDRESS: 004e6900
// PROTOTYPE: ulong __thiscall GetElementModify(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetSoulResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1550
// RVA: 0x000E6970
// ADDRESS: 004e6970
// PROTOTYPE: ushort __thiscall GetSoulResistant(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetHpRecoverSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1557
// RVA: 0x000E6990
// ADDRESS: 004e6990
// PROTOTYPE: ushort __thiscall GetHpRecoverSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetHit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1564
// RVA: 0x000E69B0
// ADDRESS: 004e69b0
// PROTOTYPE: ushort __thiscall GetHit(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1569
// RVA: 0x000E69C0
// ADDRESS: 004e69c0
// PROTOTYPE: uchar __thiscall GetLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAddSoulAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1574
// RVA: 0x000E69D0
// ADDRESS: 004e69d0
// PROTOTYPE: ushort __thiscall GetAddSoulAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAtcInterval
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1581
// RVA: 0x000E69F0
// ADDRESS: 004e69f0
// PROTOTYPE: ushort __thiscall GetAtcInterval(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetStopFrame
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1601
// RVA: 0x000E6A40
// ADDRESS: 004e6a40
// PROTOTYPE: long __thiscall GetStopFrame(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetBeAttackedPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1611
// RVA: 0x000E6AA0
// ADDRESS: 004e6aa0
// PROTOTYPE: void __thiscall GetBeAttackedPoint(long param_1, long param_2, long * param_3, long * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetMonsterKind
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1647
// RVA: 0x000E6CA0
// ADDRESS: 004e6ca0
// PROTOTYPE: eMonsterKind __thiscall GetMonsterKind(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetStrikeOutTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1656
// RVA: 0x000E6CC0
// ADDRESS: 004e6cc0
// PROTOTYPE: ulong __thiscall GetStrikeOutTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetPetLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1698
// RVA: 0x000E6CE0
// ADDRESS: 004e6ce0
// PROTOTYPE: ulong __thiscall GetPetLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetPetExperience
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1703
// RVA: 0x000E6CF0
// ADDRESS: 004e6cf0
// PROTOTYPE: ulong __thiscall GetPetExperience(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetPetLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1708
// RVA: 0x000E6D00
// ADDRESS: 004e6d00
// PROTOTYPE: void __thiscall SetPetLevel(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetPetExperience
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1716
// RVA: 0x000E6D20
// ADDRESS: 004e6d20
// PROTOTYPE: void __thiscall SetPetExperience(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::IsCarriage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1721
// RVA: 0x000E6D30
// ADDRESS: 004e6d30
// PROTOTYPE: bool __thiscall IsCarriage(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetTrackRange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:87
// RVA: 0x000E6D50
// ADDRESS: 004e6d50
// PROTOTYPE: long __thiscall GetTrackRange(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:123
// RVA: 0x000E6D80
// ADDRESS: 004e6d80
// PROTOTYPE: CBaseAI * __thiscall GetAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:201
// RVA: 0x000E6DD0
// ADDRESS: 004e6dd0
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::InitAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:323
// RVA: 0x000E6E10
// ADDRESS: 004e6e10
// PROTOTYPE: void __thiscall InitAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::OnBeenHurted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:920
// RVA: 0x000E6EF0
// ADDRESS: 004e6ef0
// PROTOTYPE: void __thiscall OnBeenHurted(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::AdjustPetProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:959
// RVA: 0x000E6FF0
// ADDRESS: 004e6ff0
// PROTOTYPE: void __thiscall AdjustPetProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::UpgradePetLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:977
// RVA: 0x000E7090
// ADDRESS: 004e7090
// PROTOTYPE: void __thiscall UpgradePetLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::IncreasePetExperience
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1010
// RVA: 0x000E7150
// ADDRESS: 004e7150
// PROTOTYPE: void __thiscall IncreasePetExperience(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::IsAttackAble
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1028
// RVA: 0x000E7230
// ADDRESS: 004e7230
// PROTOTYPE: bool __thiscall IsAttackAble(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAddElementAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1525
// RVA: 0x000E7950
// ADDRESS: 004e7950
// PROTOTYPE: ulong __thiscall GetAddElementAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1591
// RVA: 0x000E79B0
// ADDRESS: 004e79b0
// PROTOTYPE: float __thiscall GetSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::NotifyMasterWhenPetDied
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1664
// RVA: 0x000E79E0
// ADDRESS: 004e79e0
// PROTOTYPE: void __thiscall NotifyMasterWhenPetDied(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::Evanish
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1689
// RVA: 0x000E7A60
// ADDRESS: 004e7a60
// PROTOTYPE: void __thiscall Evanish(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::CalculateExperienceCorrective
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:468
// RVA: 0x000E7A70
// ADDRESS: 004e7a70
// PROTOTYPE: ulong __thiscall CalculateExperienceCorrective(CPlayer * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::~CMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:80
// RVA: 0x000E7C10
// ADDRESS: 004e7c10
// PROTOTYPE: void __thiscall ~CMonster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetFigure
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:49
// RVA: 0x000E7D20
// ADDRESS: 004e7d20
// PROTOTYPE: uchar __thiscall GetFigure(eDIR param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:166
// RVA: 0x000E7D30
// ADDRESS: 004e7d30
// PROTOTYPE: ulong __thiscall GetExp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:168
// RVA: 0x000E7D40
// ADDRESS: 004e7d40
// PROTOTYPE: ulong __thiscall GetHP(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetReAnk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:191
// RVA: 0x000E7D50
// ADDRESS: 004e7d50
// PROTOTYPE: ushort __thiscall GetReAnk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:199
// RVA: 0x000E7D60
// ADDRESS: 004e7d60
// PROTOTYPE: void __thiscall SetHP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetMinAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:200
// RVA: 0x000E7D70
// ADDRESS: 004e7d70
// PROTOTYPE: void __thiscall SetMinAtk(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetMaxAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:201
// RVA: 0x000E7D80
// ADDRESS: 004e7d80
// PROTOTYPE: void __thiscall SetMaxAtk(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetHit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:202
// RVA: 0x000E7D90
// ADDRESS: 004e7d90
// PROTOTYPE: void __thiscall SetHit(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetDef
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:203
// RVA: 0x000E7DA0
// ADDRESS: 004e7da0
// PROTOTYPE: void __thiscall SetDef(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetDodge
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:204
// RVA: 0x000E7DB0
// ADDRESS: 004e7db0
// PROTOTYPE: void __thiscall SetDodge(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetAtcSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:205
// RVA: 0x000E7DC0
// ADDRESS: 004e7dc0
// PROTOTYPE: void __thiscall SetAtcSpeed(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetElementResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:206
// RVA: 0x000E7DD0
// ADDRESS: 004e7dd0
// PROTOTYPE: void __thiscall SetElementResistant(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetSoulResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:207
// RVA: 0x000E7DE0
// ADDRESS: 004e7de0
// PROTOTYPE: void __thiscall SetSoulResistant(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetHpRecoverSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:208
// RVA: 0x000E7DF0
// ADDRESS: 004e7df0
// PROTOTYPE: void __thiscall SetHpRecoverSpeed(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetAddSoulAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:211
// RVA: 0x000E7E00
// ADDRESS: 004e7e00
// PROTOTYPE: void __thiscall SetAddSoulAtk(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetElementModify
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:212
// RVA: 0x000E7E10
// ADDRESS: 004e7e10
// PROTOTYPE: void __thiscall SetElementModify(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAckRangeMin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:216
// RVA: 0x000E7E20
// ADDRESS: 004e7e20
// PROTOTYPE: long __thiscall GetAckRangeMin(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAckRangeMax
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:217
// RVA: 0x000E7E30
// ADDRESS: 004e7e30
// PROTOTYPE: long __thiscall GetAckRangeMax(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetFightRange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:218
// RVA: 0x000E7E40
// ADDRESS: 004e7e40
// PROTOTYPE: long __thiscall GetFightRange(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetChaseRange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:220
// RVA: 0x000E7E50
// ADDRESS: 004e7e50
// PROTOTYPE: long __thiscall GetChaseRange(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetGuardRange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:219
// RVA: 0x000E7E60
// ADDRESS: 004e7e60
// PROTOTYPE: long __thiscall GetGuardRange(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::CMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// IMPLEMENTED_SUBCHAIN: scalar/property defaults материализованы выше;
// constructor-side `CMoveShape::InitSkills` остаётся у незакрытого skill owner.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:37
// RVA: 0x000E7E70
// ADDRESS: 004e7e70
// PROTOTYPE: undefined __thiscall CMonster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::CalculateExperienceQuota
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:369
// RVA: 0x000E7FE0
// ADDRESS: 004e7fe0
// PROTOTYPE: ulong __thiscall CalculateExperienceQuota(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::Talk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:345
// RVA: 0x000E8220
// ADDRESS: 004e8220
// PROTOTYPE: void __thiscall Talk(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:151
// RVA: 0x000E8400
// ADDRESS: 004e8400
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::InitSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:257
// RVA: 0x000E86B0
// ADDRESS: 004e86b0
// PROTOTYPE: void __thiscall InitSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetBeneficiary
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:551
// RVA: 0x000E8780
// ADDRESS: 004e8780
// PROTOTYPE: CPlayer * __thiscall GetBeneficiary(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::OnDied
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:644
// RVA: 0x000E8960
// ADDRESS: 004e8960
// PROTOTYPE: void __thiscall OnDied(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
