//! Общий исполняемый конвейер навыков GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/states/state.cpp`, `skill.cpp`, `attackskill.cpp`, `defenseskill.cpp` и
//! `stateskill.cpp`. Подтверждённый общий контракт —
//! последовательность `Begin → Check → Calculate → Attack → Apply`, хранение
//! времени начала и одно конечное состояние выполнения. Старую C++-иерархию
//! с виртуальными конструкторами и RTTI не воспроизводим: intrinsic-категория
//! берётся из фабричного owner-каталога, имя — из актуальных свойств ID/уровня.
//! Выбранный ID принадлежит CMoveShape, команды — CPlayerAI, а путь, поворот,
//! формулы, RNG и сетевые эффекты остаются у конкретного владельца навыка.
//! Типы исполнения игрока и боевого духа отделены от очередей CPlayerAI:
//! их данные принадлежат зарегистрированному экземпляру CMoveShape. Общий
//! enum и доступ к kernel не выполняют Begin либо concrete End автоматически.
//! SkillLifecycle хранит постоянную базу CState/CSkill. CState constructor
//! (0x005DBCA0) задаёт ended=true и нулевые source/target/coords/time;
//! CSkill constructor (0x004D8120) задаёт available=true, prepared=false.
//! Native сохраняет только type/ID; GUID не вводит дополнительный фильтр.
//! Begin объектов (0x005DBD70) обновляет лишь ненулевые стороны, Begin точки
//! (0x005DBDD0) очищает sufferer и сохраняет координаты. Оба сбрасывают ended
//! до OnBeginSkill; отказ не откатывает базу и не очищает prepared. Успех
//! CSkill::Begin (0x004D83E0) очищает prepared после callback.
//! Реестр переносит единственную базу между неактивным экземпляром и concrete
//! kernel; параллельного базового поля, Arc или обратного копирования нет.
//! Kernel::begin создаёт только временные данные конкретного исполнения,
//! без выдуманной identity; это не конструктор зарегистрированного навыка.
//! CSkill::End(0) (0x004D84C0) разрешает старый GetUser и вызывает OnEndSkill
//! до очистки полей; он не проверяет IsEnded и сохраняет available/reuse.
//! reset_after_end ниже очищает данные, затем вызывает переданное владельцем
//! удаление visual и лишь после этого выставляет ended. Это не callback и не
//! полный concrete End. Option<SkillVisualEffect> принадлежит самому экземпляру,
//! а не копируемой скалярной базе; пакет эффекта не заменяет ресурс.
//! Подключённый CRageEffect хранит только свою базу; остальные производные
//! visual остаются отдельной задачей. Diagnostic termination
//! не подменяет native ended; полное подключение registered End ещё требуется.
//!
//! Отложенные межвладельческие действия формируются до постановки команды
//! через `GameEffectJournal`; уже выполняемые синхронно боевые действия в
//! журнал не копируются.
//! Общий текст недостатка маны боевого духа получает fixed-point стоимость
//! через `double 0.0001` и x87-усечение; это одинаковый контракт всех
//! достигнутых concrete skill owner-ов.
//! Все достигнутые monster-skill reuse-gate вызывают общий `IsRestored` ниже,
//! включая исходный нулевой timestamp до первого применения; stage, missile
//! и periodic duration продолжают использовать elapsed-часы.
//! CState::Begin (0x005DBD70/0x005DBDD0) читает часы до OnBeginSkill.
//! Расписание пишет ранний отсчёт сразу в базу экземпляра, до конкретных
//! проверок ресурсов. Установщик переносит эту же базу в concrete kernel,
//! сохраняя уже выполненный первый переход в Check. Отдельного маркера
//! времени в CPlayerAI нет. Подключённые собственные Begin вызывают общий
//! переход в своей подтверждённой точке до concrete-проверок; отказ сохраняет
//! достигнутую базу. Остальные собственные Begin требуют отдельного связывания.
//! m_bSkillPrepared — независимый флаг CSkill (+0x44), не стадия Attack:
//! OnFighting (0x005092B0) переносит подготовленный экземпляр в фон до End.
//! Конкретный owner устанавливает его в подтверждённой точке выпуска.

use crate::gameserver::appserver::player::{BattleFairySkillDispatch, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::public::guid::CGuid;

use super::agility::AgilityFamilyExecutionState;
use super::archery::ArcheryExecutionState;
use super::armybreak::ArmyBreakExecutionState;
use super::basemagic::BaseMagicExecutionState;
use super::battlefairybasemagic::BattleFairyBaseMagicExecutionState;
use super::bloodrose::BloodRoseExecutionState;
use super::boalock::BoaLockExecutionState;
use super::bossbluequake::PlayerBossBlueQuakeExecutionState;
use super::bossfiendpenetrate::PlayerBossFiendPenetrateExecutionState;
use super::callosity::CallosityExecutionState;
use super::chainlightning::ChainLightningExecutionState;
use super::chaossphere::ChaosSphereExecutionState;
use super::directprojectile::PlayerDirectProjectileExecutionState;
use super::energybolt::PlayerPathProjectileExecutionState;
use super::explosivearrow::ExplosiveArrowExecutionState;
use super::fallingstar::FallingStarExecutionState;
use super::flash::FlashExecutionState;
use super::ghostcut::GhostCutExecutionState;
use super::heartlessarrow::HeartlessArrowExecutionState;
use super::heartlessarrow2::HeartlessArrowAreaExecutionState;
use super::knightcut::KnightCutExecutionState;
use super::lightingarrow::LightingArrowExecutionState;
use super::lightingarrow2::LightingArrow2ExecutionState;
use super::lightning::LightningExecutionState;
use super::littleflash::LittleFlashExecutionState;
use super::littlestar::PlayerLittleStarExecutionState;
use super::lordfastattack::LordFastAttackExecutionState;
use super::meteorarrow::MeteorArrowExecutionState;
use super::meteorarrowmass::MeteorArrowMassExecutionState;
use super::monsterthorn::PlayerMonsterThornExecutionState;
use super::poisonmoth::PoisonMothExecutionState;
use super::rage::RageExecutionState;
use super::rainarrow::RainArrowExecutionState;
use super::scorpion::ScorpionExecutionState;
use super::seal::SealExecutionState;
use super::sevenshootingstar::SevenShootingStarExecutionState;
use super::spidermist::PlayerSpiderMistExecutionState;
use super::spiderweb::PlayerSpiderWebExecutionState;
use super::spriteburn::SpriteBurnExecutionState;
use super::strike::StrikeExecutionState;
use super::summoncreatureskill::PlayerSummonCreatureExecutionState;
use super::swallow::SwallowExecutionState;
use super::yakshaslash::YakshaSlashExecutionState;
use super::yunshenglightning::PlayerYunShengLightningExecutionState;

// Типы игровых данных перечислены один раз: из них выводятся хранение,
// доступ к kernel и безопасное извлечение конкретного состояния.
macro_rules! player_skill_states {
    ($($variant:ident($state:ty)),+ $(,)?) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub(crate) enum PlayerSkillExecution {
            State(SkillExecutionKernel<PlayerSkillDispatch>),
            PoisonFog {
                kernel: SkillExecutionKernel<PlayerSkillDispatch>,
                destination: (i32, i32),
            },
            $($variant($state),)+
        }

        impl PlayerSkillExecution {
            pub(crate) fn kernel(&self) -> SkillExecutionKernel<PlayerSkillDispatch> {
                match self {
                    Self::State(kernel) | Self::PoisonFog { kernel, .. } => *kernel,
                    $(Self::$variant(state) => *state.kernel(),)+
                }
            }

            pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
                match self {
                    Self::State(kernel) | Self::PoisonFog { kernel, .. } => kernel,
                    $(Self::$variant(state) => state.kernel_mut(),)+
                }
            }

            pub(crate) fn lifecycle(&self) -> &SkillLifecycle {
                match self {
                    Self::State(kernel) | Self::PoisonFog { kernel, .. } => kernel.lifecycle(),
                    $(Self::$variant(state) => state.kernel().lifecycle(),)+
                }
            }

            pub(crate) fn lifecycle_mut(&mut self) -> &mut SkillLifecycle {
                self.kernel_mut().lifecycle_mut()
            }
        }

        $(
            impl From<$state> for PlayerSkillExecution {
                fn from(state: $state) -> Self { Self::$variant(state) }
            }

            impl PlayerSkillState for $state {
                fn from_execution(execution: &PlayerSkillExecution) -> Option<&Self> {
                    match execution {
                        PlayerSkillExecution::$variant(state) => Some(state),
                        _ => None,
                    }
                }

                fn from_execution_mut(execution: &mut PlayerSkillExecution) -> Option<&mut Self> {
                    match execution {
                        PlayerSkillExecution::$variant(state) => Some(state),
                        _ => None,
                    }
                }
            }
        )+
    };
}

pub(crate) trait PlayerSkillState {
    fn from_execution(execution: &PlayerSkillExecution) -> Option<&Self>;
    fn from_execution_mut(execution: &mut PlayerSkillExecution) -> Option<&mut Self>;
}

player_skill_states! {
    HeartlessArrowArea(HeartlessArrowAreaExecutionState),
    ExplosiveArrow(ExplosiveArrowExecutionState),
    AgilityFamily(AgilityFamilyExecutionState),
    GhostCut(GhostCutExecutionState),
    ArmyBreak(ArmyBreakExecutionState),
    LittleFlash(LittleFlashExecutionState),
    PathProjectile(PlayerPathProjectileExecutionState),
    DirectProjectile(PlayerDirectProjectileExecutionState),
    SummonCreature(PlayerSummonCreatureExecutionState),
    LordFastAttack(LordFastAttackExecutionState),
    Callosity(CallosityExecutionState),
    Archery(ArcheryExecutionState),
    HeartlessArrow(HeartlessArrowExecutionState),
    LightingArrow(LightingArrowExecutionState),
    LightingArrow2(LightingArrow2ExecutionState),
    MeteorArrowMass(MeteorArrowMassExecutionState),
    MeteorArrow(MeteorArrowExecutionState),
    RainArrow(RainArrowExecutionState),
    PoisonMoth(PoisonMothExecutionState),
    BloodRose(BloodRoseExecutionState),
    Scorpion(ScorpionExecutionState),
    BoaLock(BoaLockExecutionState),
    FallingStar(FallingStarExecutionState),
    Strike(StrikeExecutionState),
    YakshaSlash(YakshaSlashExecutionState),
    BaseMagic(BaseMagicExecutionState),
    ChainLightning(ChainLightningExecutionState),
    KnightCut(KnightCutExecutionState),
    Rage(RageExecutionState),
    Flash(FlashExecutionState),
    Swallow(SwallowExecutionState),
    SevenShootingStar(SevenShootingStarExecutionState),
    LittleStar(PlayerLittleStarExecutionState),
    YunshengLightning(PlayerYunShengLightningExecutionState),
    MonsterThorn(PlayerMonsterThornExecutionState),
    SpiderMist(PlayerSpiderMistExecutionState),
    SpiderWeb(PlayerSpiderWebExecutionState),
    BossBlueQuake(PlayerBossBlueQuakeExecutionState),
    BossFiendPenetrate(PlayerBossFiendPenetrateExecutionState),
    SpriteBurn(SpriteBurnExecutionState),
    ChaosSphere(ChaosSphereExecutionState),
    Lightning(LightningExecutionState),
    Seal(SealExecutionState),
}

impl From<SkillExecutionKernel<PlayerSkillDispatch>> for PlayerSkillExecution {
    fn from(state: SkillExecutionKernel<PlayerSkillDispatch>) -> Self { Self::State(state) }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyExecution {
    State(SkillExecutionKernel<BattleFairySkillDispatch>),
    BaseMagic(BattleFairyBaseMagicExecutionState),
}

impl BattleFairyExecution {
    pub(crate) fn kernel(&self) -> SkillExecutionKernel<BattleFairySkillDispatch> {
        match self {
            Self::State(state) => *state,
            Self::BaseMagic(state) => *state.kernel(),
        }
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<BattleFairySkillDispatch> {
        match self {
            Self::State(state) => state,
            Self::BaseMagic(state) => state.kernel_mut(),
        }
    }

    pub(crate) fn lifecycle(&self) -> &SkillLifecycle {
        match self {
            Self::State(state) => state.lifecycle(),
            Self::BaseMagic(state) => state.kernel().lifecycle(),
        }
    }

    pub(crate) fn lifecycle_mut(&mut self) -> &mut SkillLifecycle {
        self.kernel_mut().lifecycle_mut()
    }
}

pub(crate) fn battle_fairy_mana_text_cost(cost: u32) -> u32 {
    (f64::from(cost) * 0.0001_f64).trunc() as i64 as u32
}

/// Точная беззнаковая проверка `CSkill::IsRestored`: сложение выполняется в
/// `u32`, после чего результат сравнивается с текущими миллисекундами. Это не
/// устойчивый к переполнению срок и потому намеренно отличается от
/// периодических часов.
pub(crate) const fn skill_is_restored(
    last_used_ms: u32,
    reuse_delay_ms: u32,
    now_ms: u32,
) -> bool {
    last_used_ms.wrapping_add(reuse_delay_ms) <= now_ms
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum SkillStage {
    Begin,
    Check,
    Calculate,
    Attack,
    Apply,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SkillTermination {
    Completed,
    Rejected,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SkillLifecycle {
    user: (i32, ShapeIdentity),
    sufferer: (i32, ShapeIdentity),
    destination: (i32, i32),
    started_at_ms: u32,
    ended: bool,
    available: bool,
    prepared: bool,
    termination: Option<SkillTermination>,
}

impl Default for SkillLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillLifecycle {
    const EMPTY_IDENTITY: ShapeIdentity = ShapeIdentity {
        object_type: 0,
        id: 0,
        ex_id: CGuid::GUID_INVALID,
    };

    const fn new() -> Self {
        Self {
            user: (0, Self::EMPTY_IDENTITY),
            sufferer: (0, Self::EMPTY_IDENTITY),
            destination: (0, 0),
            started_at_ms: 0,
            ended: true,
            available: true,
            prepared: false,
            termination: None,
        }
    }

    const fn for_execution(started_at_ms: u32) -> Self {
        Self {
            started_at_ms,
            ended: false,
            ..Self::new()
        }
    }

    const fn native_identity(identity: ShapeIdentity) -> ShapeIdentity {
        ShapeIdentity {
            ex_id: CGuid::GUID_INVALID,
            ..identity
        }
    }

    pub(crate) const fn user(&self) -> (i32, ShapeIdentity) {
        self.user
    }

    pub(crate) const fn sufferer(&self) -> (i32, ShapeIdentity) {
        self.sufferer
    }

    pub(crate) const fn destination(&self) -> (i32, i32) {
        self.destination
    }

    pub(crate) const fn started_at_ms(&self) -> u32 {
        self.started_at_ms
    }

    pub(crate) const fn is_ended(&self) -> bool {
        self.ended
    }

    pub(crate) const fn is_available(&self) -> bool {
        self.available
    }

    pub(crate) fn set_available(&mut self, available: bool) {
        self.available = available;
    }

    pub(crate) const fn is_prepared(&self) -> bool {
        self.prepared
    }

    pub(crate) fn mark_prepared(&mut self) {
        self.prepared = true;
    }

    pub(crate) const fn termination(&self) -> Option<SkillTermination> {
        self.termination
    }

    /// Записи CState предшествуют OnBeginSkill. Нулевой объектный аргумент
    /// сохраняет соответствующую прежнюю сторону, а не очищает её.
    pub(crate) fn begin_objects(
        &mut self,
        source: Option<(i32, ShapeIdentity)>,
        target: Option<(i32, ShapeIdentity)>,
        now: impl FnOnce() -> u32,
    ) {
        if let Some((region_id, identity)) = source {
            self.started_at_ms = now();
            self.user = (region_id, Self::native_identity(identity));
        }
        if let Some((region_id, identity)) = target {
            self.sufferer = (region_id, Self::native_identity(identity));
            self.destination = (0, 0);
        }
        self.ended = false;
        self.termination = None;
    }

    pub(crate) fn begin_point(
        &mut self,
        source: (i32, ShapeIdentity),
        destination: (i32, i32),
        now: impl FnOnce() -> u32,
    ) {
        self.started_at_ms = now();
        self.user = (source.0, Self::native_identity(source.1));
        self.sufferer = (0, Self::EMPTY_IDENTITY);
        self.destination = destination;
        self.ended = false;
        self.termination = None;
    }

    pub(crate) fn finish_begin(&mut self, success: bool) -> bool {
        if success {
            self.prepared = false;
        }
        success
    }

    /// Сброс полей и удаление visual после внешних OnEndSkill/concrete cleanup.
    /// Повторный вызов допустим; этот метод сам не исполняет полный native End.
    pub(crate) fn reset_after_end(
        &mut self,
        termination: SkillTermination,
        clear_visual: impl FnOnce(),
    ) {
        self.user = (0, Self::EMPTY_IDENTITY);
        self.sufferer = (0, Self::EMPTY_IDENTITY);
        self.destination = (0, 0);
        self.started_at_ms = 0;
        self.prepared = false;
        clear_visual();
        self.ended = true;
        self.termination = Some(termination);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SkillExecutionKernel<Dispatch> {
    dispatch: Dispatch,
    stage: SkillStage,
    lifecycle: SkillLifecycle,
}

impl<Dispatch: Copy + Eq> SkillExecutionKernel<Dispatch> {
    pub(crate) const fn begin(dispatch: Dispatch, started_at_ms: u32) -> Self {
        Self {
            dispatch,
            stage: SkillStage::Begin,
            lifecycle: SkillLifecycle::for_execution(started_at_ms),
        }
    }

    pub(crate) const fn dispatch(self) -> Dispatch {
        self.dispatch
    }

    pub(crate) const fn started_at_ms(self) -> u32 {
        self.lifecycle.started_at_ms()
    }

    pub(crate) const fn stage(self) -> SkillStage {
        self.stage
    }

    pub(crate) const fn termination(self) -> Option<SkillTermination> {
        self.lifecycle.termination()
    }

    pub(crate) const fn is_prepared(self) -> bool {
        self.lifecycle.is_prepared()
    }

    pub(crate) fn mark_prepared(&mut self) {
        if self.lifecycle.termination().is_none() {
            self.lifecycle.mark_prepared();
        }
    }

    pub(crate) fn advance(&mut self, expected: SkillStage, next: SkillStage) -> bool {
        if self.lifecycle.termination().is_some() || self.stage != expected || next <= expected {
            return false;
        }
        self.stage = next;
        true
    }

    pub(crate) const fn lifecycle(&self) -> &SkillLifecycle {
        &self.lifecycle
    }

    pub(crate) fn lifecycle_mut(&mut self) -> &mut SkillLifecycle {
        &mut self.lifecycle
    }

    pub(crate) fn replace_lifecycle(&mut self, lifecycle: SkillLifecycle) -> SkillLifecycle {
        std::mem::replace(&mut self.lifecycle, lifecycle)
    }
}
