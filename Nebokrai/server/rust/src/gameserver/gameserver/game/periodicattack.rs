//! Общая координация заимствований для уже рассчитанных атак навыков.
//!
//! Конкретные формулы, периодический шаг состояний и визуальное сетевое
//! представление принадлежат модулям навыка и состояния. Этот дочерний модуль
//! временно извлекает игрока, монстра и регион, проводит обычный
//! `OnBeenAttacked` через общую защиту и
//! выполняет обработку смерти и сетевые побочные эффекты, требующие нескольких
//! независимых владельцев `CGame`.

use super::*;

impl CGame {
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
        let murderer_delivery = if disposition == KillPkDisposition::ReportMurderer {
            let pk_count_per_kill = self.globe_setup.pk_count_per_kill();
            let (pk_count, kill_count) = {
                let attacker = self.find_player_mut(attacker_id)?;
                let pk_count = attacker.report_murderer(pk_count_per_kill, || {
                    runtime.now_milliseconds()
                });
                (pk_count, attacker.kill_count())
            };
            let mut message = CMessage::new(0x000b_f70e);
            message.add_long(attacker_id);
            message.base_mut().add_short(pk_count as i16);
            message.add_ulong(kill_count);
            self.send_player_shape_around(attacker_id, None, &message)
        } else {
            None
        };
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
            ?murderer_delivery,
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
        let Some((attacker_properties, attacker_occupation, target_properties, target_health, target_mana, target_war_soul_mana)) =
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
                &mut defense_shields,
                pillar_damage_factor,
            );
            if let Some(target) = self.find_player_mut(target_id) {
                target.restore_defense_shields(defense_shields);
            }
        }
        let (damage, mana_damage) =
            Self::applied_attack_damage(&attack, target_health, target_mana);
        if attack.full_miss != 0 {
            let mut missed = CMessage::new(0x000b_f612);
            missed.add_byte(attack.full_miss);
            missed.add_long(PLAYER_TYPE);
            missed.add_long(target_id);
            let _ = self.send_player_shape_around(target_id, None, &missed);
            return;
        }
        if damage == 0 && mana_damage == 0 {
            return;
        }
        let current_health = target_health - damage;
        if let Some(target) = self.find_player_mut(target_id) {
            target.set_health(current_health);
            target.set_mana(target_mana - mana_damage);
            target
                .movement_shape_mut()
                .set_action(if current_health == 0 { 6 } else { 5 });
        }
        if current_health != 0 {
            let _ = super::finish_player_blind_states_on_defense(self, target_id, 0);
        }
        if current_health == 0 {
            let mut died = CMessage::new(0x000b_f60b);
            died.add_long(master.master_type);
            died.add_long(master.master_id);
            died.add_long(PLAYER_TYPE);
            died.add_long(target_id);
            died.add_ulong(damage);
            died.base_mut().add_char(1);
            Self::append_base_attack_tail(&mut died, &attack);
            let _ = self.send_player_shape_around(target_id, None, &died);
            let _ = self.player_on_death(
                PlayerKillingBlow {
                    victim_id: target_id,
                    attacker_type: master.master_type,
                    attacker_id: master.master_id,
                    attacker_faction_id: master.master_guild_id,
                },
                runtime,
            );
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
            self.damage_player_armor(target_id, runtime);
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
        let Some(mut owner) = self.take_region_owner(region_id) else { return };
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
        let mut deaths = Vec::new();
        let now_ms = runtime.now_milliseconds();
        crate::gameserver::appserver::skills::monsterattack::apply_owned_monster_attack_hit(
            self,
            owner.base_mut(),
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
            &mut deaths,
        );
        self.restore_region_owner(owner);
        self.apply_monster_attack_deaths(region_id, deaths, runtime);
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
        let damage = attack.hp_damage().min(target_health);
        let current_health = target_health - damage;
        let lord_hurt_plan = (property.ai == 100
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
        if let Some(mut owner) = self.take_region_owner(region_id) {
            if let Some(monster) = owner.base_mut().find_monster_by_id_mut(target_id) {
                monster.set_hit_points(current_health);
                if attack.full_miss == 0 && damage != 0 {
                    monster
                        .move_shape_mut()
                        .shape_mut()
                        .set_action(if current_health == 0 { 6 } else { 5 });
                    if current_health == 0 {
                        monster.when_been_killed(now_ms);
                    } else if property.ai == 1 {
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
                    } else if property.ai == 16 {
                        // Поиск AI16 выполняется после освобождения изменяемого
                        // заимствования монстра.
                    } else if property.ai == 11 {
                        // Поиск AI11 выполняется после освобождения изменяемого
                        // заимствования монстра.
                    } else if property.ai == 0x65 {
                        // AI101 разрешает владельца периодического эффекта и
                        // связывает близнеца после освобождения заимствования.
                    } else if property.ai == 100 {
                        // AI100 применяет Defense, spatial-step и выбор цели
                        // после освобождения заимствования монстра.
                    } else if matches!(property.ai, 8 | 13 | 14 | 20) {
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
                    monster.register_attacking_player(
                        master.master_id,
                        now_ms,
                        self.globe_setup.attack_monster_protection_ms(),
                    );
                }
                if current_health == 0 {
                    monster.set_killed_by(MonsterKillingAttack {
                        attacker_type: master.master_type,
                        attacker_id: master.master_id,
                        skill_id: attack.skill_id,
                        skill_level: attack.skill_level,
                        critical: attack.critical,
                        blast_attack: attack.blast_attack,
                    });
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
                    now_ms,
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
                && property.ai == 16
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
                && property.ai == 0x65
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
                    now_ms,
                );
            }
            if let Some(plan) = lord_hurt_plan {
                let _ = crate::gameserver::appserver::ai::lord::apply_lord_hurt_response(
                    self,
                    owner.base_mut(),
                    target_id,
                    &property,
                    ShapeIdentity {
                        object_type: master.master_type,
                        id: master.master_id,
                        ex_id: CGuid::GUID_INVALID,
                    },
                    now_ms,
                    plan,
                );
            }
            if attack.full_miss == 0
                && damage != 0
                && current_health != 0
                && matches!(property.ai, 8 | 13 | 14 | 20)
            {
                crate::gameserver::appserver::ai::guardcountry::retarget_special_guard_after_hurt(
                    self,
                    owner.base_mut(),
                    target_id,
                    &property,
                );
            }
            if attack.full_miss == 0 && damage != 0 && current_health != 0 {
                let _ = super::finish_blind_states_on_defense(
                    self,
                    owner.base_mut(),
                    ShapeIdentity {
                        object_type: MONSTER_TYPE,
                        id: target_id,
                        ex_id: CGuid::GUID_INVALID,
                    },
                    now_ms,
                );
            }
            self.restore_region_owner(owner);
        }
        if attack.full_miss != 0 {
            let mut missed = CMessage::new(0x000b_f612);
            missed.add_byte(attack.full_miss);
            missed.add_long(MONSTER_TYPE);
            missed.add_long(target_id);
            let _ = self.send_shape_position_around(region_id, x, y, &missed);
            return;
        }
        if damage == 0 {
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
            );
            return;
        }

        let mut died = CMessage::new(0x000b_f60b);
        died.add_long(master.master_type);
        died.add_long(master.master_id);
        died.add_long(MONSTER_TYPE);
        died.add_long(target_id);
        died.add_ulong(damage);
        died.base_mut().add_char(1);
        Self::append_base_attack_tail(&mut died, &attack);
        let _ = self.send_shape_position_around(region_id, x, y, &died);
    }

}
