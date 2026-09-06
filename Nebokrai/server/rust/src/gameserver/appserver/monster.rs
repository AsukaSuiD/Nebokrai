//! Достигнутая часть свойств и жизненного цикла `CMonster`.
//! CPassiveGladiator::OnBeenHurted (0x00610E40) ставит SearchEnemy после
//! base-handler каждого Defense, до pop в ProcessPassiveAction. Реакции
//! не откладываются на конец пачки: следующий Defense очищает новый
//! SearchEnemy, если перед ним нет сохраняемой границы Attack/Move.
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
//! `InitAI` и `GetAI` материализованы типизированным binding-ом из
//! `ai/aifactory.rs`; `InitSkills` использует канонические `CSkillFactory` и
//! `CMoveShape`.
//! GameSave игрока проверяет наличие auxiliary m_pCarriageAI напрямую
//! (CPlayer::AddToByteArray, 0x00441291), не тип текущего GetAI.
//! Auto-start очередь при первом AI-проходе исполняет
//! подтверждённые monster-ветви `TaiJi`/`Origin` и три состояния увеличения
//! `601..603`, а также четыре `Swordship`. Пять `WuXing` завершаются без
//! эффекта по исходному player-only gate; неизвестные state ID сохраняются.
//! Сериализация и полный автономный ИИ остаются ниже в исходном материале.
//! Достигнутая цепочка базовой атаки
//! хранит канонические
//! HP, защиту первого нападающего, снимок смертельной атаки и цель боевого ИИ;
//! `CGame` координирует урон и смерть, `Nation/GodsBattle`, награду, добычу,
//! сценарий и окончательное удаление из региона. `Defense` и `Died` проходят
//! через каноническую очередь `passive_actions` владельца `CBaseAI`, причём
//! достигнутые `Defense` и вероятностный `Stiffen` обрабатываются до активного
//! хода монстра; stun сохраняет движение, но прерывает атаку и цель.
//! Специализированный ИИ синего босса с ID `21` хранит здесь восемь
//! одноразовых HP-порогов ярости как часть жизненного цикла конкретного
//! монстра; выбор навыка остаётся в `ai/bossblue.rs`.
//! ИИ демона-босса с ID `23` аналогично хранит восемь порогов призыва и
//! исходный таймер повторного призыва, инициализируемый вместе с созданием
//! конкретного монстра; выбор принадлежит `ai/bossfiend.rs`.
//!
//! Для обычного монстра со списком навыков `0x2bd`, `0x2d1`, `0x2ef`,
//! `0x197`, `0x191`, `0x198`, `0x199`, `0x19a`, `0x19b`, `0x19c`, `0x19d`,
//! `0x19e`, `0x19f`, `0x1a0`, `0x1a1`, `0x1a2`, `0x1a3`, `0x1a4` и `0x1a5`
//! тот же владелец
//! хранит цель, выбранный по исходным `odds` навык, выполнение и задержку
//! повторного применения каждого установленного навыка отдельно; timestamp
//! расписания `CMonsterAI` остаётся независимым. Быстрая атака дополнительно хранит визуальную фазу
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
//! `Talk` формирует принадлежащий монстру пакет `0xBF801` и сохраняет строгий
//! прямоугольный предел `AREA_WIDTH/AREA_HEIGHT`; обход игроков и доставка
//! остаются у регионального runtime-владельца.
//! City/country guard refresh восстанавливает HP, очищает существующий
//! `CBaseAI` и оставляет формирование `0xBF60F` координирующему `CGame`.
//! Pet attack/speed/timing и elemental modifier getters применяют факторы
//! только при валидной player-owner связи; целочисленные результаты сохраняют
//! x87 truncation. Некоммутативные attack/element property-state обходятся в
//! общем byte-exact порядке `m_vStates`, включая повторяемые Fury и BattleFairy.
//! Два направления virtual `IsAttackAble` разведены явно: этот owner
//! проверяет monster-target относительно player/monster attacker-а, а
//! обратную player-target политику хранит `CPlayer` и координирует `CGame`.
//! Формулы групповой квоты и поправки опыта также принадлежат этому owner-у:
//! таблица квоты индексируется числом живых участников, а оба результата
//! усекаются к нулю после x87-порядка операций. Состав живой группы,
//! поэтапные масштабы игрока/региона и выдачу координирует `CGame`.
//! Там же разрешается `GetBeneficiary`: при непригодности прямого кандидата
//! используется первый участник его типизированного командного сеанса в том
//! же регионе и в исходном порядке списка подключений.
//! Для ненулевой figure унаследованный `CSkill::GetTargetPath` выбирает
//! ближайшую клетку footprint, а не центральную tile-позицию монстра.

use std::collections::BTreeMap;
use super::skills::swordship::is_swordship_skill;
use super::skills::wuxing::is_wuxing_skill;

use super::ai::aifactory::{ActiveMonsterAi, MonsterAiBinding, MonsterAiKind};
use super::ai::baseai::{
    AiShapeAction, CBaseAI, PassiveDeathAction, PassiveStiffenAction,
};
use super::ai::bossblue::BossBlueAiState;
use super::ai::bossfiend::BossFiendAiState;
use super::ai::carriage::{
    CarriageLifecycleState, CarriageMasterFacts, CarriageMasterOutcome,
};
use super::ai::guardtarget::GuardStationState;
use super::ai::jiumai::JiuMaiAiState;
use super::ai::monsterai::{MonsterAiScheduleState, accepts_hurt_target};
use super::ai::passivegladiator::PassiveGladiatorState;
use super::ai::pet::{PetBehaviorState, PetLifecycleFacts, PetLifecycleOutcome};
use super::ai::smartgladiator::SmartGladiatorState;
use super::masterinfo::MasterInfo;
use super::legacycodec::LegacyWriter;
use super::summonedcreature::{SummonedCreatureLifecycle, SummonedCreatureTick};
use super::moveshape::{CMoveShape, MoveShapePositionFacts};
use super::shape::{SHAPE_CHANGE_DELETE, ShapeFigure, ShapeIdentity, ShapeView};
use super::skills::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use super::skills::energybolt::PathProjectileProgress;
use super::skills::bossfiendpenetrate::BossFiendPenetrateProgress;
use super::skills::enlargefullmiss::ENLARGE_FULL_MISS_SKILL_ID;
use super::skills::enlargemaxhp::ENLARGE_MAX_HP_SKILL_ID;
use super::skills::enlargemaxmp::ENLARGE_MAX_MP_SKILL_ID;
use super::skills::littlestar::LittleStarProgress;
use super::skills::monsterfastattack::MonsterFastAttackProgress;
use super::skills::monsterprojectile::MonsterProjectileProgress;
use super::skills::origin::ORIGIN_SKILL_ID;
use super::skills::spiderweb::SpiderWebProgress;
use super::skills::spidermist::{SPIDER_MIST_SKILL_ID, SpiderMistProgress};
use super::skills::taiji::TAIJI_SKILL_ID;
use super::skills::yunshenglightning::YunShengLightningProgress;
use super::skills::skillfactory::CSkillFactory;
use crate::nets::netserver::message::CMessage;
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
    pet_behavior: PetBehaviorState,
    carriage_lifecycle: CarriageLifecycleState,
    first_attack_player_id: i32,
    last_attack_timer_ms: u32,
    killed_by: Option<MonsterKillingAttack>,
    base_attack_cast: Option<MonsterBaseAttackCast>,
    fast_attack_progress: Option<MonsterFastAttackProgress>,
    monster_projectile_progress: Option<MonsterProjectileProgress>,
    path_projectile_progress: Option<PathProjectileProgress>,
    boss_fiend_penetrate_progress: Option<BossFiendPenetrateProgress>,
    little_star_progress: Option<LittleStarProgress>,
    spider_web_progress: Option<SpiderWebProgress>,
    spider_mist_progress: Option<SpiderMistProgress>,
    yunsheng_lightning_progress: Option<YunShengLightningProgress>,
    summoned_creature: Option<SummonedCreatureLifecycle>,
    skill_last_used_ms: BTreeMap<u32, u32>,
    ai_schedule: MonsterAiScheduleState,
    base_attack_owned_tick: bool,
    attack_completion_action: AiShapeAction,
    boss_blue_ai: BossBlueAiState,
    boss_fiend_ai: Option<BossFiendAiState>,
    passive_gladiator_ai: Option<PassiveGladiatorState>,
    smart_gladiator_ai: Option<SmartGladiatorState>,
    guard_station_ai: Option<GuardStationState>,
    jiu_mai_ai: Option<JiuMaiAiState>,
    ai_binding: Option<MonsterAiBinding>,
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

/// Параметры точных `CalculateExperienceQuota` и
/// `CalculateExperienceCorrective`; состав группы и поэтапное применение
/// результата остаются у `CGame`. Вычисления ведутся через `f64` как безопасный
/// адаптер для x87-стека оригинала, а коэффициенты сохраняют исходную `f32`
/// точность.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MonsterExperienceFormula {
    ratios: [f32; 8],
    difference: f32,
    limit: f32,
    amerce: f32,
    amerce_limit: f32,
    amerce_start_level: i32,
    hit_base_level: i32,
    hit_prize: f32,
    maximum_hit_prize: f32,
}

impl MonsterExperienceFormula {
    pub(crate) const fn new(
        experience: ([f32; 8], f32, f32, f32, f32, i32),
        continuous_kill: (i32, f32, f32),
    ) -> Self {
        Self {
            ratios: experience.0,
            difference: experience.1,
            limit: experience.2,
            amerce: experience.3,
            amerce_limit: experience.4,
            amerce_start_level: experience.5,
            hit_base_level: continuous_kill.0,
            hit_prize: continuous_kill.1,
            maximum_hit_prize: continuous_kill.2,
        }
    }

    pub(crate) fn quota(
        self,
        property: &MonsterProperties,
        team_id: i32,
        average_level: f32,
        alive_amount: u32,
        player_level: u8,
    ) -> u32 {
        if team_id <= 0 {
            return property.experience;
        }
        let factor = ((1.0
            - f64::from(self.difference)
                * (f64::from(average_level) - f64::from(player_level)))
            / f64::from(alive_amount))
        .max(f64::from(self.limit));
        let ratio = self.ratios[alive_amount.min(7) as usize];
        (f64::from(property.experience) * factor * f64::from(ratio)).trunc() as i32 as u32
    }

    pub(crate) fn corrective(
        self,
        property: &MonsterProperties,
        quota: u32,
        player_level: u8,
        is_first_attacker: bool,
        continuous_kill_amount: u32,
    ) -> u32 {
        let level_delta = i32::from(player_level) - property.level as i32;
        let amerce_level = level_delta.wrapping_sub(self.amerce_start_level).max(0);
        let corrective_factor = (1.0
            - f64::from(amerce_level) * f64::from(self.amerce))
        .max(f64::from(self.amerce_limit));
        let mut corrected = (f64::from(quota) * corrective_factor)
            .max(0.0)
            .trunc() as i32 as u32;
        corrected = corrected.min(property.experience);
        if level_delta.max(0) <= self.hit_base_level
            && is_first_attacker
            && continuous_kill_amount != 0
        {
            let prize = (f64::from(continuous_kill_amount) * f64::from(self.hit_prize))
                .min(f64::from(self.maximum_hit_prize));
            corrected = (f64::from(corrected) * (prize + 1.0))
                .max(0.0)
                .trunc() as i32 as u32;
        }
        corrected
    }
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
            pet_behavior: PetBehaviorState::default(),
            carriage_lifecycle: CarriageLifecycleState::default(),
            first_attack_player_id: 0,
            last_attack_timer_ms: 0,
            killed_by: None,
            base_attack_cast: None,
            fast_attack_progress: None,
            monster_projectile_progress: None,
            path_projectile_progress: None,
            boss_fiend_penetrate_progress: None,
            little_star_progress: None,
            spider_web_progress: None,
            spider_mist_progress: None,
            yunsheng_lightning_progress: None,
            summoned_creature: None,
            skill_last_used_ms: BTreeMap::new(),
            ai_schedule: MonsterAiScheduleState::default(),
            base_attack_owned_tick: false,
            attack_completion_action: AiShapeAction::ChangeSkill,
            boss_blue_ai: BossBlueAiState::default(),
            boss_fiend_ai: None,
            passive_gladiator_ai: None,
            smart_gladiator_ai: None,
            guard_station_ai: None,
            jiu_mai_ai: None,
            ai_binding: None,
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

    /// Точный fresh-monster `AddToByteArray` tail поверх client-prefix
    /// `CMoveShape`. `master_name` уже разрешён владельцем `CGame`: обычный
    /// spawn передаёт локализованный `GS0119`, pet/carriage — имя игрока.
    pub(crate) fn encode_fresh_client_snapshot(
        &self,
        property: &MonsterProperties,
        master_name: &[u8],
    ) -> Option<Vec<u8>> {
        let mut payload = self
            .move_shape
            .encode_fresh_client_snapshot(true, self.hit_points == 0)?;
        self.append_client_snapshot_tail(&mut payload, property, master_name);
        Some(payload)
    }

    pub(crate) fn encode_client_snapshot(
        &self,
        property: &MonsterProperties,
        master_name: &[u8],
        now_ms: u32,
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        let mut payload = self.move_shape.encode_client_snapshot(
            true,
            self.hit_points == 0,
            now_ms,
            timed_state_now_milliseconds,
        )?;
        self.append_client_snapshot_tail(&mut payload, property, master_name);
        Some(payload)
    }

    fn append_client_snapshot_tail(
        &self,
        payload: &mut Vec<u8>,
        property: &MonsterProperties,
        master_name: &[u8],
    ) {
        let mut writer = LegacyWriter::new(payload);
        writer.write_u32(self.maximum_hp(property));
        writer.write_u32(self.hit_points);
        writer.write_u8(property.kind as u8);
        writer.write_u8(property.figure as u8);
        writer.write_u16(property.sound_id as u16);
        writer.write_u8(property.picture_level as u8);
        writer.write_u8(property.name_color as u8);
        writer.write_u8(property.hp_bar_color as u8);

        if self.tamed && self.master_info.master_type == 400 && self.master_info.master_id != 0 {
            writer.write_u8(1);
            writer.write_i32(self.master_info.master_type);
            writer.write_i32(self.master_info.master_id);
            writer.write_c_string(master_name);
            writer.write_u32(self.pet_level);
            writer.write_u32(self.pet_experience);
        } else if property.tamable == 1 && property.maximum_tame_attempt_count == 0 {
            writer.write_u8(2);
            writer.write_i32(self.master_info.master_type);
            writer.write_i32(self.master_info.master_id);
            writer.write_c_string(master_name);
        } else {
            writer.write_u8(0);
        }
    }

    /// Формирует exact `CServerRegion::AddMonster` envelope `0xBF502` для
    /// только что созданного monster. Выбор around-получателей остаётся у
    /// owning region/CGame и не дублируется в объекте.
    pub(crate) fn build_fresh_enter_message(
        &self,
        property: &MonsterProperties,
        master_name: &[u8],
    ) -> Option<CMessage> {
        let payload = self.encode_fresh_client_snapshot(property, master_name)?;
        let identity = self.move_shape.shape().identity();
        let mut message = CMessage::new(0x000b_f502);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id);
        message.add_long(i32::try_from(payload.len()).ok()?);
        message.base_mut().add(&payload);
        message.add_byte(0);
        Some(message)
    }

    pub(crate) fn build_enter_message(
        &self,
        property: &MonsterProperties,
        master_name: &[u8],
        now_ms: u32,
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<CMessage> {
        let payload = self.encode_client_snapshot(
            property,
            master_name,
            now_ms,
            timed_state_now_milliseconds,
        )?;
        let identity = self.move_shape.shape().identity();
        let mut message = CMessage::new(0x000b_f502);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id);
        message.add_long(i32::try_from(payload.len()).ok()?);
        message.base_mut().add(&payload);
        message.add_byte(0);
        Some(message)
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

    /// Точный `CMonster::DoesCreatureBeenTamed` (RVA `0x000E6460`): одного
    /// внутреннего знака недостаточно, требуется живой identity хозяина-игрока.
    pub(crate) const fn is_tamed(&self) -> bool {
        self.has_player_pet_master()
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
            || self.is_tamed()
        {
            return false;
        }
        self.clear_ai_target();
        self.tamed = true;
        self.master_info = master;
        self.pet_behavior.set_mode(pet_mode);
        if let Some(factors) = factors {
            self.adjust_pet_factors(factors);
        }
        true
    }

    /// Соответствует `dynamic_cast<CCarriage *>(GetAI())`: до назначения
    /// player-master учитывается первичный AI24, после назначения — отдельный
    /// auxiliary `CCarriage`, созданный для tamable-свойства с нулём попыток.
    pub(crate) fn is_carriage(&self, _property: &MonsterProperties) -> bool {
        matches!(
            self.active_ai(),
            Some(
                ActiveMonsterAi::Carriage
                    | ActiveMonsterAi::Primary(MonsterAiKind::Carriage)
            )
        )
    }

    pub(crate) const fn active_ai(&self) -> Option<ActiveMonsterAi> {
        match self.ai_binding {
            Some(binding) => binding.active(self.master_info),
            None => None,
        }
    }

    pub(crate) fn has_pet_ai(&self) -> bool {
        self.ai_binding.is_some_and(MonsterAiBinding::has_pet)
    }

    pub(crate) fn has_carriage_ai(&self) -> bool {
        self.ai_binding.is_some_and(MonsterAiBinding::has_carriage)
    }

    /// Exact `CMonster::InitSkills`: базовая защита добавляется первой,
    /// затем исходный бессодержательный `random(skill_count)` расходует RNG,
    /// после чего property skills проходят в wire-порядке. `odds` на этой
    /// границе не читается; fallback base attack в точном owner-е отсутствует.
    pub(crate) fn initialize_skills(
        &mut self,
        property: &MonsterProperties,
        factory: &CSkillFactory,
        random: &mut impl FnMut(i32) -> i32,
    ) {
        self.move_shape.clear_skills();
        self.move_shape.add_base_defense_skill(factory);
        let _discarded_roll = random(property.skills.len() as i32);
        for skill in &property.skills {
            let _loaded =
                self.move_shape
                    .add_skill(u32::from(skill.id), i32::from(skill.level), factory);
        }
    }

    /// `CBaseAI::OnExecuteBackStageSkills` для подтверждённых monster-ветвей
    /// пяти немедленных state-owner-ов и Swordship. WuXing снимается по
    /// подтверждённому `End(0)` для type != 400; неизвестные ID остаются в очереди.
    pub(crate) fn take_reached_back_stage_skills(&mut self) -> Vec<(u32, i32)> {
        self.move_shape
            .take_matching_back_stage_skill_ids(|skill_id| {
                matches!(
                    skill_id,
                    TAIJI_SKILL_ID
                        | ORIGIN_SKILL_ID
                        | ENLARGE_MAX_HP_SKILL_ID
                        | ENLARGE_MAX_MP_SKILL_ID
                        | ENLARGE_FULL_MISS_SKILL_ID
                ) || is_swordship_skill(skill_id) || is_wuxing_skill(skill_id)
            })
            .into_iter()
            .filter(|skill_id| !is_wuxing_skill(*skill_id))
            .map(|skill_id| (skill_id, self.move_shape.skill_level(skill_id)))
            .collect()
    }

    pub(crate) const fn carriage_action(&self) -> i32 {
        self.carriage_lifecycle.action()
    }

    pub(crate) const fn set_carriage_action(&mut self, action: i32) {
        self.carriage_lifecycle.set_action(action);
    }

    pub(crate) fn advance_carriage_schedule(&mut self, now_ms: u32) -> bool {
        self.carriage_lifecycle.advance_schedule(now_ms)
    }

    pub(crate) const fn block_carriage_schedule(&mut self, now_ms: u32, delay_ms: u32) {
        self.carriage_lifecycle.block_schedule(now_ms, delay_ms);
    }

    pub(crate) fn tick_carriage_master(
        &mut self,
        facts: CarriageMasterFacts,
    ) -> CarriageMasterOutcome {
        self.carriage_lifecycle.tick_master(facts)
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
        current_factors: Option<[f32; 10]>,
        next_factors: Option<[f32; 10]>,
    ) -> Option<PetExperienceUpdate> {
        self.pet_experience = self.pet_experience.wrapping_add(experience);
        if self.pet_level < 10 {
            let threshold = self.maximum_hp(property) as f32 * experience_factor;
            if self.pet_experience as f32 <= threshold {
                if self.pet_level == 0 && self.pet_experience == 0 {
                    if let Some(factors) = current_factors {
                        self.adjust_pet_factors(factors);
                    }
                }
            } else if self.pet_level + 1 < 10 {
                self.pet_level += 1;
                self.pet_experience = 0;
                if let Some(factors) = next_factors {
                    self.adjust_pet_factors(factors);
                }
                self.hit_points = self.maximum_hp(property);
            }
        }
        (experience != 0).then(|| PetExperienceUpdate {
            level: self.pet_level,
            experience: self.pet_experience,
            maximum_hp: self.maximum_hp(property),
            hit_points: self.hit_points,
        })
    }

    pub(crate) fn adjust_pet_factors(&mut self, factors: [f32; 10]) {
        self.factors = factors.map(f32::to_bits);
    }

    /// Exact `CMonster::GetMaxHP` (RVA `0x000E65A0`): factor `6` действует
    /// только при валидной player-owner связи, а x87 результат усекается.
    pub(crate) fn maximum_hp(&self, property: &MonsterProperties) -> u32 {
        if !self.has_player_pet_master() {
            return property.maximum_hp;
        }
        let scaled = f64::from(property.maximum_hp)
            * f64::from(f32::from_bits(self.factors[6]));
        scaled.trunc() as i32 as u32
    }

    pub(crate) fn roll_stiffen(
        &mut self,
        damage: u32,
        property: &MonsterProperties,
        setup: crate::setup::globesetup::GlobeStiffenSetup,
        now_ms: impl FnMut() -> u32,
        random: impl FnMut(i32) -> i32,
    ) -> u32 {
        let maximum_hp = self.maximum_hp(property);
        self.move_shape.stiffen(
            damage as u16,
            maximum_hp,
            property.re_ank as u16,
            setup,
            now_ms,
            random,
        )
    }

    pub(crate) const fn set_pet_mode(&mut self, mode: i32) {
        self.pet_behavior.set_mode(mode);
    }

    pub(crate) const fn pet_mode(&self) -> i32 {
        self.pet_behavior.mode()
    }

    pub(crate) fn set_pet_action(&mut self, action: i32) {
        if self.pet_behavior.set_action(action) {
            self.base_ai.lose_target();
            self.cancel_base_attack_cast();
            self.base_ai.cancel_active_move();
        }
    }

    pub(crate) const fn pet_action(&self) -> i32 {
        self.pet_behavior.action()
    }

    /// Состояние точного `CPet::OnSchedule`; поиск хозяина, региона и навыка,
    /// а также наблюдаемые сетевые эффекты и удаление остаются у `CGame`.
    pub(crate) fn tick_pet_lifecycle(&mut self, facts: PetLifecycleFacts) -> PetLifecycleOutcome {
        if !self.tamed {
            return PetLifecycleOutcome::default();
        }
        let outcome = self.pet_behavior.tick(facts, self.ai_target().is_some());
        if outcome.clear_target {
            self.clear_ai_target();
        }
        outcome
    }

    pub(crate) fn set_pet_target(&mut self, target: ShapeIdentity) {
        self.pet_behavior.begin_target();
        self.base_ai.set_object_target(target);
        self.cancel_base_attack_cast();
        self.base_ai.cancel_active_move();
    }

    pub(crate) fn retarget_passive_pet(&mut self, target: ShapeIdentity) -> bool {
        if !self
            .pet_behavior
            .retarget_passive(self.tamed, self.ai_target().is_some())
        {
            return false;
        }
        self.base_ai.set_object_target(target);
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

    /// Формирует точный кадр `CMonster::Talk`; завершающие нули строк остаются
    /// частью `CBaseMessage::Add(char const*)` wire-контракта.
    pub(crate) fn build_talk_message(&self, text: &[u8]) -> CMessage {
        let identity = self.move_shape.shape().identity();
        let mut message = CMessage::new(0x000b_f801);
        message.add_long(0);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.base_mut().add(self.display_name());
        message.add_byte(0);
        message.base_mut().add(text);
        message.add_byte(0);
        message
    }

    /// Сохраняет две независимые строгие проверки расстояния из `Talk`.
    pub(crate) fn talk_reaches(
        &self,
        target_x: i32,
        target_y: i32,
        area_width: i32,
        area_height: i32,
    ) -> bool {
        let shape = self.move_shape.shape();
        let (Ok(source_x), Ok(source_y)) = (shape.get_tile_x(), shape.get_tile_y()) else {
            return false;
        };
        i64::from(target_x).abs_diff(i64::from(source_x)) < area_width as u64
            && i64::from(target_y).abs_diff(i64::from(source_y)) < area_height as u64
    }

    pub(crate) const fn hit_points(&self) -> u32 {
        self.hit_points
    }

    pub(crate) const fn set_hit_points(&mut self, hit_points: u32) {
        self.hit_points = hit_points;
    }

    /// Выполняет достигнутый `SetHP(GetMaxHP) -> GetAI()->Clear()` war guard-
    /// путь. `CBaseAI::Clear` сбрасывает собственную цель напрямую и не
    /// запускает расширенную потерю цели питомца или отмену текущего skill.
    pub(crate) fn refresh_war_guard(&mut self, maximum_hp: u32) {
        self.hit_points = maximum_hp;
        if self.ai_binding.is_some() {
            self.base_ai.clear_guard_refresh_state();
        }
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
    ) -> bool {
        let dormancy_interval_ms = self.base_ai.wake_up(now_ms);
        let maximum_hp = self.maximum_hp(property);
        let publish_states = self.hit_points != maximum_hp;
        if publish_states {
            if resume_timer_ms == 0 {
                tracing::warn!(
                    monster_id = self.move_shape.shape().identity().id,
                    "нулевой интервал восстановления монстра не допускает деление"
                );
            } else {
                let recovery_steps = dormancy_interval_ms / resume_timer_ms;
                let recovery_speed = u32::from(self.hp_recovery_speed(property));
                self.hit_points = self
                    .hit_points
                    .wrapping_add(recovery_speed.wrapping_mul(recovery_steps))
                    .min(maximum_hp);
            }
        }
        if matches!(
            self.ai_binding.map(MonsterAiBinding::primary),
            Some(MonsterAiKind::BossBlue)
        ) {
            self.boss_blue_ai.wake(self.hit_points, maximum_hp);
        }
        publish_states
    }

    pub(crate) fn boss_blue_ai_mut(&mut self) -> &mut BossBlueAiState {
        &mut self.boss_blue_ai
    }

    /// Материализует полный `CMonster::InitAI`: фабричный выбор первичного,
    /// pet- и carriage-владельцев выполняется до инициализации их concrete
    /// состояния. Повторный вызов заменяет прежний binding, как delete/create
    /// в исходном owner-е.
    pub(crate) fn initialize_ai(&mut self, property: &MonsterProperties, now_ms: u32) {
        let binding = MonsterAiBinding::create(property, self.tame_attempt_count);
        let primary = binding.primary();
        let ai_type = binding.ai_type();
        self.ai_binding = Some(binding);
        self.attack_completion_action = if matches!(primary, MonsterAiKind::StupidArcher) {
            AiShapeAction::SearchEnemy
        } else {
            crate::gameserver::appserver::ai::fixedpositionarcher::attack_completion_action(ai_type)
        };
        self.boss_fiend_ai =
            matches!(primary, MonsterAiKind::BossFiend).then(|| BossFiendAiState::new(now_ms));
        self.passive_gladiator_ai =
            matches!(primary, MonsterAiKind::PassiveGladiator).then(PassiveGladiatorState::default);
        self.smart_gladiator_ai =
            matches!(primary, MonsterAiKind::SmartGladiator).then(SmartGladiatorState::default);
        self.guard_station_ai = matches!(
            primary,
            MonsterAiKind::CityGuardWithSword
                | MonsterAiKind::VillageCountyGuardWithSword
                | MonsterAiKind::NationCountyGuardWithSword
        )
        .then(GuardStationState::default);
        self.jiu_mai_ai = matches!(primary, MonsterAiKind::JiuMai).then(JiuMaiAiState::default);
    }

    pub(crate) fn boss_fiend_ai_mut(&mut self) -> Option<&mut BossFiendAiState> {
        self.boss_fiend_ai.as_mut()
    }

    pub(crate) const fn boss_fiend_ai(&self) -> Option<&BossFiendAiState> {
        self.boss_fiend_ai.as_ref()
    }

    pub(crate) fn passive_gladiator_ai_mut(&mut self) -> Option<&mut PassiveGladiatorState> {
        self.passive_gladiator_ai.as_mut()
    }

    pub(crate) fn take_passive_gladiator_ai(&mut self) -> Option<PassiveGladiatorState> {
        self.passive_gladiator_ai.take()
    }

    pub(crate) fn restore_passive_gladiator_ai(&mut self, state: PassiveGladiatorState) {
        self.passive_gladiator_ai = Some(state);
    }

    pub(crate) fn smart_gladiator_ai_mut(&mut self) -> Option<&mut SmartGladiatorState> {
        self.smart_gladiator_ai.as_mut()
    }

    pub(crate) fn smart_gladiator_ai(&self) -> Option<&SmartGladiatorState> {
        self.smart_gladiator_ai.as_ref()
    }

    pub(crate) fn guard_station_ai_mut(&mut self) -> Option<&mut GuardStationState> {
        self.guard_station_ai.as_mut()
    }

    pub(crate) const fn jiu_mai_ai(&self) -> Option<&JiuMaiAiState> {
        self.jiu_mai_ai.as_ref()
    }

    pub(crate) fn jiu_mai_ai_mut(&mut self) -> Option<&mut JiuMaiAiState> {
        self.jiu_mai_ai.as_mut()
    }

    pub(crate) fn combat_properties(
        &self,
        property: &MonsterProperties,
    ) -> MonsterCombatProperties {
        let mut properties = MonsterCombatProperties {
            level: property.level as u8,
            defense: self.defense(property),
            dodge: u32::from(self.dodge(property)),
            element_resistance: self.element_resistance(property),
            soul_resistance: self.soul_resistance(property),
            attack_avoid: self.attack_avoid(property),
            element_avoid: self.element_avoid(property),
            promotion_magic_attack_factor: self.move_shape.promotion_magic_attack_factor(),
        };
        for state in self.move_shape.ordered_monster_property_states() {
            if let super::moveshape::MonsterPropertyState::PoisonFog(state) = state {
                properties = state.apply_to_monster(properties);
            }
        }
        properties
    }

    /// Виртуальный `CMonster::GetDodge`: базовое значение не ниже единицы,
    /// а приручённый monster-owner применяет свой pet-level factor до
    /// сужения результата к `ushort`.
    pub(crate) fn dodge(&self, property: &MonsterProperties) -> u16 {
        let base = (property.dodge as i32).max(1) as u16;
        if self.has_player_pet_master() {
            let scaled = f64::from(base) * f64::from(f32::from_bits(self.factors[4]));
            return scaled.trunc() as i32 as u16;
        }
        base
    }

    /// Достигнутая ресурсная часть `CMonster::GetHit` (RVA `0x000E6760`):
    /// signed DWORD не выше нуля становится единицей до сужения к `ushort`.
    pub(crate) fn hit(&self, property: &MonsterProperties) -> u16 {
        (property.hit as i32).max(1) as u16
    }

    /// Достигнутая ресурсная часть `CMonster::GetAttackAvoid`
    /// (RVA `0x000E64F0`): неположительное signed значение становится нулём,
    /// а положительное ограничивается `99`.
    pub(crate) fn attack_avoid(&self, property: &MonsterProperties) -> u16 {
        (property.attack_avoid as i32).clamp(0, 99) as u16
    }

    /// Достигнутая ресурсная часть `CMonster::GetElementAvoid`
    /// (RVA `0x000E6520`): контракт совпадает с physical avoid, кроме
    /// разрешённой верхней границы `100`.
    pub(crate) fn element_avoid(&self, property: &MonsterProperties) -> u16 {
        (property.element_avoid as i32).clamp(0, 100) as u16
    }

    /// Exact `CMonster::GetDef` (RVA `0x000E6780`): отрицательная сумма
    /// свойства и runtime modifier сначала становится нулём, затем pet factor
    /// `5` усекается x87 к signed DWORD.
    pub(crate) fn defense(&self, property: &MonsterProperties) -> u32 {
        let base = (property.defence as i32).max(0) as u32;
        if !self.has_player_pet_master() {
            return base;
        }
        let scaled = f64::from(base) * f64::from(f32::from_bits(self.factors[5]));
        scaled.trunc() as i32 as u32
    }

    /// Exact `CMonster::GetElementResistant` (RVA `0x000E6880`): runtime
    /// modifier (достигнутый Taiji state) входит до нижней границы и pet
    /// factor `3`; последующие defense states применяются к готовому getter-у.
    pub(crate) fn element_resistance(&self, property: &MonsterProperties) -> u32 {
        let mut value = property.element_resistant;
        if let Some(state) = self.move_shape.taiji_state() {
            value = state.apply_to_monster(value);
        }
        let base = (value as i32).max(1) as u32;
        if !self.has_player_pet_master() {
            return base;
        }
        let scaled = f64::from(base) * f64::from(f32::from_bits(self.factors[3]));
        scaled.trunc() as i32 as u32
    }

    /// Достигнутая часть `CMonster::GetSoulResistant` (RVA `0x000E6970`):
    /// текущая цепочка не имеет setter-а runtime modifier, но ресурсное
    /// значение всё равно проходит исходную нижнюю границу до `ushort`.
    pub(crate) fn soul_resistance(&self, property: &MonsterProperties) -> u16 {
        (property.soul_resistant as i32).max(1) as u16
    }

    /// Достигнутая часть `CMonster::GetHpRecoverSpeed` (RVA `0x000E6990`):
    /// dormant AI использует ресурсное значение после нижней границы `1` и
    /// исходного сужения к `ushort`.
    pub(crate) fn hp_recovery_speed(&self, property: &MonsterProperties) -> u16 {
        (property.hp_recover_speed as i32).max(1) as u16
    }

    /// Достигнутая ресурсная часть `CMonster::GetAddSoulAtk`
    /// (RVA `0x000E69D0`): signed DWORD не выше нуля даёт `0`, положительное
    /// значение сужается к младшим шестнадцати битам.
    pub(crate) fn resource_soul_attack(property: &MonsterProperties) -> u16 {
        let value = property.yao_attack as i32;
        if value > 0 { value as u16 } else { 0 }
    }

    /// Exact `CMonster::GetStopFrame` (RVA `0x000E6A40`): только приручённый
    /// монстр с живой player-owner связью применяет pet factor `9`. Оригинал
    /// умножает signed DWORD на f32 в x87 и временно включает truncation.
    pub(crate) fn stop_frame(&self, property: &MonsterProperties) -> u32 {
        if self.has_player_pet_master() {
            let scaled = f64::from(property.stop_frame as i32)
                * f64::from(f32::from_bits(self.factors[9]));
            return scaled.trunc() as i32 as u32;
        }
        property.stop_frame
    }

    const fn has_player_pet_master(&self) -> bool {
        self.tamed && self.master_info.master_type == 400 && self.master_info.master_id != 0
    }

    fn pet_scaled_attack(&self, value: u32, factor_index: usize) -> u32 {
        let base = (value as i32).max(1) as u32;
        if !self.has_player_pet_master() {
            return base;
        }
        let scaled = f64::from(base)
            * f64::from(f32::from_bits(self.factors[factor_index]));
        let scaled = scaled.trunc() as i32;
        if scaled > 0 { scaled as u32 } else { base }
    }

    /// Exact `CMonster::GetAtcInterval` (RVA `0x000E69F0`): базовый virtual
    /// сначала сужает интервал к WORD, pet factor `7` затем усекается `__ftol2`.
    pub(crate) fn attack_interval(&self, property: &MonsterProperties) -> u32 {
        let base = property.attack_speed as u16;
        if !self.has_player_pet_master() {
            return u32::from(base);
        }
        let scaled = f64::from(base) * f64::from(f32::from_bits(self.factors[7]));
        u32::from(scaled.trunc() as i32 as u16)
    }

    /// Exact `CMonster::GetSpeed` (RVA `0x000E79B0`): x87 умножает два f32
    /// только для валидной player-owner связи; Rust округляет результат при
    /// сохранении canonical f32 скорости.
    pub(crate) fn speed(&self) -> f32 {
        let base = self.move_shape.shape().get_speed();
        if !self.has_player_pet_master() {
            return base;
        }
        (f64::from(base) * f64::from(f32::from_bits(self.factors[8]))) as f32
    }

    pub(crate) fn pet_attack_properties(
        &self,
        property: &MonsterProperties,
    ) -> PetAttackProperties {
        let (minimum_attack, maximum_attack) = self.state_attack_bounds(
            self.pet_scaled_attack(property.minimum_attack, 1),
            self.pet_scaled_attack(property.maximum_attack, 0),
        );
        PetAttackProperties {
            minimum_attack,
            maximum_attack,
            attack_interval: self.attack_interval(property),
            stop_frame: self.stop_frame(property),
            speed_bits: self.speed().to_bits(),
        }
    }

    pub(crate) fn state_attack_bounds(
        &self,
        mut minimum: u32,
        mut maximum: u32,
    ) -> (u32, u32) {
        for state in self.move_shape.ordered_monster_property_states() {
            match state {
                super::moveshape::MonsterPropertyState::Swordship(state) => {
                    (minimum, maximum) = state.apply_to_monster(minimum, maximum);
                }
                super::moveshape::MonsterPropertyState::BattleFairyAttribute(state) => {
                    minimum = state.apply_to_monster_attack(minimum);
                    maximum = state.apply_to_monster_attack(maximum);
                }
                super::moveshape::MonsterPropertyState::Fury(state) => {
                    maximum = state.apply_to_monster_max_attack(maximum);
                }
                super::moveshape::MonsterPropertyState::Weak(state) => {
                    (minimum, maximum) = state.apply_to_monster(minimum, maximum);
                }
                super::moveshape::MonsterPropertyState::GodBless(state) => {
                    (minimum, maximum, _) = state.apply_to_monster(minimum, maximum, 0);
                }
                super::moveshape::MonsterPropertyState::Roar(state) => {
                    (minimum, maximum, _) = state.apply_to_monster(minimum, maximum, 0);
                }
                super::moveshape::MonsterPropertyState::BossBlueFury(state) => {
                    minimum = state.apply_to_monster_attack(minimum);
                    maximum = state.apply_to_monster_attack(maximum);
                }
                super::moveshape::MonsterPropertyState::TaiJi(_)
                | super::moveshape::MonsterPropertyState::Origin(_)
                | super::moveshape::MonsterPropertyState::PoisonFog(_) => {}
            }
        }
        (minimum, maximum)
    }

    /// Exact `CMonster::GetElementModify` (RVA `0x000E6900`): накопленный
    /// runtime modifier сначала ограничивается нулём, затем pet factor `2`
    /// умножается в x87 и усекается к signed DWORD.
    pub(crate) fn element_modifier(&self, mut value: i32) -> u32 {
        for state in self.move_shape.ordered_monster_property_states() {
            match state {
                super::moveshape::MonsterPropertyState::Origin(state) => {
                    value = state.apply_to_monster(value);
                }
                super::moveshape::MonsterPropertyState::GodBless(state) => {
                    (_, _, value) = state.apply_to_monster(0, 0, value);
                }
                super::moveshape::MonsterPropertyState::Roar(state) => {
                    (_, _, value) = state.apply_to_monster(0, 0, value);
                }
                super::moveshape::MonsterPropertyState::BattleFairyAttribute(state) => {
                    value = state.apply_to_monster_element(value);
                }
                super::moveshape::MonsterPropertyState::TaiJi(_)
                | super::moveshape::MonsterPropertyState::Swordship(_)
                | super::moveshape::MonsterPropertyState::Fury(_)
                | super::moveshape::MonsterPropertyState::Weak(_)
                | super::moveshape::MonsterPropertyState::PoisonFog(_)
                | super::moveshape::MonsterPropertyState::BossBlueFury(_) => {}
            }
        }
        let base = value.max(0) as u32;
        if !self.has_player_pet_master() {
            return base;
        }
        let scaled = f64::from(base) * f64::from(f32::from_bits(self.factors[2]));
        scaled.trunc() as i32 as u32
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

    pub(crate) fn when_been_hurted_by(
        &mut self,
        attacker: ShapeIdentity,
        attacker_is_tamed: bool,
        now_ms: u32,
    ) {
        self.when_been_hurted(now_ms);
        if accepts_hurt_target(self.ai_target(), attacker, attacker_is_tamed) {
            self.base_ai.set_object_target(attacker);
        }
    }

    /// Общая часть `CBaseAI::WhenBeenHurted` без политики выбора цели
    /// конкретного производного ИИ.
    pub(crate) fn when_been_hurted(&mut self, now_ms: u32) {
        self.base_ai.when_been_hurted(now_ms);
    }

    pub(crate) fn when_been_stiffened(&mut self, delay_ms: u32, now_ms: u32) {
        self.base_ai.when_been_stiffened(delay_ms, now_ms);
    }

    pub(crate) fn when_passive_gladiator_hurted_by(
        &mut self,
        attacker: ShapeIdentity,
        now_ms: u32,
        attacker_is_owned_creature: bool,
    ) {
        self.when_been_hurted(now_ms);
        let already_fighting = self.ai_target().is_some();
        let selected = self.passive_gladiator_ai.as_mut().and_then(|state| {
            state.on_hurt(attacker, already_fighting, attacker_is_owned_creature)
        });
        if let Some(selected) = selected {
            self.base_ai.set_object_target(selected);
        }
    }

    pub(crate) fn when_pet_been_hurted_by(&mut self, attacker: ShapeIdentity, now_ms: u32) {
        self.base_ai.when_been_hurted(now_ms);
        if self.pet_behavior.on_hurt(self.ai_target(), attacker) {
            self.base_ai.set_object_target(attacker);
        }
    }

    pub(crate) fn when_been_killed(&mut self, now_ms: u32) {
        self.base_ai.when_been_killed(now_ms);
    }

    pub(crate) fn process_reached_defense_actions(
        &mut self,
        mut now_ms: impl FnMut() -> u32,
    ) -> usize {
        let passive_gladiator = self.passive_gladiator_ai.is_some();
        self.base_ai.process_reached_defense_actions(|ai| {
            if passive_gladiator {
                ai.begin_active_search_enemy(now_ms());
            }
        })
    }

    pub(crate) fn process_reached_stiffen_action(
        &mut self,
        now_ms: u32,
    ) -> PassiveStiffenAction {
        let action = self.base_ai.begin_reached_stiffen_action();
        if action.interrupts_attack() {
            while self.base_ai.stiffen_attack_pending() {
                let release_target = self.base_ai.stiffen_attack_needs_end()
                    && self.base_attack_cast().is_some();
                if release_target {
                    self.cancel_base_attack_cast();
                }
                self.base_ai.finish_stiffen_attack(release_target);
                self.base_ai.discard_active_prefix();
            }
        }
        self.base_ai.finish_reached_stiffen_action(action, now_ms)
    }

    pub(crate) fn begin_reached_death_action(&mut self) -> bool {
        self.base_ai.begin_reached_death_action()
    }

    /// Общий `CMonsterAI::OnLoseTarget` смерти очищает только target-поля и
    /// не отменяет сохранённый `ASA_MOVE`; расширенный `clear_ai_target`
    /// намеренно остаётся для обычных schedule/interruption путей.
    pub(crate) fn release_ai_target_for_death(&mut self) {
        self.base_ai.lose_target();
    }

    pub(crate) fn reached_death_action_state(&self) -> PassiveDeathAction {
        self.base_ai.reached_death_action_state()
    }

    pub(crate) fn finish_reached_death_action(&mut self, now_ms: u32) {
        self.base_ai.finish_reached_death_action(now_ms);
    }

    pub(crate) fn advance_active_ai_stand(&mut self, now_ms: u32) -> bool {
        self.base_ai.advance_active_stand(now_ms)
    }

    pub(crate) fn advance_handled_active_ai_action(&mut self, now_ms: u32) -> bool {
        self.base_ai.advance_handled_active_action(|| now_ms)
    }

    pub(crate) fn advance_handled_passive_ai_action(&mut self, now: impl FnOnce() -> u32) -> Option<bool> {
        self.base_ai.advance_handled_passive_action(now)
    }

    pub(crate) fn active_ai_change_skill_pending(&self) -> bool {
        self.base_ai.active_change_skill_pending()
    }

    pub(crate) fn active_ai_search_enemy_pending(&self) -> bool {
        self.base_ai.active_search_enemy_pending()
    }

    pub(crate) fn active_ai_attack_pending(&self) -> bool {
        self.base_ai.active_attack_pending()
    }

    pub(crate) fn queue_search_after_active_move(&mut self, ai_type: u32, now_ms: u32) {
        if !self.base_ai.active_move_unhandled() {
            return;
        }
        let alive = !CMoveShape::is_died(self.hit_points);
        let pet_search = self.tamed
            && alive
            && self.move_shape.current_skill_id().is_none();
        let passive_gladiator_search = ai_type == 1
            && alive
            && self
                .passive_gladiator_ai
                .as_ref()
                .is_some_and(PassiveGladiatorState::has_enemy_players);
        // `CGuardWithSword::OnMoving` RVA `0x0020E260` добавляет SearchEnemy
        // после успешного общего OnMoving и наследуется AI10/12/16; базовый
        // factory type AI9 обязан проходить тот же путь. Отдельный
        // `CGuardCountry::OnMoving` RVA `0x0020C4F0` делает то же для живых
        // factory-типов AI17/100, но не для самостоятельного AI101.
        // `CPassiveGladiator::OnMoving` RVA `0x00210E70` дополнительно требует
        // непустой `m_vEnemy`, которой соответствует owned IndexSet AI1.
        if (alive && matches!(ai_type, 4 | 17 | 100))
            || matches!(ai_type, 9 | 10 | 12 | 16)
            || passive_gladiator_search
            || pet_search
        {
            self.base_ai.begin_active_search_enemy(now_ms);
        }
    }

    pub(crate) fn begin_active_ai_move(&mut self, delay_ms: u32, now_ms: u32) {
        self.base_ai.begin_active_move(delay_ms, now_ms);
    }

    pub(crate) fn begin_active_ai_stand(&mut self, delay_ms: u32, now_ms: u32) {
        self.base_ai.begin_active_stand(delay_ms, now_ms);
    }

    pub(crate) fn begin_active_ai_search_enemy(&mut self, now_ms: u32) {
        self.base_ai.begin_active_search_enemy(now_ms);
    }

    pub(crate) fn begin_active_ai_change_skill(&mut self, now_ms: u32) {
        self.base_ai
            .add_ai_event(AiShapeAction::ChangeSkill, 0, 0, now_ms);
    }

    pub(crate) fn advance_active_ai_move(&mut self, now_ms: u32) -> bool {
        self.base_ai.advance_active_move(now_ms)
    }

    pub(crate) fn finish_active_ai_attack(&mut self, now_ms: u32) {
        self.base_ai.finish_active_attack(now_ms);
    }

    pub(crate) fn finish_active_ai_change_skill(&mut self, now_ms: u32) {
        self.base_ai.finish_active_change_skill(now_ms);
    }

    pub(crate) fn finish_active_ai_search_enemy(&mut self, now_ms: u32) {
        self.base_ai.finish_active_search_enemy(now_ms);
    }

    pub(crate) fn primary_ai_queues_idle(&self) -> bool {
        self.base_ai.primary_queues_idle()
    }

    pub(crate) const fn ai_target(&self) -> Option<ShapeIdentity> {
        if !self.base_ai.has_object_target() {
            return None;
        }
        self.base_ai.object_target()
    }

    pub(crate) fn set_ai_target(&mut self, target: ShapeIdentity) {
        self.pet_behavior.begin_ai_target(self.tamed);
        self.base_ai.set_object_target(target);
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
        if skill_id == SPIDER_MIST_SKILL_ID {
            self.move_shape.register_curable_skill_state(skill_id);
        }
        self.base_ai
            .add_ai_event(AiShapeAction::Attack, 0, 0, now_ms);
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
        self.finish_base_attack_cast_with_reuse(now_ms, true)
    }

    /// `CSkill::End(false)` завершает самостоятельное AI-действие, но не
    /// запускает reuse навыка. Это отличается от внешней отмены cast-а:
    /// derived AI всё равно получает своё обычное completion action.
    pub(crate) fn finish_base_attack_cast_without_reuse(
        &mut self,
        now_ms: u32,
    ) -> Option<MonsterBaseAttackCast> {
        self.finish_base_attack_cast_with_reuse(now_ms, false)
    }

    fn finish_base_attack_cast_with_reuse(
        &mut self,
        now_ms: u32,
        mark_reuse: bool,
    ) -> Option<MonsterBaseAttackCast> {
        let mut execution = self.base_attack_cast.take()?;
        let skill_id = execution.dispatch().skill_id;
        if skill_id == SPIDER_MIST_SKILL_ID {
            self.move_shape.finish_curable_skill_state(skill_id);
        }
        self.fast_attack_progress = None;
        self.monster_projectile_progress = None;
        self.path_projectile_progress = None;
        self.boss_fiend_penetrate_progress = None;
        self.little_star_progress = None;
        self.spider_web_progress = None;
        self.spider_mist_progress = None;
        self.yunsheng_lightning_progress = None;
        if self.attack_completion_action == AiShapeAction::ChangeSkill {
            self.move_shape.set_current_skill_id(None);
        }
        let _ = execution.terminate(SkillTermination::Completed);
        if mark_reuse {
            self.skill_last_used_ms.insert(skill_id, now_ms);
        }
        self.base_ai
            .add_ai_event(self.attack_completion_action, 0, 0, now_ms);
        Some(execution)
    }

    /// Завершает немедленный self-state навык без искусственного attack-cast.
    /// `CSkill::End(1)` фиксирует reuse независимо от активной/фоновой очереди.
    pub(crate) fn mark_immediate_skill_used(&mut self, skill_id: u32, now_ms: u32) {
        self.skill_last_used_ms.insert(skill_id, now_ms);
    }

    /// Активный `OnFighting` завершает и `End(0)`: очередь меняет навык,
    /// но отметка восстановления остаётся прежней. Фоновый вызов сюда не идёт.
    pub(crate) fn finish_active_immediate_skill(&mut self, now_ms: u32) {
        self.move_shape.shape_mut().set_action(1);
        if self.attack_completion_action == AiShapeAction::ChangeSkill {
            self.move_shape.set_current_skill_id(None);
        }
        self.base_ai
            .add_ai_event(self.attack_completion_action, 0, 0, now_ms);
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
        self.base_ai.lose_target();
        self.cancel_base_attack_cast();
        self.base_ai.cancel_active_move();
        self.pet_behavior.target_cleared(self.tamed);
    }

    pub(crate) fn lose_ai_target_and_search(&mut self, now_ms: u32) {
        self.clear_ai_target();
        self.base_ai.begin_active_search_enemy(now_ms);
    }

    pub(crate) fn cancel_base_attack_cast(&mut self) {
        self.fast_attack_progress = None;
        self.monster_projectile_progress = None;
        self.path_projectile_progress = None;
        self.boss_fiend_penetrate_progress = None;
        self.little_star_progress = None;
        self.spider_web_progress = None;
        self.spider_mist_progress = None;
        self.yunsheng_lightning_progress = None;
        if let Some(mut execution) = self.base_attack_cast.take() {
            let skill_id = execution.dispatch().skill_id;
            if skill_id == SPIDER_MIST_SKILL_ID {
                self.move_shape.finish_curable_skill_state(skill_id);
            }
            self.move_shape.set_current_skill_id(None);
            let _ = execution.terminate(SkillTermination::Cancelled);
        }
    }

    pub(crate) fn skill_last_used_ms(&self, skill_id: u32) -> u32 {
        self.skill_last_used_ms
            .get(&skill_id)
            .copied()
            .unwrap_or_default()
    }

    pub(crate) const fn begin_ai_attack_attempt(
        &mut self,
        now_ms: u32,
        interval_ms: u32,
    ) -> bool {
        self.ai_schedule
            .begin_attack_attempt(now_ms, interval_ms)
    }

    pub(crate) const fn set_base_attack_owned_tick(&mut self, owned: bool) {
        self.base_attack_owned_tick = owned;
    }

    pub(crate) fn take_base_attack_owned_tick(&mut self) -> bool {
        std::mem::take(&mut self.base_attack_owned_tick)
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

    /// Exact `CMonster::GetBeAttackedPoint` (RVA `0x000E6AA0`). Нулевая
    /// figure сохраняет базовую центральную точку `CMoveShape`.
    pub(crate) fn be_attacked_point(
        &self,
        property: &MonsterProperties,
        attacker_x: i32,
        attacker_y: i32,
    ) -> Option<(i32, i32)> {
        let view = self.shape_view(property)?;
        if property.figure as u8 == 0 {
            return Some((view.tile_x, view.tile_y));
        }
        Some(CMoveShape::nearest_figure_attack_point(
            view.tile_x,
            view.tile_y,
            view.figure,
            attacker_x,
            attacker_y,
        ))
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
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1389
// RVA: 0x000E65A0
// ADDRESS: 004e65a0
// PROTOTYPE: ulong __thiscall GetMaxHP(void)
//
// Реализовано выше как `maximum_hp`.
//

// ============================================================================
// FUNCTION: CMonster::GetMinAtk
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1401
// RVA: 0x000E6620
// ADDRESS: 004e6620
// PROTOTYPE: ulong __thiscall GetMinAtk(void)
//
// Реализовано выше через `pet_scaled_attack` и `state_attack_bounds`.
//

// ============================================================================
// FUNCTION: CMonster::GetMaxAtk
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1433
// RVA: 0x000E66C0
// ADDRESS: 004e66c0
// PROTOTYPE: ulong __thiscall GetMaxAtk(void)
//
// Реализовано выше через `pet_scaled_attack` и `state_attack_bounds`.
//

// ============================================================================
// FUNCTION: CMonster::GetDef
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1474
// RVA: 0x000E6780
// ADDRESS: 004e6780
// PROTOTYPE: ulong __thiscall GetDef(void)
//
// Реализовано выше как `defense`.
//

// ============================================================================
// FUNCTION: CMonster::GetDodge
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1489
// RVA: 0x000E6800
// ADDRESS: 004e6800
// PROTOTYPE: ushort __thiscall GetDodge(void)
//
// Реализовано выше как `dodge`.
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
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1510
// RVA: 0x000E6880
// ADDRESS: 004e6880
// PROTOTYPE: ulong __thiscall GetElementResistant(void)
//
// Реализовано выше как `element_resistance`.
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
// FUNCTION: CMonster::GetAtcInterval
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1581
// RVA: 0x000E69F0
// ADDRESS: 004e69f0
// PROTOTYPE: ushort __thiscall GetAtcInterval(void)
//
// Реализовано выше как `attack_interval`.
//

// ============================================================================
// FUNCTION: CMonster::GetStopFrame
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1601
// RVA: 0x000E6A40
// ADDRESS: 004e6a40
// PROTOTYPE: long __thiscall GetStopFrame(void)
//
// Реализовано выше как `stop_frame`.
//

// ============================================================================
// FUNCTION: CMonster::GetBeAttackedPoint
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1611
// RVA: 0x000E6AA0
// ADDRESS: 004e6aa0
// PROTOTYPE: void __thiscall GetBeAttackedPoint(long param_1, long param_2, long * param_3, long * param_4)
//
// Реализовано выше как `be_attacked_point`; сохранённый raw ниже фиксирует
// отличие нулевой figure от footprint-ветви.
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
// FUNCTION: CMonster::IsAttackAble
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// MATERIALIZED: player-attacker, monster-attacker и carriage/tamed ветви
// связаны через `CGame::monster_attackable_by_player`,
// `monster_attackable_by_monster` и `carriage_attackable_by_monster`.
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
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1591
// RVA: 0x000E79B0
// ADDRESS: 004e79b0
// PROTOTYPE: float __thiscall GetSpeed(void)
//
// Реализовано выше как `speed`.
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
// FUNCTION: CMonster::AddToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:151
// RVA: 0x000E8400
// ADDRESS: 004e8400
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Реализовано выше: CMoveShape client snapshot, maxHP/HP, kind/figure,
// sound/colors и exact ordinary/pet/carriage master tail. Property и имя
// master-а разрешает канонический владелец CGame.
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
