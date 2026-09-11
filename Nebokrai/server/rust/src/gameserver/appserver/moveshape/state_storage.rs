//! Хранилище экземпляров `CMoveShape::m_vStates` из GameServer.exe/GameServer.pdb.
//! Исходный владелец — CMoveShape; сохранённые RAW RemoveState (0x004CDAB0,
//! 0x004CDB20) и AddExStatesToByteArray (0x004D10F0) находятся в родительском
//! moveshape.rs, общий UpdateAbnormality (0x004CFD00) — в states/state.rs.
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
//! Здесь нет игрового End-dispatch: оставшееся подключение точных обработчиков
//! не подменяется пустым End или общим сбросом payload.

use super::*;

new_key_type! {
    pub(crate) struct StateKey;
}

pub(crate) trait AppliedState: Sized + 'static {
    fn into_data(self) -> StateData;
    fn as_data_ref(data: &StateData) -> Option<&Self>;
    fn as_data_mut(data: &mut StateData) -> Option<&mut Self>;
    fn from_data(data: StateData) -> Option<Self>;
}

macro_rules! applied_states {
    ($($variant:ident($payload:ty) => |$state:ident| $id:expr),+ $(,)?) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub(crate) enum StateData {
            $($variant($payload)),+
        }

        impl StateData {
            pub(crate) fn state_id(&self) -> u32 {
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
    Particular(ParticularState) => |_state| PARTICULAR_STATE_ID,
    Team(CTeamState) => |_state| TEAM_STATE_ID,
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

#[derive(Clone, Debug, Default)]
pub(crate) struct AppliedStateEntries {
    instances: SlotMap<StateKey, Option<StateData>>,
    order: Vec<Option<StateKey>>,
}

impl StateData {
    pub(crate) fn is_curable(&self) -> bool {
        matches!(self, Self::Seal(_) | Self::PoisonFog(_) | Self::SpiderPoison(_)
            | Self::SpriteBurn(_) | Self::SpiderWeb(_) | Self::KnockOut(_)
            | Self::BoaLock(_) | Self::Rush(_) | Self::Rush2(_)
            | Self::BossBlueQuake(_) | Self::KnightCut(_))
    }

    pub(crate) fn is_blind(&self) -> bool {
        matches!(self, Self::Blind(_) | Self::KnockOut(_) | Self::SpiderWeb(_)
            | Self::Seal(_) | Self::Strike(_))
    }

}

#[derive(Debug)]
pub(crate) struct StateBatch<T> {
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
    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
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

impl AppliedStateEntries {
    pub(crate) fn append<T: AppliedState>(&mut self, state: T) -> StateKey {
        self.append_data(state.into_data())
    }

    pub(crate) fn append_data(&mut self, state: StateData) -> StateKey {
        let key = self.instances.insert(Some(state));
        self.order.push(Some(key));
        key
    }

    pub(crate) fn get(&self, key: StateKey) -> Option<&StateData> {
        self.instances.get(key)?.as_ref()
    }

    pub(crate) fn get_mut(&mut self, key: StateKey) -> Option<&mut StateData> {
        self.instances.get_mut(key)?.as_mut()
    }

    pub(crate) fn address(&self, index: usize) -> Option<StateKey> {
        self.order.get(index).copied().flatten()
    }

    pub(crate) fn len(&self) -> usize {
        self.order.len()
    }

    pub(crate) fn index_of(&self, key: StateKey) -> Option<usize> {
        self.order
            .iter()
            .position(|address| *address == Some(key))
    }

    pub(crate) fn remove_at(&mut self, index: usize) -> Option<StateData> {
        let key = self.order.get_mut(index)?.take()?;
        self.instances.remove(key).flatten()
    }

    pub(crate) fn first_key<T: AppliedState>(&self) -> Option<StateKey> {
        self.key_at::<T>(0)
    }

    pub(crate) fn key_at<T: AppliedState>(&self, index: usize) -> Option<StateKey> {
        self.order.iter().filter_map(|address| {
            let key = (*address)?;
            T::as_data_ref(self.get(key)?).map(|_| key)
        }).nth(index)
    }

    pub(crate) fn entries(&self) -> impl DoubleEndedIterator<Item = (StateKey, &StateData)> {
        self.order.iter().filter_map(|address| {
            let key = (*address)?;
            Some((key, self.get(key)?))
        })
    }

    pub(crate) fn iter_data(&self) -> impl DoubleEndedIterator<Item = &StateData> {
        self.entries().map(|(_, state)| state)
    }

    pub(crate) fn keys<T: AppliedState>(&self) -> Vec<StateKey> {
        self.order
            .iter()
            .filter_map(|address| {
                let key = (*address)?;
                T::as_data_ref(self.get(key)?).map(|_| key)
            })
            .collect()
    }

    pub(crate) fn first<T: AppliedState>(&self) -> Option<&T> {
        T::as_data_ref(self.get(self.first_key::<T>()?)?)
    }

    pub(crate) fn first_mut<T: AppliedState>(&mut self) -> Option<&mut T> {
        let key = self.first_key::<T>()?;
        T::as_data_mut(self.get_mut(key)?)
    }

    pub(crate) fn nth<T: AppliedState>(&self, position: usize) -> Option<&T> {
        self.iter::<T>().nth(position)
    }

    pub(crate) fn nth_mut<T: AppliedState>(&mut self, position: usize) -> Option<&mut T> {
        let key = self.key_at::<T>(position)?;
        T::as_data_mut(self.get_mut(key)?)
    }

    pub(crate) fn take_nth<T: AppliedState>(&mut self, position: usize) -> Option<T> {
        let key = self.key_at::<T>(position)?;
        self.take(key)
    }

    pub(crate) fn iter<T: AppliedState>(&self) -> impl Iterator<Item = &T> {
        self.order.iter().filter_map(|address| {
            let key = (*address)?;
            T::as_data_ref(self.get(key)?)
        })
    }

    pub(crate) fn for_each_mut<T: AppliedState>(&mut self, mut update: impl FnMut(&mut T)) {
        for address in &self.order {
            let Some(key) = address else {
                continue;
            };
            if let Some(state) = self.instances
                .get_mut(*key)
                .and_then(Option::as_mut)
                .and_then(T::as_data_mut)
            {
                update(state);
            }
        }
    }

    pub(crate) fn take<T: AppliedState>(&mut self, key: StateKey) -> Option<T> {
        T::as_data_ref(self.get(key)?)?;
        let index = self.index_of(key)?;
        T::from_data(self.remove_at(index)?)
    }

    pub(crate) fn take_first<T: AppliedState>(&mut self) -> Option<T> {
        self.take(self.first_key::<T>()?)
    }

    pub(crate) fn replace_first<T: AppliedState>(&mut self, state: T) -> Option<T> {
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
    pub(crate) fn replace_at<T: AppliedState>(&mut self, index: usize, state: T) -> Option<StateData> {
        let address = &mut self.order[index];
        let key = self.instances.insert(Some(state.into_data()));
        match address.replace(key) {
            Some(previous) => self.instances.remove(previous).flatten(),
            None => None,
        }
    }

    pub(crate) fn take_batch<T: AppliedState>(&mut self) -> StateBatch<T> {
        let keys = self.keys::<T>();
        let values = keys
            .iter()
            .map(|key| {
                self.instances
                    .get_mut(*key)
                    .and_then(Option::take)
                    .and_then(T::from_data)
                    .expect("выбранный типизированный ключ содержит тот же payload до расчёта")
            })
            .collect();
        StateBatch { keys, values }
    }

    pub(crate) fn restore_batch<T: AppliedState>(&mut self, batch: StateBatch<T>) {
        for (key, value) in batch.keys.into_iter().zip(batch.values) {
            if let Some(payload) = self.instances.get_mut(key)
                && payload.is_none()
            {
                *payload = Some(value.into_data());
            }
        }
    }

    /// Техническая замена DB/runtime snapshot, не игровой ClearAllStates/End.
    /// Сама SlotMap остаётся на месте, чтобы прежние ключи не ожили после загрузки.
    pub(crate) fn clear(&mut self) {
        self.instances.clear();
        self.order.clear();
    }

    pub(crate) fn compact(&mut self) -> bool {
        let previous_len = self.order.len();
        self.order.retain(Option::is_some);
        self.order.len() != previous_len
    }
}
