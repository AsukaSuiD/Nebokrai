//! Runtime-spawn owner World response `0x7F80A`, локальных script-команд и
//! world-login recreate форм.
//!
//! Источник: `gameserver.exe`/`GameServer.pdb`, `OnServerMessage` RVA
//! `0x0009D300`, monster branch `0x0009ECDD`, base `AddMonster`
//! `0x0007EC50`. Wire decode остаётся у message owner-а; этот модуль владеет
//! общей RNG-последовательностью `CGame`, concrete region mutation, city-only
//! одноаргументной guard-регистрацией и spatial entry publication. Локальные
//! `CreateNpc/CreateMonster`, persisted companions и GodsBattle faction spawn
//! используют тот же owner без теневого process context; удалённый регион
//! по-прежнему маршрутизируется самим script wire.
//! Выбор случайной клетки заимствует только CRegion и общий RNG: живые формы,
//! реестры навыков и их уникальные ресурсы для него не копируются.

use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::npc::CNpc;
use crate::gameserver::appserver::region::RegionRandomContext;
use crate::gameserver::appserver::region::{CRegion, RegionCellAccessBlock, RegionRandomPosition};
use crate::gameserver::appserver::serverregion::{
    CServerRegion, RegionMembershipBlock, ServerRegionMonsterContext,
    ServerRegionMonsterEffectsContext,
    ServerRegionMonsterSpawnEffectsContext, ServerRegionNpcContext, ServerRegionNpcSetup,
    ServerRegionNpcSpawnBlock, ServerRegionNpcSpawnEffectsContext, ServerRegionNpcSpawnOutcome,
};
use crate::gameserver::appserver::shape::CShape;
use crate::nets::netserver::message::CMessage;
use crate::public::tools::add_game_log_text;
use crate::setup::monsterlist::MonsterRegistry;

use super::game::{
    CGame, GameLegacyRandomStream, GodsBattleNpcLog, LegacyFormatArgument, ServerRegionOwner,
    format_legacy_mixed, game_tick_milliseconds, legacy_atoi_i32, record_gods_battle_log,
};

enum GameRuntimeSpawnEffect {
    Log(Vec<u8>),
    NpcEntry(CShape, CMessage),
    MonsterEntry(CShape, CMessage),
}

/// Exact `OnServerMessage 0x7F80A` adapter. RVA `0x0007EC50` вызывает
/// одноаргументный virtual guard slot только для AI 10/11: city переопределяет
/// его, а country использует отдельный camp-overload и здесь наследует no-op.
struct GameRuntimeSpawnContext<'a> {
    random: &'a mut GameLegacyRandomStream,
    monster_registry: MonsterRegistry,
    default_master_name: Vec<u8>,
    npc_position_failure_template: Vec<u8>,
    monster_variant_failure_template: Vec<u8>,
    monster_position_failure_template: Vec<u8>,
    guard_monsters: Vec<i32>,
    guard_indices: Vec<i32>,
    effects: Vec<GameRuntimeSpawnEffect>,
}

impl RegionRandomContext for GameRuntimeSpawnContext<'_> {
    fn random_below(&mut self, bound: i32) -> i32 {
        self.random.random_below(bound)
    }
}

impl ServerRegionNpcSpawnEffectsContext for GameRuntimeSpawnContext<'_> {
    fn log_npc_position_failure(&mut self, npc_name: &[u8]) {
        self.effects.push(GameRuntimeSpawnEffect::Log(format_legacy_mixed(
            &self.npc_position_failure_template,
            &[LegacyFormatArgument::Bytes(npc_name)],
            0xff,
        )));
    }
}

impl ServerRegionNpcContext for GameRuntimeSpawnContext<'_> {
    fn send_npc_entered_around(&mut self, npc: &CNpc) {
        let identity = npc.move_shape().shape().identity();
        let Some(payload) = npc.encode_client_snapshot(true) else {
            return;
        };
        let Some(message) = CGame::shape_enter_message(identity, &payload) else {
            return;
        };
        self.effects.push(GameRuntimeSpawnEffect::NpcEntry(
            npc.move_shape().shape().clone(),
            message,
        ));
    }
}

impl ServerRegionMonsterSpawnEffectsContext for GameRuntimeSpawnContext<'_> {
    fn log_monster_variant_failure(&mut self, region_id: i32, refresh_index: i32) {
        self.effects.push(GameRuntimeSpawnEffect::Log(format_legacy_mixed(
            &self.monster_variant_failure_template,
            &[
                LegacyFormatArgument::Signed(region_id),
                LegacyFormatArgument::Signed(refresh_index),
            ],
            0xff,
        )));
    }

    fn log_monster_position_failure(&mut self, origin_name: &[u8]) {
        self.effects.push(GameRuntimeSpawnEffect::Log(format_legacy_mixed(
            &self.monster_position_failure_template,
            &[LegacyFormatArgument::Bytes(origin_name)],
            0xff,
        )));
    }
}

impl ServerRegionMonsterEffectsContext for GameRuntimeSpawnContext<'_> {
    fn send_monster_entered_around(&mut self, _region: &CServerRegion, monster: &CMonster) {
        let Some(property) = monster
            .base_property_key()
            .and_then(|key| self.monster_registry.get(key))
        else {
            return;
        };
        let Some(message) =
            monster.build_fresh_enter_message(property, &self.default_master_name)
        else {
            return;
        };
        self.effects.push(GameRuntimeSpawnEffect::MonsterEntry(
            monster.move_shape().shape().clone(),
            message,
        ));
    }
}

impl ServerRegionMonsterContext for GameRuntimeSpawnContext<'_> {
    fn register_guard_monster(&mut self, monster_id: i32) {
        self.guard_monsters.push(monster_id);
    }

    fn register_guard_index(&mut self, refresh_index: i32) {
        if !self.guard_indices.contains(&refresh_index) {
            self.guard_indices.push(refresh_index);
        }
    }
}

impl CGame {
    pub(crate) fn random_region_position_owned(
        &mut self,
        region: &CRegion,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
    ) -> Result<RegionRandomPosition, RegionCellAccessBlock> {
        self.with_legacy_random_stream(|_, random| {
            region
                .get_random_pos_in_range(left, top, width, height, random)
        })
    }

    /// Общий canonical owner для `CSummonedCreature`: caller удерживает
    /// concrete region, а `CGame` предоставляет единую RNG, clock,
    /// membership и fresh-entry без process-owned callbacks.
    #[allow(clippy::too_many_arguments, reason = "literal AddSummonedCreature сохраняет spawn fields")]
    pub(crate) fn add_summoned_creature_owned(
        &mut self,
        region: &mut CServerRegion,
        property: &crate::setup::monsterlist::MonsterProperties,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
        lifetime_ms: u32,
    ) -> Result<i32, RegionMembershipBlock> {
        self.with_legacy_random_stream(|game, random| {
            let mut context = GameRuntimeSpawnContext {
                random,
                monster_registry: game.monster_registry().clone(),
                default_master_name: game.get_string_by_id(b"GS0119").to_vec(),
                npc_position_failure_template: game.get_string_by_id(b"GS0233").to_vec(),
                monster_variant_failure_template: game.get_string_by_id(b"GS0231").to_vec(),
                monster_position_failure_template: game.get_string_by_id(b"GS0232").to_vec(),
                guard_monsters: Vec::new(),
                guard_indices: Vec::new(),
                effects: Vec::new(),
            };
            let (area_width, area_height) = game.area_dimensions();
            let result = region.add_summoned_creature(
                property,
                master,
                tile_x,
                tile_y,
                direction,
                lifetime_ms,
                area_width,
                area_height,
                game.skill_factory(),
                &mut context,
                |_| game_tick_milliseconds(),
            );
            let effects = std::mem::take(&mut context.effects);
            drop(context);
            for effect in effects {
                match effect {
                    GameRuntimeSpawnEffect::Log(text) => add_game_log_text(&text),
                    GameRuntimeSpawnEffect::NpcEntry(origin, message)
                    | GameRuntimeSpawnEffect::MonsterEntry(origin, message) => {
                        if let Err(error) =
                            game.send_game_shape_around(region, &origin, None, &message)
                        {
                            tracing::warn!(
                                region_id = region.id,
                                ?error,
                                "не опубликован вход призванного существа"
                            );
                        }
                    }
                }
            }
            result
        })
    }

    /// Contribution-death item spawn: общий RNG, concrete region mutation,
    /// city guard hooks и fresh-entry принадлежат `CGame`, а death owner
    /// передаёт только уже вычисленные gameplay inputs.
    pub(crate) fn spawn_contribution_item_monster_group(
        &mut self,
        region_id: i32,
        property: &crate::setup::monsterlist::MonsterProperties,
        count: u32,
        tile_x: i32,
        tile_y: i32,
    ) {
        self.with_legacy_random_stream(|game, random| {
            let Some(mut owner) = game.take_region_owner(region_id) else {
                return;
            };
            let mut context = GameRuntimeSpawnContext {
                random,
                monster_registry: game.monster_registry().clone(),
                default_master_name: game.get_string_by_id(b"GS0119").to_vec(),
                npc_position_failure_template: game.get_string_by_id(b"GS0233").to_vec(),
                monster_variant_failure_template: game.get_string_by_id(b"GS0231").to_vec(),
                monster_position_failure_template: game.get_string_by_id(b"GS0232").to_vec(),
                guard_monsters: Vec::new(),
                guard_indices: Vec::new(),
                effects: Vec::new(),
            };
            let (area_width, area_height) = game.area_dimensions();
            for _ in 0..count {
                let position = owner
                    .base()
                    .region
                    .get_random_pos_in_range(
                        tile_x.wrapping_sub(5),
                        tile_y.wrapping_sub(5),
                        10,
                        10,
                        &mut context,
                    )
                    .ok();
                let Some(position) = position.filter(|position| position.found) else {
                    continue;
                };
                let _ = owner.base_mut().add_monster(
                    property,
                    position.x,
                    position.y,
                    0,
                    false,
                    false,
                    game_tick_milliseconds(),
                    area_width,
                    area_height,
                    game.skill_factory(),
                    &mut context,
                );
            }
            if let ServerRegionOwner::City(region) = &mut owner {
                for monster_id in context.guard_monsters.drain(..) {
                    region.add_gurd_monster(monster_id);
                }
                for refresh_index in context.guard_indices.drain(..) {
                    region.add_guard_index(refresh_index);
                }
            }
            let effects = std::mem::take(&mut context.effects);
            drop(context);
            game.restore_region_owner(owner);
            game.publish_runtime_spawn_effects(region_id, effects);
        });
    }

    /// Reached GodsBattle `ChangeNpcFaction` spawn tail. Concrete region,
    /// monster registry mutation, RNG, entry wire и GodsBattle log остаются у
    /// `CGame`; contend runtime больше не изображает region effect owner.
    pub(crate) fn spawn_gods_battle_configured_monsters(
        &mut self,
        region_id: i32,
        configuration: &crate::setup::godsbattleconf::GodsBattleFactionNpcName,
        faction: i32,
    ) -> Option<(usize, usize)> {
        self.with_legacy_random_stream(|game, random| {
            let owner = game.take_region_owner(region_id)?;
            let ServerRegionOwner::GodsBattle(mut region) = owner else {
                game.restore_region_owner(owner);
                return None;
            };
            let mut context = GameRuntimeSpawnContext {
                random,
                monster_registry: game.monster_registry().clone(),
                default_master_name: game.get_string_by_id(b"GS0119").to_vec(),
                npc_position_failure_template: game.get_string_by_id(b"GS0233").to_vec(),
                monster_variant_failure_template: game.get_string_by_id(b"GS0231").to_vec(),
                monster_position_failure_template: game.get_string_by_id(b"GS0232").to_vec(),
                guard_monsters: Vec::new(),
                guard_indices: Vec::new(),
                effects: Vec::new(),
            };
            let (area_width, area_height) = game.area_dimensions();
            let mut spawned_monsters = 0usize;
            let mut blocked_spawns = 0usize;
            for token in configuration
                .monsters
                .split(|byte| *byte == b',')
                .filter(|token| !token.is_empty())
            {
                let fields = token
                    .split(|byte| *byte == b'|')
                    .filter(|field| !field.is_empty())
                    .collect::<Vec<_>>();
                if fields.len() != 3 {
                    record_gods_battle_log(GodsBattleNpcLog::InvalidMonsterToken {
                        token: token.to_vec(),
                    });
                    blocked_spawns = blocked_spawns.wrapping_add(1);
                    tracing::warn!(
                        fields = fields.len(),
                        token_bytes = token.len(),
                        "некорректное описание монстра NPC битвы богов"
                    );
                    break;
                }
                let original_name = fields[0];
                let Some(property) = game
                    .find_monster_property_by_origin_name(original_name)
                    .cloned()
                else {
                    record_gods_battle_log(GodsBattleNpcLog::MonsterSpawnFailed {
                        npc_name: configuration.name.clone(),
                        monster: original_name.to_vec(),
                    });
                    blocked_spawns = blocked_spawns.wrapping_add(1);
                    tracing::warn!(
                        original_name_bytes = original_name.len(),
                        "свойства монстра NPC битвы богов отсутствуют"
                    );
                    continue;
                };
                let spawn = region.war.base.add_monster(
                    &property,
                    legacy_atoi_i32(fields[1]),
                    legacy_atoi_i32(fields[2]),
                    -1,
                    true,
                    false,
                    game_tick_milliseconds(),
                    area_width,
                    area_height,
                    game.skill_factory(),
                    &mut context,
                );
                if let Err(block) = spawn {
                    record_gods_battle_log(GodsBattleNpcLog::MonsterSpawnFailed {
                        npc_name: configuration.name.clone(),
                        monster: original_name.to_vec(),
                    });
                    blocked_spawns = blocked_spawns.wrapping_add(1);
                    tracing::warn!(?block, "создание монстра NPC битвы богов заблокировано");
                    continue;
                }
                if let Some(property) =
                    game.find_monster_property_by_origin_name_mut(original_name)
                {
                    property.race = faction as u32;
                }
                record_gods_battle_log(GodsBattleNpcLog::MonsterSpawned {
                    npc_name: configuration.name.clone(),
                    monster: original_name.to_vec(),
                    faction,
                });
                spawned_monsters = spawned_monsters.wrapping_add(1);
            }
            let effects = std::mem::take(&mut context.effects);
            drop(context);
            game.restore_region_owner(ServerRegionOwner::GodsBattle(region));
            game.publish_runtime_spawn_effects(region_id, effects);
            Some((spawned_monsters, blocked_spawns))
        })
    }

    /// Пространственная половина world-login pet restore. Taming limit и
    /// progression factors вычисляет gameplay owner; здесь сохраняются
    /// placement fallback, persisted pet state и fresh monster entry.
    #[allow(clippy::too_many_arguments, reason = "login pet сохраняет persisted progression fields")]
    pub(crate) fn spawn_login_pet_monster(
        &mut self,
        region_id: i32,
        property: &crate::setup::monsterlist::MonsterProperties,
        player_id: i32,
        tile_x: i32,
        tile_y: i32,
        level: u32,
        experience: u32,
        health: u32,
        factors: Option<[f32; 10]>,
        now_ms: u32,
    ) -> Option<(i32, CShape, u32)> {
        self.with_legacy_random_stream(|game, random| {
            let mut owner = game.take_region_owner(region_id)?;
            let mut context = GameRuntimeSpawnContext {
                random,
                monster_registry: game.monster_registry().clone(),
                default_master_name: game.get_string_by_id(b"GS0119").to_vec(),
                npc_position_failure_template: game.get_string_by_id(b"GS0233").to_vec(),
                monster_variant_failure_template: game.get_string_by_id(b"GS0231").to_vec(),
                monster_position_failure_template: game.get_string_by_id(b"GS0232").to_vec(),
                guard_monsters: Vec::new(),
                guard_indices: Vec::new(),
                effects: Vec::new(),
            };
            let (area_width, area_height) = game.area_dimensions();
            let position = owner
                .base()
                .region
                .get_random_pos_in_range(
                    tile_x.wrapping_sub(3),
                    tile_y.wrapping_sub(3),
                    7,
                    7,
                    &mut context,
                )
                .unwrap_or(crate::gameserver::appserver::region::RegionRandomPosition {
                    x: tile_x,
                    y: tile_y,
                    found: false,
                });
            let spawned = owner
                .base_mut()
                .add_monster(
                    property,
                    position.x,
                    position.y,
                    -1,
                    true,
                    false,
                    now_ms,
                    area_width,
                    area_height,
                    game.skill_factory(),
                    &mut context,
                )
                .ok()
                .map(|monster_id| {
                    let pet = owner
                        .base_mut()
                        .find_monster_by_id_mut(monster_id)
                        .expect("login pet AddMonster публикует concrete owner");
                    pet.set_tamed(property.tamable == 1 && property.maximum_tame_attempt_count > 0);
                    pet.set_master_info(crate::gameserver::appserver::masterinfo::MasterInfo {
                        master_type: 400,
                        master_id: player_id,
                        ..crate::gameserver::appserver::masterinfo::MasterInfo::default()
                    });
                    pet.set_pet_mode(1);
                    pet.set_pet_progress(level, experience);
                    if let Some(factors) = factors {
                        pet.adjust_pet_factors(factors);
                    }
                    pet.set_hit_points(health);
                    (
                        monster_id,
                        pet.move_shape().shape().clone(),
                        pet.maximum_hp(property),
                    )
                });
            if let ServerRegionOwner::City(region) = &mut owner {
                for monster_id in context.guard_monsters.drain(..) {
                    region.add_gurd_monster(monster_id);
                }
                for refresh_index in context.guard_indices.drain(..) {
                    region.add_guard_index(refresh_index);
                }
            }
            let effects = std::mem::take(&mut context.effects);
            drop(context);
            game.restore_region_owner(owner);
            game.publish_runtime_spawn_effects(region_id, effects);
            spawned
        })
    }

    /// Пространственная половина world-login recreate повозки. Login owner
    /// передаёт сохранённые direction/HP/script; этот owner отвечает за RNG,
    /// region membership, guard hooks и fresh monster entry.
    #[allow(clippy::too_many_arguments, reason = "login carriage сохраняет persisted shape fields")]
    pub(crate) fn spawn_login_carriage_monster(
        &mut self,
        region_id: i32,
        property: &crate::setup::monsterlist::MonsterProperties,
        player_id: i32,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
        health: u32,
        script_file: &[u8],
        now_ms: u32,
    ) -> Option<(i32, CShape, u32)> {
        self.with_legacy_random_stream(|game, random| {
            let mut owner = game.take_region_owner(region_id)?;
            let mut context = GameRuntimeSpawnContext {
                random,
                monster_registry: game.monster_registry().clone(),
                default_master_name: game.get_string_by_id(b"GS0119").to_vec(),
                npc_position_failure_template: game.get_string_by_id(b"GS0233").to_vec(),
                monster_variant_failure_template: game.get_string_by_id(b"GS0231").to_vec(),
                monster_position_failure_template: game.get_string_by_id(b"GS0232").to_vec(),
                guard_monsters: Vec::new(),
                guard_indices: Vec::new(),
                effects: Vec::new(),
            };
            let (area_width, area_height) = game.area_dimensions();
            let spawned = (|| {
                let position = owner
                    .base()
                    .region
                    .get_random_pos_in_range(tile_x, tile_y, 10, 10, &mut context)
                    .ok()?;
                let monster_id = owner
                    .base_mut()
                    .add_monster(
                        property,
                        position.x,
                        position.y,
                        -1,
                        true,
                        false,
                        now_ms,
                        area_width,
                        area_height,
                        game.skill_factory(),
                        &mut context,
                    )
                    .ok()?;
                let carriage = owner
                    .base_mut()
                    .find_monster_by_id_mut(monster_id)
                    .expect("login carriage AddMonster публикует concrete owner");
                carriage.set_master_info(crate::gameserver::appserver::masterinfo::MasterInfo {
                    master_type: 400,
                    master_id: player_id,
                    ..crate::gameserver::appserver::masterinfo::MasterInfo::default()
                });
                carriage.move_shape_mut().shape_mut().set_direction(direction);
                carriage.set_hit_points(health);
                carriage.set_script_file(script_file);
                Some((
                    monster_id,
                    carriage.move_shape().shape().clone(),
                    carriage.hit_points(),
                ))
            })();
            if let ServerRegionOwner::City(region) = &mut owner {
                for monster_id in context.guard_monsters.drain(..) {
                    region.add_gurd_monster(monster_id);
                }
                for refresh_index in context.guard_indices.drain(..) {
                    region.add_guard_index(refresh_index);
                }
            }
            let effects = std::mem::take(&mut context.effects);
            drop(context);
            game.restore_region_owner(owner);
            game.publish_runtime_spawn_effects(region_id, effects);
            spawned
        })
    }

    /// Пространственная половина script `AddCarriage`: возвращает уже
    /// опубликованного canonical monster после fresh-entry и guard hooks.
    #[allow(clippy::too_many_arguments, reason = "literal AddCarriage сохраняет spawn inputs")]
    pub(crate) fn spawn_script_carriage_monster(
        &mut self,
        region_id: i32,
        property: &crate::setup::monsterlist::MonsterProperties,
        player_id: i32,
        tile_x: i32,
        tile_y: i32,
        script_file: Option<&[u8]>,
        now_ms: u32,
    ) -> Option<(i32, CShape, u32)> {
        self.with_legacy_random_stream(|game, random| {
            let mut owner = game.take_region_owner(region_id)?;
            let mut context = GameRuntimeSpawnContext {
                random,
                monster_registry: game.monster_registry().clone(),
                default_master_name: game.get_string_by_id(b"GS0119").to_vec(),
                npc_position_failure_template: game.get_string_by_id(b"GS0233").to_vec(),
                monster_variant_failure_template: game.get_string_by_id(b"GS0231").to_vec(),
                monster_position_failure_template: game.get_string_by_id(b"GS0232").to_vec(),
                guard_monsters: Vec::new(),
                guard_indices: Vec::new(),
                effects: Vec::new(),
            };
            let (area_width, area_height) = game.area_dimensions();
            let spawned = (|| {
                let position = owner
                    .base()
                    .region
                    .get_random_pos_in_range(
                        tile_x.wrapping_sub(3),
                        tile_y.wrapping_sub(3),
                        7,
                        7,
                        &mut context,
                    )
                    .ok()?;
                let monster_id = owner
                    .base_mut()
                    .add_monster(
                        property,
                        position.x,
                        position.y,
                        -1,
                        true,
                        false,
                        now_ms,
                        area_width,
                        area_height,
                        game.skill_factory(),
                        &mut context,
                    )
                    .ok()?;
                let carriage = owner
                    .base_mut()
                    .find_monster_by_id_mut(monster_id)
                    .expect("script AddCarriage сохраняет spawned owner");
                carriage.set_master_info(crate::gameserver::appserver::masterinfo::MasterInfo {
                    master_type: 400,
                    master_id: player_id,
                    ..crate::gameserver::appserver::masterinfo::MasterInfo::default()
                });
                carriage.set_carriage_action(0);
                if let Some(script_file) = script_file.filter(|script| *script != b"0") {
                    carriage.set_script_file(script_file);
                }
                Some((
                    monster_id,
                    carriage.move_shape().shape().clone(),
                    carriage.hit_points(),
                ))
            })();
            if let ServerRegionOwner::City(region) = &mut owner {
                for monster_id in context.guard_monsters.drain(..) {
                    region.add_gurd_monster(monster_id);
                }
                for refresh_index in context.guard_indices.drain(..) {
                    region.add_guard_index(refresh_index);
                }
            }
            let effects = std::mem::take(&mut context.effects);
            drop(context);
            game.restore_region_owner(owner);
            game.publish_runtime_spawn_effects(region_id, effects);
            spawned
        })
    }

    /// Локальная ветвь script `CreateNpc`: script owner выбирает регион, а
    /// `CGame` сохраняет единую RNG-последовательность и исполняет region
    /// entry/log effects после возврата временно извлечённого owner-а.
    pub(crate) fn spawn_script_npc(
        &mut self,
        region_id: i32,
        setup: &ServerRegionNpcSetup,
        mut now_ms: impl FnMut() -> u32,
    ) -> Option<Result<ServerRegionNpcSpawnOutcome, ServerRegionNpcSpawnBlock>> {
        self.with_legacy_random_stream(|game, random| {
            let mut owner = game.take_region_owner(region_id)?;
            let mut context = GameRuntimeSpawnContext {
                random,
                monster_registry: game.monster_registry().clone(),
                default_master_name: game.get_string_by_id(b"GS0119").to_vec(),
                npc_position_failure_template: game.get_string_by_id(b"GS0233").to_vec(),
                monster_variant_failure_template: game.get_string_by_id(b"GS0231").to_vec(),
                monster_position_failure_template: game.get_string_by_id(b"GS0232").to_vec(),
                guard_monsters: Vec::new(),
                guard_indices: Vec::new(),
                effects: Vec::new(),
            };
            let spawn = game.add_region_npc_with_clock(
                &mut owner,
                setup,
                false,
                true,
                &mut context,
                |_| now_ms(),
            );
            let effects = std::mem::take(&mut context.effects);
            drop(context);
            game.restore_region_owner(owner);
            game.publish_runtime_spawn_effects(region_id, effects);
            Some(spawn)
        })
    }

    /// Локальная ветвь script `CreateMonster`: placement, monster init,
    /// city guard hooks и fresh-entry wire принадлежат одному `CGame` owner-у.
    #[allow(clippy::too_many_arguments, reason = "script wire передаёт literal spawn rectangle")]
    pub(crate) fn spawn_script_monsters(
        &mut self,
        region_id: i32,
        property: &crate::setup::monsterlist::MonsterProperties,
        count: i32,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        script_file: &[u8],
        mut now_ms: impl FnMut() -> u32,
    ) -> Option<i32> {
        self.with_legacy_random_stream(|game, random| {
            let mut owner = game.take_region_owner(region_id)?;
            let (area_width, area_height) = game.area_dimensions();
            let mut context = GameRuntimeSpawnContext {
                random,
                monster_registry: game.monster_registry().clone(),
                default_master_name: game.get_string_by_id(b"GS0119").to_vec(),
                npc_position_failure_template: game.get_string_by_id(b"GS0233").to_vec(),
                monster_variant_failure_template: game.get_string_by_id(b"GS0231").to_vec(),
                monster_position_failure_template: game.get_string_by_id(b"GS0232").to_vec(),
                guard_monsters: Vec::new(),
                guard_indices: Vec::new(),
                effects: Vec::new(),
            };
            let width = right.wrapping_sub(left);
            let height = bottom.wrapping_sub(top);
            let mut first_monster_id = 0;
            for _ in 0..count {
                let Ok(position) = owner
                    .base()
                    .region
                    .get_random_pos_in_range(left, top, width, height, &mut context)
                else {
                    continue;
                };
                let Ok(monster_id) = owner.base_mut().add_monster(
                    property,
                    position.x,
                    position.y,
                    -1,
                    true,
                    false,
                    now_ms(),
                    area_width,
                    area_height,
                    game.skill_factory(),
                    &mut context,
                ) else {
                    continue;
                };
                if first_monster_id == 0 {
                    first_monster_id = monster_id;
                }
                if !script_file.is_empty()
                    && script_file != b"0"
                    && let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id)
                {
                    monster.set_script_file(script_file);
                }
            }
            if let ServerRegionOwner::City(region) = &mut owner {
                for monster_id in context.guard_monsters.drain(..) {
                    region.add_gurd_monster(monster_id);
                }
                for refresh_index in context.guard_indices.drain(..) {
                    region.add_guard_index(refresh_index);
                }
            }
            let effects = std::mem::take(&mut context.effects);
            drop(context);
            game.restore_region_owner(owner);
            game.publish_runtime_spawn_effects(region_id, effects);
            Some(first_monster_id)
        })
    }

    /// Материализует monster-ветвь `0x7F80A`: discarded pre-roll, exact
    /// count-loop, script assignment после `AddMonster` и ordered entry effects.
    pub(crate) fn spawn_runtime_monsters(
        &mut self,
        region_id: i32,
        name: &[u8],
        requested_count: i32,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        script: &[u8],
    ) {
        self.with_legacy_random_stream(|game, random| {
            let Some(mut owner) = game.take_region_owner(region_id) else {
                tracing::warn!(region_id, "runtime-создание монстров пропущено: регион отсутствует");
                return;
            };
            let range_width = right.wrapping_sub(left);
            let range_height = bottom.wrapping_sub(top);
            let (area_width, area_height) = game.area_dimensions();
            let property = game.find_monster_property_by_origin_name(name).cloned();
            let mut context = GameRuntimeSpawnContext {
                random,
                monster_registry: game.monster_registry().clone(),
                default_master_name: game.get_string_by_id(b"GS0119").to_vec(),
                npc_position_failure_template: game.get_string_by_id(b"GS0233").to_vec(),
                monster_variant_failure_template: game.get_string_by_id(b"GS0231").to_vec(),
                monster_position_failure_template: game.get_string_by_id(b"GS0232").to_vec(),
                guard_monsters: Vec::new(),
                guard_indices: Vec::new(),
                effects: Vec::new(),
            };

            // Exact branch выполняет один discarded GetRandomPosInRange до
            // проверки count, сохраняя расход общей RNG-последовательности.
            let initial_position = owner.base().region.get_random_pos_in_range(
                left,
                top,
                range_width,
                range_height,
                &mut context,
            );
            let mut completed = 0_i32;
            if initial_position.is_ok() {
                let mut remaining = requested_count;
                while remaining > 0 {
                    let position = match owner.base().region.get_random_pos_in_range(
                        left,
                        top,
                        range_width,
                        range_height,
                        &mut context,
                    ) {
                        Ok(position) => position,
                        Err(block) => {
                            tracing::warn!(region_id, requested_count, completed, ?block, "runtime-создание монстров остановлено: позиция недоступна");
                            break;
                        }
                    };
                    match &property {
                        None => tracing::warn!(region_id, ?position, name_bytes = name.len(), "runtime-создание монстра пропущено: свойства отсутствуют"),
                        Some(property) => match owner.base_mut().add_monster(
                            property,
                            position.x,
                            position.y,
                            -1,
                            true,
                            false,
                            game_tick_milliseconds(),
                            area_width,
                            area_height,
                            game.skill_factory(),
                            &mut context,
                        ) {
                            Ok(monster_id) => {
                                owner
                                    .base_mut()
                                    .find_monster_by_id_mut(monster_id)
                                    .expect("успешный AddMonster публикует owned monster")
                                    .set_script_file(script);
                                tracing::trace!(region_id, ?position, monster_id, "runtime-монстр создан");
                            }
                            Err(block) => tracing::warn!(region_id, ?position, ?block, "runtime-монстр не создан"),
                        },
                    }
                    completed = completed.wrapping_add(1);
                    remaining = remaining.wrapping_sub(1);
                }
            }

            if let ServerRegionOwner::City(region) = &mut owner {
                for monster_id in context.guard_monsters.drain(..) {
                    region.add_gurd_monster(monster_id);
                }
                for refresh_index in context.guard_indices.drain(..) {
                    region.add_guard_index(refresh_index);
                }
            }
            let effects = std::mem::take(&mut context.effects);
            drop(context);
            game.restore_region_owner(owner);
            game.publish_runtime_spawn_effects(region_id, effects);
            tracing::trace!(region_id, requested_count, completed, ?initial_position, name_bytes = name.len(), script_bytes = script.len(), "runtime-создание монстров обработано");
        });
    }

    /// Материализует NPC-ветвь `0x7F80A` через concrete region owner.
    pub(crate) fn spawn_runtime_npc(&mut self, region_id: i32, setup: &ServerRegionNpcSetup) {
        self.with_legacy_random_stream(|game, random| {
            let Some(mut owner) = game.take_region_owner(region_id) else {
                tracing::warn!(region_id, "runtime-создание NPC пропущено: регион отсутствует");
                return;
            };
            let mut context = GameRuntimeSpawnContext {
                random,
                monster_registry: game.monster_registry().clone(),
                default_master_name: game.get_string_by_id(b"GS0119").to_vec(),
                npc_position_failure_template: game.get_string_by_id(b"GS0233").to_vec(),
                monster_variant_failure_template: game.get_string_by_id(b"GS0231").to_vec(),
                monster_position_failure_template: game.get_string_by_id(b"GS0232").to_vec(),
                guard_monsters: Vec::new(),
                guard_indices: Vec::new(),
                effects: Vec::new(),
            };
            let spawn = game.add_region_npc_with_clock(
                &mut owner,
                setup,
                true,
                true,
                &mut context,
                |_| game_tick_milliseconds(),
            );
            if let ServerRegionOwner::City(region) = &mut owner {
                for monster_id in context.guard_monsters.drain(..) {
                    region.add_gurd_monster(monster_id);
                }
                for refresh_index in context.guard_indices.drain(..) {
                    region.add_guard_index(refresh_index);
                }
            }
            let effects = std::mem::take(&mut context.effects);
            drop(context);
            game.restore_region_owner(owner);
            game.publish_runtime_spawn_effects(region_id, effects);
            tracing::trace!(region_id, ?setup, ?spawn, "runtime-создание NPC обработано");
        });
    }

    fn publish_runtime_spawn_effects(
        &mut self,
        region_id: i32,
        effects: Vec<GameRuntimeSpawnEffect>,
    ) {
        for effect in effects {
            match effect {
                GameRuntimeSpawnEffect::Log(text) => add_game_log_text(&text),
                GameRuntimeSpawnEffect::NpcEntry(origin, message)
                | GameRuntimeSpawnEffect::MonsterEntry(origin, message) => {
                    let Some(owner) = self.find_region(region_id) else {
                        return;
                    };
                    if let Err(error) =
                        self.send_game_shape_around(owner.base(), &origin, None, &message)
                    {
                        tracing::warn!(region_id, ?error, "runtime shape-entry не опубликован вокруг");
                    }
                }
            }
        }
    }
}
