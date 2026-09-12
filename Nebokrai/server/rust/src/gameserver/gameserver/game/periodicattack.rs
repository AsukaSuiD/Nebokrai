//! Общая координация заимствований для уже рассчитанных атак навыков.
//! Источник: gameserver.exe + GameServer.pdb, appserver/moveshape.cpp
//! и appserver/skills/fightdefense.cpp.
//!
//! Конкретные формулы, периодический шаг состояний и визуальное сетевое
//! представление принадлежат модулям навыка и состояния. Этот дочерний модуль
//! временно извлекает игрока, монстра и регион, проводит обычный
//! `OnBeenAttacked` через общую защиту и
//! выполняет обработку смерти и сетевые побочные эффекты, требующие нескольких
//! независимых владельцев `CGame`.
//! Зарегистрированные защиты проходят собственные Begin/Defense/End по живым
//! позициям арены. После них ApplyFinalDamage читает актуальные HP/MP;
//! клиентские записи отражают положительную знаковую разницу, не сумму powers.
//! OnAction предшествует реакции выбранного AI; общий удар не создаёт Stiffen
//! и не меняет action синхронно вместо отложенной Defense-команды.
//! Прямые попадания используют общий OnBeenAttacked-пролог смерти до 0xBF60B:
//! StopAllSkills, OnBeenMurdered и WhenBeenKilled видят опубликованный регион.
//! После пакета общий CMoveShape сохраняет identity убийцы и action 6;
//! OnDied остаётся у пассивной AI FIFO, а не вызывается синхронно из удара.
//! OnBeenHurted вызывается только после нелетального BF60A: общий координатор
//! выполняет Nation-уведомление, затем регистрацию первого атакующего.
//! Общий синхронный хвост после action 6 выполняет ClearAllStates(true) и
//! prison_check; частичная Cure-очистка не заменяет эту границу смерти.
//! Наложение состояний SpiderMist/SpriteBurn использует живой IsAttackAble,
//! а не допуск рассчитанного урона:
//! без god-фильтра, с текущими правами игрока и владельца питомца. Смерть,
//! Cure и исключение самого источника остаются отдельными правилами caller-а.

use super::*;
use crate::gameserver::appserver::skills::monsterattack::{
    owned_monster_attackable, resolve_owned_monster_attack_target,
};
use crate::gameserver::appserver::skills::skillfactory::SkillCategory;
use crate::gameserver::appserver::states::skill::RegisteredSkill;

impl CGame {
    // Единственный factory-owner категории Defense — CFightDefense. Читаем
    // живые позиции, чтобы отсутствие или повторение A сохраняло native цикл.
    fn begin_received_base_defense(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        index: &mut usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Option<RegisteredSkill> {
        loop {
            let address = self.registered_move_shape_skill_at(region_id, target, SkillCategory::Defense, *index)?;
            *index += 1;
            let skill = self.registered_skill_mut(address)?;
            if skill.id() != SKILL_BASE_DEFENSE { continue; }
            skill.lifecycle_mut().begin_objects(Some((region_id, target)), Some((region_id, target)), &mut *now);
            skill.lifecycle_mut().finish_begin(true);
            return Some(address);
        }
    }

    fn live_player_target_attackable(&self, source_id: i32, target_id: i32) -> bool {
        if self.find_player(target_id).is_none_or(|player| player.city_war_died_state()) {
            return false;
        }
        if let Some((text, limit)) = self.player_attack_level_block(source_id, target_id) {
            self.send_base_attack_level_block(source_id, text, limit);
            return false;
        }
        self.player_attack_pk_allowed(source_id, target_id)
    }

    pub(crate) fn live_skill_target_attackable(
        &self,
        region_id: i32,
        source: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool {
        if matches!(target.object_type, 1100 | 1200) {
            return source.object_type == 400
                && self.stationary_build_attackable_by_player(source.id, region_id, target);
        }
        if !matches!(target.object_type, 400 | 600) { return false; }
        let Some(region) = self.find_region(region_id).map(|owner| owner.base()) else { return false; };
        match source.object_type {
            400 => {
                if target.object_type == 400 {
                    return self.live_player_target_attackable(source.id, target.id);
                }
                let Some(monster) = region.find_monster_by_id(target.id) else { return false; };
                let Some(property) = monster.base_property_key()
                    .and_then(|name| self.find_monster_property_by_origin_name(name))
                else { return false; };
                let target_master = monster.master_info();
                if (monster.is_tamed() || monster.is_carriage(property))
                    && target_master.master_type == 400 && target_master.master_id != 0
                {
                    let Some(owner) = self.find_player(target_master.master_id) else { return true; };
                    if target_master.master_id == source.id { return owner.pk_permissions().criminal; }
                    return self.live_player_target_attackable(source.id, target_master.master_id);
                }
                self.monster_attackable_by_player(source.id, region_id, property)
            }
            600 => {
                let Some(monster) = region.find_monster_by_id(source.id) else { return false; };
                let Some(property) = monster.base_property_key()
                    .and_then(|name| self.find_monster_property_by_origin_name(name))
                else { return false; };
                let Some(target_snapshot) = resolve_owned_monster_attack_target(self, region, target)
                else { return false; };
                owned_monster_attackable(
                    self, region_id, property, monster.is_tamed(), monster.master_info(),
                    target, &target_snapshot,
                )
            }
            _ => false,
        }
    }

    fn increase_owned_skill_attacker_rp(&mut self, player_id: i32, skill_id: u32) {
        // Только эти Attack вызывают IncreaseRp после возврата OnBeenAttacked.
        // Отказ цели и Clear внутри Defense не отменяют caller-хвост.
        if matches!(skill_id,
            ARMY_BREAK_SKILL_ID | ARMY_BREAK_2_SKILL_ID | BOSS_BLUE_QUAKE_SKILL_ID
            | CHAIN_LIGHTNING_SKILL_ID | FLASH_SKILL_ID | JU_CUT_SKILL_ID
            | LITTLE_FLASH_SKILL_ID | LIGHTNING_SWORD_SKILL_ID | MOSOU_SKILL_ID
            | LITTLE_FLASH_2_SKILL_ID | LIGHTNING_SWORD_2_SKILL_ID
            | LIGHTNING_SWORD_3_SKILL_ID | LIGHTNING_SWORD_4_SKILL_ID
            | SWALLOW_SKILL_ID | GHOST_CUT_SKILL_ID | GHOST_CUT_2_SKILL_ID
            | GHOST_CUT_3_SKILL_ID
        ) {
            self.increase_owned_player_rp(player_id, true, 0);
        }
    }
    /// Эти навыки вызывают общий OnBeenAttacked здания без начисления RP.
    /// Проверка допустимости остаётся перед расчётом у конкретного caller-а.
    pub(crate) fn apply_owned_skill_attack_to_stationary_build<Runtime: GameMainLoopRuntime>(
        &mut self, player_id: i32, region_id: i32, identity: ShapeIdentity,
        attack: AttackInformation, runtime: &mut Runtime,
    ) {
        self.apply_direct_player_skill_attack_to_stationary_build(
            player_id, region_id, identity, attack, runtime,
        );
    }

    /// Общий CMoveShape::OnBeenAttacked у здания — другой virtual slot, чем
    /// отдельный CBuild::OnBeenAttacked. Здесь нет IsAttackAble и death-script;
    /// после защиты работают обычные OnAction, death-пролог и очистка состояний.
    pub(crate) fn apply_direct_player_skill_attack_to_stationary_build<Runtime: GameMainLoopRuntime>(
        &mut self,
        _player_id: i32,
        region_id: i32,
        identity: ShapeIdentity,
        mut attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        let Some(build) = self.find_region(region_id).and_then(|region| region.stationary_build(identity))
        else { return; };
        if build.hp() == 0 || build.move_shape().is_god()
            || !build.move_shape().shape().is_assigned_to_server_region()
        { return; }
        let region_id = build.move_shape().shape().get_region_id();
        let mut defense_index = 0;
        while let Some(defense_skill) = self.begin_received_base_defense(
            region_id, identity, &mut defense_index, &mut || runtime.now_milliseconds(),
        ) {
            let Some(build) = self.find_region(region_id).and_then(|region| region.stationary_build(identity))
            else { return; };
            let (defense, element_resistance) = (build.defence(), build.element_resistance());
            let source_id = attack.attacker_id;
            if attack.attacker_type == PLAYER_TYPE && CSkillFactory::is_war_soul_skill(attack.skill_id)
                && self.find_player(source_id).is_none()
            {
                let _ = self.end_registered_instance_without_after_use(defense_skill, SkillTermination::Completed);
                continue;
            }
            let defense_source = (attack.attacker_type == PLAYER_TYPE)
                .then(|| self.find_player(source_id)).flatten().map(|source| {
                let occupation = source.occupation();
                if let Some((properties, _, restored)) = self.war_soul_defense_projection(source_id, attack.skill_id) {
                    (properties, occupation, Some(restored))
                } else {
                    (source.combat_properties(), occupation, None)
                }
            });
            if self.received_player_defense_allowed(region_id, identity, source_id)
                && let Some((properties, occupation, _)) = defense_source
            {
                let mut random = |maximum| game_legacy_random(&mut self.random_state, maximum);
                defend_build_base_attack(
                    &mut attack, properties, occupation, defense, element_resistance,
                    &self.globe_setup, &mut random,
                );
            } else {
                attack.clear();
            }
            self.restore_war_soul_defense_projection(source_id, defense_source.and_then(|(_, _, restored)| restored));
            let _ = self.end_registered_instance_without_after_use(defense_skill, SkillTermination::Completed);
        }
        let (health, damage) = {
            let Some(build) = self.find_region_mut(region_id).and_then(|region| region.stationary_build_mut(identity))
            else { return; };
            let result = attack.final_damage_values(build.hp(), u32::MAX, None);
            build.set_hp(result.health);
            (result.health, result.hp_record)
        };
        if health == 0 {
            let attacker = KillingAttackIdentity::from(&attack);
            self.begin_move_shape_death(region_id, identity, attacker, runtime);
            let mut died = CMessage::new(0x000b_f60b);
            died.add_long(attack.attacker_type);
            died.add_long(attack.attacker_id);
            died.add_long(identity.object_type);
            died.add_long(identity.id);
            if damage > 0 { died.add_ulong(damage); }
            died.base_mut().add_char(1);
            Self::append_base_attack_tail(&mut died, &attack);
            let _ = self.send_move_shape_around(region_id, identity, &died);
            self.record_move_shape_death(region_id, identity, attacker, runtime);
        } else if attack.full_miss != 0 {
            let mut missed = CMessage::new(0x000b_f612);
            missed.add_byte(attack.full_miss);
            missed.add_long(identity.object_type);
            missed.add_long(identity.id);
            let _ = self.send_move_shape_around(region_id, identity, &missed);
        } else if damage > 0 {
            let _ = super::finish_blind_states_on_defense(self, region_id, identity, 0);
            let Some(health) = self.find_region(region_id).and_then(|region| region.stationary_build(identity))
                .map(CBuild::hp)
            else { return; };
            let mut hurt = CMessage::new(0x000b_f60a);
            hurt.add_long(attack.attacker_type);
            hurt.add_long(attack.attacker_id);
            hurt.add_long(identity.object_type);
            hurt.add_long(identity.id);
            Self::append_hurt_damage_records(&mut hurt, damage, 0);
            hurt.add_ulong(health);
            Self::append_base_attack_tail(&mut hurt, &attack);
            let _ = self.send_move_shape_around(region_id, identity, &hurt);
            if identity.object_type == CITY_GATE_OBJECT_TYPE as i32 {
                let _ = self.city_gate_on_been_hurted(region_id, identity.id, attack.attacker_type, attack.attacker_id);
            }
        }
    }


    // OnBeenAttacked использует права снимка атаки и актуального владельца
    // цели. IsAttackAble до попадания имеет другие security/region-фильтры.
    fn received_skill_pk_allowed(
        &self,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        victim_id: i32,
        controlled: bool,
    ) -> bool {
        let Some(attacker) = self.find_player(master.master_id) else { return true; };
        let Some(victim) = self.find_player(victim_id) else { return true; };
        if (!controlled && attacker.country() != victim.country())
            || attacker.is_enemy_faction_member(victim.faction_id())
            || attacker.is_city_war_enemy_faction_member(victim.faction_id())
        { return true; }
        let own = controlled && victim_id == master.master_id;
        let badman = victim.is_badman(self.globe_setup.pk_count_per_kill());
        if master.permitted_to_kill_player == 0 && !badman && !own { return false; }
        if master.permitted_to_kill_teammate == 0
            && (own || (master.master_team_id != 0 && master.master_team_id == victim.team_id()))
        { return false; }
        if master.permitted_to_kill_guild_member == 0
            && ((master.master_guild_id != 0 && master.master_guild_id == victim.faction_id())
                || (master.master_union_id != 0 && master.master_union_id == victim.union_id()))
        { return false; }
        master.permitted_to_kill_criminal != 0 || !badman || own
    }

    // CFightDefense::Defense (fightdefense.cpp): отказ очищает атаку после
    // OnFirstAttack, но не прерывает внешний OnBeenAttacked и его armor-tail.
    fn received_player_defense_allowed(
        &self,
        region_id: i32,
        target: ShapeIdentity,
        attacker_id: i32,
    ) -> bool {
        let Some(region) = self.find_region(region_id) else { return false; };
        let Some(source) = region.base().find_child_object(
            PLAYER_TYPE, attacker_id, CGuid::GUID_INVALID, self,
        ) else { return false; };
        if target.object_type != PLAYER_TYPE { return true; }
        let Some(target) = self.find_player(target.id) else { return false; };
        let (Ok(target_x), Ok(target_y)) = (
            target.shape().get_tile_x(), target.shape().get_tile_y(),
        ) else { return false; };
        let target_cell = region.base().region.get_cell(target_x, target_y).ok().flatten();
        let source_cell = region.base().region.get_cell(source.tile_x, source.tile_y).ok().flatten();
        // При NULL target-cell оригинал пропускает оба GetSecurity.
        if target_cell.is_some()
            && (region.get_security(target_x, target_y).ok() == Some(RegionSecurity::SAFE)
                || region.get_security(source.tile_x, source.tile_y).ok() == Some(RegionSecurity::SAFE))
        { return false; }
        if region.base().region.region_type() == 3
            && let (Some(target_cell), Some(source_cell)) = (target_cell, source_cell)
            && target_cell.city_war_marker() != source_cell.city_war_marker()
        { return false; }
        true
    }

    fn owned_skill_player_attackable(
        &self,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        victim_id: i32,
        region_id: i32,
    ) -> bool {
        let Some(attacker) = self.find_player(master.master_id) else {
            return false;
        };
        let Some(victim) = self.find_player(victim_id) else {
            return false;
        };
        if master.master_id == victim_id
            || victim.is_dead()
            || victim.is_god_mode()
            || victim.is_nation_war_player_weak()
            || attacker.server_region_id() != Some(region_id)
            || victim.server_region_id() != Some(region_id)
        {
            return false;
        }
        let Some(region) = self.find_region(region_id) else {
            return false;
        };
        let (Ok(attacker_x), Ok(attacker_y), Ok(victim_x), Ok(victim_y)) = (
            attacker.shape().get_tile_x(),
            attacker.shape().get_tile_y(),
            victim.shape().get_tile_x(),
            victim.shape().get_tile_y(),
        ) else {
            return false;
        };
        if region.base().no_pk
            || region.get_security(attacker_x, attacker_y).ok() == Some(RegionSecurity::SAFE)
            || region.get_security(victim_x, victim_y).ok() == Some(RegionSecurity::SAFE)
        {
            return false;
        }
        if attacker.is_enemy_faction_member(victim.faction_id())
            || attacker.is_city_war_enemy_faction_member(victim.faction_id())
        {
            return true;
        }
        let victim_badman = victim.is_badman(self.globe_setup.pk_count_per_kill());
        if master.permitted_to_kill_player == 0
            && !victim_badman
            && master.master_country_id == i32::from(victim.country())
        {
            return false;
        }
        if master.permitted_to_kill_teammate == 0
            && master.master_team_id != 0
            && master.master_team_id == victim.team_id()
        {
            return false;
        }
        if master.permitted_to_kill_guild_member == 0
            && ((master.master_guild_id != 0 && master.master_guild_id == victim.faction_id())
                || (master.master_union_id != 0 && master.master_union_id == victim.union_id()))
        {
            return false;
        }
        master.permitted_to_kill_criminal != 0 || !victim_badman
    }

    /// Разрешает прямую атаку уже рассчитанного навыка до её применения.
    /// Проверка остаётся у `CGame`, потому что одновременно читает игрока,
    /// монстра, регион и принадлежность; конкретный навык передаёт только
    /// идентичность цели и снимок прав владельца.
    pub(crate) fn owned_player_skill_target_attackable(
        &self,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        target: ShapeIdentity,
        region_id: i32,
    ) -> bool {
        if target.object_type == PLAYER_TYPE {
            return self.owned_skill_player_attackable(master, target.id, region_id);
        }
        if target.object_type != MONSTER_TYPE {
            return false;
        }
        let Some((property, health, god, tamed, carriage, target_master)) = self
            .find_region(region_id)
            .and_then(|owner| {
                let monster = owner.base().find_monster_by_id(target.id)?;
                let property = self
                    .find_monster_property_by_origin_name(monster.base_property_key()?)?;
                Some((
                    property,
                    monster.hit_points(),
                    monster.move_shape().is_god(),
                    monster.is_tamed(),
                    monster.is_carriage(property),
                    monster.master_info(),
                ))
            })
        else {
            return false;
        };
        health != 0
            && !god
            && self.monster_attackable_by_player(master.master_id, region_id, property)
            && (!(tamed || carriage)
                || self.owned_skill_monster_attackable(master, target_master))
    }

    pub(crate) fn apply_owned_skill_attack_to_player<Runtime: GameMainLoopRuntime>(
        &mut self,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        target_id: i32,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        let skill_id = attack.skill_id;
        self.receive_player_skill_attack(
            master, target_id, region_id, attack, false, runtime,
        );
        self.increase_owned_skill_attacker_rp(master.master_id, skill_id);
    }

    /// Замыкает virtual `CPlayer::IncreaseRp -> PropertiesChanged` на
    /// canonical player и адресный `0xBF721`.
    pub(crate) fn increase_owned_player_rp(
        &mut self,
        player_id: i32,
        attacking: bool,
        damage: u16,
    ) {
        let changed = {
            let (players, globe_setup) = (&mut self.players, &self.globe_setup);
            players
                .get_mut(&player_id)
                .is_some_and(|player| player.increase_rp(attacking, damage, globe_setup))
        };
        if changed && let Some(player) = self.find_player(player_id) {
            let _ = self.send_player_properties_changed(player);
        }
    }

    /// Применяет уже рассчитанное попадание к вынесенной боевой фее игрока.
    /// Эта ветвь меняет экипировку и выполняет соответствующую доставку,
    /// не проходя обычные Defense и ApplyFinalDamage.
    pub(crate) fn apply_owned_skill_attack_to_war_soul<Runtime: GameMainLoopRuntime>(
        &mut self,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        target_id: i32,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        self.receive_player_skill_attack(
            master, target_id, region_id, attack, true, runtime,
        );
    }

    pub(crate) fn receive_player_skill_attack<Runtime: GameMainLoopRuntime>(
        &mut self,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        target_id: i32,
        _region_id: i32,
        mut attack: AttackInformation,
        war_soul_hit: bool,
        runtime: &mut Runtime,
    ) {
        let Some(target) = self.find_player(target_id) else { return; };
        if target.is_dead() || target.is_god_mode()
            || target.in_changing_server() || target.in_changing_region()
            || !target.shape().is_assigned_to_server_region()
        { return; }
        let region_id = target.shape().get_region_id();
        if self.find_region(region_id).is_none()
            || !self.received_skill_pk_allowed(master, target_id, false)
        {
            return;
        }
        let _ = self.player_on_first_attack_at_victim(
            master.master_id, target_id, Some(region_id), runtime,
        );
        if war_soul_hit && target_id != attack.attacker_id {
            let raw_damage = attack.damages.iter().fold(0_i32, |total, power| {
                total.wrapping_add(power.hp_damage)
            });
            let da_kong_key = self.globe_setup.da_kong_key();
            let (players, goods_factory) = (&mut self.players, &self.goods_factory);
            let outcome = players.get_mut(&target_id).and_then(|target| {
                target.apply_war_soul_hit(raw_damage, goods_factory, da_kong_key)
            });
            if let Some(outcome) = outcome {
                if outcome.broken {
                    let mut broken = CMessage::new(0x000b_f92f);
                    broken.add_long(PLAYER_TYPE);
                    broken.add_long(target_id);
                    let _ = self.send_player_shape_around(target_id, None, &broken);
                }
                if outcome.broadcast_previous_status {
                    let mut status = CMessage::new(0x000b_f930);
                    status.add_long(PLAYER_TYPE);
                    status.add_long(target_id);
                    let _ = self.send_player_shape_around(target_id, None, &status);
                }
                let _ = self.send_battle_fairy_goods_update(&outcome.update);
            }
            return;
        } else {
            let identity = ShapeIdentity {
                object_type: PLAYER_TYPE, id: target_id, ex_id: CGuid::GUID_INVALID,
            };
            let mut defense_index = 0;
            while let Some(defense_skill) = self.begin_received_base_defense(
                region_id, identity, &mut defense_index, &mut || runtime.now_milliseconds(),
            ) {
                let Some(target) = self.find_player(target_id) else { return; };
                let (target_properties, target_mana, target_war_soul_mana, pillar_damage_factor) = (
                    target.combat_properties(), target.mana(),
                    target.war_soul_mana(&self.goods_factory),
                    target.pillar_state().map(|state| state.damage_factor()),
                );
                // Clear предыдущего Defense меняет также источник и skill-id.
                let source_id = attack.attacker_id;
                if attack.attacker_type == PLAYER_TYPE && CSkillFactory::is_war_soul_skill(attack.skill_id)
                    && self.find_player(source_id).is_none()
                {
                    let _ = self.end_registered_instance_without_after_use(defense_skill, SkillTermination::Completed);
                    continue;
                }
                let defense_source = (attack.attacker_type == PLAYER_TYPE)
                    .then(|| self.find_player(source_id)).flatten().map(|source| {
                        let occupation = source.occupation();
                        if let Some((properties, _, restored)) = self.war_soul_defense_projection(source_id, attack.skill_id) {
                            (properties, occupation, Some(restored))
                        } else {
                            (source.combat_properties(), occupation, None)
                        }
                    });
                let mut defense_shields = self.find_player_mut(target_id)
                    .map(CPlayer::take_defense_shields).unwrap_or_default();
                if self.received_player_defense_allowed(region_id, identity, source_id)
                    && let Some((attacker_properties, attacker_occupation, _)) = defense_source
                {
                    let mut random = |maximum| game_legacy_random(&mut self.random_state, maximum);
                    defend_player_base_attack(
                        &mut attack, attacker_properties, attacker_occupation,
                        target_properties, target_mana, target_war_soul_mana,
                        &self.globe_setup, &mut random, defense_shields.as_mut_slice(),
                        pillar_damage_factor,
                    );
                } else {
                    attack.clear();
                }
                if let Some(target) = self.find_player_mut(target_id) {
                    target.restore_defense_shields(defense_shields);
                }
                self.restore_war_soul_defense_projection(
                    source_id, defense_source.and_then(|(_, _, restored)| restored),
                );
                let _ = self.end_registered_instance_without_after_use(defense_skill, SkillTermination::Completed);
            }
        }
        let (current_health, damage, mana_damage) = {
            let Some(target) = self.find_player_mut(target_id) else { return; };
            let result = attack.final_damage_values(
                target.health(), target.maximum_health(),
                Some((target.mana(), target.maximum_mana())),
            );
            // Пустая атака не вызывает SetHP/MP: повторный setter мог бы
            // обрезать уже существующее значение выше текущего максимума.
            if !attack.damages.is_empty() || attack.damage_modifier != 0 {
                target.set_health(result.health);
            }
            if !attack.damages.is_empty() {
                target.set_mana(result.mana.unwrap_or(target.mana()));
            }
            (result.health, result.hp_record, result.mp_record)
        };
        if damage != 0 {
            self.increase_owned_player_rp(target_id, false, damage as u16);
        }
        let _ = self.publish_player_states(target_id);
        if current_health != 0 && damage == 0 && mana_damage == 0 {
            if attack.full_miss != 0 {
                let mut missed = CMessage::new(0x000b_f612);
                missed.add_byte(attack.full_miss);
                missed.add_long(PLAYER_TYPE);
                missed.add_long(target_id);
                let _ = self.send_player_shape_around(target_id, None, &missed);
            } else {
                // OnBeenAttacked изнашивает броню и при пустом результате,
                // хотя hurt-пакета и реакции AI в этой ветке нет.
                self.damage_player_armor(target_id);
            }
            return;
        }
        if current_health != 0 && attack.full_miss == 0 {
            self.enter_player_combat_state(target_id);
            let _ = super::finish_player_blind_states_on_defense(self, target_id, 0);
            if let Some(target) = self.find_player_mut(target_id) {
                target.player_ai_mut().when_been_hurted(runtime.now_milliseconds());
            }
        }
        if current_health == 0 {
            let victim = ShapeIdentity {
                object_type: PLAYER_TYPE,
                id: target_id,
                ex_id: CGuid::GUID_INVALID,
            };
            let attacker = KillingAttackIdentity::from(&attack);
            self.begin_move_shape_death(region_id, victim, attacker, runtime);
            let mut died = CMessage::new(0x000b_f60b);
            died.add_long(attack.attacker_type);
            died.add_long(attack.attacker_id);
            died.add_long(PLAYER_TYPE);
            died.add_long(target_id);
            if damage > 0 { died.add_ulong(damage); }
            died.base_mut().add_char(1);
            Self::append_base_attack_tail(&mut died, &attack);
            let _ = self.send_player_shape_around(target_id, None, &died);
            self.record_move_shape_death(region_id, victim, attacker, runtime);
        } else if attack.full_miss != 0 {
            let mut missed = CMessage::new(0x000b_f612);
            missed.add_byte(attack.full_miss);
            missed.add_long(PLAYER_TYPE);
            missed.add_long(target_id);
            let _ = self.send_player_shape_around(target_id, None, &missed);
        } else {
            let Some(health) = self.find_player(target_id).map(CPlayer::health) else { return; };
            let mut hurt = CMessage::new(0x000b_f60a);
            hurt.add_long(attack.attacker_type);
            hurt.add_long(attack.attacker_id);
            hurt.add_long(PLAYER_TYPE);
            hurt.add_long(target_id);
            Self::append_hurt_damage_records(&mut hurt, damage, mana_damage);
            hurt.add_ulong(health);
            Self::append_base_attack_tail(&mut hurt, &attack);
            let _ = self.send_player_shape_around(target_id, None, &hurt);
            let attacker = ShapeIdentity {
                object_type: attack.attacker_type, id: attack.attacker_id, ex_id: CGuid::GUID_INVALID,
            };
            self.damage_player_armor(target_id);
            let _ = self.retarget_passive_pets_after_player_hurt(target_id, attacker);
            let _ = self.notify_country_after_player_hurt(target_id, attacker, runtime);
        }
    }

    pub(crate) fn apply_monster_periodic_state_attack<Runtime: GameMainLoopRuntime>(
        &mut self,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        target: ShapeIdentity,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        let Some(owner) = self.take_region_owner(region_id) else { return };
        let Some(snapshot) = crate::gameserver::appserver::skills::monsterattack::resolve_owned_monster_attack_target(
            self,
            owner.base(),
            target,
        ) else {
            self.restore_region_owner(owner);
            return;
        };
        let attack = crate::gameserver::appserver::skills::monsterattack::defend_owned_monster_attack(
            self,
            target,
            snapshot.mana,
            snapshot.war_soul_mana,
            snapshot.player_properties,
            snapshot.monster_properties,
            attack,
        );
        let now_ms = runtime.now_milliseconds();
        let mut owner = Some(owner);
        crate::gameserver::appserver::skills::monsterattack::apply_owned_monster_attack_hit(
            self,
            &mut owner,
            runtime,
            now_ms,
            master.master_id,
            crate::gameserver::appserver::masterinfo::MasterInfo::default(),
            target,
            &snapshot.shape,
            snapshot.health,
            snapshot.mana,
            snapshot.master,
            snapshot.monster_property,
            snapshot.tamed,
            snapshot.carriage,
            attack,
        );
        let Some(owner) = owner else { return };
        self.restore_region_owner(owner);
    }

    fn owned_skill_monster_attackable(
        &self,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        owner: crate::gameserver::appserver::masterinfo::MasterInfo,
    ) -> bool {
        if owner.master_type != PLAYER_TYPE || owner.master_id == 0 {
            return true;
        }
        let Some(attacker) = self.find_player(master.master_id) else {
            return false;
        };
        let Some(victim) = self.find_player(owner.master_id) else {
            return false;
        };
        if attacker.is_enemy_faction_member(victim.faction_id())
            || attacker.is_city_war_enemy_faction_member(victim.faction_id())
        {
            return true;
        }
        let victim_badman = victim.is_badman(self.globe_setup.pk_count_per_kill());
        if master.permitted_to_kill_player == 0
            && !victim_badman
            && owner.master_id != master.master_id
        {
            return false;
        }
        if master.permitted_to_kill_teammate == 0
            && (owner.master_id == master.master_id
                || (master.master_team_id != 0 && master.master_team_id == owner.master_team_id))
        {
            return false;
        }
        if master.permitted_to_kill_guild_member == 0
            && ((master.master_guild_id != 0 && master.master_guild_id == owner.master_guild_id)
                || (master.master_union_id != 0 && master.master_union_id == owner.master_union_id))
        {
            return false;
        }
        master.permitted_to_kill_criminal != 0
            || !victim_badman
            || owner.master_id == master.master_id
    }

    pub(super) fn enter_player_criminal_state<Runtime: GameClockContext>(
        &mut self, player_id: i32, runtime: &mut Runtime,
    ) -> Option<bool> {
        let threshold = self.globe_setup.pk_count_per_kill();
        let player = self.find_player(player_id)?;
        if u32::from(player.pk_count()) > threshold { return None; }
        if player.criminal_state_timestamp_ms() == 0 {
            let mut message = CMessage::new(0x000b_f60e);
            message.add_long(player_id);
            message.add_byte(1);
            let _ = self.send_player_shape_around(player_id, None, &message);
        }
        self.find_player_mut(player_id)?
            .enter_criminal_state(threshold, || runtime.now_milliseconds())
    }

    /// CMoveShape::OnBeenAttacked различает номер текущего GetAI и класс
    /// монстра. Здесь не действует предварительный IsAttackAble: после
    /// отбрасывания PK вызывается до проверки безопасной клетки питомца.
    fn received_player_attack_by_monster<Runtime: GameMainLoopRuntime>(
        &mut self, master: crate::gameserver::appserver::masterinfo::MasterInfo,
        region_id: i32, target_id: i32, property: &crate::setup::monsterlist::MonsterProperties,
        runtime: &mut Runtime,
    ) -> bool {
        let identity = ShapeIdentity { object_type: MONSTER_TYPE, id: target_id, ex_id: CGuid::GUID_INVALID };
        if !self.received_player_defense_allowed(region_id, identity, master.master_id) { return true; }
        let Some(attacker) = self.find_player(master.master_id) else { return true; };
        let permissions = attacker.pk_permissions();
        let country = attacker.country();
        let gods_faction = attacker.gods_battle_faction();
        let Some(region) = self.find_region(region_id) else { return false; };
        let Some(monster) = region.base().find_monster_by_id(target_id) else { return false; };
        // Auxiliary AI не проходит SetAIType фабрики и сохраняет ctor-ноль.
        let ai_type = monster.active_ai().map(|_| monster.active_primary_ai_type().unwrap_or(0));
        let controlled = monster.is_tamed() || monster.is_carriage(property);
        let controller = monster.master_info().master_id;
        let region_country = region.base().country;
        if property.kind == 5 {
            let Some(ai_type) = ai_type else { return false; };
            let own_country = match ai_type {
                8 | 9 => Some(true),
                13 | 14 => Some(u32::from(country) == property.race),
                15 => Some(country == region_country),
                19 | 20 | 21 => {
                    if u32::from(country) != property.race { return true; }
                    Some(true)
                }
                _ => None,
            };
            if let Some(own_country) = own_country {
                if !own_country { return permissions.country; }
                if !permissions.player { return false; }
                let _ = self.enter_player_criminal_state(master.master_id, runtime);
                return true;
            }
        }
        if property.kind == 0
            || (property.kind == 5 && ai_type.is_some_and(|kind| !matches!(kind, 10 | 11)))
        {
            let Some(ai_type) = ai_type else { return false; };
            if ai_type != 23 || gods_faction as u32 == property.race {
                let _ = self.enter_player_criminal_state(master.master_id, runtime);
            }
            return true;
        }
        if !controlled { return true; }
        if !self.received_skill_pk_allowed(master, controller, true) { return false; }
        if self.find_player(controller).is_some() {
            let Some(shape) = self.find_region(region_id)
                .and_then(|region| region.base().find_monster_by_id(target_id))
                .map(CMonster::move_shape).map(|shape| shape.shape())
            else { return false; };
            let (Ok(y), Ok(x)) = (shape.get_tile_y(), shape.get_tile_x()) else { return false; };
            let _ = self.player_on_first_attack_at_position(
                master.master_id, controller, Some(region_id), (x, y), runtime,
            );
        }
        let Some(region) = self.find_region(region_id) else { return false; };
        let Some(target) = region.base().find_monster_by_id(target_id) else { return false; };
        let (Ok(y), Ok(x)) = (target.move_shape().shape().get_tile_y(), target.move_shape().shape().get_tile_x())
        else { return false; };
        if region.get_security(x, y).ok() == Some(RegionSecurity::SAFE) { return false; }
        let Some(attacker) = self.find_player(master.master_id) else { return false; };
        let (Ok(y), Ok(x)) = (attacker.shape().get_tile_y(), attacker.shape().get_tile_x()) else { return false; };
        region.get_security(x, y).ok() != Some(RegionSecurity::SAFE)
    }

    pub(crate) fn apply_owned_skill_attack_to_monster<Runtime: GameMainLoopRuntime>(
        &mut self,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        target_id: i32,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        let skill_id = attack.skill_id;
        self.receive_monster_skill_attack(master, target_id, region_id, attack, runtime);
        self.increase_owned_skill_attacker_rp(master.master_id, skill_id);
    }

    pub(crate) fn receive_monster_skill_attack<Runtime: GameMainLoopRuntime>(
        &mut self,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        target_id: i32,
        region_id: i32,
        mut attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        let Some(property) = self
            .find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(target_id))
            .and_then(CMonster::base_property_key)
            .and_then(|key| self.find_monster_property_by_origin_name(key))
            .cloned()
        else {
            return;
        };
        let Some((target_health, god)) =
            self.find_region(region_id).and_then(|owner| {
                let monster = owner.base().find_monster_by_id(target_id)?;
                Some((
                    monster.hit_points(),
                    monster.move_shape().is_god(),
                ))
            })
        else {
            return;
        };
        if target_health == 0
            || god
            || !self.received_player_attack_by_monster(master, region_id, target_id, &property, runtime)
        {
            return;
        }
        let identity = ShapeIdentity { object_type: MONSTER_TYPE, id: target_id, ex_id: CGuid::GUID_INVALID };
        let mut defense_index = 0;
        while let Some(defense_skill) = self.begin_received_base_defense(
            region_id, identity, &mut defense_index, &mut || runtime.now_milliseconds(),
        ) {
            let source_id = attack.attacker_id;
            if attack.attacker_type == PLAYER_TYPE && CSkillFactory::is_war_soul_skill(attack.skill_id)
                && self.find_player(source_id).is_none()
            {
                let _ = self.end_registered_instance_without_after_use(defense_skill, SkillTermination::Completed);
                continue;
            }
            let defense_source = (attack.attacker_type == PLAYER_TYPE)
                .then(|| self.find_player(source_id)
                    .map(|attacker| (attacker.combat_properties(), attacker.occupation(), attacker.level())))
                .flatten()
                .map(|(properties, occupation, level)| {
                    if let Some((properties, level, restored)) =
                        self.war_soul_defense_projection(source_id, attack.skill_id)
                    {
                        (properties, occupation, level, Some(restored))
                    } else {
                        (properties, occupation, level, None)
                    }
                });
            if self.received_player_defense_allowed(region_id, identity, source_id)
                && let Some((properties, occupation, level, _)) = defense_source
            {
                let Some(target_properties) = self.find_region(region_id)
                    .and_then(|region| region.base().find_monster_by_id(target_id))
                    .map(|monster| monster.combat_properties(&property))
                else { return; };
                let mut random = |maximum| game_legacy_random(&mut self.random_state, maximum);
                defend_monster_base_attack(
                    &mut attack, properties, occupation, level, target_properties, &self.globe_setup, &mut random,
                );
            } else {
                attack.clear();
            }
            self.restore_war_soul_defense_projection(source_id, defense_source.and_then(|(_, _, _, restored)| restored));
            let _ = self.end_registered_instance_without_after_use(defense_skill, SkillTermination::Completed);
        }
        let Some(monster) = self.find_region_mut(region_id)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(target_id))
        else { return; };
        let final_damage = attack.final_damage_values(monster.hit_points(), u32::MAX, None);
        monster.set_hit_points(final_damage.health);
        let damage = final_damage.hp_record;
        let current_health = final_damage.health;
        let hurt_response = attack.full_miss == 0 && damage != 0 && current_health != 0;
        if hurt_response {
            let _ = super::finish_blind_states_on_defense(self, region_id, identity, 0);
        }
        let Some(monster) = self.find_region(region_id)
            .and_then(|region| region.base().find_monster_by_id(target_id))
        else { return; };
        let active_ai = monster.active_ai();
        let ai_type = monster.active_primary_ai_type().unwrap_or(0);
        let react = hurt_response && active_ai.is_some();
        let now_ms = if react && !matches!(ai_type, 2 | 19 | 20) {
            runtime.now_milliseconds()
        } else { 0 };
        let lord_hurt_plan = (react && ai_type == 19)
            .then(|| {
                crate::gameserver::appserver::ai::lord::plan_lord_hurt_response(
                    self,
                    region_id,
                    target_id,
                    &property,
                )
            });
        if let Some(mut owner) = self.take_region_owner(region_id) {
            if let Some(monster) = owner.base_mut().find_monster_by_id_mut(target_id) {
                if react {
                    if matches!(active_ai, Some(crate::gameserver::appserver::ai::aifactory::ActiveMonsterAi::Pet)) {
                        monster.when_pet_been_hurted_by(ShapeIdentity {
                            object_type: attack.attacker_type,
                            id: attack.attacker_id,
                            ex_id: CGuid::GUID_INVALID,
                        }, now_ms);
                    } else if ai_type == 1 {
                        monster.when_passive_gladiator_hurted_by(
                            ShapeIdentity {
                                object_type: master.master_type,
                                id: master.master_id,
                                ex_id: CGuid::GUID_INVALID,
                            },
                            now_ms,
                            false,
                        );
                    } else if ai_type == 2 {
                        // Владелец AI2 применит реакцию после освобождения
                        // изменяемого заимствования монстра.
                    } else if ai_type == 13 {
                        // Поиск AI13 выполняется после освобождения изменяемого
                        // заимствования монстра.
                    } else if ai_type == 11 {
                        // Поиск AI11 выполняется после освобождения изменяемого
                        // заимствования монстра.
                    } else if ai_type == 20 {
                        // AI20 разрешает владельца периодического эффекта и
                        // связывает близнеца после освобождения заимствования.
                    } else if ai_type == 19 {
                        // AI19 применяет Defense, spatial-step и выбор цели
                        // после освобождения заимствования монстра.
                    } else if matches!(ai_type, 8 | 17 | 100 | 101) {
                        monster.when_been_hurted(now_ms);
                    } else {
                        monster.when_been_hurted_by(
                            ShapeIdentity {
                                object_type: master.master_type,
                                id: master.master_id,
                                ex_id: CGuid::GUID_INVALID,
                            },
                            false,
                            now_ms,
                        );
                    }
                }
            }
            if react
                && ai_type == 2
            {
                crate::gameserver::appserver::ai::smartgladiator::apply_player_hurt_response(
                    self,
                    owner.base_mut(),
                    target_id,
                    &property,
                    master.master_id,
                    runtime,
                );
            }
            if react
                && ai_type == 11
            {
                crate::gameserver::appserver::ai::cityguardwithbow::retarget_city_bow_guard_after_hurt(
                    self,
                    owner.base_mut(),
                    target_id,
                    &property,
                    now_ms,
                );
            }
            if react
                && ai_type == 13
            {
                crate::gameserver::appserver::ai::vilcouguardwithbow::retarget_village_bow_guard_after_hurt(
                    self,
                    owner.base_mut(),
                    target_id,
                    &property,
                    now_ms,
                );
            }
            if react
                && ai_type == 20
            {
                let _ = retarget_jiumai_after_hurt(
                    self,
                    owner.base_mut(),
                    target_id,
                    ShapeIdentity {
                        object_type: master.master_type,
                        id: master.master_id,
                        ex_id: CGuid::GUID_INVALID,
                    },
                    runtime,
                );
            }
            if let Some(plan) = lord_hurt_plan {
                let _ = crate::gameserver::appserver::ai::lord::apply_lord_hurt_response(
                    self,
                    owner.base_mut(),
                    target_id,
                    ShapeIdentity {
                        object_type: master.master_type,
                        id: master.master_id,
                        ex_id: CGuid::GUID_INVALID,
                    },
                    || runtime.now_milliseconds(),
                    plan,
                );
            }
            if react
                && matches!(ai_type, 8 | 17 | 100 | 101)
            {
                crate::gameserver::appserver::ai::guardcountry::retarget_special_guard_after_hurt(
                    self,
                    owner.base_mut(),
                    target_id,
                    &property,
                );
            }
            self.restore_region_owner(owner);
        }
        if attack.full_miss != 0 && current_health != 0 {
            let mut missed = CMessage::new(0x000b_f612);
            missed.add_byte(attack.full_miss);
            missed.add_long(MONSTER_TYPE);
            missed.add_long(target_id);
            let _ = self.send_move_shape_around(region_id, identity, &missed);
            return;
        }
        if damage == 0 && current_health != 0 {
            return;
        }
        if current_health != 0 {
            let Some(health) = self.find_region(region_id)
                .and_then(|region| region.base().find_monster_by_id(target_id)).map(CMonster::hit_points)
            else { return; };
            let mut hurt = CMessage::new(0x000b_f60a);
            hurt.add_long(attack.attacker_type);
            hurt.add_long(attack.attacker_id);
            hurt.add_long(MONSTER_TYPE);
            hurt.add_long(target_id);
            hurt.add_byte(1);
            hurt.add_byte(0);
            hurt.add_ulong(damage);
            hurt.add_ulong(health);
            Self::append_base_attack_tail(&mut hurt, &attack);
            let _ = self.send_move_shape_around(region_id, identity, &hurt);
            let _ = self.monster_on_been_hurted(
                region_id,
                target_id,
                master.master_type,
                master.master_id,
                runtime,
            );
            return;
        }

        let victim = ShapeIdentity {
            object_type: MONSTER_TYPE,
            id: target_id,
            ex_id: CGuid::GUID_INVALID,
        };
        let attacker = KillingAttackIdentity::from(&attack);
        self.begin_move_shape_death(region_id, victim, attacker, runtime);
        let mut died = CMessage::new(0x000b_f60b);
        died.add_long(attack.attacker_type);
        died.add_long(attack.attacker_id);
        died.add_long(MONSTER_TYPE);
        died.add_long(target_id);
        if damage != 0 { died.add_ulong(damage); }
        died.base_mut().add_char(1);
        Self::append_base_attack_tail(&mut died, &attack);
        let _ = self.send_move_shape_around(region_id, victim, &died);
        self.record_move_shape_death(region_id, victim, attacker, runtime);
    }

}
