//! Статус корпуса: MIXED (`CGodsBattleMgr` startup snapshot реализован,
//! остальной owner сохранён как RAW pseudocode).
//! Декомпилятор: Ghidra 12.1.2
//! Сырой C++ ниже после typed owner-а является комментарием, а не
//! Rust-реализацией.
//!
//! Реализованный RVA `0x000AB260` сохраняет exact wire, section-local clear,
//! намеренное append-поведение faction rules и обе внутренние audit-записи.
//! Безразмерный pointer и 256-байтный временный C-string buffer заменены
//! bounded slice/cursor и owned bytes; обрыв возвращает typed error после уже
//! завершённого prefix-а вместо неназначаемого legacy UB. Region/gameplay
//! lifecycle и остальные методы manager-а пока остаются неизвестными здесь.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.h

use crate::setup::godsbattleconf::{
    CGodsBattleConf, GodsBattleDecodeError, GodsBattleDecodeReport,
};

// Точные GBK payload из GameServer .rdata VA `0x00651870` и `0x00651850`.
const REVISE_MONEY_CONFIGURATION_ERROR: &[u8] =
    b"\xC9\xF1\xD6\xAE\xC1\xA6\xD0\xDE\xD5\xFD\xD6\xB5\xC5\xE4\xD6\xC3\xB4\xED\xCE\xF3\xA3\xA1";
const EMPTY_DIE_BACK_CONFIGURATION: &[u8] =
    b"\xA1\xBE\xD6\xEE\xC9\xF1\xD6\xAE\xD5\xBD\xA1\xBF\xCB\xC0\xCD\xF6\xBB\xD8\xB3\xC7\xB5\xC4\xC5\xE4\xD6\xC3\xCE\xAA\xBF\xD5";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CGodsBattleMgr {
    configuration: CGodsBattleConf,
}

impl CGodsBattleMgr {
    pub(crate) const fn configuration(&self) -> &CGodsBattleConf {
        &self.configuration
    }

    /// Воспроизводит `CGodsBattleMgr::DecordFromByteArray` RVA `0x000AB260`,
    /// включая оба внутренних audit side effect-а в исходных позициях.
    pub(crate) fn decord_from_byte_array<AddLogText, PutStringToFile>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        add_log_text: &mut AddLogText,
        put_string_to_file: &mut PutStringToFile,
    ) -> Result<GodsBattleDecodeReport, GodsBattleDecodeError>
    where
        AddLogText: FnMut(&[u8]),
        PutStringToFile: FnMut(&str, &[u8]),
    {
        self.configuration.decord_from_byte_array(
            source,
            cursor,
            || add_log_text(REVISE_MONEY_CONFIGURATION_ERROR),
            || put_string_to_file("godsbattleLog", EMPTY_DIE_BACK_CONFIGURATION),
        )
    }
}

// ============================================================================
// FUNCTION: CGodsBattleMgr::AddRegionSet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.h:173
// RVA: 0x0009D160
// ADDRESS: 0049d160
// PROTOTYPE: void __thiscall AddRegionSet(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetFactionXYD
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:932
// RVA: 0x000A5B50
// ADDRESS: 004a5b50
// PROTOTYPE: ulong __thiscall GetFactionXYD(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::UpdateXYD
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:940
// RVA: 0x000A5B70
// ADDRESS: 004a5b70
// PROTOTYPE: void __thiscall UpdateXYD(uchar param_1, int param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetTopTenSZL
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:969
// RVA: 0x000A5C00
// ADDRESS: 004a5c00
// PROTOTYPE: void __thiscall GetTopTenSZL(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::SetFactionXYD
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:923
// RVA: 0x000A5CA0
// ADDRESS: 004a5ca0
// PROTOTYPE: bool __thiscall SetFactionXYD(int param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::IsPlayerContendSymbol
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:438
// RVA: 0x000A5E00
// ADDRESS: 004a5e00
// PROTOTYPE: bool __thiscall IsPlayerContendSymbol(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetPlayerSZLLev
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1025
// RVA: 0x000A5EF0
// ADDRESS: 004a5ef0
// PROTOTYPE: bool __thiscall GetPlayerSZLLev(ulong * param_1, ulong * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::AssignPlayerFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1038
// RVA: 0x000A5F60
// ADDRESS: 004a5f60
// PROTOTYPE: bool __thiscall AssignPlayerFaction(CPlayer * param_1, ulong * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::CalAddSZL
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:846
// RVA: 0x000A6040
// ADDRESS: 004a6040
// PROTOTYPE: bool __thiscall CalAddSZL(CPlayer * param_1, CPlayer * param_2, ulong * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::CalMinSZL
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:978
// RVA: 0x000A6120
// ADDRESS: 004a6120
// PROTOTYPE: bool __thiscall CalMinSZL(CPlayer * param_1, CPlayer * param_2, ulong * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::IsGodsBattleRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1216
// RVA: 0x000A6200
// ADDRESS: 004a6200
// PROTOTYPE: bool __thiscall IsGodsBattleRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetReturnPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1340
// RVA: 0x000A6230
// ADDRESS: 004a6230
// PROTOTYPE: bool __thiscall GetReturnPoint(long param_1, ulong param_2, long * param_3, long * param_4, long * param_5, long * param_6, long * param_7, long * param_8)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::DelObj
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:183
// RVA: 0x000A6640
// ADDRESS: 004a6640
// PROTOTYPE: void __thiscall DelObj(int param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnEnterContend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:410
// RVA: 0x000A66E0
// ADDRESS: 004a66e0
// PROTOTYPE: void __thiscall OnEnterContend(CPlayer * param_1, long param_2, char * param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetNpcNameByMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1220
// RVA: 0x000A67B0
// ADDRESS: 004a67b0
// PROTOTYPE: bool __thiscall GetNpcNameByMonster(CMonster * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetAlreadyDieCount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1320
// RVA: 0x000A6850
// ADDRESS: 004a6850
// PROTOTYPE: ulong __thiscall GetAlreadyDieCount(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::OnSZLMin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1372
// RVA: 0x000A6920
// ADDRESS: 004a6920
// PROTOTYPE: void __thiscall OnSZLMin(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::RemoveObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:138
// RVA: 0x000A7270
// ADDRESS: 004a7270
// PROTOTYPE: void __thiscall RemoveObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::CancelContendByPlayerID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:452
// RVA: 0x000A8010
// ADDRESS: 004a8010
// PROTOTYPE: bool __thiscall CancelContendByPlayerID(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::~CServerGodsBattleRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:19
// RVA: 0x000A86B0
// ADDRESS: 004a86b0
// PROTOTYPE: void __thiscall ~CServerGodsBattleRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::CServerGodsBattleRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:14
// RVA: 0x000A90F0
// ADDRESS: 004a90f0
// PROTOTYPE: undefined __thiscall CServerGodsBattleRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::GetObjFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:312
// RVA: 0x000A91B0
// ADDRESS: 004a91b0
// PROTOTYPE: Fation __thiscall GetObjFaction(int param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::AddContend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:361
// RVA: 0x000A9270
// ADDRESS: 004a9270
// PROTOTYPE: void __thiscall AddContend(CPlayer * param_1, long param_2, char * param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::CancelContendBySymbol
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:568
// RVA: 0x000A9590
// ADDRESS: 004a9590
// PROTOTYPE: void __thiscall CancelContendBySymbol(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:592
// RVA: 0x000A9660
// ADDRESS: 004a9660
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004a9779
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:639
// RVA: 0x000A9779
// ADDRESS: 004a9779
// PROTOTYPE: undefined Catch@004a9779()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::BroadCasetToPlayers
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1113
// RVA: 0x000A9860
// ADDRESS: 004a9860
// PROTOTYPE: void __thiscall BroadCasetToPlayers(Fation param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::SetXYD
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:953
// RVA: 0x000A9C40
// ADDRESS: 004a9c40
// PROTOTYPE: void __thiscall SetXYD(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::StrSplit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1161
// RVA: 0x000A9C70
// ADDRESS: 004a9c70
// PROTOTYPE: bool __thiscall StrSplit(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, vector<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>,std::allocator<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>_>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetMonsterCountByNpcName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1237
// RVA: 0x000A9DB0
// ADDRESS: 004a9db0
// PROTOTYPE: ulong __thiscall GetMonsterCountByNpcName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::OnNpcMonsterDie
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1253
// RVA: 0x000A9FC0
// ADDRESS: 004a9fc0
// PROTOTYPE: void __thiscall OnNpcMonsterDie(CMonster * param_1, CBaseObject * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::DecordTopTenFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1054
// RVA: 0x000AA4B0
// ADDRESS: 004aa4b0
// PROTOTYPE: bool __thiscall DecordTopTenFromByteArray(uchar * param_1, long * param_2, vector<_TOPTEN,std::allocator<_TOPTEN>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::OnEnterContend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1078
// RVA: 0x000AA5D0
// ADDRESS: 004aa5d0
// PROTOTYPE: bool __thiscall OnEnterContend(CPlayer * param_1, CNpc * param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::OnNpcUpdate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1186
// RVA: 0x000AA8A0
// ADDRESS: 004aa8a0
// PROTOTYPE: void __thiscall OnNpcUpdate(CNpc * param_1, ulong param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::CGodsBattleMgr
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:893
// RVA: 0x000AAAF0
// ADDRESS: 004aaaf0
// PROTOTYPE: undefined __thiscall CGodsBattleMgr(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:912
// RVA: 0x000AABF0
// ADDRESS: 004aabf0
// PROTOTYPE: CGodsBattleMgr * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnNpcSetFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:648
// RVA: 0x000AAC60
// ADDRESS: 004aac60
// PROTOTYPE: void __thiscall OnNpcSetFaction(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnMonsterDie
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:698
// RVA: 0x000AB150
// ADDRESS: 004ab150
// PROTOTYPE: void __thiscall OnMonsterDie(CMonster * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::GetReturnPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:715
// RVA: 0x000AB1C0
// ADDRESS: 004ab1c0
// PROTOTYPE: void __thiscall GetReturnPoint(CPlayer * param_1, long * param_2, long * param_3, long * param_4, long * param_5, long * param_6, long * param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::RefreshMonsterForNpc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1150
// RVA: 0x000AB740
// ADDRESS: 004ab740
// PROTOTYPE: bool __thiscall RefreshMonsterForNpc(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, ulong param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::AddObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:24
// RVA: 0x000AB7C0
// ADDRESS: 004ab7c0
// PROTOTYPE: void __thiscall AddObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnChangeFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:226
// RVA: 0x000ABC30
// ADDRESS: 004abc30
// PROTOTYPE: bool __thiscall OnChangeFaction(int param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnContendTimeOver
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:476
// RVA: 0x000ABDF0
// ADDRESS: 004abdf0
// PROTOTYPE: void __thiscall OnContendTimeOver(tagContend * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f3739
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp
// RVA: 0x000F3739
// ADDRESS: 004f3739
// PROTOTYPE: undefined Catch@004f3739()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f38a6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp
// RVA: 0x000F38A6
// ADDRESS: 004f38a6
// PROTOTYPE: undefined Catch@004f38a6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

















































































































































































// COMPONENT_VARIANT_END: GameServer
