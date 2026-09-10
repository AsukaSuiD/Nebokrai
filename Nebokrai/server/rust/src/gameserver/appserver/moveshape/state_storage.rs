//! Хранилище экземпляров `CMoveShape::m_vStates` из GameServer.exe/GameServer.pdb.
//! Исходный владелец — CMoveShape; сохранённые RAW RemoveState (0x004CDAB0,
//! 0x004CDB20), UpdateAbnormality (0x004CFD00) и AddExStatesToByteArray
//! (0x004D10F0) находятся в родительском moveshape.rs.
//! Порядок добавления и пустые позиции после удаления принадлежат этому
//! контейнеру; уплотнение выполняется только явно. Новый экземпляр получает
//! новый поколенческий ключ даже при замене в прежней позиции. Общий End
//! вызывается снаружи и может изменить тот же список до перечитывания позиции.
//! Ссылка на CStateSkill обозначает уже зарегистрированный навык: удаление
//! такой ссылки не уничтожает его и не вызывает CSkill::End вместо CState::End.
//! SlotMap заменяет владение сырыми указателями, Vec сохраняет нативный порядок.
//! Для пяти участков чистого расчёта CFightDefense существует временный slice-
//! адаптер StateBatch: ключи и позиции остаются живыми при вынутом payload.
//! Между take_batch и restore_batch не допускаются игровые callbacks; возврат
//! заполняет только ещё живые пустые слоты, не воскрешая удалённый экземпляр.
//! Единственный enum-каталог объединяет существующие typed payload, не вводя
//! второго каталога игровых ID. Codec и таймеры повторного приёма предметов
//! остаются отдельными данными владельца, а не дополнительными состояниями.
//! Здесь нет игрового End-dispatch: перенос оставшихся владельцев и их точных
//! обработчиков не подменяется пустым End или общим сбросом payload.

use super::*;

new_key_type! {
    pub(crate) struct StateKey;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StateAddress {
    Applied(StateKey),
    Skill(SkillSlot),
}

pub(crate) trait AppliedState: Sized + 'static {
    fn into_data(self) -> StateData;
    fn as_data_ref(data: &StateData) -> Option<&Self>;
    fn as_data_mut(data: &mut StateData) -> Option<&mut Self>;
    fn from_data(data: StateData) -> Option<Self>;
}

macro_rules! applied_states {
    ($($variant:ident($payload:ty)),+ $(,)?) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub(crate) enum StateData {
            $($variant($payload)),+
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
    PersistentAgility(PersistentAgilityFamilyState),
    Agility2(AgilityState2),
    Callosity(CallosityFamilyState),
    TaiJi(TaiJiState),
    EnlargeFullMiss(EnlargeFullMissState),
    EnlargeMaxHp(EnlargeMaxHpState),
    EnlargeMaxMp(EnlargeMaxMpState),
    Origin(OriginState),
    Hearten(HeartenState),
    Heal(HealState),
    Fury(FuryState),
    RageBreak(RageBreakState),
    BossBlueFury(BossBlueFuryState),
    BossBlueQuake(BossBlueQuakeState),
    Cure(CureState),
    DaubPoison(DaubPoisonState),
    Seal(SealState),
    PoisonArrow(PoisonArrowState),
    PoisonFog(PoisonFogState),
    MeteorArrow(MeteorArrowState),
    SpiderPoison(SpiderPoisonState),
    SpriteBurn(SpriteBurnState),
    SpiderWeb(SpiderWebState),
    Weak(WeakState),
    GodBless(GodBlessState),
    SoulCollect(SoulCollectState),
    KnockOut(KnockOutState),
    Blind(BlindState),
    BoaLock(BoaLockState),
    Rush(RushState),
    Rush2(Rush2State),
    Roar(RoarState),
    EnergyHolding(EnergyHoldingState),
    Pillar(PillarState),
    KnightCut(KnightCutState),
    BloodLoss(BloodLossState),
    LeafCut(LeafCutState),
    LeafCut2(LeafCutState2),
    LeafCut3(LeafCutState3),
    Kerosene(KeroseneState),
    Swordship(SwordshipState),
    Strike(StrikeState),
    WuXing(WuXingState),
    AutomaticRestore(AutomaticRestoreState),
    ConsumableRestore(ConsumableRestoreState),
    Particular(ParticularState),
    Team(CTeamState),
    BattleFairyAttribute(BattleFairyAttributeState),
    TianShenXiaFan(TianShenXiaFanState),
    Wangsheng(WangshengState),
    DefenseShield(DefenseShieldState),
    ChangeBody(ChangeBodyState),
    Extended(ExtendedState),
    Undead(UndeadState),
    Script(ScriptMoveState),
    Ride(RideState),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct AppliedStateEntries {
    instances: SlotMap<StateKey, Option<StateData>>,
    order: Vec<Option<StateAddress>>,
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
                    (Some(StateAddress::Applied(left)), Some(StateAddress::Applied(right))) => {
                        self.instances.get(*left) == other.instances.get(*right)
                    }
                    (Some(StateAddress::Skill(left)), Some(StateAddress::Skill(right))) => {
                        left == right
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
        self.order.push(Some(StateAddress::Applied(key)));
        key
    }

    pub(crate) fn append_skill(&mut self, slot: SkillSlot) {
        self.order.push(Some(StateAddress::Skill(slot)));
    }

    pub(crate) fn get(&self, key: StateKey) -> Option<&StateData> {
        self.instances.get(key)?.as_ref()
    }

    pub(crate) fn get_mut(&mut self, key: StateKey) -> Option<&mut StateData> {
        self.instances.get_mut(key)?.as_mut()
    }

    pub(crate) fn address(&self, index: usize) -> Option<StateAddress> {
        self.order.get(index).copied().flatten()
    }

    pub(crate) fn len(&self) -> usize {
        self.order.len()
    }

    pub(crate) fn index_of(&self, key: StateKey) -> Option<usize> {
        self.order
            .iter()
            .position(|address| *address == Some(StateAddress::Applied(key)))
    }

    pub(crate) fn remove_at(&mut self, index: usize) -> Option<StateData> {
        match self.order.get_mut(index)?.take()? {
            StateAddress::Applied(key) => self.instances.remove(key).flatten(),
            StateAddress::Skill(_) => None,
        }
    }

    pub(crate) fn first_key<T: AppliedState>(&self) -> Option<StateKey> {
        self.order.iter().find_map(|address| {
            let StateAddress::Applied(key) = (*address)? else {
                return None;
            };
            T::as_data_ref(self.get(key)?).map(|_| key)
        })
    }

    pub(crate) fn keys<T: AppliedState>(&self) -> Vec<StateKey> {
        self.order
            .iter()
            .filter_map(|address| {
                let StateAddress::Applied(key) = (*address)? else {
                    return None;
                };
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

    pub(crate) fn iter<T: AppliedState>(&self) -> impl Iterator<Item = &T> {
        self.order.iter().filter_map(|address| {
            let StateAddress::Applied(key) = (*address)? else {
                return None;
            };
            T::as_data_ref(self.get(key)?)
        })
    }

    pub(crate) fn for_each_mut<T: AppliedState>(&mut self, mut update: impl FnMut(&mut T)) {
        for address in &self.order {
            let Some(StateAddress::Applied(key)) = address else {
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
        match address.replace(StateAddress::Applied(key)) {
            Some(StateAddress::Applied(previous)) => self.instances.remove(previous).flatten(),
            Some(StateAddress::Skill(_)) | None => None,
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
