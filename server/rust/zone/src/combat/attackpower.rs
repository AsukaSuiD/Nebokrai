//! Типизированное описание рассчитанной атаки Zone.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/states/attackpower.cpp` и ApplyFinalDamage из `appserver/moveshape.cpp`.
//! Пара: EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB SHA-256 `B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016`.
//! Машинный код `tagAttackInformation` (VA `0x005D3CA0`, `0x005D3D70`)
//! подтверждает исходный ID, уровень и сброс; `CMoveShape::ApplyFinalDamage`
//! (VA `0x004D0DA0`) — порядок DWORD-вычитаний, ограничение HP/MP и записи
//! только положительной знаковой разницы. Поведение иных приёмников остаётся
//! у вызывающего кода.
//! Сохраняются порядок составляющих урона,
//! идентификаторы навыка и атакующего, сведения PK и признаки завершающей
//! защиты. `Vec` заменяет исходный вектор указателей без изменения числовой и
//! сетевой семантики полей. Значение формируется на стадии `Calculate`, а
//! применяет его конкретный владелец стадии `Attack`.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttackPowerType {
    Physical,
    Element,
    Soul,
    Poison,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttackPower {
    pub kind: AttackPowerType,
    pub hp_damage: i32,
    pub mp_damage: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttackInformation {
    pub skill_id: u32,
    pub skill_level: u8,
    pub attacker_type: i32,
    pub attacker_id: i32,
    pub attacker_team_id: i32,
    pub attacker_faction_id: i32,
    pub attacker_union_id: i32,
    pub hit_modifier: i32,
    pub damage_factor: f32,
    pub damage_modifier: i32,
    pub critical: bool,
    pub blast_attack: bool,
    pub full_miss: u8,
    pub damages: Vec<AttackPower>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FinalAttackDamage {
    pub health: u32,
    pub mana: Option<u32>,
    pub hp_record: u32,
    pub mp_record: u32,
}

impl AttackInformation {
    pub fn for_master(master: super::MasterInfo) -> Self {
        Self {
            skill_id: super::UNKNOWN_SKILL_ID,
            skill_level: 1,
            attacker_type: master.master_type,
            attacker_id: master.master_id,
            attacker_team_id: master.master_team_id,
            attacker_faction_id: master.master_guild_id,
            attacker_union_id: master.master_union_id,
            hit_modifier: 0,
            damage_factor: 1.0,
            damage_modifier: 0,
            critical: false,
            blast_attack: false,
            full_miss: 0,
            damages: Vec::new(),
        }
    }

    /// CMoveShape::ApplyFinalDamage: компоненты читаются как DWORD по порядку,
    /// затем применяется modifier. SetHP игрока ограничивает каждый результат
    /// текущим максимумом; у остальных владельцев передаётся u32::MAX.
    /// Записи damage используют положительную знаковую разницу, поэтому
    /// отсутствие записи само по себе не означает отсутствие изменения HP.
    pub fn final_damage_values(
        &self,
        health: u32,
        maximum_health: u32,
        mana: Option<(u32, u32)>,
    ) -> FinalAttackDamage {
        let mut remaining_health = health;
        let mut remaining_mana = mana.map(|(current, _)| current);
        for power in &self.damages {
            remaining_health = remaining_health
                .saturating_sub(power.hp_damage as u32)
                .min(maximum_health);
            if let (Some(current), Some((_, maximum))) = (&mut remaining_mana, mana) {
                *current = current.saturating_sub(power.mp_damage as u32).min(maximum);
            }
        }
        if self.damage_modifier != 0 {
            remaining_health = remaining_health
                .saturating_sub(self.damage_modifier as u32)
                .min(maximum_health);
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
    pub fn clear(&mut self) {
        self.skill_id = super::UNKNOWN_SKILL_ID;
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

    pub fn clear_damage(&mut self) {
        self.damages.clear();
        self.damage_modifier = 0;
    }

    pub fn hp_damage(&self) -> u32 {
        self.damages
            .iter()
            .map(|power| power.hp_damage.max(0) as u32)
            .fold(self.damage_modifier.max(0) as u32, u32::saturating_add)
    }

    pub fn mp_damage(&self) -> u32 {
        self.damages
            .iter()
            .map(|power| power.mp_damage.max(0) as u32)
            .fold(0, u32::saturating_add)
    }
}
