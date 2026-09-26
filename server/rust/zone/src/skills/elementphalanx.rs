//! Снимок и числовой расчёт элементального удара призванных областей.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! EXE SHA-256 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E,
//! PDB SHA-256 B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016.
//! appserver/skills/firewallphalanx.cpp/.h,
//! yinyangphalanx{,2}.cpp/.h, godthunderphalanx{,2}.cpp/.h
//! и chaosspherephalanx.cpp/.h.
//! CalculateAttackPower VA 0x00600120, 0x005FE6E0, 0x005F2600,
//! 0x005F5E90, 0x005EF3F0 и 0x005FEFB0 соответственно.
//! Тела `apply_element_phalanx_attack`/`apply_element_phalanx_war_soul`
//! перенесены из старого `appserver/skills/elementphalanxattack.rs` буквально:
//! IsDied→PK seed→Calculate→receipt без RP;
//! Player по attacker ID и оружейный множитель разрешаются при каждом
//! попадании живыми швами `ZonalCastGame`/`ZonalCastContact`
//! (`skills/zonalcast.rs`). Поиск Player не зависит от master type; его
//! отсутствие сохраняет пустую исходную атаку, не отменяя raw receipt.
//! GodThunder2 и ChaosSphere сначала проверяют глобальных Player
//! цели/источника и передают признак попадания по боевому духу; body-допуск
//! хранится в обходе региона.

use crate::combat::{AttackInformation, AttackPower, AttackPowerType, MasterInfo, truncate_original};
use crate::regions::ShapeIdentity;
use super::zonalcast::{ZonalCastContact, ZonalCastGame};

/// Живые свойства источника, которые читает призыв элементальной области.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementSummonLiveField { CriticalChance, AddElementAttack }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ElementPhalanxAttack {
    pub master: MasterInfo,
    pub skill_id: u32,
    pub skill_level: i32,
    pub minimum: i32,
    pub maximum: i32,
    pub element: i32,
    pub critical_chance: i32,
}

impl ElementPhalanxAttack {
    pub fn attack_master(self) -> MasterInfo {
        if self.master.master_type == 400 { return self.master; }
        MasterInfo {
            master_type: self.master.master_type, master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }

    /// Вызывается после разрешения Player по attacker ID и до поиска уровня цели.
    /// Отсутствующий Player оставляет исходную пустую AttackInformation.
    pub fn begin_calculation(self, attack: &mut AttackInformation) {
        attack.skill_id = self.skill_id;
        attack.skill_level = self.skill_level as u8;
        attack.damage_modifier = 0;
    }

    /// Вызывается после разрешения уровня цели и живого оружейного множителя.
    /// Возвращает необходимость позднего чтения критического множителя.
    pub fn roll_damage(
        self, attack: &mut AttackInformation, weapon_modifier: f32,
        mut random_below: impl FnMut(i32) -> i32,
    ) -> bool {
        attack.damage_factor = weapon_modifier;
        attack.hit_modifier = 100;
        let width = self.maximum.wrapping_sub(self.minimum).wrapping_abs().wrapping_add(1);
        let damage = random_below(width).wrapping_add(self.minimum)
            .wrapping_add(self.element).max(0);
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0,
        });
        if random_below(100) < self.critical_chance {
            attack.critical = true;
            return true;
        }
        false
    }

    pub fn scale_critical(attack: &mut AttackInformation, critical_rate: f32) {
        for power in &mut attack.damages {
            if matches!(power.kind, AttackPowerType::Physical | AttackPowerType::Element | AttackPowerType::Soul) {
                power.hp_damage = truncate_original(
                    f64::from(power.hp_damage) * f64::from(critical_rate),
                );
            }
        }
    }
}

/// Живой расчёт одного попадания: отсутствие Player оставляет исходную
/// пустую атаку; уровень цели и оружейный множитель читаются у владельца.
fn calculate_element_phalanx<Game: ZonalCastGame>(
    game: &mut Game,
    snapshot: ElementPhalanxAttack,
    target: (i32, ShapeIdentity),
    attack: &mut AttackInformation,
) {
    if game.find_player(attack.attacker_id).is_none() { return; }
    snapshot.begin_calculation(attack);
    let Some(level) = game.move_shape_level(target.0, target.1) else { return; };
    let Some(weapon_modifier) = game.element_phalanx_weapon_modifier(attack.attacker_id, i32::from(level)) else { return; };
    if snapshot.roll_damage(attack, weapon_modifier, |width| game.skill_random_below(width)) {
        ElementPhalanxAttack::scale_critical(attack, game.element_phalanx_critical_rate());
    }
}

/// Применение попадания к живой цели: IsDied→мастер снимка→Calculate→
/// общий контакт или попадание по боевому духу.
pub fn apply_element_phalanx_attack<Game, Runtime>(
    game: &mut Game,
    snapshot: ElementPhalanxAttack,
    target: (i32, ShapeIdentity),
    war_soul: bool,
    runtime: &mut Runtime,
)
where
    Game: ZonalCastContact<Runtime>,
{
    if game.move_shape_health(target.0, target.1).is_none_or(|hp| hp == 0) { return; }
    let master = snapshot.attack_master();
    let mut attack = AttackInformation::for_master(master);
    calculate_element_phalanx(game, snapshot, target, &mut attack);
    if war_soul {
        game.apply_owned_skill_attack_to_war_soul(master, target.1.id, target.0, attack, runtime);
    } else {
        game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    }
}

/// Отдельный проход боевых духов (GodThunder2 и ChaosSphere): PK-допуск
/// глобальных Player цели и источника у владельца, затем тот же контакт.
pub fn apply_element_phalanx_war_soul<Game, Runtime>(
    game: &mut Game,
    snapshot: ElementPhalanxAttack,
    target_id: i32,
    runtime: &mut Runtime,
)
where
    Game: ZonalCastContact<Runtime>,
{
    let Some(target) = game.element_phalanx_war_soul_target(snapshot.master, target_id) else { return; };
    apply_element_phalanx_attack(game, snapshot, target, true, runtime);
}
