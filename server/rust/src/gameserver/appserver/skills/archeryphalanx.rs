//! Прицельный региональный снаряд базовой стрельбы Archery.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/archeryphalanx.cpp.
//! Снимок конструктора, физический roll и общий серверный decoder перенесены
//! в `nebokrai_zone::skills::projectile` (основание и статусы см. там);
//! обёртка сохраняет прежние имена и интерфейс для владельцев game/. Здесь
//! остаются CGame-разрешение живых полей: поиск игрока по attacker ID
//! независимо от сохранённого типа, живая таблица навыка, уровень цели,
//! живой weapon modifier, source_property и порог смерти, затем доставка
//! сырым OnBeenAttacked без допуска, DaubPoison и RP. Отсутствие игрока или
//! таблицы оставляет исходную пустую атаку, не отменяя контакт. После
//! задержки цель заново ищется в фактическом регионе формы по type/id с
//! GUID_INVALID; End только отмечает удаление, после контакта, без
//! немедленного сообщения выхода. Выделение CScope конструктора и
//! неиспользуемые MIN/MAX/ELEMENT не дублируются: они не участвуют ни в
//! выборе цели, ни в расчёте.

use super::archery::ARCHERY_SKILL_ID;
use super::baseprojectilephalanx::BaseProjectileFlight;
use super::weaponattack::{SourceProperty, source_property};
use nebokrai_zone::skills::{
    ARCHERY_HIT_MODIFIER_PROPERTY, ArcheryProjectileAttack as ArcheryProjectileRule,
    ArcheryProjectileLiveField,
};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArcheryAttack {
    rule: ArcheryProjectileRule,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CArcheryPhalanx {
    flight: BaseProjectileFlight,
    attack: ArcheryAttack,
}

impl CArcheryPhalanx {
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, attack_delay_ms: u32, target: ShapeIdentity,
    ) -> Self {
        Self {
            flight: BaseProjectileFlight::new(id, started_at_ms, lifetime_ms, attack_delay_ms, target),
            attack: ArcheryAttack {
                rule: ArcheryProjectileRule {
                    master, skill_id: ARCHERY_SKILL_ID, skill_level,
                },
            },
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { self.flight.shape() }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { self.flight.shape_mut() }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.rule.master }
    pub(crate) const fn flight(&self) -> &BaseProjectileFlight { &self.flight }
    pub(crate) const fn flight_mut(&mut self) -> &mut BaseProjectileFlight { &mut self.flight }
    pub(crate) const fn attack_snapshot(&self) -> ArcheryAttack { self.attack }

    pub(crate) fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        self.flight.encode_client_snapshot(
            ARCHERY_SKILL_ID, self.attack.rule.skill_level, self.attack.rule.master,
            now_milliseconds,
        )
    }
}

impl ArcheryAttack {
    fn attack_master(self) -> MasterInfo { self.rule.attack_master() }
}

fn calculate_archery_attack(
    game: &mut CGame, snapshot: ArcheryAttack, target: (i32, ShapeIdentity),
    attack: &mut AttackInformation,
) {
    let Some(player) = game.find_player(attack.attacker_id) else { return; };
    let Some(properties) = game.skill_base_properties(ARCHERY_SKILL_ID, snapshot.rule.skill_level)
    else { return; };
    let source = (player.shape().get_region_id(), player.shape().identity());
    snapshot.rule.begin_calculation(attack);
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
    let weapon_modifier = player.weapon_modifier(
        game.goods_factory(), i32::from(target_level), divisor, minimum_factor,
    );
    let hit_modifier = properties.query_property(ARCHERY_HIT_MODIFIER_PROPERTY);
    let critical_rate = game.globe_setup().critical_rate();
    snapshot.rule.roll_damage(
        attack, weapon_modifier, critical_rate,
        move |_property| hit_modifier,
        |field| match field {
            ArcheryProjectileLiveField::RandomBelow(bound) => Some(game.skill_random_below(bound)),
            ArcheryProjectileLiveField::MinimumAttack => {
                source_property(game, source, SourceProperty::Minimum).map(|value| value as i32)
            }
            ArcheryProjectileLiveField::MaximumAttack => {
                source_property(game, source, SourceProperty::Maximum).map(|value| value as i32)
            }
            ArcheryProjectileLiveField::AddElementAttack => {
                source_property(game, source, SourceProperty::Element).map(|value| value as i32)
            }
            ArcheryProjectileLiveField::AddSoulAttack => {
                source_property(game, source, SourceProperty::Soul)
                    .map(|value| i32::from(value as u16))
            }
            ArcheryProjectileLiveField::CriticalChance => {
                source_property(game, source, SourceProperty::CriticalChance)
                    .map(|value| i32::from(value as u16))
            }
        },
    );
}

pub(crate) fn apply_archery_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, snapshot: ArcheryAttack, target: (i32, ShapeIdentity),
    runtime: &mut Runtime,
) {
    if game.move_shape_health(target.0, target.1).is_none_or(|hp| hp == 0) { return; }
    let master = snapshot.attack_master();
    let mut attack = AttackInformation::for_master(master);
    calculate_archery_attack(game, snapshot, target, &mut attack);
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}

// Неподключённый общий серверный decoder (pub `1:001ea070`, RVA `0x1EB070`;
// линкером слит с CBaseMagicPhalanx и CBFBaseAttackPhalanx) перенесён в
// `nebokrai_zone::skills::BaseProjectileFlight::decode_server_snapshot`;
// основание и статусы см. там.
