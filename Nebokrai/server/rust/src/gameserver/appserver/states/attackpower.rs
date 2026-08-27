//! Типизированное описание рассчитанной атаки GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/states/attackpower.cpp`. Сохраняются порядок составляющих урона,
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

impl AttackInformation {
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
// FUNCTION: tagAttackInformation::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\attackpower.cpp:44
// RVA: 0x001D3CA0
// ADDRESS: 005d3ca0
// PROTOTYPE: void __thiscall Clear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagAttackInformation::~tagAttackInformation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\attackpower.cpp:39
// RVA: 0x001D3D40
// ADDRESS: 005d3d40
// PROTOTYPE: void __thiscall ~tagAttackInformation(void)
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
