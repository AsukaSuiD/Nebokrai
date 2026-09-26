//! Делегации и hub-реализации швов Zone для областных призывов Weak,
//! PoisonFog, SnowStorm, YinYang/YinYang2, GodThunder/GodThunder2, FireWall,
//! ChaosSphere и SoulMirror.
//! Источник: gameserver.exe + GameServer.pdb (точная пара `4F5C98E0…`,
//! RSDS match), appserver/skills/weak.cpp, poisonfog.cpp, snowstorm.cpp,
//! yinyang.cpp/yinyang2.cpp, godthunder.cpp/godthunder2.cpp, firewall.cpp,
//! chaossphere.cpp и soulmirror.cpp. Тела скелета Begin/Check/AI/visual/End,
//! общий префикс Summon и state-операции перенесены буквально в
//! `nebokrai_zone::skills::{zonalcast,weak,poisonfog,snowstorm,yinyang,
//! godthunder,chaossphere,elementphalanx,weakstate,poisonfogstate}` порцией
//! T5 «zonalcast-хаб» (основание, конвенция RVA и машинные статусы — в
//! шапке `zone/skills/zonalcast.rs`). Здесь — фасадные реализации трейтов
//! `ZonalCast*` над прежними методами `CGame`/`CPlayer`/`CMoveShape` и
//! делегации с прежними сигнатурами; зарегистрированный вход
//! (playercast/monster stateskill), полный End, перечень владельцев visual
//! и потребители (playercast, monsterbaseattack, states/skill.rs) не меняются.
//!
//! Тела `rangedweaponcast`, мастер `weaponattack`, обвязка арены
//! `states/state.rs`, регистрация областей в регионе и рассылка BF502
//! остаются в этом пакете: объявленные швы делегируют им вызовы в прежних
//! точках. Применение огненной стены перенаправлено прежнему телу
//! `appserver/skills/firewall.rs`, обход зеркала душ — делегату
//! `appserver/skills/soulmirror.rs`.

use super::kernel::SkillExecutionKernel;
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{
    CastManaRule, CastPathBlock, RangedWeaponKind, check_cast_mana,
    check_cast_mana_without_text, check_ranged_weapon_and_mana, check_skill_path,
    prepare_ranged_weapon_player, spend_cast_mana, spend_cast_mana_without_text,
};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
    state_skill_outcome,
};
use super::weaponattack::{SourceProperty, source_master, source_property};
use crate::gameserver::appserver::monster::MonsterSkillExecution;
use crate::gameserver::appserver::moveshape::{CMoveShape, MoveShapeSkill};
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::region::RegionSecurity;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    remove_applied_state_from, resolve_applied_state_sufferer, resolve_skill_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut,
    update_applied_state_end_visual, update_property_state_visual,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner,
};
use crate::nets::netserver::message::{CMessage, GameMessageDomainOps};
use nebokrai_zone::combat::{AttackInformation, MasterInfo, PlayerCombatProperties};
use nebokrai_zone::effects::WeakState;
use nebokrai_zone::skills::execution::RegisteredSkillRecord;
use nebokrai_zone::skills::state::{AppliedState, StateKey};
use nebokrai_zone::skills::zonalcast::{
    ZonalCastAiOutcome, ZonalCastContact, ZonalCastGame, ZonalCastMoveShape,
    ZonalCastPathBlock, ZonalCastPlayer, ZonalCastPropertyTarget,
};
use nebokrai_zone::skills::ElementSummonLiveField;

pub(crate) use nebokrai_zone::skills::is_zonal_cast_skill;

impl ZonalCastPlayer for CPlayer {
    fn mana(&self) -> u32 { self.mana() }

    fn set_mana(&mut self, mana: u32) { self.set_mana(mana) }

    fn combat_properties(&self) -> PlayerCombatProperties { self.combat_properties() }

    fn update_state_combat_properties(
        &mut self,
        update: impl FnOnce(PlayerCombatProperties) -> PlayerCombatProperties,
    ) {
        self.update_state_combat_properties(update);
    }

    fn level(&self) -> u8 { self.level() }
}

impl ZonalCastMoveShape for CMoveShape {
    fn shape(&self) -> &CShape { self.shape() }

    fn shape_mut(&mut self) -> &mut CShape { self.shape_mut() }

    fn set_moveable(&mut self, moveable: bool) { self.set_moveable(moveable) }

    fn applied_state<T: AppliedState>(&self, key: StateKey) -> Option<&T> {
        self.applied_state(key)
    }

    fn append_applied_state_record<T: AppliedState>(&mut self, state: T, record: &[u8]) -> StateKey {
        self.append_applied_state_record(state, record)
    }

    fn mark_applied_state_begun(&mut self, key: StateKey) -> bool {
        self.mark_applied_state_begun(key)
    }

    fn set_applied_state_user(
        &mut self,
        key: StateKey,
        user: Option<(i32, ShapeIdentity)>,
    ) -> bool {
        self.set_applied_state_user(key, user)
    }

    fn set_applied_state_sufferer(
        &mut self,
        key: StateKey,
        sufferer: Option<(i32, ShapeIdentity)>,
    ) -> bool {
        self.set_applied_state_sufferer(key, sufferer)
    }

    fn set_applied_state_sufferer_region(&mut self, key: StateKey, region_id: i32) -> bool {
        self.set_applied_state_sufferer_region(key, region_id)
    }
}

impl ZonalCastGame for CGame {
    type MonsterExecution = MonsterSkillExecution;
    type SkillAddress = RegisteredSkill;
    type Player = CPlayer;
    type MoveShape = CMoveShape;

    fn registered_skill(
        &self,
        address: RegisteredSkill,
    ) -> Option<&RegisteredSkillRecord<MonsterSkillExecution>> {
        self.registered_skill(address)
    }

    fn registered_skill_mut(
        &mut self,
        address: RegisteredSkill,
    ) -> Option<&mut RegisteredSkillRecord<MonsterSkillExecution>> {
        self.registered_skill_mut(address)
    }

    fn update_registered_skill_visual(&mut self, address: RegisteredSkill, mode: u32) {
        self.update_registered_skill_visual(address, mode);
    }

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties> {
        self.skill_base_properties(skill_id, level)
    }

    fn resolve_skill_sufferer(
        &self,
        lifecycle: &nebokrai_zone::skills::SkillLifecycle,
    ) -> Option<(i32, ShapeIdentity)> {
        resolve_skill_sufferer(self, lifecycle)
    }

    fn resolve_state_move_shape(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&CMoveShape> {
        resolve_state_move_shape(self, region_id, identity)
    }

    fn resolve_state_move_shape_mut(
        &mut self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&mut CMoveShape> {
        resolve_state_move_shape_mut(self, region_id, identity)
    }

    fn find_player(&self, player_id: i32) -> Option<&CPlayer> { self.find_player(player_id) }

    fn find_player_mut(&mut self, player_id: i32) -> Option<&mut CPlayer> {
        self.find_player_mut(player_id)
    }

    fn skill_target_path(
        &self,
        lifecycle: &nebokrai_zone::skills::SkillLifecycle,
    ) -> Vec<(i32, i32, u8)> {
        self.skill_target_path(lifecycle)
    }

    fn skill_target_path_with_length(
        &self,
        lifecycle: &nebokrai_zone::skills::SkillLifecycle,
        length: u32,
    ) -> Vec<(i32, i32, u8)> {
        self.skill_target_path_with_length(lifecycle, length)
    }

    fn move_shape_health(&self, region_id: i32, target: ShapeIdentity) -> Option<u32> {
        self.move_shape_health(region_id, target)
    }

    fn move_shape_level(&self, region_id: i32, target: ShapeIdentity) -> Option<u8> {
        self.move_shape_level(region_id, target)
    }

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]) {
        self.send_skill_system_info(player_id, text);
    }

    fn zonal_source_master(&self, source: (i32, ShapeIdentity)) -> Option<MasterInfo> {
        source_master(self, source)
    }

    fn zonal_source_property(
        &self,
        source: (i32, ShapeIdentity),
        property: ElementSummonLiveField,
    ) -> Option<u32> {
        let property = match property {
            ElementSummonLiveField::CriticalChance => SourceProperty::CriticalChance,
            ElementSummonLiveField::AddElementAttack => SourceProperty::Element,
        };
        source_property(self, source, property)
    }

    fn player_weapon_damage_level(&self, player: &CPlayer) -> u32 {
        player.weapon_damage_level(self.goods_factory()) as u32
    }

    fn zonal_area_safe(&self, region_id: i32, x: i32, y: i32) -> Option<bool> {
        self.find_region(region_id)
            .map(|region| region.get_security(x, y).ok() == Some(RegionSecurity::SAFE))
    }

    fn allocate_summon_shape_id(&mut self) -> i32 { self.allocate_summon_shape_id() }

    fn skill_random_below(&mut self, maximum: i32) -> i32 { self.skill_random_below(maximum) }

    fn check_zonal_skill_path(
        &mut self,
        address: RegisteredSkill,
        properties: &CSkillBaseProperties,
        path: &[(i32, i32, u8)],
        player: Option<i32>,
        block: ZonalCastPathBlock,
    ) -> bool {
        let block = match block {
            ZonalCastPathBlock::Ignore => CastPathBlock::Ignore,
            ZonalCastPathBlock::Generic => CastPathBlock::Generic,
            ZonalCastPathBlock::GroundAndFly => CastPathBlock::GroundAndFly,
        };
        check_skill_path(self, address, properties, path, player, block)
    }

    fn check_crossbow_cast(
        &mut self,
        address: RegisteredSkill,
        source: (i32, ShapeIdentity),
        properties: &CSkillBaseProperties,
    ) -> bool {
        check_ranged_weapon_and_mana(
            self, address, source, properties, RangedWeaponKind::Crossbow, CastManaRule::RequireCost,
        )
    }

    fn check_cast_mana(
        &mut self,
        address: RegisteredSkill,
        source: (i32, ShapeIdentity),
        properties: &CSkillBaseProperties,
    ) -> bool {
        check_cast_mana(self, address, source, properties)
    }

    fn check_cast_mana_without_text(
        &mut self,
        address: RegisteredSkill,
        source: (i32, ShapeIdentity),
        properties: &CSkillBaseProperties,
    ) -> bool {
        check_cast_mana_without_text(self, address, source, properties)
    }

    fn spend_cast_mana(
        &mut self,
        address: RegisteredSkill,
        player: Option<i32>,
        properties: &CSkillBaseProperties,
    ) -> bool {
        spend_cast_mana(self, address, player, properties)
    }

    fn spend_cast_mana_without_text(
        &mut self,
        address: RegisteredSkill,
        player: Option<i32>,
        properties: &CSkillBaseProperties,
    ) -> bool {
        spend_cast_mana_without_text(self, address, player, properties)
    }

    fn prepare_crossbow_player(
        &mut self,
        address: RegisteredSkill,
        player: Option<i32>,
        properties: &CSkillBaseProperties,
    ) -> bool {
        prepare_ranged_weapon_player(self, address, player, properties, RangedWeaponKind::Crossbow)
    }

    fn element_phalanx_weapon_modifier(&self, attacker_id: i32, target_level: i32) -> Option<f32> {
        let player = self.find_player(attacker_id)?;
        let (divisor, minimum) = self.globe_setup().weapon_damage_factors();
        Some(player.weapon_modifier(self.goods_factory(), target_level, divisor, minimum))
    }

    fn element_phalanx_critical_rate(&self) -> f32 { self.globe_setup().critical_rate() }

    fn element_phalanx_war_soul_target(
        &self,
        master: MasterInfo,
        target_id: i32,
    ) -> Option<(i32, ShapeIdentity)> {
        let target = self.find_player(target_id);
        let source = self.find_player(master.master_id);
        let (Some(target), Some(source)) = (target, source) else { return None; };
        if target_id == master.master_id { return None; }
        if target.shape().get_action() == 6 || target.is_dead() { return None; }
        let target = (target.shape().get_region_id(), target.shape().identity());
        let source = (source.shape().get_region_id(), source.shape().identity());
        if !self.live_skill_target_attackable_between(source, target) { return None; }
        Some(target)
    }

    fn monster_property_level(&self, region_id: i32, monster_id: i32) -> Option<u8> {
        let monster = self.find_region(region_id)?.base().find_monster_by_id(monster_id)?;
        let properties = self.find_monster_property_by_origin_name(monster.base_property_key()?)?;
        Some(properties.level as u8)
    }

    fn apply_weak_monster_attacks(
        &mut self,
        region_id: i32,
        monster_id: i32,
        state: &WeakState,
    ) -> bool {
        let Some(monster) = self.find_region_mut(region_id)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(monster_id))
        else { return false; };
        let modifiers = monster.move_shape_mut().property_modifiers_mut();
        let (minimum_attack, maximum_attack) = state.apply_to_monster_attacks(
            modifiers.minimum_attack, modifiers.maximum_attack,
        );
        modifiers.minimum_attack = minimum_attack;
        modifiers.maximum_attack = maximum_attack;
        true
    }

    fn subtract_poison_fog_monster_losses(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        defense_loss: u32,
        element_resistance_loss: u32,
    ) -> bool {
        let Some(shape) = resolve_state_move_shape_mut(self, region_id, target) else { return false };
        let modifiers = shape.property_modifiers_mut();
        modifiers.defense = modifiers.defense.wrapping_sub(defense_loss as i32);
        modifiers.element_resistance = modifiers.element_resistance
            .wrapping_sub(element_resistance_loss as i32);
        true
    }

    fn begin_base_applied_state(&mut self, region_id: i32, holder: ShapeIdentity, key: StateKey) -> bool {
        begin_base_applied_state(self, region_id, holder, key)
    }

    fn begin_applied_state_visual(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        loop_value: i32,
    ) -> bool {
        begin_applied_state_visual(self, region_id, holder, key, loop_value)
    }

    fn resolve_applied_state_sufferer(
        &self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
    ) -> Option<(i32, ShapeIdentity)> {
        resolve_applied_state_sufferer(self, region_id, holder, key)
    }

    fn update_property_state_visual<S: AppliedState>(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: ZonalCastPropertyTarget,
        now: &mut dyn FnMut() -> u32,
        client_time: impl FnOnce(&S, &mut dyn FnMut() -> u32) -> u32,
    ) -> bool {
        let target = match target {
            ZonalCastPropertyTarget::User => StatePropertyTarget::User,
            ZonalCastPropertyTarget::Sufferer => StatePropertyTarget::Sufferer,
        };
        update_property_state_visual(self, region_id, holder, key, target, now, client_time)
    }

    fn update_applied_state_end_visual(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: ZonalCastPropertyTarget,
    ) -> bool {
        let target = match target {
            ZonalCastPropertyTarget::User => StatePropertyTarget::User,
            ZonalCastPropertyTarget::Sufferer => StatePropertyTarget::Sufferer,
        };
        update_applied_state_end_visual(self, region_id, holder, key, target)
    }

    fn remove_applied_state_from(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: (i32, ShapeIdentity),
        bytes: usize,
    ) -> bool {
        remove_applied_state_from(self, region_id, holder, key, target, bytes)
    }

    fn send_zonal_cast_visual_to_player(&self, player_id: i32, message: &CMessage) {
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    fn send_zonal_cast_visual_around(&self, region_id: i32, origin: &CShape, message: &CMessage) {
        if let Some(region) = self.find_region(region_id) {
            let _ = self.send_game_shape_around(region.base(), origin, None, message);
        }
    }
}

impl<Runtime: GameMainLoopRuntime> ZonalCastContact<Runtime> for CGame {
    fn add_weak_phalanx(
        &mut self,
        region_id: i32,
        phalanx: nebokrai_zone::skills::CWeakPhalanx,
        tile_x: i32,
        tile_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<Result<i32, ()>> {
        self.add_weak_phalanx(region_id, phalanx, tile_x, tile_y, started_at_ms, runtime)
            .map(|result| result.map_err(|_| ()))
    }

    fn send_weak_phalanx_entry(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        self.send_weak_phalanx_entry(region_id, phalanx_id, runtime)
    }

    fn add_poison_fog_phalanx(
        &mut self,
        region_id: i32,
        phalanx: nebokrai_zone::skills::CPoisonFogPhalanx,
        tile_x: i32,
        tile_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<Result<i32, ()>> {
        self.add_poison_fog_phalanx(region_id, phalanx, tile_x, tile_y, started_at_ms, runtime)
            .map(|result| result.map_err(|_| ()))
    }

    fn send_poison_fog_phalanx_entry(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        self.send_poison_fog_phalanx_entry(region_id, phalanx_id, runtime)
    }

    fn add_snow_storm_phalanx(
        &mut self,
        region_id: i32,
        phalanx: nebokrai_zone::skills::CSnowStormPhalanx,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        self.add_snow_storm_phalanx(region_id, phalanx, started_at_ms, runtime)
    }

    fn add_god_thunder_phalanx(
        &mut self,
        region_id: i32,
        phalanx: nebokrai_zone::skills::CGodThunderPhalanx,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        self.add_god_thunder_phalanx(region_id, phalanx, started_at_ms, runtime)
    }

    fn add_masked_element_phalanx(
        &mut self,
        region_id: i32,
        phalanx: nebokrai_zone::skills::MaskedElementPhalanx,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        self.add_masked_element_phalanx(region_id, phalanx, started_at_ms, runtime, None)
    }

    fn add_chaos_sphere_phalanx(
        &mut self,
        region_id: i32,
        phalanx: nebokrai_zone::skills::CChaosSpherePhalanx,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        self.add_chaos_sphere_phalanx(region_id, phalanx, started_at_ms, runtime)
    }

    fn apply_owned_skill_contact(
        &mut self,
        master: MasterInfo,
        target: ShapeIdentity,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        self.apply_owned_skill_contact(master, target, region_id, attack, runtime);
    }

    fn apply_owned_skill_attack_to_war_soul(
        &mut self,
        master: MasterInfo,
        target_id: i32,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        self.apply_owned_skill_attack_to_war_soul(master, target_id, region_id, attack, runtime);
    }

    fn summon_fire_wall(
        &mut self,
        address: RegisteredSkill,
        source: (i32, ShapeIdentity),
        destination: (i32, i32),
        runtime: &mut Runtime,
    ) {
        super::firewall::summon_fire_wall(self, address, source, destination, runtime);
    }

    fn apply_soul_mirror_area(
        &mut self,
        address: RegisteredSkill,
        source: (i32, ShapeIdentity),
        properties: &CSkillBaseProperties,
        runtime: &mut Runtime,
    ) {
        super::soulmirror::apply_soul_mirror_area(self, address, source, properties, runtime);
    }
}

pub(crate) fn execute_player_zonal_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::ZonalCast,
        |game, instance, _, runtime| nebokrai_zone::skills::zonalcast::check_zonal_cast(
            game, instance, original_user, runtime.now_milliseconds(),
        ),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(),
        |game, instance, runtime| {
            let outcome = nebokrai_zone::skills::zonalcast::run_zonal_cast_ai(
                game, instance, runtime, &mut |runtime: &mut Runtime| runtime.now_milliseconds(),
            );
            state_skill_outcome(match outcome {
                ZonalCastAiOutcome::Pending => QueuedSkillExecutionState::Pending,
                ZonalCastAiOutcome::Rejected => QueuedSkillExecutionState::Rejected,
                ZonalCastAiOutcome::Completed => QueuedSkillExecutionState::Completed,
            })
        },
    )
}

struct ZonalCastSkill<const ID: u32>;
impl<const ID: u32> RegisteredStateSkill for ZonalCastSkill<ID> {
    const ID: u32 = ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::ZonalCast;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, _target: StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool {
        let user = game.registered_skill(address)
            .and_then(|skill| nebokrai_zone::skills::zonalcast::zonal_cast_resolved_user(game, skill));
        nebokrai_zone::skills::zonalcast::check_zonal_cast(
            game, address, user, runtime.now_milliseconds(),
        )
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = nebokrai_zone::skills::zonalcast::run_zonal_cast_ai(
            game, address, runtime, &mut |runtime: &mut Runtime| runtime.now_milliseconds(),
        );
        match outcome {
            ZonalCastAiOutcome::Rejected => end_state_skill(game, address, 0, runtime),
            ZonalCastAiOutcome::Completed => end_state_skill(game, address, 1, runtime),
            ZonalCastAiOutcome::Pending => state_skill_outcome(QueuedSkillExecutionState::Pending),
        }
    }
}

pub(crate) fn execute_owned_monster_zonal_cast<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<ZonalCastSkill<ID>, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}

pub(crate) fn publish_zonal_cast_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    nebokrai_zone::skills::zonalcast::publish_zonal_cast_visual(game, skill, mode);
}

pub(super) fn prepare_element_summon(
    game: &CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
) -> Option<(MasterInfo, CSkillBaseProperties, i32)> {
    nebokrai_zone::skills::zonalcast::prepare_element_summon(game, instance, source)
}
