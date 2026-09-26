//! Реестр навыков живой фигуры `CMoveShape`. Исходный владелец —
//! `appserver/moveshape.h/.cpp`; сверка по точной паре `gameserver.exe` +
//! `GameServer.pdb`. Переходный агрегат `CMoveShape` старого пакета хранит
//! этот реестр внутри себя и делегирует сюда его поведение без изменения
//! сигнатур своих методов.
//!
//! Разрез записи навыка: скалярная база `CSkill` (ID (+0x04), level, concrete
//! owner, item position, reuse timestamp +0x40 и owned visual) живёт здесь
//! типом `SkillIdentity`, а execution kernel (Player/BattleFairy/Monster) и
//! retained данные полёта — типом `skills/execution::RegisteredSkillRecord`
//! (там же payload исполнения монстра и alias `MoveShapeSkill`).
//! Реестр связан с записью только швом `SkillIdentityAccess`
//! (generic-trait сварка по прецеденту `StateRecordTarget`).
//! Конструирование полной записи при AddSkill/CFightDefense также задаёт
//! владелец записи своим handler: категория и concrete owner выбираются здесь
//! по единственному каталогу `CSkillFactory`.
//!
//! Четыре независимые категории сохраняют экземпляры, их порядок вставки и
//! повторные ID: native AddSkill (RVA `0x000D1C70`) допускает повторный ID,
//! когда уровень первого найденного экземпляра равен нулю; повышение
//! ненулевого уровня не понижается, а удаляет первое совпадение и добавляет
//! экземпляр в хвост. В каждой категории SlotMap владеет записями, а Vec
//! ключей задаёт только native-порядок: поколенческий `SkillSlot` переживает
//! сдвиги этого Vec и не разрешает вложенному callback завершить новую
//! одноимённую регистрацию; порядок самого SlotMap не используется. Удаление
//! и очистка инвалидируют ключи, а не пересоздают хранилище с прежними
//! поколениями. Это техническая замена указателей экземпляров, не
//! дополнительный каталог ID либо owners.
//!
//! GetSkill/DelSkill (RVA `0x000CF320`) выбирают категорию отдельно через
//! актуальный `QuerySkillType(ID, 1)`; DelSkill отвергает UNKNOWN до current
//! cleanup, ID 0 проходит cleanup и только потом category lookup, и удаляет
//! только первый найденный экземпляр. ClearSkills (RVA `0x000CDCE0`) сохраняет
//! неразрешённый current ID и очищает категории в порядке Attack → Defense →
//! Summon → State. GetCurrentSkill (RVA `0x000CDC10`) разрешает выбранный ID
//! через реестр: выбранный ID сам по себе не доказывает наличие навыка.
//! GetDefaultAttackSkillID (RVA `0x000CE240`) выбирает ID 2 только по
//! attack-вектору, иначе ID 3 по summon-вектору, иначе ID 1 — сверено по
//! машинному коду (скан поля ID `CSkill +0x04` по векторам `+0x130`/`+0x150`,
//! без зависимости от QuerySkillType и без раннего выхода).
//!
//! Публичные символы семейства: `?AddSkill@CMoveShape@@QAEHW4tagSkillID@@J@Z`
//! (`0x000D1C70`), `?AddSkill@CMoveShape@@QAEHPBDJ@Z` (`0x000D3C70`,
//! name-вариант не перенесён), `?DelSkill@CMoveShape@@QAEHW4tagSkillID@@@Z`
//! (`0x000CF320`), `?DelSkill@CMoveShape@@QAEHPBD@Z` (`0x000CF560`,
//! name-вариант не перенесён), `?ClearSkills@CMoveShape@@QAEXXZ`
//! (`0x000CDCE0`), `?GetCurrentSkill@CMoveShape@@QAEPAVCSkill@@XZ`
//! (`0x000CDC10`), `?SetCurrentSkill@CMoveShape@@UAEXW4tagSkillID@@@Z`
//! (`0x000CEEE0`), `?SetItemSkill@CMoveShape@@QAEXW4tagSkillID@@@Z`
//! (`0x000D1570`), `?GetDefaultAttackSkillID@CMoveShape@@UAE?AW4tagSkillID@@XZ`
//! (`0x000CE240`).

use slotmap::{SlotMap, new_key_type};

use crate::combat::UNKNOWN_SKILL_ID;
use crate::skills::skillfactory::{CSkillFactory, SkillCategory, SkillOwner};
use crate::skills::{SkillLifecycle, SkillTermination, SkillVisualEffect};

/// ID intrinsic защиты: `CSkillFactory::QuerySkill(SKILL_BASE_DEFENSE, 1)`
/// создавал `CFightDefense` отдельной ветвью даже без reloadable properties.
pub const SKILL_BASE_DEFENSE: u32 = 10;

/// Скалярная база зарегистрированного навыка (база `CSkill`): ID, level,
/// concrete owner, item position, reuse timestamp +0x40 и owned visual.
/// Execution kernel и retained данные полёта живут в той же записи Zone
/// `skills/execution::RegisteredSkillRecord`.
#[derive(Debug, Eq, PartialEq)]
pub struct SkillIdentity {
    id: u32,
    level: i32,
    owner: SkillOwner,
    item_position: i32,
    last_used_ms: u32,
    current_visual_effect: Option<SkillVisualEffect>,
}

impl SkillIdentity {
    /// Свежая регистрация: `CSkill` constructor (0x004D8120) задаёт timestamp
    /// +0x40 равным нулю; новая регистрация не наследует его от удалённого
    /// экземпляра того же ID.
    pub const fn new_registered(id: u32, level: i32, owner: SkillOwner) -> Self {
        Self {
            id,
            level,
            owner,
            item_position: -1,
            last_used_ms: 0,
            current_visual_effect: None,
        }
    }

    pub const fn id(&self) -> u32 {
        self.id
    }

    pub const fn level(&self) -> i32 {
        self.level
    }

    pub const fn owner(&self) -> SkillOwner {
        self.owner
    }

    pub const fn item_position(&self) -> i32 {
        self.item_position
    }

    pub const fn set_item_position(&mut self, position: i32) {
        self.item_position = position;
    }

    /// Общий reuse timestamp +0x40 единственный для этого экземпляра.
    pub const fn last_used_ms(&self) -> u32 {
        self.last_used_ms
    }

    pub fn mark_used(&mut self, now_ms: u32) {
        self.last_used_ms = now_ms;
    }

    /// Визуальный ресурс принадлежит самому экземпляру отдельно от копируемой
    /// скалярной базы и от execution payload: Rage/KnightCut создают эффект до
    /// cast-проверок, поэтому он существует и при failed Begin без payload.
    pub fn replace_visual_effect(&mut self, effect: SkillVisualEffect) {
        self.current_visual_effect = Some(effect);
    }

    pub fn visual_effect_mut(&mut self) -> Option<&mut SkillVisualEffect> {
        self.current_visual_effect.as_mut()
    }

    pub fn visual_effect(&self) -> Option<&SkillVisualEffect> {
        self.current_visual_effect.as_ref()
    }

    /// Visual и IsEnded следуют после отдельного native reuse-clock: сначала
    /// очищаются source/target/time базы (caller записи), затем удаляется
    /// `Option<SkillVisualEffect>` и только потом выставляется ended.
    pub fn finish_cleared_base_end(
        &mut self,
        lifecycle: &mut SkillLifecycle,
        termination: SkillTermination,
    ) {
        let visual = &mut self.current_visual_effect;
        lifecycle.finish_end(termination, || drop(visual.take()));
    }

    pub const fn skill_type(&self) -> u32 {
        self.owner.category() as u32
    }

    /// GetMinDistance общий для клиента и ИИ: конкретный usage владельца и
    /// signed-положительная граница 1.
    pub fn minimum_range(&self, factory: &CSkillFactory) -> u32 {
        self.owner
            .minimum_range_usage()
            .and_then(|usage| {
                factory
                    .query_skill_base_properties(self.id, self.level)
                    .map(|properties| properties.query_property(usage))
            })
            .filter(|value| (*value as i32) > 0)
            .unwrap_or(1)
    }

    /// `CSkill::GetSkillName` (0x004D86E0) читает актуальные свойства по
    /// ID/уровню, а не имя времени регистрации. None в name означает отсутствие
    /// записи; локализованный GS0318 и пустой fallback разрешаются владельцем
    /// публикации.
    pub fn name<'a>(&self, factory: &'a CSkillFactory) -> Option<&'a [u8]> {
        factory
            .query_skill_base_properties(self.id, self.level)
            .map(|properties| properties.skill_name())
    }
}

/// Скалярная связка реестра с полной записью навыка: коллекция владеет
/// записью `S`, но правила реестра смотрят только в `SkillIdentity`;
/// execution/retained payload подключает владелец `S` со своей стороны.
pub trait SkillIdentityAccess {
    fn identity(&self) -> &SkillIdentity;
    fn identity_mut(&mut self) -> &mut SkillIdentity;
}

new_key_type! {
    /// Поколенческий ключ записи в одной категории: переживает сдвиги order и
    /// не разрешает вложенному callback завершить новую одноимённую регистрацию.
    pub struct SkillEntity;
}

/// Адрес конкретного экземпляра в одной категории данного владельца.
/// После удаления ключ не разрешается в новую запись с тем же ID или индексом.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillSlot {
    pub category: SkillCategory,
    pub entity: SkillEntity,
}

/// Одна категория реестра: SlotMap владеет записями, Vec ключей задаёт только
/// native-порядок (порядок самого SlotMap не используется).
#[derive(Debug)]
pub struct SkillCollection<S> {
    instances: SlotMap<SkillEntity, S>,
    order: Vec<SkillEntity>,
}

impl<S> Default for SkillCollection<S> {
    fn default() -> Self {
        Self {
            instances: SlotMap::default(),
            order: Vec::new(),
        }
    }
}

impl<S: PartialEq> PartialEq for SkillCollection<S> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<S: Eq> Eq for SkillCollection<S> {}

impl<S> Drop for SkillCollection<S> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<S> SkillCollection<S> {
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &S> {
        self.order.iter().map(|entity| {
            self.instances
                .get(*entity)
                .expect("порядок категории содержит только живые экземпляры навыков")
        })
    }

    pub fn entities(&self) -> &[SkillEntity] {
        &self.order
    }

    pub fn push(&mut self, skill: S) {
        self.order.push(self.instances.insert(skill));
    }

    pub fn remove(&mut self, index: usize) -> S {
        let entity = self.order.remove(index);
        self.instances
            .remove(entity)
            .expect("удаляемый индекс категории принадлежит живому экземпляру навыка")
    }

    pub fn clear(&mut self) {
        for entity in self.order.drain(..) {
            drop(self.instances.remove(entity));
        }
    }

    pub fn get(&self, entity: SkillEntity) -> Option<&S> {
        self.instances.get(entity)
    }

    pub fn get_mut(&mut self, entity: SkillEntity) -> Option<&mut S> {
        self.instances.get_mut(entity)
    }
}

/// Реестр навыков одной фигуры: четыре независимые категории, выбранный
/// current ID и ordered-список item-skill. Скалярные правила реестра живут
/// здесь; конструирование записи и execution lifecycle остаются владельцу `S`.
#[derive(Debug)]
pub struct SkillRegistry<S: SkillIdentityAccess> {
    categories: [SkillCollection<S>; 4],
    current_skill_id: Option<u32>,
    item_skill_ids: Vec<u32>,
}

impl<S: SkillIdentityAccess> Default for SkillRegistry<S> {
    fn default() -> Self {
        Self {
            categories: std::array::from_fn(|_| SkillCollection::default()),
            current_skill_id: None,
            item_skill_ids: Vec::new(),
        }
    }
}

impl<S: SkillIdentityAccess + PartialEq> PartialEq for SkillRegistry<S> {
    fn eq(&self, other: &Self) -> bool {
        self.categories == other.categories
            && self.current_skill_id == other.current_skill_id
            && self.item_skill_ids == other.item_skill_ids
    }
}

impl<S: SkillIdentityAccess + Eq> Eq for SkillRegistry<S> {}

impl<S: SkillIdentityAccess> SkillRegistry<S> {
    /// Все навыки фигуры по категориям, каждая в своём native-порядке вставки.
    pub fn skills(&self) -> impl Iterator<Item = &S> {
        self.categories.iter().flat_map(SkillCollection::iter)
    }

    pub fn skills_in_category(
        &self,
        category: SkillCategory,
    ) -> impl ExactSizeIterator<Item = &S> {
        self.categories[category as usize].iter()
    }

    pub fn skill_count_in_category(&self, category: SkillCategory) -> usize {
        self.categories[category as usize].entities().len()
    }

    pub fn skill(&self, skill_id: u32, factory: &CSkillFactory) -> Option<&S> {
        self.skill_at(self.skill_slot(skill_id, factory)?)
    }

    pub fn skill_mut(&mut self, skill_id: u32, factory: &CSkillFactory) -> Option<&mut S> {
        self.skill_at_mut(self.skill_slot(skill_id, factory)?)
    }

    pub fn skill_slot(&self, skill_id: u32, factory: &CSkillFactory) -> Option<SkillSlot> {
        let category = SkillCategory::from_raw(factory.query_skill_type(skill_id, 1))?;
        let index = self.categories[category as usize]
            .iter()
            .position(|skill| skill.identity().id == skill_id)?;
        self.skill_slot_at(category, index)
    }

    /// Прямой native-обход берёт текущий индекс категории, без QuerySkillType.
    /// Возвращённый ключ сохраняет идентичность через последующие callbacks.
    pub fn skill_slot_at(&self, category: SkillCategory, index: usize) -> Option<SkillSlot> {
        let entity = *self.categories[category as usize].entities().get(index)?;
        Some(SkillSlot { category, entity })
    }

    pub fn skill_at(&self, slot: SkillSlot) -> Option<&S> {
        self.categories[slot.category as usize].get(slot.entity)
    }

    pub fn skill_at_mut(&mut self, slot: SkillSlot) -> Option<&mut S> {
        self.categories[slot.category as usize].get_mut(slot.entity)
    }

    /// Общий reuse timestamp +0x40: отсутствующий навык читается нулём.
    pub fn skill_last_used_ms(&self, skill_id: u32, factory: &CSkillFactory) -> u32 {
        self.skill(skill_id, factory)
            .map_or(0, |skill| skill.identity().last_used_ms())
    }

    pub fn mark_skill_used(&mut self, skill_id: u32, now_ms: u32, factory: &CSkillFactory) {
        if let Some(skill) = self.skill_mut(skill_id, factory) {
            skill.identity_mut().mark_used(now_ms);
        }
    }

    pub fn skill_visual_effect_mut(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> Option<&mut SkillVisualEffect> {
        self.skill_mut(skill_id, factory)?
            .identity_mut()
            .current_visual_effect
            .as_mut()
    }

    pub fn replace_skill_visual_effect(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
        effect: SkillVisualEffect,
    ) -> bool {
        let Some(skill) = self.skill_mut(skill_id, factory) else {
            return false;
        };
        skill.identity_mut().replace_visual_effect(effect);
        true
    }

    /// Полный сброс реестра при очистке persisted runtime-состояния владельца:
    /// записи всех категорий в порядке массива, выбранный ID и ordered
    /// item-список.
    pub fn clear(&mut self) {
        for category in &mut self.categories {
            category.clear();
        }
        self.current_skill_id = None;
        self.item_skill_ids.clear();
    }

    /// Удаление реестра сохраняет неразрешённый current ID. Полный concrete
    /// End перед удалением вызывает владелец записи отдельно: lifecycle
    /// execution живёт в записи (Zone `skills/execution`) и этой границей
    /// не подменяется.
    pub fn clear_skills(&mut self, factory: &CSkillFactory) {
        if self.current_skill(factory).is_some() {
            self.current_skill_id = None;
        }
        for category in [
            SkillCategory::Attack,
            SkillCategory::Defense,
            SkillCategory::Summon,
            SkillCategory::State,
        ] {
            self.categories[category as usize].clear();
        }
    }

    /// Reloadable properties не могут отменить intrinsic defense или изменить
    /// его категорию; имя читается только при обращении к экземпляру.
    pub fn add_base_defense_skill(&mut self, make: impl FnOnce(SkillOwner) -> S) {
        let owner = CSkillFactory::factory_owner(SKILL_BASE_DEFENSE)
            .expect("CFightDefense входит в native factory");
        self.categories[SkillCategory::Defense as usize].push(make(owner));
    }

    pub fn set_item_skill_position(
        &mut self,
        skill_id: u32,
        position: i32,
        factory: &CSkillFactory,
    ) -> bool {
        let Some(skill) = self.skill_mut(skill_id, factory) else {
            return false;
        };
        skill.identity_mut().set_item_position(position);
        true
    }

    /// Выбранный ID независимо от наличия зарегистрированного навыка.
    pub const fn current_skill_id(&self) -> Option<u32> {
        self.current_skill_id
    }

    /// Проекция GetCurrentSkill в реестр; execution и End остаются у владельца
    /// записи.
    pub fn current_skill(&self, factory: &CSkillFactory) -> Option<&S> {
        self.current_skill_id
            .and_then(|skill_id| self.skill(skill_id, factory))
    }

    /// GetDefaultAttackSkillID (0x004CE240): порядок категорий важнее порядка
    /// ID; прямой скан intrinsic-категорий не зависит от QuerySkillType.
    /// Отсутствующий default ID самопроизвольно не сбрасывается.
    pub fn default_attack_skill_id(&self) -> u32 {
        if self
            .skills_in_category(SkillCategory::Attack)
            .any(|skill| skill.identity().id == 2)
        {
            2
        } else if self
            .skills_in_category(SkillCategory::Summon)
            .any(|skill| skill.identity().id == 3)
        {
            3
        } else {
            1
        }
    }

    /// Typed boundary для snapshot/skill caller-а. Полное semantic действие
    /// `SetCurrentSkill` (завершение прежнего concrete skill) не подменяется
    /// записью ID и остаётся у соответствующего owner-а.
    pub const fn set_current_skill_id(&mut self, skill_id: Option<u32>) {
        self.current_skill_id = skill_id;
    }

    /// Exact `SetItemSkill`: native owner только добавляет ID в ordered vector
    /// непосредственно перед передачей item-skill в его AI.
    pub fn set_item_skill(&mut self, skill_id: u32) {
        self.item_skill_ids.push(skill_id);
    }

    /// AddSkill (0x004D1C70): ненулевой уровень не понижается, повышение
    /// удаляет первое совпадение и добавляет новый экземпляр в хвост.
    /// Нулевой уровень прежнего экземпляра допускает повторный ID.
    pub fn add_skill(
        &mut self,
        skill_id: u32,
        level: i32,
        factory: &CSkillFactory,
        make: impl FnOnce(SkillOwner) -> S,
    ) -> bool {
        if let Some(existing) = self.skill(skill_id, factory) {
            let existing_level = existing.identity().level;
            if existing_level != 0 {
                if level <= existing_level {
                    return true;
                }
                self.delete_skill(skill_id, factory);
            }
        }
        self.insert_new_skill(skill_id, make)
    }

    /// Категорию вставки задаёт concrete constructor записанного владельца
    /// (единственный каталог `CSkillFactory`), а не запись runtime-свойств.
    pub fn insert_new_skill(&mut self, skill_id: u32, make: impl FnOnce(SkillOwner) -> S) -> bool {
        let Some(owner) = CSkillFactory::factory_owner(skill_id) else {
            return false;
        };
        self.categories[owner.category() as usize].push(make(owner));
        true
    }

    /// DelSkill (0x004CF320) удаляет только первый найденный экземпляр.
    /// До category lookup обрабатывается разрешённый current, даже если
    /// удаляется другой ID. UNKNOWN отвергается до этого, а ID 0 — после.
    pub fn delete_skill(&mut self, skill_id: u32, factory: &CSkillFactory) -> bool {
        if skill_id == UNKNOWN_SKILL_ID {
            return false;
        }
        if self.current_skill(factory).is_some() {
            self.current_skill_id = None;
        }
        let Some(category) = SkillCategory::from_raw(factory.query_skill_type(skill_id, 1)) else {
            return false;
        };
        self.delete_skill_in_category(skill_id, category);
        true
    }

    pub fn delete_skill_in_category(&mut self, skill_id: u32, category: SkillCategory) {
        let index = self.categories[category as usize]
            .iter()
            .position(|skill| skill.identity().id == skill_id);
        if let Some(index) = index {
            self.categories[category as usize].remove(index);
        }
    }
}
