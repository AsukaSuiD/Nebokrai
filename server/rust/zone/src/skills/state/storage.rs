//! Арена экземпляров `CMoveShape::m_vStates` из GameServer.exe/GameServer.pdb
//! (пара gameserver.exe SHA-256 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
//! ↔ GameServer.pdb RSDS 5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53 age 2, совпадают).
//! Перенесена из переходного Game: хранилище — бывший
//! `appserver/moveshape/state_storage.rs`, `CanonicalStateStorage` и
//! `LegacyStateCodec` — бывший `appserver/moveshape.rs` (шаг B moveshape п3).
//! Сохранённые RAW RemoveState (0x004CDAB0, 0x004CDB20): append/remove/insert
//! записей — соседний `mutations` (волна Z-M2b),
//! AddExStatesToByteArray (0x004D10F0) — `serialization`,
//! общий UpdateAbnormality (0x004CFD00) остаётся у владельца hub moveshape
//! в его states/state.rs.
//! Порядок добавления и пустые позиции после удаления принадлежат этому
//! контейнеру; уплотнение выполняется только явно. Новый экземпляр получает
//! новый поколенческий ключ даже при замене в прежней позиции. Общий End
//! вызывается снаружи и может изменить тот же список до перечитывания позиции.
//! Активный CSpiderMist не входит в этот контейнер: exact RTTI 0x0066F16C
//! задаёт CSummonSkill→CSkill→CState, а три Begin не вызывают AddState.
//! Проверка ID 0x198 внутри CastCure сама по себе не создаёт state-owner.
//! SlotMap заменяет владение сырыми указателями, Vec сохраняет нативный порядок.
//! Для пяти участков чистого расчёта CFightDefense существует временный slice-
//! адаптер StateBatch: ключи и позиции остаются живыми при вынутом payload.
//! Между take_batch и restore_batch не допускаются игровые callbacks; возврат
//! заполняет только ещё живые пустые слоты, не воскрешая удалённый экземпляр.
//! Единственный enum-каталог объединяет существующие typed payload, не вводя
//! второго каталога игровых ID. Codec и таймеры повторного приёма предметов
//! остаются отдельными данными владельца, а не дополнительными состояниями.
//! Игровой AI/End-dispatch находится в hub states/state.rs; это хранилище не
//! вызывает callbacks при Drop и не подменяет End общим сбросом payload.
//! Запись арены различает runtime-установку и загрузку: StartAllStates
//! (0x004CE050) вызывает Begin(nullptr, holder), не превращая пустой GetUser
//! в текущего держателя. Это существенно для базового End (0x005DBCE0).
//! Loaded ctor начинает с ended=true и visual=NULL. Runtime-регистрация
//! фиксирует результат уже исполненного concrete Begin: ended=false и
//! состояние base visual из единого callback-каталога, без повторной рассылки.
//! Различаются loop=1, живой loop=0 GodBless2, уже завершённые one-shot
//! Agility2/Promotion и owners без ресурса; повторный Begin заменяет ресурс.
//! DecodeExStates 0x004D1B18 записывает sufferer type/id держателя, но оставляет
//! region=0 из CState ctor 0x005DBCA0. Object Begin устанавливает текущий
//! sufferer-region. Первичная установка GodBless/Fog/BF/Ex/CHBY/Undead/Ride записывает фактические
//! User/Sufferer identity и region, включая допустимый NULL; Begin(NULL,holder)
//! при повторном входе сохраняет User. Для ещё не перенесённых primary owners
//! остаётся явная holder-привязка. Отдельный признак from_save больше не
//! подменяет GetUser; сохранённый нулевой region не заменяется fallback.
//! Cache-span загруженной записи принадлежит тому же поколенческому ключу.
//! Это технические границы Serialize-cache, не native input-offset:
//! Tian читает 10 байт, но пишет 12. Удаление/вставка сдвигает общие spans,
//! не разрешая отдельному typed payload перезаписать неизвестный raw-tail.
//! Serializer заимствует ту же арену в ReadOnly либо Save-режиме. Только Save
//! допускает запись уже вычисленного remaining в текущий ключ до следующей
//! позиции; адаптер не знает игровых типов, не копирует арену и не читает часы.
//! Клиентская проекция записей и runtime-план visual — соседний `catalog`.

use std::ops::{Deref, DerefMut};

use slotmap::{SlotMap, new_key_type};

use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};

use crate::effects::{
    AgilityState2, AutomaticRestoreState, BattleFairyAttributeState, BlindState, BloodLossState,
    BoaLockState, BossBlueFuryState, BossBlueQuakeState, CHANGE_BODY_STATE_ID, CTeamState,
    CVisualEffect, CallosityFamilyState, ChangeBodyState, ConsumableRestoreIntervals,
    ConsumableRestoreState, CureState, DaubPoisonState, DefenseShieldState, EnergyHoldingState,
    EnlargeFullMissState, EnlargeMaxHpState, EnlargeMaxMpState, ExtendedState, FuryState,
    GodBlessState, HealState, HeartenState, KnightCutState, KnockOutState, LeafCutState,
    MeteorArrowState, OriginState, ParticularState, PersistentAgilityFamilyState, PillarState,
    PoisonFogState, RIDE_STATE_ID, RageBreakState, RideState, RoarState, ScriptMoveState,
    SoulCollectState, SpiderWebState, SwordshipState, TaiJiState, TianShenXiaFanState,
    UNDEAD_STATE_ID, UndeadState, WangshengState, WeakState, WuXingState,
};
use crate::regions::ShapeIdentity;
use crate::skills::is_cure_removable_state_id;
use crate::skills::statefactory::{
    KeroseneState, LeafCutState2, LeafCutState3, PoisonArrowState, Rush2State, RushState,
    SealState, SpiderPoisonState, SpriteBurnState, StrikeState,
};

use super::catalog::registered_runtime_state_visual;

new_key_type! {
    pub struct StateKey;
}

pub trait AppliedState: Sized + 'static {
    fn into_data(self) -> StateData;
    fn as_data_ref(data: &StateData) -> Option<&Self>;
    fn as_data_mut(data: &mut StateData) -> Option<&mut Self>;
    fn from_data(data: StateData) -> Option<Self>;
}

macro_rules! applied_states {
    ($($variant:ident($payload:ty) => |$state:ident| $id:expr),+ $(,)?) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum StateData {
            $($variant($payload)),+
        }

        impl StateData {
            pub fn state_id(&self) -> u32 {
                match self {
                    $(Self::$variant($state) => $id as u32),+
                }
            }
        }

        $(
            impl AppliedState for $payload {
                fn into_data(self) -> StateData {
                    StateData::$variant(self)
                }

                fn as_data_ref(data: &StateData) -> Option<&Self> {
                    match data {
                        StateData::$variant(state) => Some(state),
                        _ => None,
                    }
                }

                fn as_data_mut(data: &mut StateData) -> Option<&mut Self> {
                    match data {
                        StateData::$variant(state) => Some(state),
                        _ => None,
                    }
                }

                fn from_data(data: StateData) -> Option<Self> {
                    match data {
                        StateData::$variant(state) => Some(state),
                        _ => None,
                    }
                }
            }
        )+
    };
}

applied_states! {
    PersistentAgility(PersistentAgilityFamilyState) => |state| state.skill_id(),
    Agility2(AgilityState2) => |state| state.skill_id(),
    Callosity(CallosityFamilyState) => |state| state.skill_id(),
    TaiJi(TaiJiState) => |state| state.skill_id(),
    EnlargeFullMiss(EnlargeFullMissState) => |state| state.skill_id(),
    EnlargeMaxHp(EnlargeMaxHpState) => |state| state.skill_id(),
    EnlargeMaxMp(EnlargeMaxMpState) => |state| state.skill_id(),
    Origin(OriginState) => |state| state.skill_id(),
    Hearten(HeartenState) => |state| state.skill_id(),
    Heal(HealState) => |state| state.skill_id(),
    Fury(FuryState) => |state| state.skill_id(),
    RageBreak(RageBreakState) => |state| state.skill_id(),
    BossBlueFury(BossBlueFuryState) => |state| state.skill_id(),
    BossBlueQuake(BossBlueQuakeState) => |state| state.skill_id(),
    Cure(CureState) => |state| state.skill_id(),
    DaubPoison(DaubPoisonState) => |state| state.skill_id(),
    Seal(SealState) => |state| state.skill_id(),
    PoisonArrow(PoisonArrowState) => |state| state.skill_id(),
    PoisonFog(PoisonFogState) => |state| state.skill_id(),
    MeteorArrow(MeteorArrowState) => |state| state.skill_id(),
    SpiderPoison(SpiderPoisonState) => |state| state.skill_id(),
    SpriteBurn(SpriteBurnState) => |state| state.skill_id(),
    SpiderWeb(SpiderWebState) => |state| state.skill_id(),
    Weak(WeakState) => |state| state.skill_id(),
    GodBless(GodBlessState) => |state| state.skill_id(),
    SoulCollect(SoulCollectState) => |state| state.skill_id(),
    KnockOut(KnockOutState) => |state| state.skill_id(),
    Blind(BlindState) => |state| state.skill_id(),
    BoaLock(BoaLockState) => |state| state.skill_id(),
    Rush(RushState) => |state| state.skill_id(),
    Rush2(Rush2State) => |state| state.skill_id(),
    Roar(RoarState) => |state| state.skill_id(),
    EnergyHolding(EnergyHoldingState) => |state| state.skill_id(),
    Pillar(PillarState) => |state| state.skill_id(),
    KnightCut(KnightCutState) => |state| state.skill_id(),
    BloodLoss(BloodLossState) => |state| state.skill_id(),
    LeafCut(LeafCutState) => |state| state.skill_id(),
    LeafCut2(LeafCutState2) => |state| state.skill_id(),
    LeafCut3(LeafCutState3) => |state| state.skill_id(),
    Kerosene(KeroseneState) => |state| state.skill_id(),
    Swordship(SwordshipState) => |state| state.skill_id(),
    Strike(StrikeState) => |state| state.skill_id(),
    WuXing(WuXingState) => |state| state.skill_id(),
    AutomaticRestore(AutomaticRestoreState) => |state| state.state_id(),
    ConsumableRestore(ConsumableRestoreState) => |state| state.state_id(),
    Particular(ParticularState) => |state| state.state_id(),
    Team(CTeamState) => |state| state.state_id(),
    BattleFairyAttribute(BattleFairyAttributeState) => |state| state.skill_id(),
    TianShenXiaFan(TianShenXiaFanState) => |state| state.state_id(),
    Wangsheng(WangshengState) => |state| state.state_id(),
    DefenseShield(DefenseShieldState) => |state| state.skill_id(),
    ChangeBody(ChangeBodyState) => |_state| CHANGE_BODY_STATE_ID,
    Extended(ExtendedState) => |state| state.kind.state_id(),
    Undead(UndeadState) => |_state| UNDEAD_STATE_ID,
    Script(ScriptMoveState) => |state| state.state_id(),
    Ride(RideState) => |_state| RIDE_STATE_ID,
}

#[derive(Debug, Default)]
pub struct AppliedStateEntries {
    instances: SlotMap<StateKey, AppliedStateInstance>,
    order: Vec<Option<StateKey>>,
}

#[derive(Debug, Eq, PartialEq)]
enum StateParticipant {
    Holder { region_id: Option<i32> },
    Identity { region_id: i32, identity: ShapeIdentity },
}

impl StateParticipant {
    fn resolve(&self, holder_region: i32, holder: ShapeIdentity) -> (i32, ShapeIdentity) {
        match *self {
            Self::Holder { region_id } => (region_id.unwrap_or(holder_region), holder),
            Self::Identity { region_id, identity } => (region_id, identity),
        }
    }

    fn set_region(&mut self, region: i32) {
        match self {
            Self::Holder { region_id } => *region_id = Some(region),
            Self::Identity { region_id, .. } => *region_id = region,
        }
    }

    fn from_address((region_id, identity): (i32, ShapeIdentity)) -> Self {
        Self::Identity { region_id, identity }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct AppliedStateInstance {
    payload: Option<StateData>,
    user: Option<StateParticipant>,
    sufferer: Option<StateParticipant>,
    ended: Option<bool>,
    visual: Option<CVisualEffect>,
    serialized_span: Option<(usize, usize)>,
}

impl AppliedStateInstance {
    fn new(payload: StateData, from_save: bool) -> Self {
        let visual = (!from_save)
            .then(|| registered_runtime_state_visual(&payload))
            .flatten();
        Self {
            payload: Some(payload),
            user: (!from_save).then_some(StateParticipant::Holder { region_id: None }),
            sufferer: Some(StateParticipant::Holder { region_id: from_save.then_some(0) }),
            ended: Some(from_save), visual, serialized_span: None,
        }
    }
}

impl StateData {
    pub fn is_curable(&self) -> bool {
        is_cure_removable_state_id(self.state_id())
    }

    pub fn is_blind(&self) -> bool {
        matches!(self, Self::Blind(_) | Self::KnockOut(_) | Self::SpiderWeb(_)
            | Self::Seal(_) | Self::Strike(_) | Self::KnightCut(_))
    }

}

#[derive(Debug)]
pub struct StateBatch<T> {
    keys: Vec<StateKey>,
    values: Vec<T>,
}

impl<T> Default for StateBatch<T> {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
        }
    }
}

impl<T> StateBatch<T> {
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.values
    }
}

impl PartialEq for AppliedStateEntries {
    fn eq(&self, other: &Self) -> bool {
        self.order.len() == other.order.len()
            && self.order.iter().zip(&other.order).all(|(left, right)| {
                match (left, right) {
                    (None, None) => true,
                    (Some(left), Some(right)) => {
                        self.instances.get(*left) == other.instances.get(*right)
                    }
                    _ => false,
                }
            })
    }
}

impl Eq for AppliedStateEntries {}

pub enum StateSerialization<'a> {
    ReadOnly(&'a AppliedStateEntries),
    Save(&'a mut AppliedStateEntries),
}

impl StateSerialization<'_> {
    pub fn entries(&self) -> &AppliedStateEntries {
        match self {
            Self::ReadOnly(entries) => entries,
            Self::Save(entries) => entries,
        }
    }

    pub fn get_mut(&mut self, key: StateKey) -> Option<&mut StateData> {
        match self {
            Self::ReadOnly(_) => None,
            Self::Save(entries) => entries.get_mut(key),
        }
    }
}

impl AppliedStateEntries {
    pub fn append<T: AppliedState>(&mut self, state: T) -> StateKey {
        self.insert_data(state.into_data(), false)
    }

    pub fn append_loaded_data(&mut self, state: StateData, span: (usize, usize)) -> StateKey {
        let key = self.insert_data(state, true);
        self.set_serialized_span(key, span);
        key
    }

    pub fn serialized_span(&self, key: StateKey) -> Option<(usize, usize)> {
        self.instances.get(key)?.serialized_span
    }

    pub fn set_serialized_span(&mut self, key: StateKey, span: (usize, usize)) {
        if let Some(instance) = self.instances.get_mut(key) {
            instance.serialized_span = Some(span);
        }
    }

    pub fn shift_serialized_spans_after_remove(&mut self, offset: usize, amount: usize) {
        let Some(end) = offset.checked_add(amount) else { return };
        for instance in self.instances.values_mut() {
            let Some((start, size)) = instance.serialized_span else { continue };
            if start >= end {
                instance.serialized_span = Some((start - amount, size));
            } else if start.checked_add(size).is_none_or(|state_end| state_end > offset) {
                instance.serialized_span = None;
            }
        }
    }

    pub fn shift_serialized_spans_for_insert(&mut self, offset: usize, amount: usize) {
        for instance in self.instances.values_mut() {
            let Some((start, size)) = instance.serialized_span else { continue };
            if start >= offset {
                instance.serialized_span = start.checked_add(amount).map(|start| (start, size));
            } else if start.checked_add(size).is_none_or(|end| end > offset) {
                instance.serialized_span = None;
            }
        }
    }

    fn insert_data(&mut self, state: StateData, from_save: bool) -> StateKey {
        let key = self.instances.insert(AppliedStateInstance::new(state, from_save));
        self.order.push(Some(key));
        key
    }

    pub fn get(&self, key: StateKey) -> Option<&StateData> {
        self.instances.get(key)?.payload.as_ref()
    }

    pub fn get_mut(&mut self, key: StateKey) -> Option<&mut StateData> {
        self.instances.get_mut(key)?.payload.as_mut()
    }

    pub fn mark_ended(&mut self, key: StateKey) -> bool {
        let Some(instance) = self.instances.get_mut(key) else { return false };
        instance.ended = Some(true);
        true
    }

    pub fn mark_begun(&mut self, key: StateKey, region_id: i32) -> bool {
        let Some(instance) = self.instances.get_mut(key) else { return false };
        instance.ended = Some(false);
        instance.sufferer = Some(StateParticipant::Holder { region_id: Some(region_id) });
        true
    }

    pub fn user(&self, key: StateKey, holder_region: i32, holder: ShapeIdentity) -> Option<(i32, ShapeIdentity)> {
        Some(self.instances.get(key)?.user.as_ref()?.resolve(holder_region, holder))
    }

    pub fn sufferer(&self, key: StateKey, holder_region: i32, holder: ShapeIdentity) -> Option<(i32, ShapeIdentity)> {
        Some(self.instances.get(key)?.sufferer.as_ref()?.resolve(holder_region, holder))
    }

    pub fn set_user(&mut self, key: StateKey, user: Option<(i32, ShapeIdentity)>) -> bool {
        let Some(instance) = self.instances.get_mut(key) else { return false };
        instance.user = user.map(StateParticipant::from_address);
        true
    }

    pub fn set_sufferer(&mut self, key: StateKey, sufferer: Option<(i32, ShapeIdentity)>) -> bool {
        let Some(instance) = self.instances.get_mut(key) else { return false };
        instance.sufferer = sufferer.map(StateParticipant::from_address);
        true
    }

    pub fn set_user_region(&mut self, key: StateKey, region_id: i32) -> bool {
        let Some(instance) = self.instances.get_mut(key) else { return false };
        if let Some(user) = &mut instance.user { user.set_region(region_id); }
        true
    }

    pub fn set_sufferer_region(&mut self, key: StateKey, region_id: i32) -> bool {
        let Some(instance) = self.instances.get_mut(key) else { return false };
        if let Some(sufferer) = &mut instance.sufferer { sufferer.set_region(region_id); }
        true
    }

    pub fn has_visual(&self, key: StateKey) -> bool {
        self.instances.get(key).is_some_and(|instance| instance.visual.is_some())
    }

    pub fn visual_ended(&self, key: StateKey) -> Option<bool> {
        Some(self.instances.get(key)?.visual.as_ref()?.is_ended())
    }

    pub fn begin_visual(&mut self, key: StateKey, loop_value: i32) -> bool {
        let Some(instance) = self.instances.get_mut(key) else { return false };
        let mut visual = CVisualEffect::new();
        visual.begin_visual_effect(loop_value);
        instance.visual = Some(visual);
        true
    }

    pub fn update_visual_base(&mut self, key: StateKey) -> bool {
        let Some(visual) = self.instances.get_mut(key).and_then(|entry| entry.visual.as_mut()) else { return false };
        visual.update_visual_effect();
        true
    }

    pub fn address(&self, index: usize) -> Option<StateKey> {
        self.order.get(index).copied().flatten()
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn index_of(&self, key: StateKey) -> Option<usize> {
        self.order
            .iter()
            .position(|address| *address == Some(key))
    }

    pub fn remove_at(&mut self, index: usize) -> Option<StateData> {
        let key = self.order.get_mut(index)?.take()?;
        self.instances.remove(key)?.payload
    }

    pub fn first_key<T: AppliedState>(&self) -> Option<StateKey> {
        self.key_at::<T>(0)
    }

    pub fn key_at<T: AppliedState>(&self, index: usize) -> Option<StateKey> {
        self.order.iter().filter_map(|address| {
            let key = (*address)?;
            T::as_data_ref(self.get(key)?).map(|_| key)
        }).nth(index)
    }

    pub fn entries(&self) -> impl DoubleEndedIterator<Item = (StateKey, &StateData)> {
        self.order.iter().filter_map(|address| {
            let key = (*address)?;
            Some((key, self.get(key)?))
        })
    }

    pub fn iter_data(&self) -> impl DoubleEndedIterator<Item = &StateData> {
        self.entries().map(|(_, state)| state)
    }

    pub fn keys<T: AppliedState>(&self) -> Vec<StateKey> {
        self.order
            .iter()
            .filter_map(|address| {
                let key = (*address)?;
                T::as_data_ref(self.get(key)?).map(|_| key)
            })
            .collect()
    }

    pub fn first<T: AppliedState>(&self) -> Option<&T> {
        T::as_data_ref(self.get(self.first_key::<T>()?)?)
    }

    pub fn first_mut<T: AppliedState>(&mut self) -> Option<&mut T> {
        let key = self.first_key::<T>()?;
        T::as_data_mut(self.get_mut(key)?)
    }

    pub fn nth<T: AppliedState>(&self, position: usize) -> Option<&T> {
        self.iter::<T>().nth(position)
    }

    pub fn nth_mut<T: AppliedState>(&mut self, position: usize) -> Option<&mut T> {
        let key = self.key_at::<T>(position)?;
        T::as_data_mut(self.get_mut(key)?)
    }

    pub fn take_nth<T: AppliedState>(&mut self, position: usize) -> Option<T> {
        let key = self.key_at::<T>(position)?;
        self.take(key)
    }

    pub fn iter<T: AppliedState>(&self) -> impl Iterator<Item = &T> {
        self.order.iter().filter_map(|address| {
            let key = (*address)?;
            T::as_data_ref(self.get(key)?)
        })
    }

    pub fn for_each_mut<T: AppliedState>(&mut self, mut update: impl FnMut(&mut T)) {
        for address in &self.order {
            let Some(key) = address else {
                continue;
            };
            if let Some(state) = self.instances
                .get_mut(*key)
                .and_then(|instance| instance.payload.as_mut())
                .and_then(T::as_data_mut)
            {
                update(state);
            }
        }
    }

    pub fn take<T: AppliedState>(&mut self, key: StateKey) -> Option<T> {
        T::as_data_ref(self.get(key)?)?;
        let index = self.index_of(key)?;
        T::from_data(self.remove_at(index)?)
    }

    pub fn take_first<T: AppliedState>(&mut self) -> Option<T> {
        self.take(self.first_key::<T>()?)
    }

    pub fn replace_first<T: AppliedState>(&mut self, state: T) -> Option<T> {
        match self.first_key::<T>().and_then(|key| self.index_of(key)) {
            Some(index) => self.replace_at(index, state).and_then(T::from_data),
            None => {
                self.append(state);
                None
            }
        }
    }

    /// Позиция уже существует, в том числе после удаления старого экземпляра.
    /// Его End вызывает владелец до замены, а не этот контейнер.
    pub fn replace_at<T: AppliedState>(&mut self, index: usize, state: T) -> Option<StateData> {
        let address = &mut self.order[index];
        let key = self.instances.insert(AppliedStateInstance::new(state.into_data(), false));
        match address.replace(key) {
            Some(previous) => self.instances.remove(previous)?.payload,
            None => None,
        }
    }

    pub fn take_batch<T: AppliedState>(&mut self) -> StateBatch<T> {
        let keys = self.keys::<T>();
        let values = keys
            .iter()
            .map(|key| {
                self.instances
                    .get_mut(*key)
                    .and_then(|instance| instance.payload.take())
                    .and_then(T::from_data)
                    .expect("выбранный типизированный ключ содержит тот же payload до расчёта")
            })
            .collect();
        StateBatch { keys, values }
    }

    pub fn restore_batch<T: AppliedState>(&mut self, batch: StateBatch<T>) {
        for (key, value) in batch.keys.into_iter().zip(batch.values) {
            if let Some(instance) = self.instances.get_mut(key)
                && instance.payload.is_none()
            {
                instance.payload = Some(value.into_data());
            }
        }
    }

    /// Техническая замена DB/runtime snapshot, не игровой ClearAllStates/End.
    /// Сама SlotMap остаётся на месте, чтобы прежние ключи не ожили после загрузки.
    pub fn clear(&mut self) {
        self.instances.clear();
        self.order.clear();
    }

    pub fn compact(&mut self) -> bool {
        let previous_len = self.order.len();
        self.order.retain(Option::is_some);
        self.order.len() != previous_len
    }
}

/// Каноническое хранилище применённых состояний владельца: живая арена,
/// таймеры повторного приёма расходуемых предметов и сырой codec проекции
/// GameSave. Исходный владелец — `CMoveShape`; поля доступны hub moveshape
/// напрямую через `Deref`.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct CanonicalStateStorage {
    pub state_entries: AppliedStateEntries,
    pub consumable_restore_intervals: ConsumableRestoreIntervals,
    pub ex_states: LegacyStateCodec,
}

/// Сырой Vec-проекция Save-потока состояний с непрозрачным хвостом:
/// записи с неизвестным ID или усечённые сохраняются как есть и дописываются
/// обратно после типизированной части. Служит только Save/Load проекции.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LegacyStateCodec {
    pub payload: Vec<u8>,
    pub opaque_tail: Vec<u8>,
    pub opaque_count: u32,
    pub header_was_present: bool,
}

impl LegacyStateCodec {
    pub fn replace(&mut self, payload: Vec<u8>) {
        self.header_was_present = payload.len() >= 4;
        self.payload = payload;
        self.opaque_tail.clear();
        self.opaque_count = 0;
    }

    pub fn clear(&mut self) {
        self.replace(Vec::new());
    }

    pub fn with_opaque_tail(&self, mut payload: Vec<u8>) -> Vec<u8> {
        if !self.header_was_present && read_u32(&payload, 0).unwrap_or(0) == 0 {
            return self.opaque_tail.clone();
        }
        if self.opaque_count == 0 && self.opaque_tail.is_empty() {
            return payload;
        }
        if payload.len() < 4 {
            payload = 0u32.to_le_bytes().to_vec();
        }
        let count = read_u32(&payload, 0).unwrap_or(0);
        write_u32(&mut payload, 0, count.wrapping_add(self.opaque_count));
        payload.extend_from_slice(&self.opaque_tail);
        payload
    }
}

impl Deref for LegacyStateCodec {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

impl DerefMut for LegacyStateCodec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.payload
    }
}

fn read_u32(source: &[u8], offset: usize) -> Option<u32> {
    LegacyReader::at(source, offset).ok()?.read_u32().ok()
}

fn write_u32(destination: &mut [u8], offset: usize, value: u32) {
    LegacyWriter::write_u32_at(destination, offset, value).expect("проверенное поле состояния");
}
