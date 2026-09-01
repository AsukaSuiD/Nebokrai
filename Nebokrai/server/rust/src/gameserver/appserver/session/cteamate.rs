//! Командный разъём `CTeamate`, принадлежащий игроку в GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/session/cteamate.cpp`. Материализована достигнутая часть
//! приглашения и входа: идентификатор разъёма, владелец-игрок, снимок региона
//! и имени, а также `Serialize`, который `OnPlugInserted` вкладывает в
//! клиентское сообщение `0xBFD03`. Base ended-флаг передаётся owner-ом
//! реестра, чтобы wire не расходился с `CPlug::Serialize`. Локальный выход
//! доведён до членства игрока
//! и сообщения `0xBFD05`. Достигнутые обработчики распределения и чата
//! создают `0xBFD08/09` из типизированных владельцев сессии. Регион, состояние
//! участника и удалённое восстановление используют тот же типизированный
//! разъём. `IsPlugAvailable` сохраняет точное пятиминутное окно remote-owner-а
//! и reconnect-сигнал для повторной публикации team snapshot; остальные
//! недостигнутые ветви сохранены ниже как RAW.

use crate::gameserver::appserver::legacycodec::LegacyWriter;

const PLAYER_LOSE_TIMEOUT_MS: u32 = 300_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TeamMateAvailability {
    Available,
    Recovered,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CTeamate {
    plug_id: i32,
    owner_type: i32,
    owner_id: i32,
    owner_region_id: i32,
    owner_name: Vec<u8>,
    player_lose_time_stamp: u32,
}

impl CTeamate {
    pub(crate) fn new(
        plug_id: i32,
        owner_id: i32,
        owner_region_id: i32,
        owner_name: &[u8],
    ) -> Self {
        Self::new_owned(plug_id, 400, owner_id, owner_region_id, owner_name)
    }

    pub(crate) fn new_owned(
        plug_id: i32,
        owner_type: i32,
        owner_id: i32,
        owner_region_id: i32,
        owner_name: &[u8],
    ) -> Self {
        Self {
            plug_id,
            owner_type,
            owner_id,
            owner_region_id,
            owner_name: owner_name
                .split(|byte| *byte == 0)
                .next()
                .unwrap_or_default()
                .to_vec(),
            player_lose_time_stamp: 0,
        }
    }

    pub(crate) const fn plug_id(&self) -> i32 {
        self.plug_id
    }

    pub(crate) const fn owner_id(&self) -> i32 {
        self.owner_id
    }

    pub(crate) const fn owner_type(&self) -> i32 {
        self.owner_type
    }

    pub(crate) const fn owner_region_id(&self) -> i32 {
        self.owner_region_id
    }

    pub(crate) const fn set_owner_region_id(&mut self, owner_region_id: i32) {
        self.owner_region_id = owner_region_id;
    }

    /// Exact `IsPlugAvailable` reconnect window. Отсутствующий локальный
    /// player запускает пятиминутный срок только если его сохранённый регион
    /// принадлежит этому GameServer; появление owner-а очищает срок и требует
    /// повторной публикации полного team snapshot.
    pub(crate) const fn availability(
        &mut self,
        now_ms: u32,
        owner_is_local: bool,
        owner_region_is_local: bool,
    ) -> TeamMateAvailability {
        if owner_is_local {
            if self.player_lose_time_stamp != 0 {
                self.player_lose_time_stamp = 0;
                return TeamMateAvailability::Recovered;
            }
            return TeamMateAvailability::Available;
        }
        if self.player_lose_time_stamp == 0 {
            if owner_region_is_local {
                self.player_lose_time_stamp = now_ms;
            }
            return TeamMateAvailability::Available;
        }
        if self
            .player_lose_time_stamp
            .wrapping_add(PLAYER_LOSE_TIMEOUT_MS)
            <= now_ms
        {
            TeamMateAvailability::Expired
        } else {
            TeamMateAvailability::Available
        }
    }

    pub(crate) fn serialize(&self, output: &mut Vec<u8>, plug_ended: i32) {
        let mut writer = LegacyWriter::new(output);
        writer.write_i32(5);
        writer.write_i32(self.owner_type);
        writer.write_i32(self.owner_id);
        writer.write_i32(plug_ended);
        writer.write_i32(self.owner_region_id);
        writer.write_c_string(&self.owner_name);
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp

// ============================================================================
// FUNCTION: CTeamate::SetOwnerRegionID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:107
// RVA: 0x000EA270
// ADDRESS: 004ea270
// PROTOTYPE: void __thiscall SetOwnerRegionID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::OnPlugEnded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:332
// RVA: 0x000EA2A0
// ADDRESS: 004ea2a0
// PROTOTYPE: int __thiscall OnPlugEnded(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:131
// RVA: 0x000EA3C0
// ADDRESS: 004ea3c0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::OnChangeState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:163
// RVA: 0x000EA430
// ADDRESS: 004ea430
// PROTOTYPE: int __thiscall OnChangeState(long param_1, long param_2, uchar * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::CTeamate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:16
// RVA: 0x000EA850
// ADDRESS: 004ea850
// PROTOTYPE: undefined __thiscall CTeamate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::~CTeamate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:22
// RVA: 0x000EA880
// ADDRESS: 004ea880
// PROTOTYPE: void __thiscall ~CTeamate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::SetOwnerName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:118
// RVA: 0x000EA8E0
// ADDRESS: 004ea8e0
// PROTOTYPE: void __thiscall SetOwnerName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:146
// RVA: 0x000EA910
// ADDRESS: 004ea910
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::OnPlugInserted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:294
// RVA: 0x000EA9C0
// ADDRESS: 004ea9c0
// PROTOTYPE: int __thiscall OnPlugInserted(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::IsPlugAvailable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp:33
// RVA: 0x000EAB20
// ADDRESS: 004eab20
// PROTOTYPE: int __thiscall IsPlugAvailable(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005b863c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp
// RVA: 0x001B863C
// ADDRESS: 005b863c
// PROTOTYPE: undefined Catch@005b863c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005b86fd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp
// RVA: 0x001B86FD
// ADDRESS: 005b86fd
// PROTOTYPE: undefined Catch@005b86fd()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005b8ed6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cteamate.cpp
// RVA: 0x001B8ED6
// ADDRESS: 005b8ed6
// PROTOTYPE: undefined Catch@005b8ed6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
