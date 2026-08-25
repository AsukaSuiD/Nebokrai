//! Достигнутая constructor/property-часть `CMonster` GameServer.
//!
//! Exact `GameServer/gameserver.exe + GameServer/GameServer.pdb`, owner
//! `server/gameserver/appserver/monster.h/.cpp`, подтверждают inheritance от
//! `CMoveShape`, type `600`, HP `1`, live/refresh index `-1`, нулевые leader
//! признаки и десять factor-ов `1.0`. `CBaseObject::CreateObject(600,id)`
//! записывает ID после derived constructor-а.
//!
//! Старый `m_pBaseProperty` указывал внутрь process-global `CMonsterList` и
//! перепривязывался после selector `0x02`. Rust хранит byte-exact original-name
//! key и разрешает текущий `MonsterProperties` у `CGame`; это устраняет
//! dangling pointer, сохраняя observable refresh semantics. Spawn snapshot
//! (имя, graphics, HP, speed) остаётся в concrete object, как в `AddMonster`.
//! `InitSkills/InitAI`, combat, serialization и полный AI остаются RAW ниже.
//! Login pet restoration и client control используют owned `tagMasterInfo`,
//! taming sign, progress, Globe factor snapshot и узкое pet-control state;
//! async CPet decision tree этим не подменяется.

use super::masterinfo::MasterInfo;
use super::moveshape::CMoveShape;
use super::shape::{SHAPE_CHANGE_DELETE, ShapeFigure, ShapeIdentity};
use crate::setup::monsterlist::MonsterProperties;

const MONSTER_TYPE: i32 = 600;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CMonster {
    move_shape: CMoveShape,
    original_name: Vec<u8>,
    base_property_key: Option<Vec<u8>>,
    script_file: Vec<u8>,
    hit_points: u32,
    live_time: i32,
    refresh_index: i32,
    sign: u16,
    leader_sign: u16,
    leader_distance: u16,
    leader_type: i32,
    leader_id: i32,
    died_remove: bool,
    factors: [u32; 10],
    master_info: MasterInfo,
    tamed: bool,
    pet_level: u32,
    pet_experience: u32,
    pet_mode: i32,
    pet_action: i32,
    pet_target: Option<ShapeIdentity>,
}

impl CMonster {
    pub(crate) fn with_constructor_defaults() -> Self {
        let mut move_shape = CMoveShape::default();
        move_shape
            .shape_mut()
            .base_object_mut()
            .set_type(MONSTER_TYPE);
        Self {
            move_shape,
            original_name: Vec::new(),
            base_property_key: None,
            script_file: Vec::new(),
            hit_points: 1,
            live_time: -1,
            refresh_index: -1,
            sign: 0,
            leader_sign: 0,
            leader_distance: 0,
            leader_type: 0,
            leader_id: 0,
            died_remove: false,
            factors: [1.0f32.to_bits(); 10],
            master_info: MasterInfo::default(),
            tamed: false,
            pet_level: 0,
            pet_experience: 0,
            pet_mode: 0,
            pet_action: 1,
            pet_target: None,
        }
    }

    pub(crate) const fn move_shape(&self) -> &CMoveShape {
        &self.move_shape
    }

    pub(crate) const fn move_shape_mut(&mut self) -> &mut CMoveShape {
        &mut self.move_shape
    }

    pub(crate) const fn master_info(&self) -> MasterInfo {
        self.master_info
    }

    pub(crate) const fn set_master_info(&mut self, master_info: MasterInfo) {
        self.master_info = master_info;
    }

    pub(crate) const fn set_tamed(&mut self, tamed: bool) {
        self.tamed = tamed;
    }

    pub(crate) const fn is_owned_pet(&self, player_id: i32) -> bool {
        self.master_info.master_type == 400 && self.master_info.master_id == player_id
    }

    pub(crate) const fn set_pet_progress(&mut self, level: u32, experience: u32) {
        self.pet_level = level;
        self.pet_experience = experience;
    }

    pub(crate) const fn pet_progress(&self) -> (u32, u32) {
        (self.pet_level, self.pet_experience)
    }

    pub(crate) fn adjust_pet_factors(&mut self, factors: [f32; 10]) {
        self.factors = factors.map(f32::to_bits);
    }

    pub(crate) fn pet_maximum_hp(&self, property: &MonsterProperties) -> u32 {
        (property.maximum_hp as f32 * f32::from_bits(self.factors[6])).round_ties_even() as u32
    }

    pub(crate) const fn set_pet_mode(&mut self, mode: i32) {
        self.pet_mode = mode;
    }

    pub(crate) const fn set_pet_action(&mut self, action: i32) {
        self.pet_action = action;
        if action != 0 {
            self.pet_target = None;
        }
    }

    pub(crate) const fn set_pet_target(&mut self, target: ShapeIdentity) {
        self.pet_action = 0;
        self.pet_target = Some(target);
    }

    pub(crate) fn evanish_pet(&mut self) {
        self.stage_for_delete();
    }

    /// Назначает exact поля, которые `AddMonster` пишет до virtual `Init`.
    pub(crate) fn bind_spawn_property(&mut self, property: &MonsterProperties) {
        let shape = self.move_shape.shape_mut();
        shape.base_object_mut().set_name(&property.name);
        shape
            .base_object_mut()
            .set_graphics_id(property.picture_id as i32);
        self.original_name = property.original_name.clone();
        self.base_property_key = Some(property.original_name.clone());
        self.hit_points = property.maximum_hp;
    }

    /// Скорость исходный spawn назначает только после `Init` и позиции.
    pub(crate) fn set_spawn_speed(&mut self, property: &MonsterProperties) {
        self.move_shape
            .shape_mut()
            .set_speed(property.move_speed as f32);
    }

    pub(crate) fn base_property_key(&self) -> Option<&[u8]> {
        self.base_property_key.as_deref()
    }

    pub(crate) fn original_name(&self) -> &[u8] {
        &self.original_name
    }

    pub(crate) fn display_name(&self) -> &[u8] {
        let name = self.move_shape.shape().base_object().get_name();
        if name.is_empty() {
            &self.original_name
        } else {
            name
        }
    }

    pub(crate) const fn hit_points(&self) -> u32 {
        self.hit_points
    }

    pub(crate) const fn set_hit_points(&mut self, hit_points: u32) {
        self.hit_points = hit_points;
    }

    /// Guards reached from `CMonster::OnBeenHurted` before Nation first-hit
    /// dispatch: action `ACT_DIED` and health-based death are independent.
    pub(crate) fn can_trigger_nation_damage(&self) -> bool {
        self.move_shape.shape().get_action() != 6 && !CMoveShape::is_died(self.hit_points)
    }

    /// Exact `OnClearWar` predicate использует ту же пару virtual action/HP,
    /// но остаётся отдельным gameplay-контрактом phase cleanup.
    pub(crate) fn can_clear_from_nation_war(&self) -> bool {
        self.move_shape.shape().get_action() != 6 && !CMoveShape::is_died(self.hit_points)
    }

    pub(crate) const fn staged_for_delete(&self) -> bool {
        self.move_shape.shape().change_state() == SHAPE_CHANGE_DELETE
    }

    pub(crate) fn stage_for_delete(&mut self) {
        self.move_shape
            .shape_mut()
            .set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub(crate) fn set_script_file(&mut self, script_file: &[u8]) {
        let prefix_len = script_file
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(script_file.len());
        self.script_file.clear();
        self.script_file
            .extend_from_slice(&script_file[..prefix_len]);
    }

    pub(crate) const fn set_refresh_data(
        &mut self,
        sign: u16,
        leader_sign: u16,
        leader_distance: u16,
        refresh_index: i32,
    ) {
        self.sign = sign;
        self.leader_sign = leader_sign;
        self.leader_distance = leader_distance;
        self.refresh_index = refresh_index;
    }

    pub(crate) fn figure(property: &MonsterProperties) -> ShapeFigure {
        let figure = property.figure as u8;
        ShapeFigure::from_directions([figure; 4])
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h

// ============================================================================
// FUNCTION: CMonster::GetScriptFile
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:249
// RVA: 0x00031410
// ADDRESS: 00431410
// PROTOTYPE: char * __thiscall GetScriptFile(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetScriptFile
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:250
// RVA: 0x00038C70
// ADDRESS: 00438c70
// PROTOTYPE: void __thiscall SetScriptFile(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d0882
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp
// RVA: 0x000D0882
// ADDRESS: 004d0882
// PROTOTYPE: undefined Catch@004d0882()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetMasterInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:107
// RVA: 0x000E63E0
// ADDRESS: 004e63e0
// PROTOTYPE: void __thiscall SetMasterInfo(tagMasterInfo * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetMasterInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:112
// RVA: 0x000E63F0
// ADDRESS: 004e63f0
// PROTOTYPE: tagMasterInfo * __thiscall GetMasterInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::Init
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:117
// RVA: 0x000E6400
// ADDRESS: 004e6400
// PROTOTYPE: void __thiscall Init(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:144
// RVA: 0x000E6420
// ADDRESS: 004e6420
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::IsTamable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:208
// RVA: 0x000E6430
// ADDRESS: 004e6430
// PROTOTYPE: int __thiscall IsTamable(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::DoesCreatureBeenTamed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:220
// RVA: 0x000E6460
// ADDRESS: 004e6460
// PROTOTYPE: int __thiscall DoesCreatureBeenTamed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::IncreaseTameAttemptCount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:244
// RVA: 0x000E6490
// ADDRESS: 004e6490
// PROTOTYPE: void __thiscall IncreaseTameAttemptCount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetTamedSign
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:249
// RVA: 0x000E64A0
// ADDRESS: 004e64a0
// PROTOTYPE: void __thiscall SetTamedSign(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAttackAvoid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:273
// RVA: 0x000E64F0
// ADDRESS: 004e64f0
// PROTOTYPE: ushort __thiscall GetAttackAvoid(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetElementAvoid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:298
// RVA: 0x000E6520
// ADDRESS: 004e6520
// PROTOTYPE: ushort __thiscall GetElementAvoid(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetAttackAvoid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:282
// RVA: 0x000E6550
// ADDRESS: 004e6550
// PROTOTYPE: void __thiscall SetAttackAvoid(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetMaxHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1389
// RVA: 0x000E65A0
// ADDRESS: 004e65a0
// PROTOTYPE: ulong __thiscall GetMaxHP(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetMinAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1401
// RVA: 0x000E6620
// ADDRESS: 004e6620
// PROTOTYPE: ulong __thiscall GetMinAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetMaxAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1433
// RVA: 0x000E66C0
// ADDRESS: 004e66c0
// PROTOTYPE: ulong __thiscall GetMaxAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetHit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1467
// RVA: 0x000E6760
// ADDRESS: 004e6760
// PROTOTYPE: ushort __thiscall GetHit(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetDef
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1474
// RVA: 0x000E6780
// ADDRESS: 004e6780
// PROTOTYPE: ulong __thiscall GetDef(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetDodge
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1489
// RVA: 0x000E6800
// ADDRESS: 004e6800
// PROTOTYPE: ushort __thiscall GetDodge(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAtcSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1504
// RVA: 0x000E6860
// ADDRESS: 004e6860
// PROTOTYPE: short __thiscall GetAtcSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetElementResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1510
// RVA: 0x000E6880
// ADDRESS: 004e6880
// PROTOTYPE: ulong __thiscall GetElementResistant(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetElementModify
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1535
// RVA: 0x000E6900
// ADDRESS: 004e6900
// PROTOTYPE: ulong __thiscall GetElementModify(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetSoulResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1550
// RVA: 0x000E6970
// ADDRESS: 004e6970
// PROTOTYPE: ushort __thiscall GetSoulResistant(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetHpRecoverSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1557
// RVA: 0x000E6990
// ADDRESS: 004e6990
// PROTOTYPE: ushort __thiscall GetHpRecoverSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetHit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1564
// RVA: 0x000E69B0
// ADDRESS: 004e69b0
// PROTOTYPE: ushort __thiscall GetHit(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1569
// RVA: 0x000E69C0
// ADDRESS: 004e69c0
// PROTOTYPE: uchar __thiscall GetLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAddSoulAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1574
// RVA: 0x000E69D0
// ADDRESS: 004e69d0
// PROTOTYPE: ushort __thiscall GetAddSoulAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAtcInterval
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1581
// RVA: 0x000E69F0
// ADDRESS: 004e69f0
// PROTOTYPE: ushort __thiscall GetAtcInterval(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetStopFrame
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1601
// RVA: 0x000E6A40
// ADDRESS: 004e6a40
// PROTOTYPE: long __thiscall GetStopFrame(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetBeAttackedPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1611
// RVA: 0x000E6AA0
// ADDRESS: 004e6aa0
// PROTOTYPE: void __thiscall GetBeAttackedPoint(long param_1, long param_2, long * param_3, long * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetMonsterKind
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1647
// RVA: 0x000E6CA0
// ADDRESS: 004e6ca0
// PROTOTYPE: eMonsterKind __thiscall GetMonsterKind(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetStrikeOutTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1656
// RVA: 0x000E6CC0
// ADDRESS: 004e6cc0
// PROTOTYPE: ulong __thiscall GetStrikeOutTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetPetLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1698
// RVA: 0x000E6CE0
// ADDRESS: 004e6ce0
// PROTOTYPE: ulong __thiscall GetPetLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetPetExperience
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1703
// RVA: 0x000E6CF0
// ADDRESS: 004e6cf0
// PROTOTYPE: ulong __thiscall GetPetExperience(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetPetLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1708
// RVA: 0x000E6D00
// ADDRESS: 004e6d00
// PROTOTYPE: void __thiscall SetPetLevel(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetPetExperience
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1716
// RVA: 0x000E6D20
// ADDRESS: 004e6d20
// PROTOTYPE: void __thiscall SetPetExperience(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::IsCarriage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1721
// RVA: 0x000E6D30
// ADDRESS: 004e6d30
// PROTOTYPE: bool __thiscall IsCarriage(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetTrackRange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:87
// RVA: 0x000E6D50
// ADDRESS: 004e6d50
// PROTOTYPE: long __thiscall GetTrackRange(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:123
// RVA: 0x000E6D80
// ADDRESS: 004e6d80
// PROTOTYPE: CBaseAI * __thiscall GetAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:201
// RVA: 0x000E6DD0
// ADDRESS: 004e6dd0
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::InitAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:323
// RVA: 0x000E6E10
// ADDRESS: 004e6e10
// PROTOTYPE: void __thiscall InitAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::OnBeenHurted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:920
// RVA: 0x000E6EF0
// ADDRESS: 004e6ef0
// PROTOTYPE: void __thiscall OnBeenHurted(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::AdjustPetProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:959
// RVA: 0x000E6FF0
// ADDRESS: 004e6ff0
// PROTOTYPE: void __thiscall AdjustPetProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::UpgradePetLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:977
// RVA: 0x000E7090
// ADDRESS: 004e7090
// PROTOTYPE: void __thiscall UpgradePetLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::IncreasePetExperience
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1010
// RVA: 0x000E7150
// ADDRESS: 004e7150
// PROTOTYPE: void __thiscall IncreasePetExperience(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::IsAttackAble
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1028
// RVA: 0x000E7230
// ADDRESS: 004e7230
// PROTOTYPE: bool __thiscall IsAttackAble(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAddElementAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1525
// RVA: 0x000E7950
// ADDRESS: 004e7950
// PROTOTYPE: ulong __thiscall GetAddElementAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1591
// RVA: 0x000E79B0
// ADDRESS: 004e79b0
// PROTOTYPE: float __thiscall GetSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::NotifyMasterWhenPetDied
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1664
// RVA: 0x000E79E0
// ADDRESS: 004e79e0
// PROTOTYPE: void __thiscall NotifyMasterWhenPetDied(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::Evanish
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:1689
// RVA: 0x000E7A60
// ADDRESS: 004e7a60
// PROTOTYPE: void __thiscall Evanish(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::CalculateExperienceCorrective
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:468
// RVA: 0x000E7A70
// ADDRESS: 004e7a70
// PROTOTYPE: ulong __thiscall CalculateExperienceCorrective(CPlayer * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::~CMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:80
// RVA: 0x000E7C10
// ADDRESS: 004e7c10
// PROTOTYPE: void __thiscall ~CMonster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetFigure
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:49
// RVA: 0x000E7D20
// ADDRESS: 004e7d20
// PROTOTYPE: uchar __thiscall GetFigure(eDIR param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:166
// RVA: 0x000E7D30
// ADDRESS: 004e7d30
// PROTOTYPE: ulong __thiscall GetExp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:168
// RVA: 0x000E7D40
// ADDRESS: 004e7d40
// PROTOTYPE: ulong __thiscall GetHP(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetReAnk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:191
// RVA: 0x000E7D50
// ADDRESS: 004e7d50
// PROTOTYPE: ushort __thiscall GetReAnk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:199
// RVA: 0x000E7D60
// ADDRESS: 004e7d60
// PROTOTYPE: void __thiscall SetHP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetMinAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:200
// RVA: 0x000E7D70
// ADDRESS: 004e7d70
// PROTOTYPE: void __thiscall SetMinAtk(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetMaxAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:201
// RVA: 0x000E7D80
// ADDRESS: 004e7d80
// PROTOTYPE: void __thiscall SetMaxAtk(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetHit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:202
// RVA: 0x000E7D90
// ADDRESS: 004e7d90
// PROTOTYPE: void __thiscall SetHit(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetDef
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:203
// RVA: 0x000E7DA0
// ADDRESS: 004e7da0
// PROTOTYPE: void __thiscall SetDef(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetDodge
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:204
// RVA: 0x000E7DB0
// ADDRESS: 004e7db0
// PROTOTYPE: void __thiscall SetDodge(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetAtcSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:205
// RVA: 0x000E7DC0
// ADDRESS: 004e7dc0
// PROTOTYPE: void __thiscall SetAtcSpeed(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetElementResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:206
// RVA: 0x000E7DD0
// ADDRESS: 004e7dd0
// PROTOTYPE: void __thiscall SetElementResistant(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetSoulResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:207
// RVA: 0x000E7DE0
// ADDRESS: 004e7de0
// PROTOTYPE: void __thiscall SetSoulResistant(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetHpRecoverSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:208
// RVA: 0x000E7DF0
// ADDRESS: 004e7df0
// PROTOTYPE: void __thiscall SetHpRecoverSpeed(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetAddSoulAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:211
// RVA: 0x000E7E00
// ADDRESS: 004e7e00
// PROTOTYPE: void __thiscall SetAddSoulAtk(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::SetElementModify
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:212
// RVA: 0x000E7E10
// ADDRESS: 004e7e10
// PROTOTYPE: void __thiscall SetElementModify(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAckRangeMin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:216
// RVA: 0x000E7E20
// ADDRESS: 004e7e20
// PROTOTYPE: long __thiscall GetAckRangeMin(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetAckRangeMax
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:217
// RVA: 0x000E7E30
// ADDRESS: 004e7e30
// PROTOTYPE: long __thiscall GetAckRangeMax(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetFightRange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:218
// RVA: 0x000E7E40
// ADDRESS: 004e7e40
// PROTOTYPE: long __thiscall GetFightRange(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetChaseRange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:220
// RVA: 0x000E7E50
// ADDRESS: 004e7e50
// PROTOTYPE: long __thiscall GetChaseRange(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetGuardRange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.h:219
// RVA: 0x000E7E60
// ADDRESS: 004e7e60
// PROTOTYPE: long __thiscall GetGuardRange(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::CMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// IMPLEMENTED_SUBCHAIN: scalar/property defaults материализованы выше;
// constructor-side `CMoveShape::InitSkills` остаётся у незакрытого skill owner.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:37
// RVA: 0x000E7E70
// ADDRESS: 004e7e70
// PROTOTYPE: undefined __thiscall CMonster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::CalculateExperienceQuota
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:369
// RVA: 0x000E7FE0
// ADDRESS: 004e7fe0
// PROTOTYPE: ulong __thiscall CalculateExperienceQuota(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::Talk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:345
// RVA: 0x000E8220
// ADDRESS: 004e8220
// PROTOTYPE: void __thiscall Talk(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:151
// RVA: 0x000E8400
// ADDRESS: 004e8400
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::InitSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:257
// RVA: 0x000E86B0
// ADDRESS: 004e86b0
// PROTOTYPE: void __thiscall InitSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetBeneficiary
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:551
// RVA: 0x000E8780
// ADDRESS: 004e8780
// PROTOTYPE: CPlayer * __thiscall GetBeneficiary(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::OnDied
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\monster.cpp:644
// RVA: 0x000E8960
// ADDRESS: 004e8960
// PROTOTYPE: void __thiscall OnDied(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
