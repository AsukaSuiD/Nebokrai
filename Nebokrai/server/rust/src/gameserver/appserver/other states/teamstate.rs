//! Состояние набора в отряд `CTeamState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/other states/teamstate.cpp`. Материализован достигнутый через
//! client `0x8FF08` lifecycle: имя/пароль, state ID `100006`, бессрочное
//! client-time и additional-data с password bit и исходным количеством один.
//! Общий polymorphic `CState` storage заменён typed player-owned списком;
//! begin/end visual wire публикует message owner. Team-session AI и
//! сериализация остальных способов создания state остаются в RAW ниже.

pub(crate) const TEAM_STATE_ID: i32 = 0x0001_86a6;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CTeamState {
    team_name: Vec<u8>,
    team_password: Vec<u8>,
    last_check_timestamp_ms: u32,
}

impl CTeamState {
    pub(crate) fn new(team_name: Vec<u8>, team_password: Vec<u8>) -> Self {
        Self {
            team_name,
            team_password,
            last_check_timestamp_ms: 0,
        }
    }

    pub(crate) const fn state_id(&self) -> i32 {
        TEAM_STATE_ID
    }

    /// Базовый `CState::GetClientStateTime` для этого бессрочного state.
    pub(crate) const fn client_state_time(&self) -> i32 {
        0
    }

    /// До создания team session исходный owner сообщает самого лидера как
    /// единственного участника; bit 16 отмечает непустой пароль.
    pub(crate) fn initial_additional_data(&self) -> u32 {
        (u32::from(!self.team_password.is_empty()) << 16) | 1
    }

    pub(crate) fn additional_data(&self, teammates: usize) -> u32 {
        (u32::from(!self.team_password.is_empty()) << 16)
            | u32::try_from(teammates).unwrap_or(u32::MAX)
    }

    pub(crate) fn team_name(&self) -> &[u8] {
        &self.team_name
    }

    pub(crate) fn team_password(&self) -> &[u8] {
        &self.team_password
    }

    pub(crate) const fn last_check_timestamp_ms(&self) -> u32 {
        self.last_check_timestamp_ms
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp

// ============================================================================
// FUNCTION: CTeamState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:59
// RVA: 0x001BF800
// ADDRESS: 005bf800
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:77
// RVA: 0x001BF8D0
// ADDRESS: 005bf8d0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:44
// RVA: 0x001BF9A0
// ADDRESS: 005bf9a0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:143
// RVA: 0x001BFA50
// ADDRESS: 005bfa50
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::GetTeamName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:162
// RVA: 0x001BFAB0
// ADDRESS: 005bfab0
// PROTOTYPE: char * __thiscall GetTeamName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::GetTeamPassword
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:167
// RVA: 0x001BFAC0
// ADDRESS: 005bfac0
// PROTOTYPE: char * __thiscall GetTeamPassword(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamStateVisualEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:205
// RVA: 0x001BFAD0
// ADDRESS: 005bfad0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::CTeamState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:34
// RVA: 0x001BFC60
// ADDRESS: 005bfc60
// PROTOTYPE: undefined __thiscall CTeamState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::~CTeamState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:40
// RVA: 0x001BFCA0
// ADDRESS: 005bfca0
// PROTOTYPE: void __thiscall ~CTeamState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:111
// RVA: 0x001BFD20
// ADDRESS: 005bfd20
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::GetAdditionalData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:172
// RVA: 0x001BFDD0
// ADDRESS: 005bfdd0
// PROTOTYPE: ulong __thiscall GetAdditionalData(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::CTeamState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:17
// RVA: 0x001BFE60
// ADDRESS: 005bfe60
// PROTOTYPE: undefined __thiscall CTeamState(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:152
// RVA: 0x001BFF20
// ADDRESS: 005bff20
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
