//! Достигнутая send-family проекция `CPlayer` исторического GameServer.
//!
//! PDB `GameServer/GameServer.pdb` подтверждает base `CMoveShape +0x0` и signed
//! `m_lTeamID +0xB20`, а также unsigned byte `m_btCountry +0xA5C`. Exact
//! `CMessage::SendToAround` RVA `0x00014420` и
//! `SendToRegionContryPlayer` RVA `0x00014760` читают inherited
//! `CBaseObject::m_lID +0x8` как numeric map/player identity, team ID и country
//! после RTTI `CShape/CMoveShape -> CPlayer`. Эти достигнутые поля имеют статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; исходники
//! `server/gameserver/appserver/player.h/.cpp`.
//! `CMessage::Run` RVA `0x000149D0` дополнительно читает inherited father
//! `+0x40` как текущий `CServerRegion*`; удалённый raw pointer выражен
//! `Option<i32>` region identity в assembly-проекции.
//!
//! Материализована также подтверждённая setter-family: боевые scalar-ы
//! насыщаются до `INT_MAX`, contribution — до `±2_000_000_000`, а fetch power
//! сравнивается с unsigned-представлением setup limit. Это минимальный owned
//! player state для будущих equipment/battle-fairy side effects, но не замена
//! полного constructor-а, property recalc или runtime player lifecycle.
//! Silence-timeout, как и оригинал, проверяется лениво при query по
//! инъецируемому wrapping `timeGetTime`-значению; отдельный scheduler для него
//! не требуется.
//! Поэтому `from_send_state` остаётся явной assembly-границей уже
//! восстановленного runtime. Figure передаётся как доказанный derived virtual
//! fact; владение spatial state остаётся у `CMoveShape`.

use super::moveshape::CMoveShape;
use super::shape::{CShape, ShapeFigure, ShapeView};

const PLAYER_TYPE: i32 = 400;
const LEGACY_COMBAT_MAXIMUM: u32 = i32::MAX as u32;
const CONTRIBUTION_MINIMUM: i32 = -2_000_000_000;
const CONTRIBUTION_MAXIMUM: i32 = 2_000_000_000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerBaseProperties {
    pub(crate) pk_count: u16,
    pub(crate) experience: u32,
    pub(crate) fetch_power: u32,
    pub(crate) battle_fairy_recall: bool,
    pub(crate) battle_fairy_died: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerCombatProperties {
    pub(crate) maximum_hp: u32,
    pub(crate) maximum_mp: u32,
    pub(crate) strength: u32,
    pub(crate) dexterity: u32,
    pub(crate) constitution: u32,
    pub(crate) intelligence: u32,
    pub(crate) minimum_attack: u32,
    pub(crate) maximum_attack: u32,
    pub(crate) defense: u32,
    pub(crate) element_resistance: u32,
    pub(crate) blast_defense_scale_bits: u32,
    pub(crate) full_miss_scale_bits: u32,
    pub(crate) critical_rate_bits: u32,
}

impl PlayerCombatProperties {
    pub(crate) const fn blast_defense_scale(self) -> f32 {
        f32::from_bits(self.blast_defense_scale_bits)
    }

    pub(crate) const fn full_miss_scale(self) -> f32 {
        f32::from_bits(self.full_miss_scale_bits)
    }

    pub(crate) const fn critical_rate(self) -> f32 {
        f32::from_bits(self.critical_rate_bits)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CPlayer {
    move_shape: CMoveShape,
    figure: ShapeFigure,
    team_id: i32,
    country: u8,
    server_region_id: Option<i32>,
    base_properties: PlayerBaseProperties,
    combat_properties: PlayerCombatProperties,
    ci_qing_open: bool,
    contribution: i32,
    silence_minutes: i32,
    silence_timestamp_minutes: u32,
}

impl CPlayer {
    /// Собирает только достигнутый send-family state уже созданного игрока;
    /// identity другого object type отвергается до регистрации.
    pub(crate) fn from_send_state(
        move_shape: CMoveShape,
        figure: ShapeFigure,
        team_id: i32,
        country: u8,
        server_region_id: Option<i32>,
    ) -> Option<Self> {
        if move_shape.shape().identity().object_type != PLAYER_TYPE {
            return None;
        }
        Some(Self {
            move_shape,
            figure,
            team_id,
            country,
            server_region_id,
            base_properties: PlayerBaseProperties::default(),
            combat_properties: PlayerCombatProperties::default(),
            ci_qing_open: false,
            contribution: 0,
            silence_minutes: 0,
            silence_timestamp_minutes: 0,
        })
    }

    pub(crate) const fn shape(&self) -> &CShape {
        self.move_shape.shape()
    }

    pub(crate) const fn player_id(&self) -> i32 {
        self.shape().identity().id
    }

    pub(crate) const fn team_id(&self) -> i32 {
        self.team_id
    }

    pub(crate) const fn country(&self) -> u8 {
        self.country
    }

    pub(crate) const fn server_region_id(&self) -> Option<i32> {
        self.server_region_id
    }

    pub(crate) const fn base_properties(&self) -> PlayerBaseProperties {
        self.base_properties
    }

    pub(crate) const fn combat_properties(&self) -> PlayerCombatProperties {
        self.combat_properties
    }

    pub(crate) const fn ci_qing_open(&self) -> bool {
        self.ci_qing_open
    }

    pub(crate) const fn contribution(&self) -> i32 {
        self.contribution
    }

    pub(crate) const fn silence_minutes(&self) -> i32 {
        self.silence_minutes
    }

    pub(crate) const fn set_pk_count(&mut self, value: u16) {
        self.base_properties.pk_count = value;
    }

    pub(crate) const fn set_ci_qing_open(&mut self, value: bool) {
        self.ci_qing_open = value;
    }

    pub(crate) const fn set_experience(&mut self, value: u32) {
        self.base_properties.experience = value;
    }

    pub(crate) const fn set_maximum_hp(&mut self, value: u32) {
        self.combat_properties.maximum_hp = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_maximum_mp(&mut self, value: u32) {
        self.combat_properties.maximum_mp = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_strength(&mut self, value: u32) {
        self.combat_properties.strength = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_dexterity(&mut self, value: u32) {
        self.combat_properties.dexterity = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_constitution(&mut self, value: u32) {
        self.combat_properties.constitution = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_intelligence(&mut self, value: u32) {
        self.combat_properties.intelligence = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_minimum_attack(&mut self, value: u32) {
        self.combat_properties.minimum_attack = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_maximum_attack(&mut self, value: u32) {
        self.combat_properties.maximum_attack = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_defense(&mut self, value: u32) {
        self.combat_properties.defense = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_element_resistance(&mut self, value: u32) {
        self.combat_properties.element_resistance = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_blast_defense_scale(&mut self, value: f32) {
        self.combat_properties.blast_defense_scale_bits =
            (if value < 0.01 { 0.01 } else { value }).to_bits();
    }

    pub(crate) const fn set_full_miss_scale(&mut self, value: f32) {
        self.combat_properties.full_miss_scale_bits =
            (if value < 0.01 { 0.01 } else { value }).to_bits();
    }

    pub(crate) const fn set_critical_rate(&mut self, value: f32) {
        self.combat_properties.critical_rate_bits =
            (if value < 1.0 { 1.0 } else { value }).to_bits();
    }

    pub(crate) const fn set_contribution(&mut self, value: i32) {
        self.contribution = if value < CONTRIBUTION_MINIMUM {
            CONTRIBUTION_MINIMUM
        } else if value > CONTRIBUTION_MAXIMUM {
            CONTRIBUTION_MAXIMUM
        } else {
            value
        };
    }

    /// `lMaxFetchPower` в exact сравнивался после unsigned cast, поэтому
    /// отрицательный setup limit становится большим unsigned пределом.
    pub(crate) const fn set_fetch_power(&mut self, value: u32, setup_maximum: i32) {
        let maximum = setup_maximum as u32;
        self.base_properties.fetch_power = if maximum < value { maximum } else { value };
    }

    pub(crate) const fn set_battle_fairy_recall(&mut self, value: bool) {
        self.base_properties.battle_fairy_recall = value;
    }

    pub(crate) const fn set_battle_fairy_died(&mut self, value: bool) {
        self.base_properties.battle_fairy_died = value;
    }

    /// Exact `SetSilence`: начало хранится в минутах `timeGetTime`, а
    /// не абсолютным deadline в миллисекундах.
    pub(crate) const fn set_silence(&mut self, minutes: i32, now_milliseconds: u32) {
        if minutes > 0 {
            self.silence_minutes = minutes;
            self.silence_timestamp_minutes = now_milliseconds / 60_000;
        } else {
            self.silence_minutes = 0;
            self.silence_timestamp_minutes = 0;
        }
    }

    /// Exact `IsInSilence`: равенство deadline ещё считается silence; после
    /// первой просроченной проверки оба legacy поля обнуляются.
    pub(crate) const fn is_in_silence(&mut self, now_milliseconds: u32) -> bool {
        if self.silence_minutes == 0 {
            return false;
        }
        let deadline =
            (self.silence_timestamp_minutes as i32).wrapping_add(self.silence_minutes) as u32;
        if now_milliseconds / 60_000 <= deadline {
            return true;
        }
        self.silence_minutes = 0;
        self.silence_timestamp_minutes = 0;
        false
    }

    pub(crate) fn shape_view(&self) -> Option<ShapeView> {
        let identity = self.shape().identity();
        Some(ShapeView {
            identity,
            tile_x: self.shape().get_tile_x().ok()?,
            tile_y: self.shape().get_tile_y().ok()?,
            figure: self.figure,
        })
    }
}

const fn clamp_combat_scalar(value: u32) -> u32 {
    if LEGACY_COMBAT_MAXIMUM < value {
        LEGACY_COMBAT_MAXIMUM
    } else {
        value
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp

// ============================================================================
// FUNCTION: CPlayer::GetAccount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:371
// RVA: 0x00002860
// ADDRESS: 00402860
// PROTOTYPE: char * __thiscall GetAccount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetPkCount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:304
// RVA: 0x0001E580
// ADDRESS: 0041e580
// PROTOTYPE: void __thiscall SetPkCount(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCiQingOpenFun
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:250
// RVA: 0x0002ACD0
// ADDRESS: 0042acd0
// PROTOTYPE: void __thiscall SetCiQingOpenFun(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:289
// RVA: 0x0002ACE0
// ADDRESS: 0042ace0
// PROTOTYPE: void __thiscall SetExp(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:390
// RVA: 0x0002ACF0
// ADDRESS: 0042acf0
// PROTOTYPE: void __thiscall SetMaxHP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxMP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:392
// RVA: 0x0002AD10
// ADDRESS: 0042ad10
// PROTOTYPE: void __thiscall SetMaxMP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetStr
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:398
// RVA: 0x0002AD30
// ADDRESS: 0042ad30
// PROTOTYPE: void __thiscall SetStr(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetDex
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:400
// RVA: 0x0002AD50
// ADDRESS: 0042ad50
// PROTOTYPE: void __thiscall SetDex(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCon
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:402
// RVA: 0x0002AD70
// ADDRESS: 0042ad70
// PROTOTYPE: void __thiscall SetCon(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetInt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:404
// RVA: 0x0002AD90
// ADDRESS: 0042ad90
// PROTOTYPE: void __thiscall SetInt(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMinAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:406
// RVA: 0x0002ADB0
// ADDRESS: 0042adb0
// PROTOTYPE: void __thiscall SetMinAtk(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:408
// RVA: 0x0002ADD0
// ADDRESS: 0042add0
// PROTOTYPE: void __thiscall SetMaxAtk(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetDef
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:416
// RVA: 0x0002ADF0
// ADDRESS: 0042adf0
// PROTOTYPE: void __thiscall SetDef(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetElementResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:422
// RVA: 0x0002AE10
// ADDRESS: 0042ae10
// PROTOTYPE: void __thiscall SetElementResistant(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetBlastDefendScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:460
// RVA: 0x0002AE30
// ADDRESS: 0042ae30
// PROTOTYPE: void __thiscall SetBlastDefendScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetFullMissScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:466
// RVA: 0x0002AE60
// ADDRESS: 0042ae60
// PROTOTYPE: void __thiscall SetFullMissScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCriticalRate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:471
// RVA: 0x0002AE90
// ADDRESS: 0042ae90
// PROTOTYPE: void __thiscall SetCriticalRate(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetContribute
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:546
// RVA: 0x0002AEC0
// ADDRESS: 0042aec0
// PROTOTYPE: void __thiscall SetContribute(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetNextExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:570
// RVA: 0x0002AF00
// ADDRESS: 0042af00
// PROTOTYPE: ulong __thiscall GetNextExp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetFetchPower
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:680
// RVA: 0x0002AF20
// ADDRESS: 0042af20
// PROTOTYPE: void __thiscall SetFetchPower(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetBFRecall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1303
// RVA: 0x0002AF40
// ADDRESS: 0042af40
// PROTOTYPE: void __thiscall SetBFRecall(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetBFDied
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1305
// RVA: 0x0002AF50
// ADDRESS: 0042af50
// PROTOTYPE: void __thiscall SetBFDied(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetPersonalShopFlag
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:507
// RVA: 0x0002AF60
// ADDRESS: 0042af60
// PROTOTYPE: void __thiscall SetPersonalShopFlag(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDefaultAttackSkillID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:562
// RVA: 0x0002AFD0
// ADDRESS: 0042afd0
// PROTOTYPE: tagSkillID __thiscall GetDefaultAttackSkillID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnDecreaseMurdererSign
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2941
// RVA: 0x0002B030
// ADDRESS: 0042b030
// PROTOTYPE: void __thiscall OnDecreaseMurdererSign(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnUpdateMurdererSign
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:3037
// RVA: 0x0002B130
// ADDRESS: 0042b130
// PROTOTYPE: void __thiscall OnUpdateMurdererSign(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsBadman
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:3089
// RVA: 0x0002B160
// ADDRESS: 0042b160
// PROTOTYPE: bool __thiscall IsBadman(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsInArea
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5078
// RVA: 0x0002B190
// ADDRESS: 0042b190
// PROTOTYPE: bool __thiscall IsInArea(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsInRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5123
// RVA: 0x0002B230
// ADDRESS: 0042b230
// PROTOTYPE: bool __thiscall IsInRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ActiveEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:6432
// RVA: 0x0002B240
// ADDRESS: 0042b240
// PROTOTYPE: void __thiscall ActiveEquip(CGoods * param_1, ulong param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountFuMoProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:7230
// RVA: 0x0002B690
// ADDRESS: 0042b690
// PROTOTYPE: int __thiscall MountFuMoProperty(GOODS_ADDON_PROPERTIES param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetCurBurden
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8564
// RVA: 0x0002C2D0
// ADDRESS: 0042c2d0
// PROTOTYPE: long __thiscall GetCurBurden(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CanMountEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8582
// RVA: 0x0002C310
// ADDRESS: 0042c310
// PROTOTYPE: long __thiscall CanMountEquip(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CanUseItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8650
// RVA: 0x0002C490
// ADDRESS: 0042c490
// PROTOTYPE: long __thiscall CanUseItem(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecodeSkillsFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9096
// RVA: 0x0002C5C0
// ADDRESS: 0042c5c0
// PROTOTYPE: void __thiscall DecodeSkillsFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnChangeProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9195
// RVA: 0x0002C620
// ADDRESS: 0042c620
// PROTOTYPE: void __thiscall OnChangeProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetSilence
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9266
// RVA: 0x0002C8A0
// ADDRESS: 0042c8a0
// PROTOTYPE: void __thiscall SetSilence(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsInSilence
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9296
// RVA: 0x0002C8F0
// ADDRESS: 0042c8f0
// PROTOTYPE: bool __thiscall IsInSilence(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateCurrentState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9313
// RVA: 0x0002C940
// ADDRESS: 0042c940
// PROTOTYPE: void __thiscall UpdateCurrentState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::EnterCriminalState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9336
// RVA: 0x0002C9A0
// ADDRESS: 0042c9a0
// PROTOTYPE: void __thiscall EnterCriminalState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::EnterResidentState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9351
// RVA: 0x0002CA40
// ADDRESS: 0042ca40
// PROTOTYPE: void __thiscall EnterResidentState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::EnterPeaceState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9361
// RVA: 0x0002CAC0
// ADDRESS: 0042cac0
// PROTOTYPE: void __thiscall EnterPeaceState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::EnterCombatState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9373
// RVA: 0x0002CB50
// ADDRESS: 0042cb50
// PROTOTYPE: void __thiscall EnterCombatState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetGoodsById
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9478
// RVA: 0x0002CC00
// ADDRESS: 0042cc00
// PROTOTYPE: CGoods * __thiscall GetGoodsById(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetGoodsById_FromPackage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9501
// RVA: 0x0002CC70
// ADDRESS: 0042cc70
// PROTOTYPE: CGoods * __thiscall GetGoodsById_FromPackage(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnBeginSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9513
// RVA: 0x0002CC90
// ADDRESS: 0042cc90
// PROTOTYPE: int __thiscall OnBeginSkill(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetCurrentProgress
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9547
// RVA: 0x0002CCB0
// ADDRESS: 0042ccb0
// PROTOTYPE: eProgress __thiscall GetCurrentProgress(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCurrentProgress
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9552
// RVA: 0x0002CCC0
// ADDRESS: 0042ccc0
// PROTOTYPE: void __thiscall SetCurrentProgress(eProgress param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendNotifyMessageA
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9558
// RVA: 0x0002CCD0
// ADDRESS: 0042ccd0
// PROTOTYPE: void __thiscall SendNotifyMessageA(char * param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendSystemInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9570
// RVA: 0x0002CD70
// ADDRESS: 0042cd70
// PROTOTYPE: void __thiscall SendSystemInfo(char * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendOtherInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9581
// RVA: 0x0002CE00
// ADDRESS: 0042ce00
// PROTOTYPE: void __thiscall SendOtherInfo(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CanMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9644
// RVA: 0x0002CE80
// ADDRESS: 0042ce80
// PROTOTYPE: int __thiscall CanMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnCannotMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9651
// RVA: 0x0002CEA0
// ADDRESS: 0042cea0
// PROTOTYPE: void __thiscall OnCannotMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10030
// RVA: 0x0002CF40
// ADDRESS: 0042cf40
// PROTOTYPE: ulong __thiscall GetMoney(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetYuanBao
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10035
// RVA: 0x0002CF50
// ADDRESS: 0042cf50
// PROTOTYPE: ulong __thiscall GetYuanBao(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDepotMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10045
// RVA: 0x0002CF60
// ADDRESS: 0042cf60
// PROTOTYPE: ulong __thiscall GetDepotMoney(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10050
// RVA: 0x0002CF70
// ADDRESS: 0042cf70
// PROTOTYPE: int __thiscall SetMoney(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetYuanBao
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10094
// RVA: 0x0002D120
// ADDRESS: 0042d120
// PROTOTYPE: int __thiscall SetYuanBao(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPacketListener::CPacketListener
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10224
// RVA: 0x0002D2D0
// ADDRESS: 0042d2d0
// PROTOTYPE: undefined __thiscall CPacketListener(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPacketListener::~CPacketListener
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10229
// RVA: 0x0002D2E0
// ADDRESS: 0042d2e0
// PROTOTYPE: void __thiscall ~CPacketListener(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPacketListener::OnTraversingContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10234
// RVA: 0x0002D2F0
// ADDRESS: 0042d2f0
// PROTOTYPE: int __thiscall OnTraversingContainer(CContainer * param_1, CBaseObject * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAtcInterval
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10285
// RVA: 0x0002D3F0
// ADDRESS: 0042d3f0
// PROTOTYPE: ushort __thiscall GetAtcInterval(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetStrikeOutTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10309
// RVA: 0x0002D430
// ADDRESS: 0042d430
// PROTOTYPE: ulong __thiscall GetStrikeOutTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RejectUseSkillRequest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10327
// RVA: 0x0002D450
// ADDRESS: 0042d450
// PROTOTYPE: void __thiscall RejectUseSkillRequest(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10892
// RVA: 0x0002D4D0
// ADDRESS: 0042d4d0
// PROTOTYPE: void __thiscall SetLevel(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::PerformEmotion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11059
// RVA: 0x0002D590
// ADDRESS: 0042d590
// PROTOTYPE: void __thiscall PerformEmotion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ClearEmotion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11083
// RVA: 0x0002D680
// ADDRESS: 0042d680
// PROTOTYPE: void __thiscall ClearEmotion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CContainerListener::OnObjectRemoved
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11134
// RVA: 0x0002D720
// ADDRESS: 0042d720
// PROTOTYPE: int __thiscall OnObjectRemoved(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetContendState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11140
// RVA: 0x0002D730
// ADDRESS: 0042d730
// PROTOTYPE: void __thiscall SetContendState(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCityWarDiedStateTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11153
// RVA: 0x0002D7C0
// ADDRESS: 0042d7c0
// PROTOTYPE: void __thiscall SetCityWarDiedStateTime(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCityWarDiedState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11169
// RVA: 0x0002D860
// ADDRESS: 0042d860
// PROTOTYPE: void __thiscall SetCityWarDiedState(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsFactionMaster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11200
// RVA: 0x0002D930
// ADDRESS: 0042d930
// PROTOTYPE: bool __thiscall IsFactionMaster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsUnionMaster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11211
// RVA: 0x0002D950
// ADDRESS: 0042d950
// PROTOTYPE: bool __thiscall IsUnionMaster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetWeaponModifier
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11410
// RVA: 0x0002D980
// ADDRESS: 0042d980
// PROTOTYPE: float __thiscall GetWeaponModifier(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetWeaponDamageLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11439
// RVA: 0x0002D9F0
// ADDRESS: 0042d9f0
// PROTOTYPE: ulong __thiscall GetWeaponDamageLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::QuestTimeBegin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11486
// RVA: 0x0002DA20
// ADDRESS: 0042da20
// PROTOTYPE: void __thiscall QuestTimeBegin(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::QuestTimeClear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11497
// RVA: 0x0002DAC0
// ADDRESS: 0042dac0
// PROTOTYPE: void __thiscall QuestTimeClear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetQuestOn
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11506
// RVA: 0x0002DB40
// ADDRESS: 0042db40
// PROTOTYPE: void __thiscall SetQuestOn(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetExploit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12073
// RVA: 0x0002DBC0
// ADDRESS: 0042dbc0
// PROTOTYPE: void __thiscall SetExploit(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::end_business
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12095
// RVA: 0x0002DBF0
// ADDRESS: 0042dbf0
// PROTOTYPE: void __thiscall end_business(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetSessionID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12173
// RVA: 0x0002DCC0
// ADDRESS: 0042dcc0
// PROTOTYPE: char * __thiscall GetSessionID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetIpAddress
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12179
// RVA: 0x0002DCD0
// ADDRESS: 0042dcd0
// PROTOTYPE: char * __thiscall GetIpAddress(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteSkillItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12861
// RVA: 0x0002DD20
// ADDRESS: 0042dd20
// PROTOTYPE: int __thiscall DeleteSkillItem(ulong param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetWarSoulGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13040
// RVA: 0x0002DF10
// ADDRESS: 0042df10
// PROTOTYPE: CGoods * __thiscall GetWarSoulGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetWarSoulXY
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13071
// RVA: 0x0002DF50
// ADDRESS: 0042df50
// PROTOTYPE: void __thiscall SetWarSoulXY(tagPOINT param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13109
// RVA: 0x0002E0A0
// ADDRESS: 0042e0a0
// PROTOTYPE: void __thiscall DelWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetWarSoulStaus
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13135
// RVA: 0x0002E190
// ADDRESS: 0042e190
// PROTOTYPE: void __thiscall SetWarSoulStaus(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ReplacePlayerData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13276
// RVA: 0x0002E260
// ADDRESS: 0042e260
// PROTOTYPE: void __thiscall ReplacePlayerData(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RestorePlayerData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13300
// RVA: 0x0002E400
// ADDRESS: 0042e400
// PROTOTYPE: void __thiscall RestorePlayerData(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::TellClientMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13315
// RVA: 0x0002E4D0
// ADDRESS: 0042e4d0
// PROTOTYPE: void __thiscall TellClientMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::TellClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13324
// RVA: 0x0002E570
// ADDRESS: 0042e570
// PROTOTYPE: void __thiscall TellClient(ulong param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RejectUseSkillRequestWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13358
// RVA: 0x0002E720
// ADDRESS: 0042e720
// PROTOTYPE: void __thiscall RejectUseSkillRequestWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckBFSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13446
// RVA: 0x0002E7A0
// ADDRESS: 0042e7a0
// PROTOTYPE: int __thiscall CheckBFSkill(CGoods * param_1, long param_2, tagSkillID param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::BuyItemFromAuction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13554
// RVA: 0x0002E820
// ADDRESS: 0042e820
// PROTOTYPE: void __thiscall BuyItemFromAuction(CGUID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13619
// RVA: 0x0002E8B0
// ADDRESS: 0042e8b0
// PROTOTYPE: bool __thiscall IsMoney(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsGoodAllowedInAuction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13682
// RVA: 0x0002EA50
// ADDRESS: 0042ea50
// PROTOTYPE: bool __thiscall IsGoodAllowedInAuction(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsCurAucNodeOK
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13941
// RVA: 0x0002EA80
// ADDRESS: 0042ea80
// PROTOTYPE: bool __thiscall IsCurAucNodeOK(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsCurAucBuyNodeOK
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13948
// RVA: 0x0002EB30
// ADDRESS: 0042eb30
// PROTOTYPE: bool __thiscall IsCurAucBuyNodeOK(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CleanCurAucNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13970
// RVA: 0x0002EBE0
// ADDRESS: 0042ebe0
// PROTOTYPE: void __thiscall CleanCurAucNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetOptMoneyYuan
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14536
// RVA: 0x0002EC10
// ADDRESS: 0042ec10
// PROTOTYPE: bool __thiscall GetOptMoneyYuan(long * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetOptMoneyJin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14575
// RVA: 0x0002ECF0
// ADDRESS: 0042ecf0
// PROTOTYPE: bool __thiscall GetOptMoneyJin(long * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::QuerySellerGoodsSelf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14715
// RVA: 0x0002EE80
// ADDRESS: 0042ee80
// PROTOTYPE: void __thiscall QuerySellerGoodsSelf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAuctionMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14782
// RVA: 0x0002EEF0
// ADDRESS: 0042eef0
// PROTOTYPE: ulong __thiscall GetAuctionMoney(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AuctionLimit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14998
// RVA: 0x0002EF00
// ADDRESS: 0042ef00
// PROTOTYPE: bool __thiscall AuctionLimit(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendSaleLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15094
// RVA: 0x0002EF60
// ADDRESS: 0042ef60
// PROTOTYPE: void __thiscall SendSaleLog(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendCutLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15115
// RVA: 0x0002F0A0
// ADDRESS: 0042f0a0
// PROTOTYPE: void __thiscall SendCutLog(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::NoticyWS_BaiTan_Over
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15135
// RVA: 0x0002F140
// ADDRESS: 0042f140
// PROTOTYPE: void __thiscall NoticyWS_BaiTan_Over(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendToGSBaiTan
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15143
// RVA: 0x0002F1C0
// ADDRESS: 0042f1c0
// PROTOTYPE: void __thiscall SendToGSBaiTan(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountScoreAdd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15155
// RVA: 0x0002F250
// ADDRESS: 0042f250
// PROTOTYPE: int __cdecl CountScoreAdd(int param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::JJcWeekClear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15176
// RVA: 0x0002F2D0
// ADDRESS: 0042f2d0
// PROTOTYPE: void __thiscall JJcWeekClear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::JJcSeasonClear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15190
// RVA: 0x0002F320
// ADDRESS: 0042f320
// PROTOTYPE: void __thiscall JJcSeasonClear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddPreItemToPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15373
// RVA: 0x0002F360
// ADDRESS: 0042f360
// PROTOTYPE: void __thiscall AddPreItemToPlayer(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddGoodsToCiQing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16045
// RVA: 0x0002F920
// ADDRESS: 0042f920
// PROTOTYPE: bool __thiscall AddGoodsToCiQing(CGoods * param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::WriteCiQingLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16333
// RVA: 0x0002FAA0
// ADDRESS: 0042faa0
// PROTOTYPE: void __thiscall WriteCiQingLog(ulong param_1, ulong param_2, ulong param_3, ulong param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCurFlash
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17480
// RVA: 0x0002FB50
// ADDRESS: 0042fb50
// PROTOTYPE: void __thiscall SetCurFlash(CGoods * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneFlash
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17528
// RVA: 0x0002FC50
// ADDRESS: 0042fc50
// PROTOTYPE: void __thiscall DoneFlash(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddLTUp60Cnt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17679
// RVA: 0x0002FD10
// ADDRESS: 0042fd10
// PROTOTYPE: void __thiscall AddLTUp60Cnt(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:368
// RVA: 0x000300D0
// ADDRESS: 004300d0
// PROTOTYPE: void __thiscall SetMP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetRP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:370
// RVA: 0x000300F0
// ADDRESS: 004300f0
// PROTOTYPE: void __thiscall SetRP(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetVigour
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:518
// RVA: 0x00030120
// ADDRESS: 00430120
// PROTOTYPE: void __thiscall SetVigour(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::PeriodicalUpdate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2348
// RVA: 0x00030140
// ADDRESS: 00430140
// PROTOTYPE: void __thiscall PeriodicalUpdate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IncreaseRp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9417
// RVA: 0x000302F0
// ADDRESS: 004302f0
// PROTOTYPE: void __thiscall IncreaseRp(int param_1, ushort param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDepotPassword
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10270
// RVA: 0x00030410
// ADDRESS: 00430410
// PROTOTYPE: char * __thiscall GetDepotPassword(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::WriteGoodsDelLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12633
// RVA: 0x00030430
// ADDRESS: 00430430
// PROTOTYPE: void __thiscall WriteGoodsDelLog(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelWarSoulSkillInPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13157
// RVA: 0x00030610
// ADDRESS: 00430610
// PROTOTYPE: void __thiscall DelWarSoulSkillInPlayer(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddWarSoulSkillToPalyer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13209
// RVA: 0x00030760
// ADDRESS: 00430760
// PROTOTYPE: void __thiscall AddWarSoulSkillToPalyer(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ComputeWarSoulXY
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13366
// RVA: 0x00030930
// ADDRESS: 00430930
// PROTOTYPE: void __thiscall ComputeWarSoulXY(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::BuyItemFromAauction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13826
// RVA: 0x00030BC0
// ADDRESS: 00430bc0
// PROTOTYPE: bool __thiscall BuyItemFromAauction(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsDonePreNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13931
// RVA: 0x00030CF0
// ADDRESS: 00430cf0
// PROTOTYPE: bool __thiscall IsDonePreNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCurAucBuyNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13977
// RVA: 0x00030D00
// ADDRESS: 00430d00
// PROTOTYPE: bool __thiscall SetCurAucBuyNode(CGoodsNode param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ReFlushSelfGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14723
// RVA: 0x00030D90
// ADDRESS: 00430d90
// PROTOTYPE: void __thiscall ReFlushSelfGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetAuctionMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14737
// RVA: 0x00030E60
// ADDRESS: 00430e60
// PROTOTYPE: bool __thiscall SetAuctionMoney(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsGM
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5044
// RVA: 0x00031430
// ADDRESS: 00431430
// PROTOTYPE: bool __thiscall IsGM(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetGMLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5052
// RVA: 0x00031480
// ADDRESS: 00431480
// PROTOTYPE: long __thiscall GetGMLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10594
// RVA: 0x000314E0
// ADDRESS: 004314e0
// PROTOTYPE: ulong __thiscall DeleteGoods(PLAYER_EXTEND_ID param_1, CGUID * param_2, ulong param_3, bool param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoodsbyGuid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12330
// RVA: 0x00031800
// ADDRESS: 00431800
// PROTOTYPE: int __thiscall DeleteGoodsbyGuid(CGUID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendAucAbBuyOptTran
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14493
// RVA: 0x00031900
// ADDRESS: 00431900
// PROTOTYPE: void __thiscall SendAucAbBuyOptTran(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddTaoZhuangSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15294
// RVA: 0x00031DE0
// ADDRESS: 00431de0
// PROTOTYPE: void __thiscall AddTaoZhuangSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddTaoZhuangPre
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15351
// RVA: 0x00031E70
// ADDRESS: 00431e70
// PROTOTYPE: void __thiscall AddTaoZhuangPre(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddCiQingTaoZhuangPre
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15362
// RVA: 0x00031EE0
// ADDRESS: 00431ee0
// PROTOTYPE: void __thiscall AddCiQingTaoZhuangPre(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteArrayLeiTing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17554
// RVA: 0x00031F50
// ADDRESS: 00431f50
// PROTOTYPE: void __thiscall AddByteArrayLeiTing(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetOneThing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17635
// RVA: 0x00032000
// ADDRESS: 00432000
// PROTOTYPE: tagThing * __thiscall GetOneThing(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DropGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:272
// RVA: 0x00032630
// ADDRESS: 00432630
// PROTOTYPE: int __thiscall DropGoods(PLAYER_EXTEND_ID param_1, CGUID * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::check_item_in_packet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:388
// RVA: 0x00032900
// ADDRESS: 00432900
// PROTOTYPE: uint __thiscall check_item_in_packet(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::remove_item_in_packet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:408
// RVA: 0x000329E0
// ADDRESS: 004329e0
// PROTOTYPE: uint __thiscall remove_item_in_packet(int param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetNumSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8968
// RVA: 0x00032AA0
// ADDRESS: 00432aa0
// PROTOTYPE: long __thiscall GetNumSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddSkillsToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8998
// RVA: 0x00032B80
// ADDRESS: 00432b80
// PROTOTYPE: void __thiscall AddSkillsToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnChangeStates
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9234
// RVA: 0x00033080
// ADDRESS: 00433080
// PROTOTYPE: void __thiscall OnChangeStates(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsEnemyFactionMember
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9397
// RVA: 0x00033210
// ADDRESS: 00433210
// PROTOTYPE: long __thiscall IsEnemyFactionMember(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsCityWarEneymyFactionMemeber
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9407
// RVA: 0x00033240
// ADDRESS: 00433240
// PROTOTYPE: long __thiscall IsCityWarEneymyFactionMemeber(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoodsInPacket
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10857
// RVA: 0x00033270
// ADDRESS: 00433270
// PROTOTYPE: void __thiscall DeleteGoodsInPacket(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddQuestDataByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11226
// RVA: 0x00033310
// ADDRESS: 00433310
// PROTOTYPE: bool __thiscall AddQuestDataByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetQuestState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11293
// RVA: 0x000333B0
// ADDRESS: 004333b0
// PROTOTYPE: long __thiscall GetQuestState(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetValidQuestNum
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11301
// RVA: 0x000333F0
// ADDRESS: 004333f0
// PROTOTYPE: long __thiscall GetValidQuestNum(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CompleteQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11352
// RVA: 0x00033480
// ADDRESS: 00433480
// PROTOTYPE: void __thiscall CompleteQuest(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11383
// RVA: 0x00033550
// ADDRESS: 00433550
// PROTOTYPE: void __thiscall UpdateQuest(ushort param_1, long param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ReUseSkillItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12832
// RVA: 0x00033610
// ADDRESS: 00433610
// PROTOTYPE: int __thiscall ReUseSkillItem(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::JudgeZhaoMuStatus
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13478
// RVA: 0x000336B0
// ADDRESS: 004336b0
// PROTOTYPE: bool __thiscall JudgeZhaoMuStatus(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CleanPreAndSkillList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15265
// RVA: 0x00033700
// ADDRESS: 00433700
// PROTOTYPE: void __thiscall CleanPreAndSkillList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelTaoZhuangSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15273
// RVA: 0x000337A0
// ADDRESS: 004337a0
// PROTOTYPE: void __thiscall DelTaoZhuangSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteCiQing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15841
// RVA: 0x00033840
// ADDRESS: 00433840
// PROTOTYPE: void __thiscall AddByteCiQing(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountCiQingFromHand
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16180
// RVA: 0x000338D0
// ADDRESS: 004338d0
// PROTOTYPE: bool __thiscall MountCiQingFromHand(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddOrgSysToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1518
// RVA: 0x00033C30
// ADDRESS: 00433c30
// PROTOTYPE: bool __thiscall AddOrgSysToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteGS2WS
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13511
// RVA: 0x00033E40
// ADDRESS: 00433e40
// PROTOTYPE: void __thiscall AddByteGS2WS(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateLeiTingToWSandClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17701
// RVA: 0x00033ED0
// ADDRESS: 00433ed0
// PROTOTYPE: void __thiscall UpdateLeiTingToWSandClient(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetDepotPassword
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10275
// RVA: 0x000342D0
// ADDRESS: 004342d0
// PROTOTYPE: void __thiscall SetDepotPassword(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsAttackAble
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10335
// RVA: 0x00034300
// ADDRESS: 00434300
// PROTOTYPE: bool __thiscall IsAttackAble(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetTileXY
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10872
// RVA: 0x00034930
// ADDRESS: 00434930
// PROTOTYPE: void __thiscall SetTileXY(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::do_coutribute
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11514
// RVA: 0x00034A00
// ADDRESS: 00434a00
// PROTOTYPE: void __thiscall do_coutribute(CServerRegion * param_1, CPlayer * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddExploitToMurdererInCountryWar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12038
// RVA: 0x00035DF0
// ADDRESS: 00435df0
// PROTOTYPE: void __thiscall AddExploitToMurdererInCountryWar(CServerRegion * param_1, CPlayer * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DrawAwards
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12205
// RVA: 0x00035F70
// ADDRESS: 00435f70
// PROTOTYPE: long __thiscall DrawAwards(long param_1, int param_2, ulong param_3, ulong param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeBodyCheck
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12918
// RVA: 0x00036080
// ADDRESS: 00436080
// PROTOTYPE: int __thiscall ChangeBodyCheck(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsAollowAuction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13583
// RVA: 0x00036200
// ADDRESS: 00436200
// PROTOTYPE: bool __thiscall IsAollowAuction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::WriteBuyAuctionLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14612
// RVA: 0x00036370
// ADDRESS: 00436370
// PROTOTYPE: void __thiscall WriteBuyAuctionLog(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OpenAuction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14788
// RVA: 0x000367B0
// ADDRESS: 004367b0
// PROTOTYPE: bool __thiscall OpenAuction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckAuctionMoneyMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14846
// RVA: 0x000369B0
// ADDRESS: 004369b0
// PROTOTYPE: bool __thiscall CheckAuctionMoneyMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoodsFromCiQing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16251
// RVA: 0x00036AA0
// ADDRESS: 00436aa0
// PROTOTYPE: void __thiscall DeleteGoodsFromCiQing(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddThingCnt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17649
// RVA: 0x00036BD0
// ADDRESS: 00436bd0
// PROTOTYPE: bool __thiscall AddThingCnt(int param_1, int param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateSZL
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17775
// RVA: 0x00036C40
// ADDRESS: 00436c40
// PROTOTYPE: void __thiscall UpdateSZL(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RemoveQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11367
// RVA: 0x00038DC0
// ADDRESS: 00438dc0
// PROTOTYPE: void __thiscall RemoveQuest(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeByteCiQing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15853
// RVA: 0x00038EA0
// ADDRESS: 00438ea0
// PROTOTYPE: void __thiscall DeByteCiQing(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:357
// RVA: 0x0003A020
// ADDRESS: 0043a020
// PROTOTYPE: int __thiscall CheckGoods(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelFriend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9163
// RVA: 0x0003A1A0
// ADDRESS: 0043a1a0
// PROTOTYPE: bool __thiscall DelFriend(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10566
// RVA: 0x0003A300
// ADDRESS: 0043a300
// PROTOTYPE: CGUID __thiscall DeleteGoods(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DropParticularGoodsWhenDead
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10712
// RVA: 0x0003A4D0
// ADDRESS: 0043a4d0
// PROTOTYPE: void __thiscall DropParticularGoodsWhenDead(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DropParticularGoodsWhenLost
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10776
// RVA: 0x0003A860
// ADDRESS: 0043a860
// PROTOTYPE: void __thiscall DropParticularGoodsWhenLost(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DropParticularGoodsWhenRecall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10818
// RVA: 0x0003AAC0
// ADDRESS: 0043aac0
// PROTOTYPE: void __thiscall DropParticularGoodsWhenRecall(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToTaoZhuangSkillList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15230
// RVA: 0x0003AD30
// ADDRESS: 0043ad30
// PROTOTYPE: void __thiscall AddItemToTaoZhuangSkillList(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToTaoZhuangPreList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15243
// RVA: 0x0003AD70
// ADDRESS: 0043ad70
// PROTOTYPE: void __thiscall AddItemToTaoZhuangPreList(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToCiQingTaoZhuangPreList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15254
// RVA: 0x0003ADB0
// ADDRESS: 0043adb0
// PROTOTYPE: void __thiscall AddItemToCiQingTaoZhuangPreList(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetCurrentTypeValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16080
// RVA: 0x0003ADF0
// ADDRESS: 0043adf0
// PROTOTYPE: void __thiscall GetCurrentTypeValue(map<unsigned_long,unsigned_long,std::less<unsigned_long>,std::allocator<std::pair<unsigned_long_const_,unsigned_long>_>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddGoodsToPacket
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:431
// RVA: 0x0003BFD0
// ADDRESS: 0043bfd0
// PROTOTYPE: bool __thiscall AddGoodsToPacket(vector<CGoods*,std::allocator<CGoods*>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecordOrgSysFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1459
// RVA: 0x0003C230
// ADDRESS: 0043c230
// PROTOTYPE: bool __thiscall DecordOrgSysFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountEquipRide
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:6516
// RVA: 0x0003C5E0
// ADDRESS: 0043c5e0
// PROTOTYPE: void __thiscall MountEquipRide(CGoods * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnExitRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9661
// RVA: 0x0003DB50
// ADDRESS: 0043db50
// PROTOTYPE: void __thiscall OnExitRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IncExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10910
// RVA: 0x0003DB60
// ADDRESS: 0043db60
// PROTOTYPE: ulong __thiscall IncExp(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddQuestDataByteArray_ForClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11240
// RVA: 0x0003E1C0
// ADDRESS: 0043e1c0
// PROTOTYPE: bool __thiscall AddQuestDataByteArray_ForClient(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToDelList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12365
// RVA: 0x0003E3F0
// ADDRESS: 0043e3f0
// PROTOTYPE: bool __thiscall AddItemToDelList(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelAllItemInDelList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12401
// RVA: 0x0003E4E0
// ADDRESS: 0043e4e0
// PROTOTYPE: bool __thiscall DelAllItemInDelList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneDelList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12427
// RVA: 0x0003E670
// ADDRESS: 0043e670
// PROTOTYPE: void __thiscall DoneDelList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ComputeTicket
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12459
// RVA: 0x0003E820
// ADDRESS: 0043e820
// PROTOTYPE: ulong __thiscall ComputeTicket(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckAddGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12497
// RVA: 0x0003E920
// ADDRESS: 0043e920
// PROTOTYPE: ulong __thiscall CheckAddGoods(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToMap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12518
// RVA: 0x0003E980
// ADDRESS: 0043e980
// PROTOTYPE: bool __thiscall AddItemToMap(ulong param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToGoodsAiTree
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12543
// RVA: 0x0003EA60
// ADDRESS: 0043ea60
// PROTOTYPE: bool __thiscall AddItemToGoodsAiTree(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelItemFromGoodsAiTree
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12567
// RVA: 0x0003EAC0
// ADDRESS: 0043eac0
// PROTOTYPE: bool __thiscall DelItemFromGoodsAiTree(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneGoodsAiTree
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12616
// RVA: 0x0003EBD0
// ADDRESS: 0043ebd0
// PROTOTYPE: void __thiscall DoneGoodsAiTree(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateGoodsGS2C
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12784
// RVA: 0x0003EC40
// ADDRESS: 0043ec40
// PROTOTYPE: void __thiscall UpdateGoodsGS2C(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetLastUseSkillItemTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12826
// RVA: 0x0003ED30
// ADDRESS: 0043ed30
// PROTOTYPE: void __thiscall SetLastUseSkillItemTime(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::TellClietAuctionOK
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13520
// RVA: 0x0003ED50
// ADDRESS: 0043ed50
// PROTOTYPE: void __thiscall TellClietAuctionOK(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteAuctionSelfToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13708
// RVA: 0x0003EF60
// ADDRESS: 0043ef60
// PROTOTYPE: void __thiscall AddByteAuctionSelfToClient(CMessage * param_1, CMessage * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteAuctionAllToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13764
// RVA: 0x0003F190
// ADDRESS: 0043f190
// PROTOTYPE: void __thiscall AddByteAuctionAllToClient(CMessage * param_1, CMessage * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendBackAucNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14323
// RVA: 0x0003F3D0
// ADDRESS: 0043f3d0
// PROTOTYPE: void __thiscall SendBackAucNode(CGoodsNode * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::TellClientScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14696
// RVA: 0x0003F500
// ADDRESS: 0043f500
// PROTOTYPE: void __thiscall TellClientScale(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendAuctionBangCondition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14938
// RVA: 0x0003F620
// ADDRESS: 0043f620
// PROTOTYPE: void __thiscall SendAuctionBangCondition(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ModifyAuctionSpace
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14977
// RVA: 0x0003F7A0
// ADDRESS: 0043f7a0
// PROTOTYPE: void __thiscall ModifyAuctionSpace(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OutputBinaryStream
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15063
// RVA: 0x0003F890
// ADDRESS: 0043f890
// PROTOTYPE: void __thiscall OutputBinaryStream(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendTaoZhuangSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15214
// RVA: 0x0003FA20
// ADDRESS: 0043fa20
// PROTOTYPE: void __thiscall SendTaoZhuangSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CleanTaoZhuangItemList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15636
// RVA: 0x0003FAF0
// ADDRESS: 0043faf0
// PROTOTYPE: void __thiscall CleanTaoZhuangItemList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToTaoZhuangItemList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15642
// RVA: 0x0003FB60
// ADDRESS: 0043fb60
// PROTOTYPE: void __thiscall AddItemToTaoZhuangItemList(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MakeCiQingNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15864
// RVA: 0x0003FCD0
// ADDRESS: 0043fcd0
// PROTOTYPE: ulong __thiscall MakeCiQingNode(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendCiQingGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16023
// RVA: 0x0003FE90
// ADDRESS: 0043fe90
// PROTOTYPE: void __thiscall SendCiQingGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecodeByteArrayLeiTing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17571
// RVA: 0x0003FFD0
// ADDRESS: 0043ffd0
// PROTOTYPE: void __thiscall DecodeByteArrayLeiTing(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeFyEnergyFlag
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17595
// RVA: 0x000400F0
// ADDRESS: 004400f0
// PROTOTYPE: bool __thiscall ChangeFyEnergyFlag(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::InitSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:530
// RVA: 0x00040C30
// ADDRESS: 00440c30
// PROTOTYPE: void __thiscall InitSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:742
// RVA: 0x00040DC0
// ADDRESS: 00440dc0
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnExit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1692
// RVA: 0x00041460
// ADDRESS: 00441460
// PROTOTYPE: void __thiscall OnExit(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044156e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1712
// RVA: 0x0004156E
// ADDRESS: 0044156e
// PROTOTYPE: undefined Catch@0044156e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0044158b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1715
// RVA: 0x0004158B
// ADDRESS: 0044158b
// PROTOTYPE: undefined FUN_0044158b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnLost
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1780
// RVA: 0x000417A0
// ADDRESS: 004417a0
// PROTOTYPE: void __thiscall OnLost(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnEquipmentWaste
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2731
// RVA: 0x000419B0
// ADDRESS: 004419b0
// PROTOTYPE: void __thiscall OnEquipmentWaste(EQUIPMENT_COLUMN param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnArmorDamaged
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2818
// RVA: 0x00041AF0
// ADDRESS: 00441af0
// PROTOTYPE: void __thiscall OnArmorDamaged(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnWeaponDamaged
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2932
// RVA: 0x00041D50
// ADDRESS: 00441d50
// PROTOTYPE: void __thiscall OnWeaponDamaged(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnBeenHurted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2966
// RVA: 0x00041D80
// ADDRESS: 00441d80
// PROTOTYPE: void __thiscall OnBeenHurted(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnBeenMurdered
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:3111
// RVA: 0x00042040
// ADDRESS: 00442040
// PROTOTYPE: void __thiscall OnBeenMurdered(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5239
// RVA: 0x00042610
// ADDRESS: 00442610
// PROTOTYPE: void __thiscall MountEquip(ulong param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00443c5b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:6069
// RVA: 0x00043C5B
// ADDRESS: 00443c5b
// PROTOTYPE: undefined Catch@00443c5b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RestoreHp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:7785
// RVA: 0x00044C80
// ADDRESS: 00444c80
// PROTOTYPE: int __thiscall RestoreHp(ulong param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RestoreMp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:7804
// RVA: 0x00044D50
// ADDRESS: 00444d50
// PROTOTYPE: int __thiscall RestoreMp(ulong param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::Mount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8513
// RVA: 0x00044E20
// ADDRESS: 00444e20
// PROTOTYPE: int __thiscall Mount(ulong param_1, ulong param_2, ulong param_3, char * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddFriend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9110
// RVA: 0x00044F80
// ADDRESS: 00444f80
// PROTOTYPE: void __thiscall AddFriend(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnObjectAdded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11095
// RVA: 0x000451A0
// ADDRESS: 004451a0
// PROTOTYPE: int __thiscall OnObjectAdded(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecordQuestDataFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11278
// RVA: 0x000452D0
// ADDRESS: 004452d0
// PROTOTYPE: bool __thiscall DecordQuestDataFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11316
// RVA: 0x00045360
// ADDRESS: 00445360
// PROTOTYPE: void __thiscall AddQuest(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::get_country_identity
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12150
// RVA: 0x00045540
// ADDRESS: 00445540
// PROTOTYPE: uchar __thiscall get_country_identity(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RestoreHpMp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12717
// RVA: 0x000455D0
// ADDRESS: 004455d0
// PROTOTYPE: void __thiscall RestoreHpMp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToAuction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13492
// RVA: 0x00045910
// ADDRESS: 00445910
// PROTOTYPE: bool __thiscall AddItemToAuction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MakeCurAucNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13999
// RVA: 0x00045A50
// ADDRESS: 00445a50
// PROTOTYPE: bool __thiscall MakeCurAucNode(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendBuyAucNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14353
// RVA: 0x00045EE0
// ADDRESS: 00445ee0
// PROTOTYPE: void __thiscall SendBuyAucNode(CGoodsNode * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendAucAbOpt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14397
// RVA: 0x00046080
// ADDRESS: 00446080
// PROTOTYPE: void __thiscall SendAucAbOpt(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AutoAddAuctionGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14862
// RVA: 0x000466A0
// ADDRESS: 004466a0
// PROTOTYPE: void __thiscall AutoAddAuctionGoods(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ComposeCiQingNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15901
// RVA: 0x00046A40
// ADDRESS: 00446a40
// PROTOTYPE: bool __thiscall ComposeCiQingNode(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateCiQingProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16110
// RVA: 0x00046E90
// ADDRESS: 00446e90
// PROTOTYPE: void __thiscall UpdateCiQingProperty(map<unsigned_long,unsigned_long,std::less<unsigned_long>,std::allocator<std::pair<unsigned_long_const_,unsigned_long>_>_> param_1, map<unsigned_long,unsigned_long,std::less<unsigned_long>,std::allocator<std::pair<unsigned_long_const_,unsigned_long>_>_> param_2, map<unsigned_long,unsigned_long,struct_std::less<unsigned_long>,class_std::allocator<struct_std::pair<unsigned_long_const_,unsigned_long>_>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountCiQingEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16353
// RVA: 0x00047000
// ADDRESS: 00447000
// PROTOTYPE: void __thiscall MountCiQingEquip(ulong param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00448346
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17020
// RVA: 0x00048346
// ADDRESS: 00448346
// PROTOTYPE: undefined Catch@00448346()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::~CPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:256
// RVA: 0x00049860
// ADDRESS: 00449860
// PROTOTYPE: void __thiscall ~CPlayer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:286
// RVA: 0x0004A2D0
// ADDRESS: 0044a2d0
// PROTOTYPE: uchar __thiscall GetLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:288
// RVA: 0x0004A2E0
// ADDRESS: 0044a2e0
// PROTOTYPE: ulong __thiscall GetExp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:365
// RVA: 0x0004A2F0
// ADDRESS: 0044a2f0
// PROTOTYPE: ulong __thiscall GetHP(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:366
// RVA: 0x0004A300
// ADDRESS: 0044a300
// PROTOTYPE: void __thiscall SetHP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMaxHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:389
// RVA: 0x0004A340
// ADDRESS: 0044a340
// PROTOTYPE: ulong __thiscall GetMaxHP(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMinAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:405
// RVA: 0x0004A350
// ADDRESS: 0044a350
// PROTOTYPE: ulong __thiscall GetMinAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMaxAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:407
// RVA: 0x0004A360
// ADDRESS: 0044a360
// PROTOTYPE: ulong __thiscall GetMaxAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetHit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:409
// RVA: 0x0004A370
// ADDRESS: 0044a370
// PROTOTYPE: ushort __thiscall GetHit(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetCCH
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:413
// RVA: 0x0004A380
// ADDRESS: 0044a380
// PROTOTYPE: ushort __thiscall GetCCH(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDef
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:415
// RVA: 0x0004A390
// ADDRESS: 0044a390
// PROTOTYPE: ulong __thiscall GetDef(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDodge
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:417
// RVA: 0x0004A3A0
// ADDRESS: 0044a3a0
// PROTOTYPE: ushort __thiscall GetDodge(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAtcSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:419
// RVA: 0x0004A3B0
// ADDRESS: 0044a3b0
// PROTOTYPE: short __thiscall GetAtcSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetElementResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:421
// RVA: 0x0004A3C0
// ADDRESS: 0044a3c0
// PROTOTYPE: ulong __thiscall GetElementResistant(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetHpRecoverSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:423
// RVA: 0x0004A3D0
// ADDRESS: 0044a3d0
// PROTOTYPE: ushort __thiscall GetHpRecoverSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMpRecoverSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:425
// RVA: 0x0004A3E0
// ADDRESS: 0044a3e0
// PROTOTYPE: ushort __thiscall GetMpRecoverSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetExalt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:428
// RVA: 0x0004A3F0
// ADDRESS: 0044a3f0
// PROTOTYPE: ulong __thiscall GetExalt(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetExalt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:429
// RVA: 0x0004A400
// ADDRESS: 0044a400
// PROTOTYPE: void __thiscall SetExalt(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetSoulResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:432
// RVA: 0x0004A410
// ADDRESS: 0044a410
// PROTOTYPE: ushort __thiscall GetSoulResistant(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAddElementAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:434
// RVA: 0x0004A420
// ADDRESS: 0044a420
// PROTOTYPE: ulong __thiscall GetAddElementAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAddSoulAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:436
// RVA: 0x0004A430
// ADDRESS: 0044a430
// PROTOTYPE: ushort __thiscall GetAddSoulAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetReAnk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:441
// RVA: 0x0004A440
// ADDRESS: 0044a440
// PROTOTYPE: ushort __thiscall GetReAnk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAttackAvoid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:444
// RVA: 0x0004A450
// ADDRESS: 0044a450
// PROTOTYPE: ushort __thiscall GetAttackAvoid(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetElementAvoid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:446
// RVA: 0x0004A460
// ADDRESS: 0044a460
// PROTOTYPE: ushort __thiscall GetElementAvoid(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetFullMiss
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:448
// RVA: 0x0004A470
// ADDRESS: 0044a470
// PROTOTYPE: ushort __thiscall GetFullMiss(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddToByteArray_ForClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:941
// RVA: 0x0004A480
// ADDRESS: 0044a480
// PROTOTYPE: bool __thiscall AddToByteArray_ForClient(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneCurAucNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14165
// RVA: 0x0004B100
// ADDRESS: 0044b100
// PROTOTYPE: void __thiscall DoneCurAucNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneCurAucBuyNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14258
// RVA: 0x0004B2A0
// ADDRESS: 0044b2a0
// PROTOTYPE: void __thiscall DoneCurAucBuyNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendBackCurBuyNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14347
// RVA: 0x0004B370
// ADDRESS: 0044b370
// PROTOTYPE: void __thiscall SendBackCurBuyNode(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendResultToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16137
// RVA: 0x0004B390
// ADDRESS: 0044b390
// PROTOTYPE: void __thiscall SendResultToClient(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteToOtherPerson
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16269
// RVA: 0x0004B5E0
// ADDRESS: 0044b5e0
// PROTOTYPE: void __thiscall AddByteToOtherPerson(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1252
// RVA: 0x0004BA80
// ADDRESS: 0044ba80
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1867
// RVA: 0x0004C400
// ADDRESS: 0044c400
// PROTOTYPE: bool __thiscall ChangeRegion(long param_1, long param_2, long param_3, long param_4, long param_5, long param_6, long param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnStandOnSwitchPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2596
// RVA: 0x0004D420
// ADDRESS: 0044d420
// PROTOTYPE: int __thiscall OnStandOnSwitchPoint(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnDied
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:3306
// RVA: 0x0004D850
// ADDRESS: 0044d850
// PROTOTYPE: void __thiscall OnDied(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:4930
// RVA: 0x00052DD0
// ADDRESS: 00452dd0
// PROTOTYPE: long __thiscall CheckLevel(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountAllEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5132
// RVA: 0x00053480
// ADDRESS: 00453480
// PROTOTYPE: void __thiscall MountAllEquip(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004535ee
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5183
// RVA: 0x000535EE
// ADDRESS: 004535ee
// PROTOTYPE: undefined Catch@004535ee()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UseItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:7833
// RVA: 0x00053840
// ADDRESS: 00453840
// PROTOTYPE: void __thiscall UseItem(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::InitNameValueMap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8693
// RVA: 0x00055000
// ADDRESS: 00455000
// PROTOTYPE: void __thiscall InitNameValueMap(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8858
// RVA: 0x000577B0
// ADDRESS: 004577b0
// PROTOTYPE: ulong __thiscall GetValue(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8889
// RVA: 0x00057AF0
// ADDRESS: 00457af0
// PROTOTYPE: ulong __thiscall SetValue(char * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8927
// RVA: 0x00057E60
// ADDRESS: 00457e60
// PROTOTYPE: ulong __thiscall ChangeValue(char * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IncreaseContinuousKill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9593
// RVA: 0x00058390
// ADDRESS: 00458390
// PROTOTYPE: void __thiscall IncreaseContinuousKill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::PlayerRunScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11397
// RVA: 0x000584E0
// ADDRESS: 004584e0
// PROTOTYPE: long __thiscall PlayerRunScript(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AdjustHonorRank
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15201
// RVA: 0x000585B0
// ADDRESS: 004585b0
// PROTOTYPE: bool __thiscall AdjustHonorRank(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RequestChangeAppellation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15208
// RVA: 0x000585C0
// ADDRESS: 004585c0
// PROTOTYPE: bool __thiscall RequestChangeAppellation(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ComputerAddValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15311
// RVA: 0x000585D0
// ADDRESS: 004585d0
// PROTOTYPE: void __thiscall ComputerAddValue(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneTaoZhuang
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15678
// RVA: 0x00058810
// ADDRESS: 00458810
// PROTOTYPE: void __thiscall DoneTaoZhuang(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RunQuestCompleteScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17798
// RVA: 0x00058940
// ADDRESS: 00458940
// PROTOTYPE: long __thiscall RunQuestCompleteScript(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:89
// RVA: 0x000589B0
// ADDRESS: 004589b0
// PROTOTYPE: undefined __thiscall CPlayer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:601
// RVA: 0x000593E0
// ADDRESS: 004593e0
// PROTOTYPE: void __thiscall UpdateProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnRelive
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1558
// RVA: 0x00059BB0
// ADDRESS: 00459bb0
// PROTOTYPE: void __thiscall OnRelive(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2264
// RVA: 0x00059FF0
// ADDRESS: 00459ff0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnEnterRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9681
// RVA: 0x0005A170
// ADDRESS: 0045a170
// PROTOTYPE: void __thiscall OnEnterRegion(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetNetExID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1037
// RVA: 0x0007BD10
// ADDRESS: 0047bd10
// PROTOTYPE: long __thiscall GetNetExID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetSeaGoodsName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1438
// RVA: 0x00087780
// ADDRESS: 00487780
// PROTOTYPE: void __thiscall SetSeaGoodsName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetLastContainerScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1173
// RVA: 0x00093440
// ADDRESS: 00493440
// PROTOTYPE: char * __thiscall GetLastContainerScript(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCharged
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:317
// RVA: 0x000AEB30
// ADDRESS: 004aeb30
// PROTOTYPE: void __thiscall SetCharged(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxEnergy
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:526
// RVA: 0x000AEB40
// ADDRESS: 004aeb40
// PROTOTYPE: void __thiscall SetMaxEnergy(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCreateFactionOperator
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1023
// RVA: 0x000AEB60
// ADDRESS: 004aeb60
// PROTOTYPE: void __thiscall SetCreateFactionOperator(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetApplyJoinOperator
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1026
// RVA: 0x000AEB70
// ADDRESS: 004aeb70
// PROTOTYPE: void __thiscall SetApplyJoinOperator(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetFactionDeclareWarOperator
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1029
// RVA: 0x000AEB80
// ADDRESS: 004aeb80
// PROTOTYPE: void __thiscall SetFactionDeclareWarOperator(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetEnergy
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:524
// RVA: 0x000AED90
// ADDRESS: 004aed90
// PROTOTYPE: void __thiscall SetEnergy(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetLastContainerScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1172
// RVA: 0x000AEFC0
// ADDRESS: 004aefc0
// PROTOTYPE: void __thiscall SetLastContainerScript(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::PushItemToCiQingList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:252
// RVA: 0x000AF1E0
// ADDRESS: 004af1e0
// PROTOTYPE: void __thiscall PushItemToCiQingList(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddStr
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:511
// RVA: 0x000FA9E0
// ADDRESS: 004fa9e0
// PROTOTYPE: void __thiscall AddStr(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddDex
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:512
// RVA: 0x000FAA00
// ADDRESS: 004faa00
// PROTOTYPE: void __thiscall AddDex(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddCon
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:513
// RVA: 0x000FAA20
// ADDRESS: 004faa20
// PROTOTYPE: void __thiscall AddCon(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddInt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:514
// RVA: 0x000FAA40
// ADDRESS: 004faa40
// PROTOTYPE: void __thiscall AddInt(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::bHasPet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1281
// RVA: 0x000FD7D0
// ADDRESS: 004fd7d0
// PROTOTYPE: bool __thiscall bHasPet(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetBlastAttackScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:458
// RVA: 0x001DFD60
// ADDRESS: 005dfd60
// PROTOTYPE: void __thiscall SetBlastAttackScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetElementBlastAttackScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:462
// RVA: 0x001DFD90
// ADDRESS: 005dfd90
// PROTOTYPE: void __thiscall SetElementBlastAttackScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetElementBlastDefendScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:464
// RVA: 0x001DFDC0
// ADDRESS: 005dfdc0
// PROTOTYPE: void __thiscall SetElementBlastDefendScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
