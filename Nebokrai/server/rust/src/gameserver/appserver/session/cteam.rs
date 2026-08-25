//! Team session `CTeam` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/session/cteam.cpp`. Материализован reached local creation/join
//! prefix: team/leader identity, default shared allocation и exact session +
//! teammate serialization, local leave/leader/kick/disband lifecycle,
//! allocation/chat transitions и их World/client publications. Remote
//! reconstruction, AI/quest и остальные state transitions остаются RAW.

use crate::gameserver::appserver::session::csession::CSession;
use crate::gameserver::appserver::session::cteamate::CTeamate;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CTeam {
    team_id: u32,
    leader_id: i32,
    allocation_scheme: i32,
    team_name: Vec<u8>,
    password: Vec<u8>,
}

impl CTeam {
    pub(crate) const fn new(team_id: u32) -> Self {
        Self {
            team_id,
            leader_id: 0,
            allocation_scheme: 1,
            team_name: Vec::new(),
            password: Vec::new(),
        }
    }

    pub(crate) const fn team_id(&self) -> u32 {
        self.team_id
    }

    pub(crate) const fn leader_id(&self) -> i32 {
        self.leader_id
    }

    pub(crate) const fn set_leader(&mut self, leader_id: i32) {
        self.leader_id = leader_id;
    }

    pub(crate) fn serialize<'a>(
        &self,
        session: &CSession,
        teammates: impl IntoIterator<Item = &'a CTeamate>,
    ) -> Vec<u8> {
        let teammates: Vec<&CTeamate> = teammates.into_iter().collect();
        let mut output = Vec::new();
        output.extend_from_slice(&1_i32.to_le_bytes());
        output.extend_from_slice(&session.minimum_plugs().to_le_bytes());
        output.extend_from_slice(&session.maximum_plugs().to_le_bytes());
        output.extend_from_slice(&session.lifetime().to_le_bytes());
        output.extend_from_slice(&self.team_id.to_le_bytes());
        output.extend_from_slice(&self.team_name);
        output.push(0);
        output.extend_from_slice(&self.password);
        output.push(0);
        output.extend_from_slice(&self.leader_id.to_le_bytes());
        output.extend_from_slice(
            &u32::try_from(teammates.len())
                .unwrap_or(u32::MAX)
                .to_le_bytes(),
        );
        for teammate in teammates {
            teammate.serialize(&mut output);
        }
        output
    }

    pub(crate) const fn allocation_scheme(&self) -> i32 {
        self.allocation_scheme
    }

    pub(crate) const fn set_allocation_scheme(&mut self, allocation_scheme: i32) {
        self.allocation_scheme = allocation_scheme;
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp

// ============================================================================
// FUNCTION: CTeam::SetTeamID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:58
// RVA: 0x00106F00
// ADDRESS: 00506f00
// PROTOTYPE: void __thiscall SetTeamID(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::GetLeader
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:87
// RVA: 0x00106F20
// ADDRESS: 00506f20
// PROTOTYPE: long __thiscall GetLeader(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::GetAllocationScheme
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:614
// RVA: 0x00106F30
// ADDRESS: 00506f30
// PROTOTYPE: ALLOCATION_SCHEME __thiscall GetAllocationScheme(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::KickPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:619
// RVA: 0x00106F40
// ADDRESS: 00506f40
// PROTOTYPE: void __thiscall KickPlayer(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::OnPlugChangeState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:640
// RVA: 0x00107000
// ADDRESS: 00507000
// PROTOTYPE: int __thiscall OnPlugChangeState(long param_1, long param_2, uchar * param_3, int param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::SetLeader
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:68
// RVA: 0x001073E0
// ADDRESS: 005073e0
// PROTOTYPE: void __thiscall SetLeader(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::OnPlugInserted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:542
// RVA: 0x00107450
// ADDRESS: 00507450
// PROTOTYPE: int __thiscall OnPlugInserted(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::OnPlugEnded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:563
// RVA: 0x001074B0
// ADDRESS: 005074b0
// PROTOTYPE: int __thiscall OnPlugEnded(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::SetAllocationScheme
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:602
// RVA: 0x00107510
// ADDRESS: 00507510
// PROTOTYPE: void __thiscall SetAllocationScheme(ALLOCATION_SCHEME param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::GetTeamatesAmount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:156
// RVA: 0x00107590
// ADDRESS: 00507590
// PROTOTYPE: ulong __thiscall GetTeamatesAmount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::GetCurrentServerTeamatesAmount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:171
// RVA: 0x001075E0
// ADDRESS: 005075e0
// PROTOTYPE: ulong __thiscall GetCurrentServerTeamatesAmount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::GetCurrentRegionTeamatesAmount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:269
// RVA: 0x00107640
// ADDRESS: 00507640
// PROTOTYPE: ulong __thiscall GetCurrentRegionTeamatesAmount(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::GetCurrentRegionTeamatesAmount_Alive
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:288
// RVA: 0x001076B0
// ADDRESS: 005076b0
// PROTOTYPE: ulong __thiscall GetCurrentRegionTeamatesAmount_Alive(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::GetCurrentRegionTeamatesAverageLevel_Alive
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:378
// RVA: 0x00107750
// ADDRESS: 00507750
// PROTOTYPE: float __thiscall GetCurrentRegionTeamatesAverageLevel_Alive(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::FindTeamatesInCurrentRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:406
// RVA: 0x00107850
// ADDRESS: 00507850
// PROTOTYPE: CPlayer * __thiscall FindTeamatesInCurrentRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:428
// RVA: 0x001078F0
// ADDRESS: 005078f0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::CTeam
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:25
// RVA: 0x001079C0
// ADDRESS: 005079c0
// PROTOTYPE: undefined __thiscall CTeam(ulong param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::~CTeam
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:33
// RVA: 0x00107A30
// ADDRESS: 00507a30
// PROTOTYPE: void __thiscall ~CTeam(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::CompleteTeamData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:48
// RVA: 0x00107AE0
// ADDRESS: 00507ae0
// PROTOTYPE: void __cdecl CompleteTeamData(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:455
// RVA: 0x00107D10
// ADDRESS: 00507d10
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::OnSessionEnded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:509
// RVA: 0x001080E0
// ADDRESS: 005080e0
// PROTOTYPE: int __thiscall OnSessionEnded(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::OnSessionAborted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:528
// RVA: 0x001081D0
// ADDRESS: 005081d0
// PROTOTYPE: int __thiscall OnSessionAborted(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::GetCurrentRegionTeamates_Alive
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:331
// RVA: 0x00108730
// ADDRESS: 00508730
// PROTOTYPE: void __thiscall GetCurrentRegionTeamates_Alive(map<long,CPlayer*,std::less<long>,std::allocator<std::pair<long_const_,CPlayer*>_>_> * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::QuestTeamData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:42
// RVA: 0x001088A0
// ADDRESS: 005088a0
// PROTOTYPE: void __cdecl QuestTeamData(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:118
// RVA: 0x001088D0
// ADDRESS: 005088d0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::AddQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:190
// RVA: 0x001089C0
// ADDRESS: 005089c0
// PROTOTYPE: void __thiscall AddQuest(ushort param_1, CPlayer * param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::RunScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:230
// RVA: 0x00108B30
// ADDRESS: 00508b30
// PROTOTYPE: void __thiscall RunScript(char * param_1, CPlayer * param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeam::OnSessionStarted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteam.cpp:488
// RVA: 0x00108CA0
// ADDRESS: 00508ca0
// PROTOTYPE: int __thiscall OnSessionStarted(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
