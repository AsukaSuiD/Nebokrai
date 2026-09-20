//! Process-owned состояние `CJJcSystem` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/jjcsystem.cpp`. Достигнутые World callbacks сохраняют две
//! ordered map: region → пара участников и player → сведения соперника.
//! Match/start/timeout/week/season и `CPlayer::OnLost -> QuitJJc` вызываются
//! реальным `CGame::ProcessMessage`; player, region, script и network effects
//! выполняет `CGame`. `CScript::JJcFunction 10000..10009` теперь достигает
//! также `ApplyJJc`, `EndPK` и `BackRegion`: этот owner выбирает сведения матча
//! и точку возврата, а `CGame` сохраняет порядок сообщений, воскрешения и
//! смены региона. Неопределённые значения EAX после исходных void-вызовов не
//! считаются игровым контрактом и нормализованы диспетчером в ноль.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct JjcInfo {
    pub(crate) jjc_level: u32,
    pub(crate) old_region_id: i32,
    pub(crate) pos_x: i32,
    pub(crate) pos_y: i32,
    pub(crate) opponent_id: i32,
    pub(crate) jjc_region_id: i32,
    pub(crate) start_time: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CJJcSystem {
    pk_list: BTreeMap<i32, (i32, i32)>,
    player_info: BTreeMap<i32, JjcInfo>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct JjcReturnTarget {
    pub(crate) region_id: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
}

impl CJJcSystem {
    pub(crate) fn on_world_closed(&mut self) {
        self.pk_list.clear();
        self.player_info.clear();
    }

    pub(crate) fn on_matched(&mut self, first: JjcInfo, second: JjcInfo) {
        self.pk_list
            .insert(first.jjc_region_id, (second.opponent_id, first.opponent_id));
        self.player_info.insert(second.opponent_id, first);
        self.player_info.insert(first.opponent_id, second);
    }

    pub(crate) fn opponent_info(
        &self,
        selector: u32,
        region_id: i32,
        player_id: i32,
    ) -> Option<i32> {
        let info = self.player_info.get(&player_id)?;
        let opponent = self.player_info.get(&info.opponent_id)?;
        let _region_members = self.pk_list.get(&region_id);
        match selector {
            1 => Some(info.opponent_id),
            2 => Some(opponent.jjc_level as i32),
            _ => None,
        }
    }

    pub(crate) fn return_target(
        &self,
        player_id: i32,
        current_region_id: i32,
        country: u8,
        region_min: i32,
        region_max: i32,
    ) -> Option<JjcReturnTarget> {
        let info = self.player_info.get(&player_id).copied().unwrap_or_default();
        if info.old_region_id != 0 {
            return Some(JjcReturnTarget {
                region_id: info.old_region_id,
                tile_x: info.pos_x,
                tile_y: info.pos_y,
            });
        }
        if !(region_min..=region_max).contains(&current_region_id) {
            return None;
        }
        match country {
            1 => Some(JjcReturnTarget {
                region_id: 11_000,
                tile_x: 0x113,
                tile_y: 0x11a,
            }),
            2 => Some(JjcReturnTarget {
                region_id: 12_000,
                tile_x: 0xdc,
                tile_y: 0x106,
            }),
            3 => Some(JjcReturnTarget {
                region_id: 13_000,
                tile_x: 0xdc,
                tile_y: 0x106,
            }),
            4 => Some(JjcReturnTarget {
                region_id: 14_000,
                tile_x: 0x113,
                tile_y: 0x11a,
            }),
            _ => None,
        }
    }

    pub(crate) fn finish_pk(&mut self, region_id: i32, player_id: i32) {
        self.player_info.remove(&player_id);
        self.pk_list.remove(&region_id);
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp

// ============================================================================
// FUNCTION: CJJcSystem::IsJJcRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:25
// RVA: 0x000D9590
// ADDRESS: 004d9590
// PROTOTYPE: bool __thiscall IsJJcRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::OnJJcPKOver
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:216
// RVA: 0x000D95C0
// ADDRESS: 004d95c0
// PROTOTYPE: void __thiscall OnJJcPKOver(CPlayer * param_1, CPlayer * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::OnWSClosed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:340
// RVA: 0x000D9900
// ADDRESS: 004d9900
// PROTOTYPE: void __thiscall OnWSClosed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::WeekUpdate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:346
// RVA: 0x000D9960
// ADDRESS: 004d9960
// PROTOTYPE: void __thiscall WeekUpdate(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::SeasonUpdate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:364
// RVA: 0x000D99E0
// ADDRESS: 004d99e0
// PROTOTYPE: void __thiscall SeasonUpdate(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::GetOpponentInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:125
// RVA: 0x000D9AA0
// ADDRESS: 004d9aa0
// PROTOTYPE: long __thiscall GetOpponentInfo(ulong param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::ApplyJJc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:33
// RVA: 0x000D9B60
// ADDRESS: 004d9b60
// PROTOTYPE: bool __thiscall ApplyJJc(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::~CJJcSystem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:13
// RVA: 0x000DAA20
// ADDRESS: 004daa20
// PROTOTYPE: void __thiscall ~CJJcSystem(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::CJJcSystem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:9
// RVA: 0x000DABD0
// ADDRESS: 004dabd0
// PROTOTYPE: undefined __thiscall CJJcSystem(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:18
// RVA: 0x000DAC60
// ADDRESS: 004dac60
// PROTOTYPE: CJJcSystem * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::QuitJJc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:72
// RVA: 0x000DACC0
// ADDRESS: 004dacc0
// PROTOTYPE: void __thiscall QuitJJc(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::UpdateJJcData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:164
// RVA: 0x000DAE00
// ADDRESS: 004dae00
// PROTOTYPE: void __thiscall UpdateJJcData(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::OnJJcMatched
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:183
// RVA: 0x000DAF00
// ADDRESS: 004daf00
// PROTOTYPE: void __thiscall OnJJcMatched(tagJJcInfo * param_1, tagJJcInfo * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::OnJJcStarted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:198
// RVA: 0x000DAF90
// ADDRESS: 004daf90
// PROTOTYPE: void __thiscall OnJJcStarted(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::OnJJcPKTimeout
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:232
// RVA: 0x000DB040
// ADDRESS: 004db040
// PROTOTYPE: void __thiscall OnJJcPKTimeout(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::BackRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:284
// RVA: 0x000DB0E0
// ADDRESS: 004db0e0
// PROTOTYPE: void __thiscall BackRegion(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::EndPK
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\jjcsystem.cpp:251
// RVA: 0x000DB240
// ADDRESS: 004db240
// PROTOTYPE: void __thiscall EndPK(long param_1, CPlayer * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
