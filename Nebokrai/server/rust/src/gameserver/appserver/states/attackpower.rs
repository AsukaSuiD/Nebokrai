//! Типизированное описание рассчитанной атаки GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/states/attackpower.cpp` и ApplyFinalDamage из appserver/moveshape.cpp.
//! Сохраняются порядок составляющих урона,
//! идентификаторы навыка и атакующего, сведения PK и признаки завершающей
//! защиты. `Vec` заменяет исходный вектор указателей без изменения числовой и
//! сетевой семантики полей. Значение формируется на стадии `Calculate`, а
//! применяет его конкретный владелец стадии `Attack`.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttackPowerType {
    Physical,
    Element,
    Soul,
    Poison,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AttackPower {
    pub(crate) kind: AttackPowerType,
    pub(crate) hp_damage: i32,
    pub(crate) mp_damage: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AttackInformation {
    pub(crate) skill_id: u32,
    pub(crate) skill_level: u8,
    pub(crate) attacker_type: i32,
    pub(crate) attacker_id: i32,
    pub(crate) attacker_team_id: i32,
    pub(crate) attacker_faction_id: i32,
    pub(crate) attacker_union_id: i32,
    pub(crate) hit_modifier: i32,
    pub(crate) damage_factor: f32,
    pub(crate) damage_modifier: i32,
    pub(crate) critical: bool,
    pub(crate) blast_attack: bool,
    pub(crate) full_miss: u8,
    pub(crate) damages: Vec<AttackPower>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FinalAttackDamage {
    pub(crate) health: u32,
    pub(crate) mana: Option<u32>,
    pub(crate) hp_record: u32,
    pub(crate) mp_record: u32,
}

impl AttackInformation {
    /// CMoveShape::ApplyFinalDamage: компоненты читаются как DWORD по порядку,
    /// затем применяется modifier. SetHP игрока ограничивает каждый результат
    /// текущим максимумом; у остальных владельцев передаётся u32::MAX.
    /// Записи damage используют положительную знаковую разницу, поэтому
    /// отсутствие записи само по себе не означает отсутствие изменения HP.
    pub(crate) fn final_damage_values(
        &self, health: u32, maximum_health: u32, mana: Option<(u32, u32)>,
    ) -> FinalAttackDamage {
        let mut remaining_health = health;
        let mut remaining_mana = mana.map(|(current, _)| current);
        for power in &self.damages {
            remaining_health = remaining_health.saturating_sub(power.hp_damage as u32).min(maximum_health);
            if let (Some(current), Some((_, maximum))) = (&mut remaining_mana, mana) {
                *current = current.saturating_sub(power.mp_damage as u32).min(maximum);
            }
        }
        if self.damage_modifier != 0 {
            remaining_health = remaining_health.saturating_sub(self.damage_modifier as u32).min(maximum_health);
        }
        FinalAttackDamage {
            health: remaining_health,
            mana: remaining_mana,
            hp_record: (health.wrapping_sub(remaining_health) as i32).max(0) as u32,
            mp_record: mana.zip(remaining_mana).map_or(0, |((old, _), current)| {
                (old.wrapping_sub(current) as i32).max(0) as u32
            }),
        }
    }

    /// Clear защиты сбрасывает всю атаку, кроме уровня навыка, а не только
    /// сумму урона. Пустой результат остаётся обычным попаданием без full-miss.
    pub(crate) fn clear(&mut self) {
        self.skill_id = super::super::skills::skillfactory::UNKNOWN_SKILL_ID;
        self.attacker_type = 0;
        self.attacker_id = 0;
        self.attacker_team_id = 0;
        self.attacker_faction_id = 0;
        self.attacker_union_id = 0;
        self.hit_modifier = 0;
        self.damage_factor = 1.0;
        self.critical = false;
        self.blast_attack = false;
        self.full_miss = 0;
        self.clear_damage();
    }

    pub(crate) fn clear_damage(&mut self) {
        self.damages.clear();
        self.damage_modifier = 0;
    }

    pub(crate) fn hp_damage(&self) -> u32 {
        self.damages
            .iter()
            .map(|power| power.hp_damage.max(0) as u32)
            .fold(self.damage_modifier.max(0) as u32, u32::saturating_add)
    }

    pub(crate) fn mp_damage(&self) -> u32 {
        self.damages
            .iter()
            .map(|power| power.mp_damage.max(0) as u32)
            .fold(0, u32::saturating_add)
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\attackpower.cpp

// ============================================================================
// FUNCTION: tagAttackPower::tagAttackPower
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\attackpower.cpp:11
// RVA: 0x001D3C80
// ADDRESS: 005d3c80
// PROTOTYPE: undefined __thiscall tagAttackPower(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: tagAttackInformation::tagAttackInformation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\attackpower.cpp:16
// RVA: 0x001D3D70
// ADDRESS: 005d3d70
// PROTOTYPE: undefined __thiscall tagAttackInformation(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
