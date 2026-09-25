//! Spawn setup data-контракты `CServerRegion::AddNpc/AddMonsterRect`
//! исторического GameServer (порция spawn setup) и ядро инициализации
//! создаваемого NPC. Исходный владелец — `appserver/serverregion.h/.cpp`.
//! Переходный агрегат `CServerRegion` и доменный `CNpc` остаются в старом
//! пакете: агрегат хранит те же setup/monster колонки, factory-ветвь
//! `CreateObject(500, id)` остаётся в `appserver/baseobject.rs`, а ядро
//! инициализации достигается узким trait-швом `SpawnedNpcAccess` без
//! изменения сигнатур методов. Batch-цикл `AddNpc` (позиционный fallback,
//! membership-вход, owned-хранилище, around-публикация), startup decoder и
//! monster spawn этой порцией не переносятся.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2). Pub `AddNpc@CServerRegion` из карты стены — VA `0x00480A40`
//! (RVA `0x00080A40`, prototype `int __thiscall AddNpc(tagNpc*, bool, bool)`,
//! `serverregion.cpp:1003`) — совпадает с метаданными исследовательской
//! ведомости старого файла и с уже подтверждённой inline-записью
//! script-колонки в Zone `regions/npc.rs`. Ведомость ctor-инициализации
//! `CNpc` (`+0x1D4` пустая MSVC-строка script, `+0x1F0` show_list=1, колонки
//! live `+0x1F4`/born `+0x1F8` ctor не пишет) имеет статус VERIFIED в шапке
//! `crate::regions::npc` и применяется здесь через generic trait-шов без
//! нового утверждения. Перенесённые тела — дословный перенос уже достигнутой
//! реконструкции старого файла; новой машинной сверки эта порция не
//! добавляет, статусы не повышает.

use super::membership::RegionMembershipBlock;
use crate::regions::region::{RegionCellAccessBlock, RegionRandomContext};
use crate::regions::shape::CShape;

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
