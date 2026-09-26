//! Тонкий путь к поклеточному арбалетному выстрелу CPoisonMoth (0xCF) в Zone.
//! Источник: gameserver.exe/GameServer.pdb, исходный владелец
//! `appserver/skills/poisonmoth.cpp`. Собственные Check (клей самонаведения)
//! и AI (фаза направления, подготовка пути с квазнотой MAX+1, полёт по одной
//! клетке за тик, двойной visual(3)) перенесены буквально в
//! `nebokrai_zone::skills::poisonmoth` (основание и статусы MATCH — в шапке
//! Zone-файла; кластер D, порция D6). Здесь — объявленные швы переноса:
//! hub-реализации `PoisonMothGame`/`PoisonMothMoveShape` и
//! `PoisonMothContact` над прежними `CGame`/`CMoveShape` с вызовами семей
//! `rangedweaponcast` (оружейный Check, MP-контракт, failure-строки) и
//! `crossbowattack` (поклеточный удар) в прежних точках; оркестрация
//! зарегистрированного входа остаётся общим hub `playercast` (Begin-запись,
//! visual-ресурс CF, второй visual(2) при отказе Check, End по исходу).
//! Делегации сохраняют прежние сигнатуры — потребители (`playercast.rs`,
//! `crossbowattack.rs`, `crossbowcastvisual.rs`, `bossfiendpenetrate.rs`)
//! не меняются; семейные helpers ниже остаются общими швами старого пакета.

use super::crossbowattack::run_poison_moth_cell;
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{
    ArrowCastPathRule, RangedWeaponKind, check_ranged_weapon_cast,
    prepare_ranged_weapon_player, ranged_weapon_failure, terminal,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use nebokrai_zone::skills::poisonmoth::{
    self as zone, PoisonMothAiOutcome, PoisonMothContact, PoisonMothGame, PoisonMothMoveShape,
};
pub(crate) use nebokrai_zone::skills::execution::PoisonMothExecutionState;

pub(crate) const POISON_MOTH_SKILL_ID: u32 = zone::POISON_MOTH_SKILL_ID;
pub(super) const PLAYER_TYPE: i32 = 400;
pub(super) const MONSTER_TYPE: i32 = 600;

impl PoisonMothMoveShape for crate::gameserver::appserver::moveshape::CMoveShape {
    fn shape(&self) -> &crate::gameserver::appserver::shape::CShape { self.shape() }
    fn shape_mut(&mut self) -> &mut crate::gameserver::appserver::shape::CShape { self.shape_mut() }
    fn set_moveable(&mut self, moveable: bool) { self.set_moveable(moveable) }
}

impl PoisonMothGame for CGame {
    type MonsterExecution = crate::gameserver::appserver::monster::MonsterSkillExecution;
    type SkillAddress = RegisteredSkill;
    type MoveShape = crate::gameserver::appserver::moveshape::CMoveShape;

    fn registered_skill(
        &self,
        address: RegisteredSkill,
    ) -> Option<&crate::gameserver::appserver::moveshape::MoveShapeSkill> {
        self.registered_skill(address)
    }

    fn registered_skill_mut(
        &mut self,
        address: RegisteredSkill,
    ) -> Option<&mut crate::gameserver::appserver::moveshape::MoveShapeSkill> {
        self.registered_skill_mut(address)
    }

    fn update_registered_skill_visual(&mut self, address: RegisteredSkill, mode: u32) {
        self.update_registered_skill_visual(address, mode);
    }

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties> {
        self.skill_base_properties(skill_id, level)
    }

    fn resolve_state_move_shape(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&Self::MoveShape> {
        resolve_state_move_shape(self, region_id, identity)
    }

    fn resolve_state_move_shape_mut(
        &mut self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&mut Self::MoveShape> {
        resolve_state_move_shape_mut(self, region_id, identity)
    }

    fn resolve_skill_sufferer(
        &self,
        lifecycle: &nebokrai_zone::skills::SkillLifecycle,
    ) -> Option<(i32, ShapeIdentity)> {
        resolve_skill_sufferer(self, lifecycle)
    }

    fn skill_target_path(
        &self,
        lifecycle: &nebokrai_zone::skills::SkillLifecycle,
    ) -> Vec<(i32, i32, u8)> {
        self.skill_target_path(lifecycle)
    }

    fn poison_moth_region_present(&self, region_id: i32) -> bool {
        self.find_region(region_id).is_some()
    }

    fn poison_moth_skill_cell_block(&self, region_id: i32, x: i32, y: i32) -> Option<u8> {
        Some(self.find_region(region_id)?.base().skill_cell_block(x, y))
    }

    fn poison_moth_weapon_failure(
        &mut self,
        address: RegisteredSkill,
        player: Option<i32>,
        mode: u32,
    ) {
        ranged_weapon_failure(self, address, player, mode, RangedWeaponKind::Crossbow);
    }

    fn prepare_poison_moth_weapon_player(
        &mut self,
        address: RegisteredSkill,
        player: Option<i32>,
        properties: &CSkillBaseProperties,
    ) -> bool {
        prepare_ranged_weapon_player(self, address, player, properties, RangedWeaponKind::Crossbow)
    }
}

impl<Runtime: GameMainLoopRuntime> PoisonMothContact<Runtime> for CGame {
    fn check_poison_moth_ranged_weapon_cast(
        &mut self,
        address: RegisteredSkill,
        original_user: (i32, ShapeIdentity),
        runtime: &mut Runtime,
    ) -> bool {
        check_ranged_weapon_cast(
            self, address, original_user, ArrowCastPathRule::DistanceAndBlocks,
            RangedWeaponKind::Crossbow, runtime,
        )
    }

    fn run_poison_moth_cell(
        &mut self,
        address: RegisteredSkill,
        source: (i32, ShapeIdentity),
        cell: (i32, i32),
        runtime: &mut Runtime,
    ) -> bool {
        run_poison_moth_cell(self, address, source, cell, runtime)
    }
}

pub(crate) fn execute_player_poison_moth<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let original_target = if game.registered_skill(instance).is_some_and(|skill| skill.player_dispatch().is_none()) {
        dispatch.object_target().and_then(|target| {
            let player = game.find_player(player_id)?;
            game.player_skill_begin_object(player.shape().get_region_id(), target)
        })
    } else { None };
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::CrossbowCast,
        |game, instance, _, runtime| {
            let target = if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
                game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()))
            } else { original_target };
            // Точный общий GetS: исходный S объектной команды, свежий S
            // координатной; отказ Check добавляет visual(2) перед общим End(0).
            let accepted = zone::check_poison_moth_cast(game, instance, original_user, target, runtime);
            if !accepted { game.update_registered_skill_visual(instance, 2); }
            accepted
        },
        |dispatch, started| PoisonMothExecutionState::begin(dispatch, started).into(),
        |game, instance, runtime| {
            match zone::execute_poison_moth_ai(game, instance, runtime, |runtime: &mut Runtime| runtime.now_milliseconds()) {
                PoisonMothAiOutcome::Pending => terminal(QueuedSkillExecutionState::Pending),
                PoisonMothAiOutcome::Rejected => terminal(QueuedSkillExecutionState::Rejected),
                PoisonMothAiOutcome::Completed => terminal(QueuedSkillExecutionState::Completed),
            }
        },
    )
}

pub(super) fn weapon_is_crossbow(game: &CGame, player: &CPlayer) -> bool { player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 4) }
pub(super) fn target_position(game: &CGame, region_id: i32, player_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch { PlayerSkillDispatch::SelfTarget { .. } => game.find_player(player_id).and_then(CPlayer::shape_view).map(|view| (view.tile_x, view.tile_y)), PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)), PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)) }
}
pub(super) fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions(); MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()), permitted_to_kill_player: i32::from(permissions.player), permitted_to_kill_teammate: i32::from(permissions.teammate), permitted_to_kill_guild_member: i32::from(permissions.guild_member), permitted_to_kill_criminal: i32::from(permissions.criminal) }
}
pub(super) fn target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<u8> {
    match target.object_type { PLAYER_TYPE => game.find_player(target.id).map(CPlayer::level), MONSTER_TYPE => game.find_region(region_id).and_then(|owner| { let monster = owner.base().find_monster_by_id(target.id)?; game.find_monster_property_by_origin_name(monster.base_property_key()?).map(|property| property.level as u8) }), _ => None }
}
pub(super) fn cell_targets(game: &CGame, region_id: i32, x: i32, y: i32) -> Vec<ShapeIdentity> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() }; let (area_width, area_height) = game.area_dimensions(); let mut shapes = Vec::new(); if region.get_shapes(x, y, area_width, area_height, game, &mut shapes).is_err() { return Vec::new() } shapes.into_iter().map(|shape| shape.identity).collect()
}
