//! Общий исполняемый конвейер навыков GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/states/skill.cpp`, `attackskill.cpp`, `defenseskill.cpp` и
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
//! Kernel описывает уже начатое исполнение, но не подменяет зарегистрированный
//! экземпляр. Полное состояние CSkill до Begin и после End, как и общий
//! registered-skill End, ещё требует соединения с реестром CMoveShape.
//! Lifecycle визуальных state-эффектов сохраняет отдельный исходный owner.
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
//! При установке нового kernel расписание может передать этот ранний отсчёт,
//! чтобы проверки ресурсов не сдвигали начало каста. Вызов ограничен
//! установщиками нового исполнения в реестре экземпляров и привязан к dispatch. Некоторые
//! владельцы до установки уже выполняют первый переход в Check; это не
//! основание терять исходный отсчёт. Активный AI сюда повторно не входит.
//! m_bSkillPrepared — независимый флаг CSkill (+0x44), не стадия Attack:
//! OnFighting (0x005092B0) переносит подготовленный экземпляр в фон до End.
//! Конкретный owner устанавливает его в подтверждённой точке выпуска.

use crate::gameserver::appserver::player::{BattleFairySkillDispatch, PlayerSkillDispatch};

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
                    $(Self::$variant(state) => state.kernel().clone(),)+
                }
            }

            pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
                match self {
                    Self::State(kernel) | Self::PoisonFog { kernel, .. } => kernel,
                    $(Self::$variant(state) => state.kernel_mut(),)+
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
            Self::BaseMagic(state) => state.kernel(),
        }
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<BattleFairySkillDispatch> {
        match self {
            Self::State(state) => state,
            Self::BaseMagic(state) => state.kernel_mut(),
        }
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
pub(crate) struct SkillExecutionKernel<Dispatch> {
    dispatch: Dispatch,
    started_at_ms: u32,
    stage: SkillStage,
    termination: Option<SkillTermination>,
    prepared: bool,
}

impl<Dispatch: Copy + Eq> SkillExecutionKernel<Dispatch> {
    pub(crate) const fn begin(dispatch: Dispatch, started_at_ms: u32) -> Self {
        Self {
            dispatch,
            started_at_ms,
            stage: SkillStage::Begin,
            termination: None,
            prepared: false,
        }
    }

    pub(crate) const fn dispatch(self) -> Dispatch {
        self.dispatch
    }

    pub(crate) const fn started_at_ms(self) -> u32 {
        self.started_at_ms
    }

    pub(crate) fn inherit_scheduled_begin(&mut self, begin: Option<(Dispatch, u32)>) {
        if self.termination.is_none()
            && let Some((dispatch, started_at_ms)) = begin
            && dispatch == self.dispatch
        {
            self.started_at_ms = started_at_ms;
        }
    }

    pub(crate) const fn stage(self) -> SkillStage {
        self.stage
    }

    pub(crate) const fn termination(self) -> Option<SkillTermination> {
        self.termination
    }

    pub(crate) const fn is_prepared(self) -> bool {
        self.prepared
    }

    pub(crate) fn mark_prepared(&mut self) {
        if self.termination.is_none() {
            self.prepared = true;
        }
    }

    pub(crate) fn advance(&mut self, expected: SkillStage, next: SkillStage) -> bool {
        if self.termination.is_some() || self.stage != expected || next <= expected {
            return false;
        }
        self.stage = next;
        true
    }

    pub(crate) fn terminate(&mut self, termination: SkillTermination) -> bool {
        if self.termination.is_some() {
            return false;
        }
        self.termination = Some(termination);
        self.prepared = false;
        true
    }
}
