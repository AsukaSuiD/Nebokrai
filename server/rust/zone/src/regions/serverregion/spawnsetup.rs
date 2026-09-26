//! Spawn setup data-контракты `CServerRegion::AddNpc/AddMonsterRect` и
//! batch-ядра их spawn-циклов: ядро инициализации создаваемого NPC,
//! batch-циклы `AddNpc` и `AddMonsterRect` с телом `AddMonster`. Исходный
//! владелец — `appserver/serverregion.h/.cpp`; сверка по точной паре
//! `gameserver.exe` + `GameServer.pdb` (идентификаторы сборки —
//! `server/rust/src/manifest/_gameserver_export_manifest.toml`). Переходный
//! агрегат `CServerRegion` и доменные `CNpc`/`CMonster` остаются в старом
//! пакете: агрегат хранит те же setup/monster колонки, factory-ветви
//! `CreateObject(500/600, id)` остаются в `appserver/baseobject.rs`, а
//! batch-ядра достигаются узкими trait-швами (`SpawnedNpcAccess`/
//! `SpawnedMonsterAccess` над доменными объектами и store-трейты над
//! переходными хранилищами) без изменения сигнатур методов переходного
//! агрегата. Ядра читают монстр-реестр и skill-фабрику как уже zone/shared
//! владельцев (`shared::resources::MonsterRegistry`,
//! `skills::skillfactory::CSkillFactory`); startup decoder, lookup-семья
//! NPC, AI refresh-фасады и context-обвязки log/send/guard не переносятся —
//! impl-трейтов старого пакета перенаправляют исходные context-вызовы в том
//! же порядке.
//!
//! Pub `AddNpc@CServerRegion` из карты стены — VA `0x00480A40`
//! (RVA `0x00080A40`, prototype `int __thiscall AddNpc(tagNpc*, bool, bool)`,
//! `serverregion.cpp:1003`) — совпадает с метаданными исследовательской
//! ведомости старого файла и с уже подтверждённой inline-записью
//! script-колонки в Zone `regions/npc.rs`; парный `AddMonsterRect` —
//! RVA `0x00084040` (`serverregion.cpp:672`, prototype
//! `int __thiscall AddMonsterRect(tagMonster*, long, bool, bool)`) из той же
//! ведомости. Ведомость ctor-инициализации
//! `CNpc` (`+0x1D4` пустая MSVC-строка script, `+0x1F0` show_list=1, колонки
//! live `+0x1F4`/born `+0x1F8` ctor не пишет) имеет статус VERIFIED в шапке
//! `crate::regions::npc` и применяется здесь через generic trait-шов без
//! нового утверждения. Перенесённые batch-тела сверены statement-в-statement
//! с телом переходного владельца на момент переноса: различия только в
//! pub-видимости, путях модулей и объявленных швах (store/SpawnedMonsterAccess/
//! context-обвязки); статусы при переносе не повышались.

use nebokrai_shared::resources::{
    MonsterProperties, MonsterRegistry, get_monster_property_by_origin_name,
};

use super::membership::RegionMembershipBlock;
use crate::regions::region::{
    RegionCellAccessBlock, RegionRandomContext, RegionRandomPosition,
};
use crate::regions::shape::{CShape, MonsterAreaClass, ShapeFigure, ShapeRuntimeFacts};
use crate::skills::skillfactory::CSkillFactory;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServerRegionNpcSetup {
    pub show_list: bool,
    pub picture_id: i32,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub count: i32,
    pub direction: i32,
    pub time: i32,
    pub name: Vec<u8>,
    pub script: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServerRegionMonsterVariant {
    pub cumulative_odds: u16,
    pub sign: u16,
    pub leader_sign: u16,
    pub leader_distance: u16,
    pub name: Vec<u8>,
    pub script: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServerRegionMonsterSetup {
    pub index: i32,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub count: i32,
    pub reset_time: i32,
    pub start_time: i32,
    pub direction: i32,
    pub living_count: i32,
    pub last_reset_time_ms: u32,
    pub variants: Vec<ServerRegionMonsterVariant>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerRegionNpcSpawnBlock {
    RandomPosition(RegionCellAccessBlock),
    Membership(RegionMembershipBlock),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ServerRegionNpcSpawnOutcome {
    pub first_created_id: Option<i32>,
    pub created: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ServerRegionMonsterSpawnOutcome {
    pub created: usize,
    pub missing_properties: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerRegionMonsterRectBlock {
    MissingRefreshSetup { index: i32 },
    RandomPosition(RegionCellAccessBlock),
    Membership(RegionMembershipBlock),
}

/// Узкий write-шов инициализации spawned NPC над доменным `CNpc` старого
/// пакета: shape-доступ переходной композиции `CMoveShape` и четыре скалярные
/// колонки NPC. Реализация живёт у переходного владельца в
/// `appserver/serverregion.rs`; имена членов сознательно отличаются от
/// inherent-methods `CNpc`, чтобы не менять ничего в `npc.rs`.
pub trait SpawnedNpcAccess {
    fn spawned_npc_shape(&mut self) -> &mut CShape;

    fn spawn_show_list(&mut self, show_list: bool);

    fn spawn_script_file(&mut self, script: &[u8]);

    fn spawn_live_time(&mut self, live_time_ms: u32);

    fn spawn_born_time(&mut self, born_time_ms: u32);
}

/// Инициализационное тело factory-объекта внутри `CServerRegion::AddNpc`
/// (`0x00080A40`) после `CreateObject(500, id)`: имя, graphics ID,
/// spawn-позиция с tile-center `+0.5`, direction guard `0..8` с fallback в
/// исходный `random(8)`, затем show_list, script-колонка (strcpy до первого
/// NUL — семантика `regions::npc`) и live/born lifetime. Clock-чтение стоит
/// на исходном месте между записью live и born-gate и питает оба потребителя:
/// born-колонку и возвращённый тик membership-входа обвязки.
pub fn initialize_created_npc<Npc: SpawnedNpcAccess + ?Sized, Context: RegionRandomContext + ?Sized>(
    npc: &mut Npc,
    setup: &ServerRegionNpcSetup,
    tile_x: i32,
    tile_y: i32,
    context: &mut Context,
    mut now_ms: impl FnMut(&mut Context) -> u32,
) -> u32 {
    let shape = npc.spawned_npc_shape();
    shape.base_object_mut().set_name(&setup.name);
    shape.base_object_mut().set_graphics_id(setup.picture_id);
    shape.set_pos_xy_move_order(tile_x as f32 + 0.5, tile_y as f32 + 0.5);
    let direction = if (0..8).contains(&setup.direction) {
        setup.direction
    } else {
        context.random_below(8)
    };
    shape.set_direction(direction);
    npc.spawn_show_list(setup.show_list);
    npc.spawn_script_file(&setup.script);
    npc.spawn_live_time(setup.time as u32);
    let spawn_tick = now_ms(context);
    if setup.time != 0 {
        npc.spawn_born_time(spawn_tick);
    }
    spawn_tick
}

/// Узкий write-шов batch-ядер spawn монстров над доменным `CMonster` старого
/// пакета: shape-доступ переходной композиции `CMoveShape`, публикация
/// исходных полей `AddMonster` до virtual `Init`, конструирование фигуры,
/// AI/скорость и refresh/script колонки. Реализация живёт у переходного
/// владельца рядом с `SpawnedNpcAccess`; имена членов сознательно отличаются
/// от inherent-methods `CMonster`, чтобы не менять ничего в `monster.rs`.
/// `initialize_spawn_skills` расходует RNG через переданную обвязку старого
/// пакета: базовая защита добавляется первой, затем исходный
/// бессодержательный `random(skill_count)`, после чего property skills
/// проходят в wire-порядке.
pub trait SpawnedMonsterAccess {
    fn spawned_monster_shape(&mut self) -> &mut CShape;

    fn bind_monster_spawn_property(&mut self, property: &MonsterProperties);

    fn initialize_spawn_skills(
        &mut self,
        property: &MonsterProperties,
        factory: &CSkillFactory,
        random: &mut impl FnMut(i32) -> i32,
    );

    fn initialize_spawn_ai(&mut self, property: &MonsterProperties, now_ms: u32);

    fn assign_spawn_speed(&mut self, property: &MonsterProperties);

    fn spawn_figure(property: &MonsterProperties) -> ShapeFigure;

    fn set_spawn_refresh_data(
        &mut self,
        sign: u16,
        leader_sign: u16,
        leader_distance: u16,
        refresh_index: i32,
    );

    fn set_spawn_script_file(&mut self, script: &[u8]);
}

/// Узкий шов ядра batch-цикла `AddNpc` (`0x00080A40`) над переходными
/// хранилищами `CServerRegion`: setup-запись, монотонные счётчики ID/total,
/// позиционный fallback `GetRandomPos`, factory type `500`, membership-вход и
/// owned-хранилище `BTreeMap<i32, CNpc>`. Реализация живёт у переходного
/// владельца и не меняет сигнатур методов агрегата; entry-effects входа
/// (`AfterEnteredArea`) выполняет та же обвязка `add_object` старого пакета.
pub trait ServerRegionNpcSpawnStore {
    type Npc: SpawnedNpcAccess;

    fn remember_npc_setup(&mut self, setup: &ServerRegionNpcSetup);

    fn take_next_npc_id(&mut self) -> i32;

    fn create_spawned_npc(&mut self, id: i32) -> Self::Npc;

    fn bump_total_spawned_npcs(&mut self);

    fn random_npc_position<Context: RegionRandomContext>(
        &self,
        left: i32,
        top: i32,
        range_width: i32,
        range_height: i32,
        context: &mut Context,
    ) -> Result<RegionRandomPosition, RegionCellAccessBlock>;

    fn enter_spawned_npc<Context: RegionRandomContext>(
        &mut self,
        npc: &mut Self::Npc,
        facts: ShapeRuntimeFacts,
        area_width: i32,
        area_height: i32,
        spawn_tick_ms: u32,
        context: &mut Context,
    ) -> Result<(), RegionMembershipBlock>;

    fn publish_owned_npc(&mut self, id: i32, npc: Self::Npc);

    fn owned_npc(&self, id: i32) -> Option<&Self::Npc>;
}

/// Batch-цикл `CServerRegion::AddNpc` (`0x00080A40`) над хранилищами
/// переходного владельца: запоминание setup, позиционный fallback с
/// owner-side log `GS0233` при неудаче (обвязка старого пакета), итерация
/// `count` созданий через factory type `500` и `initialize_created_npc`,
/// инкремент total до membership-входа, публикация в owned-хранилище,
/// завершающий entry-hook производного региона до optional круговой
/// публикации `0xBF502` и расхода RNG следующей итерации. Контекстные log и
/// send выполняет переходная обвязка в исходном порядке; `after_entry`
/// вызывается после появления NPC в каноническом хранилище и до круговой
/// публикации, как достигнутый virtual-хвост `AddObject`.
#[allow(
    clippy::too_many_arguments,
    reason = "literal AddNpc batch сохраняет исходные аргументы и объявленные context-швы переходного агрегата"
)]
pub fn add_npc_batch<Store, Context>(
    store: &mut Store,
    setup: &ServerRegionNpcSetup,
    remember_setup: bool,
    send_around: bool,
    area_width: i32,
    area_height: i32,
    context: &mut Context,
    mut now_ms: impl FnMut(&mut Context) -> u32,
    mut after_entry: impl FnMut(&mut Store, i32, &mut Context),
    mut send_entry: impl FnMut(&Store::Npc, &mut Context),
    mut on_position_failure: impl FnMut(&[u8], &mut Context),
) -> Result<ServerRegionNpcSpawnOutcome, ServerRegionNpcSpawnBlock>
where
    Store: ServerRegionNpcSpawnStore,
    Context: RegionRandomContext,
{
    if remember_setup {
        store.remember_npc_setup(setup);
    }

    let mut created = 0usize;
    let mut first_created_id = None;
    let mut remaining = setup.count;
    while remaining > 0 {
        let position = store
            .random_npc_position(
                setup.left,
                setup.top,
                setup.right.wrapping_sub(setup.left),
                setup.bottom.wrapping_sub(setup.top),
                context,
            )
            .map_err(ServerRegionNpcSpawnBlock::RandomPosition)?;
        if !position.found {
            on_position_failure(&setup.name, context);
            remaining = remaining.wrapping_sub(1);
            continue;
        }

        let id = store.take_next_npc_id();
        let mut npc = store.create_spawned_npc(id);
        let spawn_tick = initialize_created_npc(
            &mut npc, setup, position.x, position.y, context, &mut now_ms,
        );

        store.bump_total_spawned_npcs();
        let facts = ShapeRuntimeFacts {
            is_npc: true,
            is_move_shape: true,
            blocks_region_cell: true,
            figure: ShapeFigure::default(),
            ..ShapeRuntimeFacts::default()
        };
        store
            .enter_spawned_npc(&mut npc, facts, area_width, area_height, spawn_tick, context)
            .map_err(ServerRegionNpcSpawnBlock::Membership)?;

        store.publish_owned_npc(id, npc);
        after_entry(store, id, context);
        created = created.wrapping_add(1);
        first_created_id.get_or_insert(id);
        if send_around {
            send_entry(
                store
                    .owned_npc(id)
                    .expect("NPC опубликован непосредственно перед send"),
                context,
            );
        }
        remaining = remaining.wrapping_sub(1);
    }
    Ok(ServerRegionNpcSpawnOutcome {
        first_created_id,
        created,
    })
}

/// Узкий шов batch-ядер `AddMonsterRect`/`AddMonster` (`0x00084040/0x0007EC50`)
/// над переходными хранилищами `CServerRegion`: setup-записи и их living
/// счётчики, монотонные ID/total, позиционный fallback, factory type `600`,
/// membership-вход и owned `MonsterWorld`. Read обращений к setup-записям
/// повторяет последовательность исходного тела по живому storage; реализация
/// живёт у переходного владельца и не меняет сигнатур методов агрегата.
pub trait ServerRegionMonsterSpawnStore {
    type Monster: SpawnedMonsterAccess;

    fn remember_monster_setup(&mut self, setup: &ServerRegionMonsterSetup);

    fn monster_setup_index(&self, index: i32) -> Option<usize>;

    fn monster_setup(&self, setup_index: usize) -> &ServerRegionMonsterSetup;

    fn bump_monster_living_count(&mut self, setup_index: usize);

    fn take_next_monster_id(&mut self) -> i32;

    fn create_spawned_monster(&mut self, id: i32) -> Self::Monster;

    fn bump_total_spawned_monsters(&mut self);

    fn random_monster_position<Context: RegionRandomContext>(
        &self,
        left: i32,
        top: i32,
        range_width: i32,
        range_height: i32,
        context: &mut Context,
    ) -> Result<RegionRandomPosition, RegionCellAccessBlock>;

    #[allow(
        clippy::too_many_arguments,
        reason = "membership-вход сохраняет исходные аргументы AddObject переходного агрегата"
    )]
    fn enter_spawned_monster<Context: RegionRandomContext>(
        &mut self,
        monster: &mut Self::Monster,
        facts: ShapeRuntimeFacts,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<(), RegionMembershipBlock>;

    fn publish_owned_monster(&mut self, id: i32, monster: Self::Monster);

    fn owned_monster(&self, id: i32) -> Option<&Self::Monster>;

    fn owned_monster_mut(&mut self, id: i32) -> Option<&mut Self::Monster>;
}

/// Low-level тело `CServerRegion::AddMonster` (`0x0047EC50`) над хранилищами
/// переходного владельца: factory type `600`, публикация исходных полей до
/// virtual `Init`, AI/speed/direction границы (`0..8` с fallback в исходный
/// `random(8)`), guard-регистрация до speed/direction/AddObject для AI 10/11
/// (context-обвязка старого пакета), membership-вход с RTTI-фактом carriage,
/// затем owned-публикация, безусловный around-send (пятый bool exact EXE не
/// читается) и инкремент total после send.
#[allow(
    clippy::too_many_arguments,
    reason = "literal AddMonster сохраняет исходные spawn flags и owner-границы"
)]
pub fn add_monster<Store, Context>(
    store: &mut Store,
    property: &MonsterProperties,
    tile_x: i32,
    tile_y: i32,
    direction: i32,
    _unused_legacy_flag: bool,
    suppress_guard_registration: bool,
    now_ms: u32,
    area_width: i32,
    area_height: i32,
    skill_factory: &CSkillFactory,
    context: &mut Context,
    mut on_guard_monster: impl FnMut(i32, &mut Context),
    mut send_entry: impl FnMut(&Store, &Store::Monster, &mut Context),
) -> Result<i32, RegionMembershipBlock>
where
    Store: ServerRegionMonsterSpawnStore,
    Context: RegionRandomContext,
{
    let id = store.take_next_monster_id();
    let mut monster = store.create_spawned_monster(id);
    monster.bind_monster_spawn_property(property);
    monster.initialize_spawn_skills(property, skill_factory, &mut |bound| {
        context.random_below(bound)
    });
    monster.initialize_spawn_ai(property, now_ms);
    monster
        .spawned_monster_shape()
        .set_pos_xy_move_order(tile_x as f32 + 0.5, tile_y as f32 + 0.5);
    if matches!(property.ai, 10 | 11) && !suppress_guard_registration {
        on_guard_monster(id, context);
    }
    monster.assign_spawn_speed(property);
    let direction = if (0..8).contains(&direction) {
        direction
    } else {
        context.random_below(8)
    };
    monster.spawned_monster_shape().set_direction(direction);

    let facts = ShapeRuntimeFacts {
        monster: Some(
            if property.tamable == 1 && property.maximum_tame_attempt_count == 0 {
                MonsterAreaClass::Carriage
            } else {
                MonsterAreaClass::Active
            },
        ),
        is_move_shape: true,
        blocks_region_cell: true,
        figure: Store::Monster::spawn_figure(property),
        ..ShapeRuntimeFacts::default()
    };
    store.enter_spawned_monster(
        &mut monster, facts, area_width, area_height, now_ms, context,
    )?;
    store.publish_owned_monster(id, monster);
    // Compatibility quirk exact EXE 0x0047EC50: пятый bool не читается,
    // а enter message отправляется безусловно.
    let entered = store
        .owned_monster(id)
        .expect("monster опубликован непосредственно перед send");
    send_entry(&*store, entered, context);
    store.bump_total_spawned_monsters();
    Ok(id)
}

/// Batch-цикл `CServerRegion::AddMonsterRect` (`0x00084040`) над хранилищами
/// переходного владельца: запоминание setup, поиск refresh-записи, cumulative
/// RNG-выбор variant, позиционный fallback (owner-side log обвязки без
/// прерывания итерации), property lookup по original name реестра
/// `CMonsterList`, low-level `add_monster`, затем живой инкремент
/// `living_count`, refresh metadata/script колонки и guard-index регистрация
/// обвязки. Trace-обвязка остаётся у переходного пакета; ядро возвращает её
/// счётчики typed-outcome.
#[allow(
    clippy::too_many_arguments,
    reason = "literal AddMonsterRect сохраняет исходные аргументы и context-швы переходного агрегата"
)]
pub fn add_monster_rect_batch<Store, Context>(
    store: &mut Store,
    region_id: i32,
    setup: &ServerRegionMonsterSetup,
    amount: i32,
    remember_setup: bool,
    suppress_guard_registration: bool,
    now_ms: u32,
    area_width: i32,
    area_height: i32,
    monster_registry: &MonsterRegistry,
    skill_factory: &CSkillFactory,
    context: &mut Context,
    mut on_variant_failure: impl FnMut(i32, i32, &mut Context),
    mut on_position_failure: impl FnMut(&[u8], &mut Context),
    mut on_guard_monster: impl FnMut(i32, &mut Context),
    mut on_guard_index: impl FnMut(i32, &mut Context),
    mut send_entry: impl FnMut(&Store, &Store::Monster, &mut Context),
) -> Result<ServerRegionMonsterSpawnOutcome, ServerRegionMonsterRectBlock>
where
    Store: ServerRegionMonsterSpawnStore,
    Context: RegionRandomContext,
{
    if remember_setup {
        store.remember_monster_setup(setup);
    }
    let Some(setup_index) = store.monster_setup_index(setup.index) else {
        return Err(ServerRegionMonsterRectBlock::MissingRefreshSetup { index: setup.index });
    };

    let mut created = 0usize;
    let mut missing_properties = 0usize;
    let mut remaining = amount;
    while remaining > 0 {
        let random_odds = context.random_below(100);
        let selected = store
            .monster_setup(setup_index)
            .variants
            .iter()
            .find(|variant| random_odds < i32::from(variant.cumulative_odds))
            .cloned();
        let Some(selected) = selected else {
            let refresh_index = store.monster_setup(setup_index).index;
            on_variant_failure(region_id, refresh_index, context);
            remaining = remaining.wrapping_sub(1);
            continue;
        };

        let refresh = store.monster_setup(setup_index);
        let position = store
            .random_monster_position(
                refresh.left,
                refresh.top,
                refresh.right.wrapping_sub(refresh.left),
                refresh.bottom.wrapping_sub(refresh.top),
                context,
            )
            .map_err(ServerRegionMonsterRectBlock::RandomPosition)?;
        if !position.found {
            on_position_failure(&selected.name, context);
        }

        let Some(property) =
            get_monster_property_by_origin_name(monster_registry, &selected.name).cloned()
        else {
            missing_properties = missing_properties.wrapping_add(1);
            remaining = remaining.wrapping_sub(1);
            continue;
        };
        let direction = store.monster_setup(setup_index).direction;
        let id = add_monster(
            store,
            &property,
            position.x,
            position.y,
            direction,
            remember_setup,
            suppress_guard_registration,
            now_ms,
            area_width,
            area_height,
            skill_factory,
            context,
            &mut on_guard_monster,
            &mut send_entry,
        )
        .map_err(ServerRegionMonsterRectBlock::Membership)?;

        let refresh_index = store.monster_setup(setup_index).index;
        store.bump_monster_living_count(setup_index);
        let monster = store
            .owned_monster_mut(id)
            .expect("успешный AddMonster публикует owned monster");
        monster.set_spawn_refresh_data(
            selected.sign,
            selected.leader_sign,
            selected.leader_distance,
            refresh_index,
        );
        if !selected.script.is_empty() && selected.script != b"0" {
            monster.set_spawn_script_file(&selected.script);
        }
        created = created.wrapping_add(1);
        if matches!(property.ai, 10 | 11) && !suppress_guard_registration {
            on_guard_index(refresh_index, context);
        }
        remaining = remaining.wrapping_sub(1);
    }
    Ok(ServerRegionMonsterSpawnOutcome {
        created,
        missing_properties,
    })
}
