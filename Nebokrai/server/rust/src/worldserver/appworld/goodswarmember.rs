//! Владелец списка участников и faction-count Goods War WorldServer.
//!
//! `RequestCountList` RVA `0x000A1C60`, `AppendOneFaction2Count`
//! `0x000A1DD0`, `RefreshMembers/RefreshlistFid/RefreshAll`
//! `0x000A2090/0x000A21D0/0x000A22C0`, `DeleteOneMember` `0x000A27A0` и
//! `InsertOneFaction` `0x000A2960` имеют статус `IMPLEMENTED`; DB reload,
//! остальные mutations и `FactionWin` ниже пока `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Декомпилятор: Ghidra 12.1.2
//! Полный декомпилят хранится локально и не входит в распространяемый код.
//!
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`;
//! исходный owner:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp`.
//! `m_member` является signed ordered map `player ID -> faction ID`, а
//! `m_setGoodsWarFactionIdList` — signed ordered set. `BTreeMap/BTreeSet`
//! заменяют только MSVC tree. Count-list сохраняет отдельный list-order:
//! записи сортируются по убыванию count, новая запись ставится перед первой с
//! `old_count <= new_count`, прежняя запись той же faction ищется как C-строка
//! по имени и удаляется уже после вставки. Wire `0x7FF20` сохраняет operation
//! `2/4/5/16`, ordered пары и terminal `(0,0)`; `0x7FF21` публикует максимум
//! первые пять count-записей как raw `char[20] + long`.
//! В `AppendOneFaction2Count` сохранён observable ранний return: первая запись
//! имени публикует `0x7FF21`, замена уже существующей записи — нет.
//! Значимые bytes имени сравниваются до первого NUL, как старый `strcmp`;
//! остаток fixed wire-поля Rust обнуляет вместо передачи недоказанного
//! allocator residue из старого неинициализированного heap-блока.
//! Повреждённые `local_38/local_8` в RAW `DeleteOneMember/InsertOneFaction`
//! являются потерянными decompiler stack-alias входного параметра: PDB
//! сохраняет соответствующие сигнатуры, а exact caller `0x60139` кладёт
//! прочитанный literal ID непосредственно перед каждым вызовом.
//!
//! Старый unbounded copy faction-name в `char[20]` мог перезаписать count и
//! links. Это внутренний UB, а не протокол: Rust останавливает append при
//! visible имени длиннее 19 bytes до изменения faction/count state. DB/ADO
//! reload не имитируется и остаётся своим сырым owner-ом; достигнутая структура
//! принимает уже загруженное состояние через обычные safe collections.

use std::collections::{BTreeMap, BTreeSet};

use crate::nets::networld::message::CMessage;

const GOODS_WAR_STATE_MESSAGE_TYPE: i32 = 0x7FF20;
const GOODS_WAR_COUNT_MESSAGE_TYPE: i32 = 0x7FF21;
const FACTION_NAME_CAPACITY: usize = 20;
const MAX_PUBLISHED_COUNTS: usize = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoodsWarFactionSnapshot {
    pub(crate) name: Vec<u8>,
    pub(crate) goods_war_count: i32,
}

pub(crate) trait GoodsWarMemberContext {
    type Block;

    fn faction_snapshot(
        &mut self,
        faction_id: i32,
    ) -> Result<Option<GoodsWarFactionSnapshot>, Self::Block>;

    /// Выполняет concrete `CFaction::SetGoodsWarCount` и возвращает
    /// нормализованное сохранённое значение.
    fn set_faction_goods_war_count(
        &mut self,
        faction_id: i32,
        count: i32,
    ) -> Result<i32, Self::Block>;

    fn send_all(&mut self, message: &CMessage) -> i32;
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GoodsWarFactionCount {
    name: [u8; FACTION_NAME_CAPACITY],
    count: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoodsWarMutationReport {
    pub(crate) target_found: bool,
    pub(crate) state_changed: bool,
    pub(crate) delivery: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoodsWarRefreshReport {
    pub(crate) members_delivery: i32,
    pub(crate) counts_delivery: i32,
    pub(crate) faction_ids_delivery: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GoodsWarMemberBlock<ContextBlock> {
    Context(ContextBlock),
    FactionNameWouldOverflow {
        faction_id: i32,
        visible_len: usize,
        capacity: usize,
    },
}

/// Safe reached-state исходного `CGoodsWarMember` без pointer/list ABI.
#[derive(Default)]
pub(crate) struct CGoodsWarMember {
    members: BTreeMap<i32, i32>,
    faction_ids: BTreeSet<i32>,
    counts: Vec<GoodsWarFactionCount>,
}

impl CGoodsWarMember {
    pub(crate) const fn with_reached_empty_state() -> Self {
        Self {
            members: BTreeMap::new(),
            faction_ids: BTreeSet::new(),
            counts: Vec::new(),
        }
    }

    fn send_members<Context: GoodsWarMemberContext + ?Sized>(
        &self,
        context: &mut Context,
    ) -> i32 {
        let mut message = CMessage::new(GOODS_WAR_STATE_MESSAGE_TYPE);
        if self.members.is_empty() {
            message.base_mut().add_long(5);
        } else {
            message.base_mut().add_long(4);
            for (&player_id, &faction_id) in &self.members {
                message.base_mut().add_long(player_id);
                message.base_mut().add_long(faction_id);
            }
            message.base_mut().add_long(0);
            message.base_mut().add_long(0);
        }
        context.send_all(&message)
    }

    fn send_count_list<Context: GoodsWarMemberContext + ?Sized>(
        &self,
        context: &mut Context,
    ) -> i32 {
        let published_count = self.counts.len().min(MAX_PUBLISHED_COUNTS);
        let mut message = CMessage::new(GOODS_WAR_COUNT_MESSAGE_TYPE);
        message.base_mut().add_long(published_count as i32);
        for record in self.counts.iter().take(published_count) {
            message.base_mut().add(&record.name);
            message.base_mut().add_long(record.count);
        }
        context.send_all(&message)
    }

    fn send_faction_ids<Context: GoodsWarMemberContext + ?Sized>(
        &self,
        context: &mut Context,
    ) -> i32 {
        let mut message = CMessage::new(GOODS_WAR_STATE_MESSAGE_TYPE);
        message.base_mut().add_long(0x10);
        message.base_mut().add_long(self.faction_ids.len() as i32);
        for &faction_id in &self.faction_ids {
            message.base_mut().add_long(faction_id);
        }
        context.send_all(&message)
    }

    /// Удаляет literal player key и только при hit публикует operation `2`.
    pub(crate) fn delete_one_member<Context: GoodsWarMemberContext + ?Sized>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> GoodsWarMutationReport {
        if self.members.remove(&player_id).is_none() {
            return GoodsWarMutationReport::default();
        }
        let mut message = CMessage::new(GOODS_WAR_STATE_MESSAGE_TYPE);
        message.base_mut().add_long(2);
        message.base_mut().add_long(player_id);
        GoodsWarMutationReport {
            target_found: true,
            state_changed: true,
            delivery: Some(context.send_all(&message)),
        }
    }

    /// Вставляет существующую faction и при новой записи публикует весь set.
    pub(crate) fn insert_one_faction<Context>(
        &mut self,
        faction_id: i32,
        context: &mut Context,
    ) -> Result<GoodsWarMutationReport, GoodsWarMemberBlock<Context::Block>>
    where
        Context: GoodsWarMemberContext + ?Sized,
    {
        if context
            .faction_snapshot(faction_id)
            .map_err(GoodsWarMemberBlock::Context)?
            .is_none()
        {
            return Ok(GoodsWarMutationReport::default());
        }
        if !self.faction_ids.insert(faction_id) {
            return Ok(GoodsWarMutationReport {
                target_found: true,
                ..GoodsWarMutationReport::default()
            });
        }
        Ok(GoodsWarMutationReport {
            target_found: true,
            state_changed: true,
            delivery: Some(self.send_faction_ids(context)),
        })
    }

    /// Увеличивает count и переставляет запись; публикация есть только при
    /// первом появлении имени, как в exact раннем return старого owner-а.
    pub(crate) fn append_one_faction_to_count<Context>(
        &mut self,
        faction_id: i32,
        context: &mut Context,
    ) -> Result<GoodsWarMutationReport, GoodsWarMemberBlock<Context::Block>>
    where
        Context: GoodsWarMemberContext + ?Sized,
    {
        let Some(snapshot) = context
            .faction_snapshot(faction_id)
            .map_err(GoodsWarMemberBlock::Context)?
        else {
            return Ok(GoodsWarMutationReport::default());
        };
        let visible_name = legacy_c_string_prefix(&snapshot.name);
        if visible_name.len() >= FACTION_NAME_CAPACITY {
            return Err(GoodsWarMemberBlock::FactionNameWouldOverflow {
                faction_id,
                visible_len: visible_name.len(),
                capacity: FACTION_NAME_CAPACITY,
            });
        }
        let mut name = [0_u8; FACTION_NAME_CAPACITY];
        name[..visible_name.len()].copy_from_slice(visible_name);

        let old_position = self
            .counts
            .iter()
            .position(|entry| legacy_c_string_prefix(&entry.name) == visible_name);
        let next_count = snapshot.goods_war_count.wrapping_add(1);
        let saved_count = context
            .set_faction_goods_war_count(faction_id, next_count)
            .map_err(GoodsWarMemberBlock::Context)?;
        let insert_position = self
            .counts
            .iter()
            .position(|entry| entry.count <= saved_count)
            .unwrap_or(self.counts.len());
        self.counts.insert(
            insert_position,
            GoodsWarFactionCount {
                name,
                count: saved_count,
            },
        );
        if let Some(old_position) = old_position {
            let shifted_old_position = if old_position >= insert_position {
                old_position + 1
            } else {
                old_position
            };
            self.counts.remove(shifted_old_position);
        }
        Ok(GoodsWarMutationReport {
            target_found: true,
            state_changed: true,
            delivery: old_position
                .is_none()
                .then(|| self.send_count_list(context)),
        })
    }

    /// Публикует members, count top-five и faction IDs строго в этом порядке.
    pub(crate) fn refresh_all<Context: GoodsWarMemberContext + ?Sized>(
        &self,
        context: &mut Context,
    ) -> GoodsWarRefreshReport {
        GoodsWarRefreshReport {
            members_delivery: self.send_members(context),
            counts_delivery: self.send_count_list(context),
            faction_ids_delivery: self.send_faction_ids(context),
        }
    }
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    &value[..value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len())]
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp

// ============================================================================
// FUNCTION: Catch@00483d19
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp
// RVA: 0x00083D19
// ADDRESS: 00483d19
// PROTOTYPE: undefined Catch@00483d19()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00484016
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp
// RVA: 0x00084016
// ADDRESS: 00484016
// PROTOTYPE: undefined Catch@00484016()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00484145
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp
// RVA: 0x00084145
// ADDRESS: 00484145
// PROTOTYPE: undefined Catch@00484145()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::RequestCountList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:171
// RVA: 0x000A1C60
// ADDRESS: 004a1c60
// PROTOTYPE: void __thiscall RequestCountList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::AppendOneFaction2Count
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:202
// RVA: 0x000A1DD0
// ADDRESS: 004a1dd0
// PROTOTYPE: void __thiscall AppendOneFaction2Count(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::DelOneFactionfCount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:289
// RVA: 0x000A1FA0
// ADDRESS: 004a1fa0
// PROTOTYPE: bool __thiscall DelOneFactionfCount(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::RefreshMembers
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:45
// RVA: 0x000A2090
// ADDRESS: 004a2090
// PROTOTYPE: void __thiscall RefreshMembers(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::RefreshlistFid
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:325
// RVA: 0x000A21D0
// ADDRESS: 004a21d0
// PROTOTYPE: void __thiscall RefreshlistFid(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::RefreshAll
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:361
// RVA: 0x000A22C0
// ADDRESS: 004a22c0
// PROTOTYPE: void __thiscall RefreshAll(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::reInitDB
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:118
// RVA: 0x000A22E0
// ADDRESS: 004a22e0
// PROTOTYPE: void __thiscall reInitDB(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004a26f0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:158
// RVA: 0x000A26F0
// ADDRESS: 004a26f0
// PROTOTYPE: undefined Catch@004a26f0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004a2742
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:167
// RVA: 0x000A2742
// ADDRESS: 004a2742
// PROTOTYPE: undefined FUN_004a2742()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::IsInFactionIdList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:338
// RVA: 0x000A2760
// ADDRESS: 004a2760
// PROTOTYPE: bool __thiscall IsInFactionIdList(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::DeleteOneMember
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:72
// RVA: 0x000A27A0
// ADDRESS: 004a27a0
// PROTOTYPE: void __thiscall DeleteOneMember(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::DeleteMembersByFactionId
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:88
// RVA: 0x000A2850
// ADDRESS: 004a2850
// PROTOTYPE: void __thiscall DeleteMembersByFactionId(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::InsertOneFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:343
// RVA: 0x000A2960
// ADDRESS: 004a2960
// PROTOTYPE: void __thiscall InsertOneFaction(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::~CGoodsWarMember
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:12
// RVA: 0x000A29C0
// ADDRESS: 004a29c0
// PROTOTYPE: void __thiscall ~CGoodsWarMember(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::CGoodsWarMember
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:5
// RVA: 0x000A2A90
// ADDRESS: 004a2a90
// PROTOTYPE: undefined __thiscall CGoodsWarMember(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::MkOne
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:26
// RVA: 0x000A2B60
// ADDRESS: 004a2b60
// PROTOTYPE: bool __thiscall MkOne(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsWarMember::FactionWin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goodswarmember.cpp:369
// RVA: 0x000A2BD0
// ADDRESS: 004a2bd0
// PROTOTYPE: void __thiscall FactionWin(CFaction * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//























// COMPONENT_VARIANT_END: WorldServer
