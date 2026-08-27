//! Каноническое generational-хранилище монстров одного `CServerRegion`.
//!
//! `slotmap` владеет `CMonster` и инвалидирует handle только при настоящем
//! удалении сущности. `BTreeMap` остаётся тонким индексом legacy ID и задаёт
//! подтверждённый порядок обхода; порядок slot-map никогда не используется в
//! ИИ, выборе цели, рассылке или сериализации. Прирученный pet остаётся тем же
//! monster entity, поэтому смена lifecycle-роли не меняет identity. Временное
//! изъятие для вызова, одновременно меняющего region membership, сохраняет
//! slot и generation, а не изображает удаление и повторное создание сущности.

use std::collections::BTreeMap;

use slotmap::{SlotMap, new_key_type};

use super::monster::CMonster;

new_key_type! {
    struct MonsterEntity;
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MonsterWorld {
    entities: SlotMap<MonsterEntity, Option<CMonster>>,
    legacy_ids: BTreeMap<i32, MonsterEntity>,
}

impl PartialEq for MonsterWorld {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl Eq for MonsterWorld {}

pub(crate) struct TakenMonster {
    entity: MonsterEntity,
    legacy_id: i32,
    monster: CMonster,
}

impl TakenMonster {
    pub(crate) fn monster_mut(&mut self) -> &mut CMonster {
        &mut self.monster
    }
}

impl MonsterWorld {
    pub(crate) fn insert(&mut self, legacy_id: i32, monster: CMonster) -> Option<CMonster> {
        if let Some(entity) = self.legacy_ids.get(&legacy_id).copied() {
            return self.entities.get_mut(entity)?.replace(monster);
        }
        let entity = self.entities.insert(Some(monster));
        self.legacy_ids.insert(legacy_id, entity);
        None
    }

    pub(crate) fn get(&self, legacy_id: &i32) -> Option<&CMonster> {
        let entity = *self.legacy_ids.get(legacy_id)?;
        self.entities.get(entity)?.as_ref()
    }

    pub(crate) fn get_mut(&mut self, legacy_id: &i32) -> Option<&mut CMonster> {
        let entity = *self.legacy_ids.get(legacy_id)?;
        self.entities.get_mut(entity)?.as_mut()
    }

    pub(crate) fn contains_key(&self, legacy_id: &i32) -> bool {
        self.get(legacy_id).is_some()
    }

    pub(crate) fn take(&mut self, legacy_id: i32) -> Option<TakenMonster> {
        let entity = *self.legacy_ids.get(&legacy_id)?;
        let monster = self.entities.get_mut(entity)?.take()?;
        Some(TakenMonster {
            entity,
            legacy_id,
            monster,
        })
    }

    pub(crate) fn restore(&mut self, taken: TakenMonster) {
        let slot = self
            .entities
            .get_mut(taken.entity)
            .expect("временное изъятие сохраняет generational slot");
        debug_assert!(slot.is_none());
        debug_assert_eq!(self.legacy_ids.get(&taken.legacy_id), Some(&taken.entity));
        *slot = Some(taken.monster);
    }

    pub(crate) fn discard(&mut self, taken: TakenMonster) {
        debug_assert_eq!(self.legacy_ids.get(&taken.legacy_id), Some(&taken.entity));
        self.legacy_ids.remove(&taken.legacy_id);
        let removed = self.entities.remove(taken.entity);
        debug_assert!(matches!(removed, Some(None)));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&i32, &CMonster)> {
        self.legacy_ids.iter().filter_map(|(legacy_id, entity)| {
            self.entities
                .get(*entity)
                .and_then(Option::as_ref)
                .map(|monster| (legacy_id, monster))
        })
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &CMonster> {
        self.iter().map(|(_, monster)| monster)
    }

    pub(crate) fn for_each_mut(&mut self, mut visit: impl FnMut(i32, &mut CMonster)) {
        let identities: Vec<_> = self
            .legacy_ids
            .iter()
            .map(|(&id, &entity)| (id, entity))
            .collect();
        for (legacy_id, entity) in identities {
            if let Some(monster) = self.entities.get_mut(entity).and_then(Option::as_mut) {
                visit(legacy_id, monster);
            }
        }
    }
}
