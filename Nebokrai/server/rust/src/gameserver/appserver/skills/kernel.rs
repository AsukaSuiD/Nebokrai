//! Общий исполняемый конвейер навыков GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/states/state.cpp`, `skill.cpp`, `attackskill.cpp`, `defenseskill.cpp` и
//! `stateskill.cpp`. Rust-последовательность
//! `Begin → Check → Calculate → Attack → Apply` упорядочивает достигнутые фазы,
//! но не воспроизводит числовые поля native-классов. Время начала и конечное
//! состояние принадлежат единственной базе. Старую C++-иерархию
//! с виртуальными конструкторами и RTTI не воспроизводим: intrinsic-категория
//! берётся из фабричного owner-каталога, имя — из актуальных свойств ID/уровня.
//! Выбранный ID принадлежит CMoveShape, команды — CPlayerAI, а путь, поворот,
//! формулы, RNG и сетевые эффекты остаются у конкретного владельца навыка.
//! Типы исполнения игрока и боевого духа отделены от очередей CPlayerAI:
//! их данные принадлежат зарегистрированному экземпляру CMoveShape. Общий
//! enum и доступ к kernel не выполняют Begin либо concrete End автоматически.
//! Небольшой отдельный каталог BattleFairy выводит enum, доступ к единственному
//! kernel, From и узкие End-hooks для BaseMagic/FatalBlow. Полётный скаляр
//! FatalBlow принадлежит concrete исполнению, а общий visual остаётся ресурсом
//! зарегистрированного навыка. BF End(int) 0x00516FB0/0x0051A700/0x005222A0
//! обнуляет DWORD +0x4C/+0x50 до visual и AfterUse: у BloodLoss Begin
//! (0x0051A606) ставит +0x4C=1, а AI (0x0051B433) при нуле сразу выходит;
//! +0x50 в 0x0051B4B7 отдельно пропускает первую проверку. End(bool)
//! 0x0051BE50/0x005246C0 очищает BYTE +0x4C/+0x4D, но не подменяет End(int).
//! Idle представляет выключенную concrete-фазу, не новый Begin и не ended
//! базы: source/target/time/visual ещё доступны последующим End-действиям.
//! FatalBlow End дополнительно очищает flying-time; BaseMagic attack-time
//! сохраняется. Выбор применимого int/bool-пролога остаётся у владельца End.
//! Каталог Player связывает узкие derived End-переходы и освобождение путей:
//! scalar/kernel остаётся у экземпляра, а порядок относительно movement/visual
//! определяет общий End. Hooks не меняют FIFO, selection, базовую available
//! или самостоятельные региональные phalanx. Ненулевой HeartLessArrow End
//! может только выпустить удерживаемую стрелу и запретить общий хвост.
//! Hooks очищают подтверждённые поля существующей Rust-проекции. Отдельные
//! прочие derived available/condition/skill-casted, пока не представленные у owner-а,
//! не кодируются записью в base available или произвольным откатом stage.
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
//! Данные производных visual не копируются в kernel; общий ресурс хранит
//! зарегистрированный экземпляр. Diagnostic termination
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

use super::archery::ArcheryExecutionState;
use super::armybreak::ArmyBreakExecutionState;
use super::basemagic::BaseMagicExecutionState;
use super::battlefairybasemagic::BattleFairyBaseMagicExecutionState;
use super::bloodrose::BloodRoseExecutionState;
use super::boalock::BoaLockExecutionState;
use super::bossbluequake::PlayerBossBlueQuakeExecutionState;
use super::bossfiendpenetrate::PlayerBossFiendPenetrateExecutionState;
use super::chainlightning::ChainLightningExecutionState;
use super::chaossphere::ChaosSphereExecutionState;
use super::directprojectile::PlayerDirectProjectileExecutionState;
use super::energybolt::PlayerPathProjectileExecutionState;
use super::explosivearrow::ExplosiveArrowExecutionState;
use super::fatalblow::FatalBlowExecutionState;
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
use super::monsterthorn::PlayerMonsterThornExecutionState;
use super::poisonmoth::PoisonMothExecutionState;
use super::rage::RageExecutionState;
use super::rainarrow::RainArrowExecutionState;
use super::rush::RushExecutionState;
use super::scorpion::ScorpionExecutionState;
use super::seal::SealExecutionState;
use super::sevenshootingstar::SevenShootingStarExecutionState;
use super::spidermist::PlayerSpiderMistExecutionState;
use super::spiderweb::PlayerSpiderWebExecutionState;
use super::spriteburn::SpriteBurnExecutionState;
use super::strike::StrikeExecutionState;
use super::summoncreatureskill::PlayerSummonCreatureExecutionState;
use super::swallow::SwallowExecutionState;
use super::thunderblow2::ThunderBlow2Execution;
use super::yakshaslash::YakshaSlashExecutionState;
use super::yunshenglightning::PlayerYunShengLightningExecutionState;

// Типы игровых данных перечислены один раз: из них выводятся хранение,
// доступ к kernel и безопасное извлечение конкретного состояния.
macro_rules! player_skill_states {
    ($($variant:ident($state:ty) $(prepare($prepare:ident))? $(paths($paths:ident))?),+ $(,)?) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub(crate) enum PlayerSkillExecution {
            State(SkillExecutionKernel<PlayerSkillDispatch>),
            $($variant($state),)+
        }

        impl PlayerSkillExecution {
            pub(crate) fn kernel(&self) -> SkillExecutionKernel<PlayerSkillDispatch> {
                match self {
                    Self::State(kernel) => *kernel,
                    $(Self::$variant(state) => *state.kernel(),)+
                }
            }

            pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
                match self {
                    Self::State(kernel) => kernel,
                    $(Self::$variant(state) => state.kernel_mut(),)+
                }
            }

            pub(crate) fn lifecycle(&self) -> &SkillLifecycle {
                match self {
                    Self::State(kernel) => kernel.lifecycle(),
                    $(Self::$variant(state) => state.kernel().lifecycle(),)+
                }
            }

            pub(crate) fn lifecycle_mut(&mut self) -> &mut SkillLifecycle {
                self.kernel_mut().lifecycle_mut()
            }

            pub(crate) fn prepare_derived_end(&mut self, _argument: i32) -> bool {
                match self {
                    Self::State(_) => true,
                    $(Self::$variant(_state) => player_skill_states!(@prepare _state, _argument $(, $prepare)?),)+
                }
            }

            pub(crate) fn clear_end_paths(&mut self) {
                match self {
                    Self::State(_) => {},
                    $(Self::$variant(_state) => player_skill_states!(@paths _state $(, $paths)?),)+
                }
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
    (@prepare $state:ident, $argument:ident) => { true };
    (@prepare $state:ident, $argument:ident, $method:ident) => { $state.$method($argument) };
    (@paths $state:ident) => { {} };
    (@paths $state:ident, $method:ident) => { $state.$method() };
}

pub(crate) trait PlayerSkillState {
    fn from_execution(execution: &PlayerSkillExecution) -> Option<&Self>;
    fn from_execution_mut(execution: &mut PlayerSkillExecution) -> Option<&mut Self>;
}

player_skill_states! {
    HeartlessArrowArea(HeartlessArrowAreaExecutionState),
    ExplosiveArrow(ExplosiveArrowExecutionState) paths(clear_end_paths),
    GhostCut(GhostCutExecutionState) paths(clear_end_paths),
    ThunderBlow2(ThunderBlow2Execution) prepare(prepare_derived_end),
    ArmyBreak(ArmyBreakExecutionState) prepare(prepare_derived_end),
    LittleFlash(LittleFlashExecutionState) paths(clear_end_paths),
    PathProjectile(PlayerPathProjectileExecutionState) paths(clear_end_paths),
    DirectProjectile(PlayerDirectProjectileExecutionState),
    SummonCreature(PlayerSummonCreatureExecutionState),
    LordFastAttack(LordFastAttackExecutionState),
    Archery(ArcheryExecutionState),
    HeartlessArrow(HeartlessArrowExecutionState) prepare(prepare_derived_end) paths(clear_end_paths),
    LightingArrow(LightingArrowExecutionState) paths(clear_end_paths),
    LightingArrow2(LightingArrow2ExecutionState) paths(clear_end_paths),
    RainArrow(RainArrowExecutionState) paths(clear_end_paths),
    PoisonMoth(PoisonMothExecutionState) paths(clear_end_paths),
    BloodRose(BloodRoseExecutionState) paths(clear_end_paths),
    Scorpion(ScorpionExecutionState),
    BoaLock(BoaLockExecutionState),
    Strike(StrikeExecutionState),
    YakshaSlash(YakshaSlashExecutionState),
    BaseMagic(BaseMagicExecutionState),
    ChainLightning(ChainLightningExecutionState),
    KnightCut(KnightCutExecutionState),
    Rage(RageExecutionState),
    Flash(FlashExecutionState) paths(clear_end_paths),
    Rush(RushExecutionState) paths(clear_end_paths),
    Swallow(SwallowExecutionState) prepare(prepare_derived_end),
    SevenShootingStar(SevenShootingStarExecutionState) paths(clear_end_paths),
    LittleStar(PlayerLittleStarExecutionState) paths(clear_end_paths),
    YunshengLightning(PlayerYunShengLightningExecutionState),
    MonsterThorn(PlayerMonsterThornExecutionState),
    SpiderMist(PlayerSpiderMistExecutionState),
    SpiderWeb(PlayerSpiderWebExecutionState) prepare(prepare_derived_end),
    BossBlueQuake(PlayerBossBlueQuakeExecutionState),
    BossFiendPenetrate(PlayerBossFiendPenetrateExecutionState) paths(clear_end_paths),
    SpriteBurn(SpriteBurnExecutionState),
    ChaosSphere(ChaosSphereExecutionState),
    Lightning(LightningExecutionState),
    Seal(SealExecutionState),
}

impl From<SkillExecutionKernel<PlayerSkillDispatch>> for PlayerSkillExecution {
    fn from(state: SkillExecutionKernel<PlayerSkillDispatch>) -> Self { Self::State(state) }
}

macro_rules! battle_fairy_skill_states {
    ($($variant:ident($state:ty) $(prepare($prepare:ident))?),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(crate) enum BattleFairyExecution {
            State(SkillExecutionKernel<BattleFairySkillDispatch>),
            $($variant($state),)+
        }

        impl BattleFairyExecution {
            pub(crate) fn kernel(&self) -> SkillExecutionKernel<BattleFairySkillDispatch> {
                match self {
                    Self::State(state) => *state,
                    $(Self::$variant(state) => *state.kernel(),)+
                }
            }

            pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<BattleFairySkillDispatch> {
                match self {
                    Self::State(state) => state,
                    $(Self::$variant(state) => state.kernel_mut(),)+
                }
            }

            pub(crate) fn lifecycle(&self) -> &SkillLifecycle {
                match self {
                    Self::State(state) => state.lifecycle(),
                    $(Self::$variant(state) => state.kernel().lifecycle(),)+
                }
            }

            pub(crate) fn lifecycle_mut(&mut self) -> &mut SkillLifecycle {
                self.kernel_mut().lifecycle_mut()
            }

            pub(crate) fn prepare_derived_end(&mut self) {
                self.kernel_mut().clear_phase_for_end();
                match self {
                    Self::State(_) => {},
                    $(Self::$variant(_state) => battle_fairy_skill_states!(@prepare _state $(, $prepare)?),)+
                }
            }
        }

        $(
            impl From<$state> for BattleFairyExecution {
                fn from(state: $state) -> Self { Self::$variant(state) }
            }
        )+
    };
    (@prepare $state:ident) => { {} };
    (@prepare $state:ident, $method:ident) => { $state.$method() };
}

battle_fairy_skill_states! {
    BaseMagic(BattleFairyBaseMagicExecutionState),
    FatalBlow(FatalBlowExecutionState) prepare(prepare_derived_end),
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
    Idle,
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

    /// Переназначение цели внутри AI меняет только её type/id, без нового
    /// Begin, сброса координат или замены региона и времени исходной базы.
    pub(crate) fn set_sufferer_identity(&mut self, identity: ShapeIdentity) {
        self.sufferer.1 = Self::native_identity(identity);
    }

    pub(crate) const fn destination(&self) -> (i32, i32) {
        self.destination
    }

    pub(crate) fn set_destination(&mut self, destination: (i32, i32)) {
        self.destination = destination;
    }

    /// Фиксирует точку вместо объекта без нового Begin и повторного чтения часов.
    pub(crate) fn set_point_target(&mut self, destination: (i32, i32)) {
        self.sufferer = (0, Self::EMPTY_IDENTITY);
        self.destination = destination;
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
        self.set_point_target(destination);
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
        self.clear_end_context();
        self.finish_end(termination, clear_visual);
    }

    /// CSkill::End очищает участников и prepared до чтения часов reuse.
    pub(crate) fn clear_end_context(&mut self) {
        self.user = (0, Self::EMPTY_IDENTITY);
        self.sufferer = (0, Self::EMPTY_IDENTITY);
        self.destination = (0, 0);
        self.started_at_ms = 0;
        self.prepared = false;
    }

    pub(crate) fn finish_end(
        &mut self,
        termination: SkillTermination,
        clear_visual: impl FnOnce(),
    ) {
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

    pub(crate) fn clear_phase_for_end(&mut self) {
        self.stage = SkillStage::Idle;
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
