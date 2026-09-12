//! Прицельный региональный снаряд базовой магии BaseMagic.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/basemagicphalanx.cpp.
//! Полёт, два абсолютных unsigned срока, поиск цели и тихий End общие с
//! Archery. После задержки Attack проверяет смерть цели, фиксирует PK и
//! доставляет сырой OnBeenAttacked без допуска, DaubPoison и RP.
//! Calculate ищет игрока по attacker ID независимо от сохранённого типа.
//! Отсутствующий игрок оставляет исходную пустую атаку, не отменяя контакт.
//! Таблица навыка повторно не запрашивается: MIN/MAX/ELEMENT сохранены
//! конструктором. Живой element_modify читается до уровня цели и weapon
//! modifier; RNG получает abs(MAX-MIN)+1, затем читается живой AddElement.
//! Единственный компонент Element использует signed wrapping и нижнюю
//! границу ноль. CCH читается после компонента, критический множитель
//! усекается в общем оружейном хвосте без промежуточного float.
//! Клиентский снимок содержит master type/id, не цель. Общий серверный
//! decoder не имеет достигнутого caller-а; его неизвестность сохранена
//! у archeryphalanx, откуда происходит та же линкерная реализация.

use super::basemagic::BASE_MAGIC_SKILL_ID;
use super::baseprojectilephalanx::BaseProjectileFlight;
use super::weaponattack::{SourceProperty, apply_weapon_critical, source_property};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BaseMagicAttack {
    master: MasterInfo,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_modifier: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CBaseMagicPhalanx {
    flight: BaseProjectileFlight,
    attack: BaseMagicAttack,
}

impl CBaseMagicPhalanx {
    #[allow(clippy::too_many_arguments, reason = "снимок конструктора BaseMagicPhalanx")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32,
        element_modifier: i32, attack_delay_ms: u32, target: ShapeIdentity,
    ) -> Self {
        Self {
            flight: BaseProjectileFlight::new(id, started_at_ms, lifetime_ms, attack_delay_ms, target),
            attack: BaseMagicAttack {
                master, skill_level, minimum_attack, maximum_attack, element_modifier,
            },
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { self.flight.shape() }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { self.flight.shape_mut() }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master }
    pub(crate) const fn flight(&self) -> &BaseProjectileFlight { &self.flight }
    pub(crate) const fn flight_mut(&mut self) -> &mut BaseProjectileFlight { &mut self.flight }
    pub(crate) const fn attack_snapshot(&self) -> BaseMagicAttack { self.attack }

    pub(crate) fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        self.flight.encode_client_snapshot(
            BASE_MAGIC_SKILL_ID, self.attack.skill_level, self.attack.master, now_milliseconds,
        )
    }
}

impl BaseMagicAttack {
    fn attack_master(self) -> MasterInfo {
        if self.master.master_type == 400 { return self.master; }
        MasterInfo {
            master_type: self.master.master_type, master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }
}

fn calculate_base_magic_attack(
    game: &mut CGame, snapshot: BaseMagicAttack, target: (i32, ShapeIdentity),
    attack: &mut AttackInformation,
) {
    let Some(player) = game.find_player(attack.attacker_id) else { return; };
    let element_modify = player.combat_properties().element_modify;
    let source = (player.shape().get_region_id(), player.shape().identity());
    attack.skill_id = BASE_MAGIC_SKILL_ID;
    attack.skill_level = snapshot.skill_level as u8;
    attack.damage_modifier = 0;
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
    attack.damage_factor = player.weapon_modifier(
        game.goods_factory(), i32::from(target_level), divisor, minimum_factor,
    );
    attack.hit_modifier = 100;

    let element = snapshot.element_modifier.wrapping_mul(element_modify).wrapping_div(100);
    let width = snapshot.maximum_attack.wrapping_sub(snapshot.minimum_attack)
        .wrapping_abs().wrapping_add(1);
    let rolled = game.skill_random_below(width).wrapping_add(snapshot.minimum_attack);
    let Some(addition) = source_property(game, source, SourceProperty::Element) else { return; };
    let damage = element.wrapping_add((addition as i32).wrapping_add(rolled)).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 });
    let Some(chance) = source_property(game, source, SourceProperty::CriticalChance) else { return; };
    apply_weapon_critical(game, i32::from(chance as u16), attack);
}

pub(crate) fn apply_base_magic_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, snapshot: BaseMagicAttack, target: (i32, ShapeIdentity),
    runtime: &mut Runtime,
) {
    if game.move_shape_health(target.0, target.1).is_none_or(|hp| hp == 0) { return; }
    let master = snapshot.attack_master();
    let mut attack = AttackInformation::for_master(master);
    calculate_base_magic_attack(game, snapshot, target, &mut attack);
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}
