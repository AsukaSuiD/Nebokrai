//! Общая координация заимствований для уже рассчитанных атак навыков.
//!
//! Конкретные формулы, периодический шаг состояний и визуальное сетевое
//! представление принадлежат модулям навыка и состояния. Этот дочерний модуль
//! временно извлекает игрока, монстра и регион, проводит обычный
//! `OnBeenAttacked` через общую защиту и
//! выполняет обработку смерти и сетевые побочные эффекты, требующие нескольких
//! независимых владельцев `CGame`.
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

impl CGame {
    fn live_player_target_attackable(&self, source_id: i32, target_id: i32) -> bool {
        if self.find_player(target_id).is_none_or(|player| player.city_war_died_state()) {
            return false;
        }
        if let Some((text, limit)) = self.player_base_attack_level_block(source_id, target_id) {
            self.send_base_attack_level_block(source_id, text, limit);
            return false;
        }
        self.player_base_attackable(source_id, target_id)
    }

    pub(crate) fn live_skill_target_attackable(
        &self,
        region_id: i32,
        source: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool {
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
        // Range/Fast Attack (VA 0x005123e0/0x00513700/0x00530fb0) не вызывают
        // IncreaseRp после OnBeenAttacked; RP защищающейся стороны обычный.
        // MachineryStomp/LordWiderangingAttack (VA 0x00532290/0x0052fdc0)
        // имеют такой же хвост без IncreaseRp.
        if !matches!(skill_id, MONSTER_RANGE_ATTACK_SKILL_ID | MONSTER_FAST_ATTACK_SKILL_ID | LORD_FAST_ATTACK_SKILL_ID | MACHINERY_STOMP_SKILL_ID | LORD_WIDERANGING_ATTACK_SKILL_ID) {
            self.increase_owned_player_rp(player_id, true, 0);
        }
    }
    /// Рассчитанная атака навыка проходит ту же защиту `CBuild/CCityGate`,
    /// что базовая атака; death-script и wire остаются у общего build-owner-а.
    pub(crate) fn apply_owned_skill_attack_to_stationary_build<Runtime: GameMainLoopRuntime>(
        &mut self, player_id: i32, region_id: i32, identity: ShapeIdentity,
        mut attack: AttackInformation, runtime: &mut Runtime,
    ) {
        if !self.stationary_build_attackable_by_player(player_id, region_id, identity) {
            return;
        }
        let Some(target) = self.stationary_build_combat_snapshot(region_id, identity) else { return };
        let Some(view) = self.base_magic_target_view(region_id, identity) else { return };
        let Some((properties, occupation)) = self.find_player(player_id)
            .map(|player| (player.combat_properties(), player.occupation())) else { return };
        let mut random = |maximum| game_legacy_random(&mut self.random_state, maximum);
        defend_build_base_attack(&mut attack, properties, occupation, target.defense,
            target.element_resistance, &self.globe_setup, &mut random);
        self.apply_defended_player_attack_to_stationary_build(player_id, region_id, identity,
            view.tile_x, view.tile_y, &attack, runtime);
        self.increase_owned_skill_attacker_rp(player_id, attack.skill_id);
    }

    fn player_on_owned_skill_attack<Runtime: GameMainLoopRuntime>(
        &mut self,
        attacker_id: i32,
        victim_id: i32,
        region_id: i32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        let attacker_country_identity = self.player_country_identity(attacker_id);
        let attacker = self.find_player(attacker_id)?;
        let victim = self.find_player(victim_id)?;
        let victim_x = victim.shape().get_tile_x().ok()?;
        let victim_y = victim.shape().get_tile_y().ok()?;
        let owner = self.find_region(region_id)?;
        let security = owner.get_security(victim_x, victim_y).ok()?;
        let disposition = CPKSys::on_kill(KillPkFacts {
            victim_is_badman: victim.is_badman(self.globe_setup.pk_count_per_kill()),
            security,
            city_war_enemies: CPKSys::is_city_war_state(Some(attacker), Some(victim)),
            faction_war_enemies: CPKSys::is_faction_war_state(Some(attacker), Some(victim)),
            gods_battle_region: owner.is_gods_battle(),
            same_gods_battle_faction: attacker.gods_battle_faction()
                == victim.gods_battle_faction(),
            same_country: attacker.country() == victim.country(),
            attacker_country_identity,
            attacker_kill_count: attacker.kill_count(),
        });
        let victim_level = victim.level();
        let murderer_report = (disposition == KillPkDisposition::ReportMurderer)
            .then(|| self.report_player_murderer(attacker_id, runtime.now_milliseconds()))
            .flatten();
        let eligible = matches!(
            disposition,
            KillPkDisposition::AllowedCombat | KillPkDisposition::ReportMurderer
        );
        let world_log_delivery =
            (eligible && self.log_system.player_killer_log_enabled()).then(|| {
                let mut message = CMessage::new(0x0006_020a);
                message.add_byte(0);
                message.add_long(victim_id);
                message.add_long(attacker_id);
                message.add_ulong(u32::from(victim_level));
                message.add_long(victim_x);
                message.add_long(victim_y);
                message.send(self, false)
            });
        tracing::debug!(
            attacker_id,
            victim_id,
            ?disposition,
            ?murderer_report,
            ?world_log_delivery,
            "обработан удар рассчитанного навыка по игроку"
        );
        Some(())
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
        self.apply_owned_skill_attack_to_player_kind(
            master, target_id, region_id, attack, false, runtime,
        );
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
    /// Флаг остаётся на runtime-root, потому что одна атомарная операция меняет
    /// экипировку, обычные HP/MP и выполняет соответствующую доставку.
    pub(crate) fn apply_owned_skill_attack_to_war_soul<Runtime: GameMainLoopRuntime>(
        &mut self,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        target_id: i32,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        self.apply_owned_skill_attack_to_player_kind(
            master, target_id, region_id, attack, true, runtime,
        );
    }

    fn apply_owned_skill_attack_to_player_kind<Runtime: GameMainLoopRuntime>(
        &mut self,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        target_id: i32,
        region_id: i32,
        mut attack: AttackInformation,
        war_soul_hit: bool,
        runtime: &mut Runtime,
    ) {
        if !self.owned_skill_player_attackable(master, target_id, region_id) {
            return;
        }
        let Some((mut attacker_properties, attacker_occupation, target_properties, target_health, target_mana, target_war_soul_mana)) =
            self.find_player(master.master_id).and_then(|attacker| {
                let target = self.find_player(target_id)?;
                Some((
                    attacker.combat_properties(),
                    attacker.occupation(),
                    target.combat_properties(),
                    target.health(),
                    target.mana(),
                    target.war_soul_mana(&self.goods_factory),
                ))
            })
        else {
            return;
        };
        let mut restored_war_soul_scales = None;
        if !war_soul_hit
            && let Some((properties, _, restored)) =
                self.war_soul_defense_projection(master.master_id, attack.skill_id)
        {
            attacker_properties = properties;
            restored_war_soul_scales = Some(restored);
        }
        if !war_soul_hit {
            let _ = self.player_on_owned_skill_attack(
                master.master_id,
                target_id,
                region_id,
                runtime,
            );
        }
        if war_soul_hit {
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
        } else {
            let pillar_damage_factor = self.find_player(target_id)
                .and_then(CPlayer::pillar_state).map(|state| state.damage_factor());
            let mut defense_shields = self
                .find_player_mut(target_id)
                .map(CPlayer::take_defense_shields)
                .unwrap_or_default();
            let mut random = |maximum| game_legacy_random(&mut self.random_state, maximum);
            defend_player_base_attack(
                &mut attack,
                attacker_properties,
                attacker_occupation,
                target_properties,
                target_mana,
                target_war_soul_mana,
                &self.globe_setup,
                &mut random,
                defense_shields.as_mut_slice(),
                pillar_damage_factor,
            );
            if let Some(target) = self.find_player_mut(target_id) {
                target.restore_defense_shields(defense_shields);
            }
            self.restore_war_soul_defense_projection(
                master.master_id,
                restored_war_soul_scales,
            );
        }
        let (damage, mana_damage) =
            Self::applied_attack_damage(&attack, target_health, target_mana);
        if damage == 0 && mana_damage == 0 {
            if attack.full_miss != 0 {
                let mut missed = CMessage::new(0x000b_f612);
                missed.add_byte(attack.full_miss);
                missed.add_long(PLAYER_TYPE);
                missed.add_long(target_id);
                let _ = self.send_player_shape_around(target_id, None, &missed);
            }
            self.increase_owned_skill_attacker_rp(master.master_id, attack.skill_id);
            return;
        }
        let current_health = target_health - damage;
        if let Some(target) = self.find_player_mut(target_id) {
            target.set_health(current_health);
            target.set_mana(target_mana - mana_damage);
            if current_health != 0 && attack.full_miss == 0 {
                target.movement_shape_mut().set_action(5);
            }
        }
        if damage != 0 {
            self.increase_owned_player_rp(target_id, false, damage as u16);
        }
        if current_health != 0 && attack.full_miss == 0 {
            let _ = self.queue_player_hurt_ai(target_id, damage, runtime);
            let _ = self.retarget_passive_pets_after_player_hurt(
                target_id,
                ShapeIdentity {
                    object_type: master.master_type,
                    id: master.master_id,
                    ex_id: CGuid::GUID_INVALID,
                },
            );
            let _ = self.notify_country_after_player_hurt(
                target_id,
                ShapeIdentity {
                    object_type: master.master_type,
                    id: master.master_id,
                    ex_id: CGuid::GUID_INVALID,
                },
                runtime,
            );
            let _ = super::finish_player_blind_states_on_defense(self, target_id, 0);
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
            died.add_long(master.master_type);
            died.add_long(master.master_id);
            died.add_long(PLAYER_TYPE);
            died.add_long(target_id);
            died.add_ulong(damage);
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
            let mut hurt = CMessage::new(0x000b_f60a);
            hurt.add_long(master.master_type);
            hurt.add_long(master.master_id);
            hurt.add_long(PLAYER_TYPE);
            hurt.add_long(target_id);
            Self::append_hurt_damage_records(&mut hurt, damage, mana_damage);
            hurt.add_ulong(current_health);
            Self::append_base_attack_tail(&mut hurt, &attack);
            let _ = self.send_player_shape_around(target_id, None, &hurt);
            self.damage_player_armor(target_id);
        }
        self.increase_owned_skill_attacker_rp(master.master_id, attack.skill_id);
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

    pub(crate) fn apply_owned_skill_attack_to_monster<Runtime: GameMainLoopRuntime>(
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
        let Some((target_properties, target_health, tamed, carriage, god, target_master, x, y)) =
            self.find_region(region_id).and_then(|owner| {
                let monster = owner.base().find_monster_by_id(target_id)?;
                let shape = monster.move_shape().shape();
                Some((
                    monster.combat_properties(&property),
                    monster.hit_points(),
                    monster.is_tamed(),
                    monster.is_carriage(&property),
                    monster.move_shape().is_god(),
                    monster.master_info(),
                    shape.get_tile_x().ok()?,
                    shape.get_tile_y().ok()?,
                ))
            })
        else {
            return;
        };
        if target_health == 0
            || god
            || !self.monster_attackable_by_player(master.master_id, region_id, &property)
            || ((tamed || carriage)
                && !self.owned_skill_monster_attackable(master, target_master))
        {
            return;
        }
        if (tamed || carriage) && target_master.master_type == PLAYER_TYPE {
            let _ = self.player_on_first_skill(
                master.master_id,
                target_master.master_id,
                Some(region_id),
                runtime,
            );
        }
        let Some((attacker_properties, attacker_occupation, attacker_level)) = self
            .find_player(master.master_id)
            .map(|attacker| {
                (
                    attacker.combat_properties(),
                    attacker.occupation(),
                    attacker.level(),
                )
            })
        else {
            return;
        };
        let (attacker_properties, attacker_level, restored_war_soul_scales) =
            if let Some((properties, level, restored)) =
                self.war_soul_defense_projection(master.master_id, attack.skill_id)
            {
                (properties, level, Some(restored))
            } else {
                (attacker_properties, attacker_level, None)
            };
        let now_ms = runtime.now_milliseconds();
        self.apply_guard_monster_first_attack(master.master_id, region_id, &property, now_ms);
        let mut random = |maximum| game_legacy_random(&mut self.random_state, maximum);
        defend_monster_base_attack(
            &mut attack,
            attacker_properties,
            attacker_occupation,
            attacker_level,
            target_properties,
            &self.globe_setup,
            &mut random,
        );
        self.restore_war_soul_defense_projection(master.master_id, restored_war_soul_scales);
        let damage = attack.hp_damage().min(target_health);
        let current_health = target_health - damage;
        let lord_hurt_plan = (property.ai == 19
            && attack.full_miss == 0
            && damage != 0
            && current_health != 0)
            .then(|| {
                crate::gameserver::appserver::ai::lord::plan_lord_hurt_response(
                    self,
                    region_id,
                    target_id,
                    &property,
                )
            });
        let stiffen_setup = self.globe_setup.stiffen_setup();
        let mut stiffen_delay = 0;
        if let Some(mut owner) = self.take_region_owner(region_id) {
            if let Some(monster) = owner.base_mut().find_monster_by_id_mut(target_id) {
                if attack.full_miss == 0 && damage != 0 && current_health != 0 {
                    stiffen_delay = monster.roll_stiffen(
                        damage,
                        &property,
                        stiffen_setup,
                        || runtime.now_milliseconds(),
                        |maximum| game_legacy_random(&mut self.random_state, maximum),
                    );
                }
                monster.set_hit_points(current_health);
                if damage != 0 && attack.full_miss == 0 && current_health != 0 {
                    monster.move_shape_mut().shape_mut().set_action(5);
                    if property.ai == 1 {
                        monster.when_passive_gladiator_hurted_by(
                            ShapeIdentity {
                                object_type: master.master_type,
                                id: master.master_id,
                                ex_id: CGuid::GUID_INVALID,
                            },
                            now_ms,
                            false,
                        );
                    } else if property.ai == 2 {
                        // Владелец AI2 применит реакцию после освобождения
                        // изменяемого заимствования монстра.
                    } else if property.ai == 13 {
                        // Поиск AI13 выполняется после освобождения изменяемого
                        // заимствования монстра.
                    } else if property.ai == 11 {
                        // Поиск AI11 выполняется после освобождения изменяемого
                        // заимствования монстра.
                    } else if property.ai == 20 {
                        // AI20 разрешает владельца периодического эффекта и
                        // связывает близнеца после освобождения заимствования.
                    } else if property.ai == 19 {
                        // AI19 применяет Defense, spatial-step и выбор цели
                        // после освобождения заимствования монстра.
                    } else if matches!(property.ai, 8 | 17 | 100 | 101) {
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
            if attack.full_miss == 0
                && damage != 0
                && current_health != 0
                && property.ai == 2
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
            if attack.full_miss == 0
                && damage != 0
                && current_health != 0
                && property.ai == 11
            {
                crate::gameserver::appserver::ai::cityguardwithbow::retarget_city_bow_guard_after_hurt(
                    self,
                    owner.base_mut(),
                    target_id,
                    &property,
                    now_ms,
                );
            }
            if attack.full_miss == 0
                && damage != 0
                && current_health != 0
                && property.ai == 13
            {
                crate::gameserver::appserver::ai::vilcouguardwithbow::retarget_village_bow_guard_after_hurt(
                    self,
                    owner.base_mut(),
                    target_id,
                    &property,
                    now_ms,
                );
            }
            if attack.full_miss == 0
                && damage != 0
                && current_health != 0
                && property.ai == 20
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
            if attack.full_miss == 0
                && damage != 0
                && current_health != 0
                && matches!(property.ai, 8 | 17 | 100 | 101)
            {
                crate::gameserver::appserver::ai::guardcountry::retarget_special_guard_after_hurt(
                    self,
                    owner.base_mut(),
                    target_id,
                    &property,
                );
            }
            if stiffen_delay != 0
                && let Some(monster) = owner.base_mut().find_monster_by_id_mut(target_id)
            {
                monster.when_been_stiffened(stiffen_delay, runtime.now_milliseconds());
            }
            if attack.full_miss == 0 && damage != 0 && current_health != 0 {
                let mut published_owner = Some(owner);
                let _ = self.with_published_region(&mut published_owner, |game| {
                    super::finish_blind_states_on_defense(
                        game,
                        region_id,
                        ShapeIdentity {
                            object_type: MONSTER_TYPE,
                            id: target_id,
                            ex_id: CGuid::GUID_INVALID,
                        },
                        now_ms,
                    )
                });
                let Some(restored_owner) = published_owner else { return };
                owner = restored_owner;
                if owner.base().find_monster_by_id(target_id).is_none() {
                    self.restore_region_owner(owner);
                    return;
                }
            }
            self.restore_region_owner(owner);
        }
        if attack.full_miss != 0 && current_health != 0 {
            let mut missed = CMessage::new(0x000b_f612);
            missed.add_byte(attack.full_miss);
            missed.add_long(MONSTER_TYPE);
            missed.add_long(target_id);
            let _ = self.send_shape_position_around(region_id, x, y, &missed);
            self.increase_owned_skill_attacker_rp(master.master_id, attack.skill_id);
            return;
        }
        if damage == 0 {
            self.increase_owned_skill_attacker_rp(master.master_id, attack.skill_id);
            return;
        }
        if current_health != 0 {
            let mut hurt = CMessage::new(0x000b_f60a);
            hurt.add_long(master.master_type);
            hurt.add_long(master.master_id);
            hurt.add_long(MONSTER_TYPE);
            hurt.add_long(target_id);
            hurt.add_byte(1);
            hurt.add_byte(0);
            hurt.add_ulong(damage);
            hurt.add_ulong(current_health);
            Self::append_base_attack_tail(&mut hurt, &attack);
            let _ = self.send_shape_position_around(region_id, x, y, &hurt);
            let _ = self.monster_on_been_hurted(
                region_id,
                target_id,
                master.master_type,
                master.master_id,
                runtime,
            );
            self.increase_owned_skill_attacker_rp(master.master_id, attack.skill_id);
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
        died.add_long(master.master_type);
        died.add_long(master.master_id);
        died.add_long(MONSTER_TYPE);
        died.add_long(target_id);
        died.add_ulong(damage);
        died.base_mut().add_char(1);
        Self::append_base_attack_tail(&mut died, &attack);
        let _ = self.send_move_shape_around(region_id, victim, &died);
        self.record_move_shape_death(region_id, victim, attacker, runtime);
        self.increase_owned_skill_attacker_rp(master.master_id, attack.skill_id);
    }

}
