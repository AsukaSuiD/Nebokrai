//! Владелец союза исторического `WorldServer`.
//!
//! Статус достигнутой save-части `CUnion`, `CUnion::CloneSaveData` RVA
//! `0x000C6380`, `SetChangeData` RVA `0x000C17D0` и `CUnion::IsMember` RVA
//! `0x000BD840`, `CUnion::GetPlayerHeader` RVA `0x000C6320`, а также
//! `CUnion::DelMember` RVA `0x000C2B30`, inert virtual-ы
//! `DubAndSetJobLvl/EditLeaveWord/OperatorTax/SetControbuter/Upgrade` RVA
//! `0x000C18B0/0x000C1EC0/0x000C1ED0/0x000C1F20/0x000C1F30`,
//! `GetEstablishedTime` RVA `0x000C1F00` и compiler-owned destructor RVA
//! `0x000C1F40`, обе перегрузки `GetMemberList` RVA
//! `0x000C1BB0/0x000C2710`, `IsUsingPV/SetMemPV/AbolishMemPV` RVA
//! `0x000C1D00/0x000C1D70/0x000C1DD0` и обе `CheckOperValidate` RVA
//! `0x000C18D0/0x000C1970`, empty enemy-set getter-ы RVA `0x000C1C50`,
//! `IsOwnedCity/IsEnemyFaction/GetOwnedCities/IsHaveEnymyFaction/
//! IsHaveCityEnemyFaction` RVA
//! `0x000C2340/0x000C23A0/0x000C2650/0x000C2810/0x000C2870`, обе
//! `AddOwnedCity`, `DelOwnedCity/ClearOwnedCity/SetOwnedCity` RVA
//! `0x000C2400/0x000C2490/0x000C2510/0x000C2570/0x000C25F0`,
//! `ClearEnemyFation/GetEnemyLeaderOrgnizingID` RVA
//! `0x000C28D0/0x000C2950` и три victor fan-out RVA
//! `0x000C29B0/0x000C2A30/0x000C2AB0`, `UpdatePlayerFactionInfo` RVA
//! `0x000C5770` и `UpdateEnemyFactionToClient/
//! UpdateCityWarEnemyFactionToClient/UpdateOwnedCityToClient` RVA
//! `0x000C5FF0/0x000C60D0/0x000C61B0` — `IMPLEMENTED`;
//! остальной корпус ниже остаётся
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.h`
//! и `union.cpp:1442`.
//!
//! Точный PDB задаёт старый `CUnion` размером `0x50`: signed ID `+0x4`, имя
//! `+0x8`, signed master ID `+0x24`, ordered member-map `+0x28`, `tagTime`
//! `+0x38` и signed dirty-mask `+0x48`. `CloneSaveData` возвращает null при
//! нулевой mask; иначе создаёт отдельного `CUnion`, копирует ровно эти поля и
//! весь member-map в signed key-order. `m_ApplyPerson` и
//! `m_dwLastDemiseTime` clone не назначает и DB-owner не читает, поэтому они
//! не получают выдуманного Rust-состояния.
//!
//! `Vec<u8>`, `BTreeMap` и обычный `Clone/Drop` заменяют только MSVC string,
//! tree и `new/delete`. Rust-layout не выдаётся за старый ABI. Конструктор
//! live-союза и `Initial` с organizing callbacks остаются raw; узкий
//! `from_reached_save_state` создаёт только уже доказанную save-проекцию.
//!
//! PDB публикует `CUnion::IsMember(long)` на том же RVA `0x000BD840`, что и
//! faction-вариант: identical-code folding допустим, потому что `m_lID` и
//! `m_Members` у обоих concrete owners имеют одинаковые offsets `+0x4/+0x28`.
//! Exact EXE ищет входной signed ID и возвращает собственный ID либо `0`.
//!
//! `DelMember` не читает union receiver. Машинный проход
//! `0x004C2B30..0x004C2B8B` подтверждает единый входной faction ID для `find`
//! и `operator[]`, вопреки повреждённому RAW имени stack-slot. Поэтому lookup
//! и два faction-callback-а материализованы у владельца map как
//! `COrganizingCtrl::detach_union_member`; это технический перенос singleton-
//! доступа, а не изменение контракта `CUnion`.
//! `GetPlayerHeader` при неположительном master-faction ID возвращает `0`.
//! Иначе он ищет master-faction в controller map и вызывает именно
//! `GetMasterID`, не рекурсивный `GetPlayerHeader`; miss/null map-value также
//! дают `0`. Недостигнутый Rust master ID у найденной concrete faction остаётся
//! typed-блокировкой caller-а, а не выдуманным нулём.
//! Пять inert virtual-ов машинно не читают receiver или аргументы: title/job
//! возвращает `true`, edit/tax/upgrade — `false`, contributor-mutator ничего не
//! делает. Эти результаты сохранены буквально, а не заменены предполагаемой
//! faction-логикой. `GetEstablishedTime` возвращал const reference на точные
//! `0x10` bytes; Copy-значение Rust отделяет lifetime без изменения данных.
//! Destructor до следующего PDB-символа освобождает только member-map, string и
//! base storage. Обычный `Drop` Rust-полей заменяет MSVC tree/string cleanup;
//! ручного destructor callback-а нет.
//! Обе `GetMemberList` сначала очищают caller-list, затем проходят member-map в
//! signed key-order. ID-вариант возвращает ключи; organizing-вариант для
//! каждого положительного ключа выполняет nullable faction lookup, пропускает
//! miss/null и не добавляет один pointer дважды. Свежий `Vec` заменяет только
//! list clear/nodes, а lookup передаётся явно вместо singleton-а.
//! Три purview-owner-а проверяют индекс `0..10` до map lookup. Query принимает
//! только точный `PST_Permit`; grant меняет лишь `PST_No`, revoke — любое
//! ненулевое состояние. Общий typed-результат перенесён к owner-у
//! `COrganizing::tagMemInfo`, но concrete map и переходы остаются в `CUnion`.
//! Validation сначала преобразует player master-а через controller-wide scan в
//! положительный faction ID. Простая перегрузка требует union-membership и
//! право; target-перегрузка дополнительно отвергает literal equality входных
//! ID, нулевой target и master-faction союза, затем требует членство обоих и
//! ровно `Permit` у manager при отсутствии `Permit` у target.
//! Оба enemy-set getter-а folded в один owner, который не читает receiver и
//! всегда строит пустой set. Остальные read-only proxy используют только
//! положительный master-faction ID и nullable controller lookup; miss даёт
//! `0`, `false` либо пустой owned-city list. Старый miss `GetOwnedCities`
//! возвращал ссылку на уже уничтоженный локальный list. Safe Rust исправляет
//! этот чистый lifetime/UB-дефект, возвращая owned snapshot либо пустой
//! `VecDeque`, не меняя задуманное значение нормальных ветвей.
//! Owned-city fan-out проходит member-map в signed key-order, пропускает
//! неположительные ID и nullable faction lookup. Одиночное и списочное
//! добавление вызывают каждую найденную faction; clear делает то же и всегда
//! возвращал `true`; set обращается только к положительной найденной
//! master-faction. Exact ASM `0x004C2510..0x004C256C` отдельно подтверждает
//! внешний legacy-баг: `DelOwnedCity(region)` не читает `region`, вызывает у
//! master-фракции безаргументный slot `ClearOwnedCity` и возвращает `true`
//! только при найденном ненулевом pointer. Старый Linux-донор исправлял это на
//! обычный delete, но Rust сохраняет машинное поведение целевой версии явно.
//! Enemy clear и три victor-owner-а снова проходят member-map в signed order,
//! пропускают неположительные ID и miss/null faction. Exact vtable slots
//! `+0xC8/+0x130/+0x134/+0x138` подтверждают concrete clear и порядок
//! defence/offense/village счётчиков. Enemy-leader proxy использует только
//! положительную master-faction и slot `+0xC4`; exact `CFaction`-реализация
//! этого slot-а возвращает literal `0`.
//! `UpdatePlayerFactionInfo(0)` проходит все положительные member-faction ID;
//! положительный аргумент выбирает ровно этот ID без membership-check, а
//! отрицательный не вызывает ничего. Каждый найденный target получает
//! concrete `CFaction::UpdatePlayerFactionInfo(0)`; lookup miss/null тихо
//! пропускается.
//! Три client-snapshot wrapper-а выбирают targets по тому же правилу и
//! полностью игнорируют входной `operation`: найденной faction всегда
//! передаётся `(player=0, OP_Update=2)`. Rust вызывает уже восстановленные
//! concrete snapshot-owner-ы напрямую и сохраняет результаты всех send;
//! owned-city serialization block останавливает обход после выполненного
//! prefix-а вместо выдуманного продолжения после safe-границы.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::faction::{
    FactionEnemyDelivery, FactionInitialPropertyBlock, FactionOwnedCityDelivery,
    FactionOwnedCityUpdateBuildError, FactionPropertyDelivery, OwnedCityMutationBuildError,
};
use super::organizing::{
    EOperator, EPurview, EPurviewOwnState, MemberPurviewMutation, TagMemInfo, TagTimeValue,
};
use crate::worldserver::worldserver::game::CGame;

/// Узкая read-only граница controller-wide `IsFactionMaster`.
pub(crate) trait UnionOperatorValidationContext {
    type Block;

    fn faction_id_by_master_player(&self, player_id: i32) -> Result<i32, Self::Block>;
}

/// Узкая read-only граница master-faction proxy без singleton-а.
pub(crate) trait UnionMasterFactionQueryContext {
    fn faction_is_owned_city(&self, faction_id: i32, region_id: i32) -> Option<i32>;

    fn faction_is_enemy_faction(&self, faction_id: i32, enemy_id: i32) -> Option<i32>;

    fn faction_owned_cities(&self, faction_id: i32) -> Option<VecDeque<i32>>;

    fn faction_has_enemy(&self, faction_id: i32) -> Option<bool>;

    fn faction_has_city_war_enemy(&self, faction_id: i32) -> Option<bool>;

    fn faction_enemy_leader_organizing_id(&self, faction_id: i32) -> Option<i32>;
}

/// Узкая mutable-граница faction-map для owned-city virtual dispatch.
pub(crate) trait UnionOwnedCityMutationContext {
    fn faction_add_owned_city(
        &mut self,
        faction_id: i32,
        game: &CGame,
        region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<bool, OwnedCityMutationBuildError>;

    fn faction_add_owned_cities(
        &mut self,
        faction_id: i32,
        game: &CGame,
        region_ids: &VecDeque<i32>,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<bool, OwnedCityMutationBuildError>;

    fn faction_clear_owned_cities(
        &mut self,
        faction_id: i32,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<bool, OwnedCityMutationBuildError>;

    fn faction_set_owned_cities(
        &mut self,
        faction_id: i32,
        game: &CGame,
        region_ids: &VecDeque<i32>,
    ) -> Result<bool, OwnedCityMutationBuildError>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionOwnedCityFanoutReport {
    pub(crate) invoked_faction_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionOwnedCityMutationBlock {
    pub(crate) faction_id: i32,
    pub(crate) completed_faction_ids: Vec<i32>,
    pub(crate) source: OwnedCityMutationBuildError,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionOwnedCityBooleanMutationReport {
    pub(crate) legacy_result: bool,
    pub(crate) invoked_faction_ids: Vec<i32>,
}

/// Узкая mutable-граница standard enemy и victor virtual dispatch.
pub(crate) trait UnionFactionStateMutationContext {
    fn faction_clear_enemy_factions(&mut self, faction_id: i32) -> bool;

    fn faction_add_defence_victor_count(
        &mut self,
        faction_id: i32,
        game: &CGame,
    ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock>;

    fn faction_add_offense_victor_count(
        &mut self,
        faction_id: i32,
        game: &CGame,
    ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock>;

    fn faction_add_village_war_victor_count(
        &mut self,
        faction_id: i32,
        game: &CGame,
    ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionFactionFanoutReport {
    pub(crate) invoked_faction_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionVictorFactionReport {
    pub(crate) faction_id: i32,
    pub(crate) deliveries: Vec<FactionPropertyDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionVictorFanoutReport {
    pub(crate) factions: Vec<UnionVictorFactionReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionVictorMutationBlock {
    pub(crate) faction_id: i32,
    pub(crate) completed_factions: Vec<UnionVictorFactionReport>,
    pub(crate) source: FactionInitialPropertyBlock,
}

/// Read-only faction callback для обновления online player-состояния.
pub(crate) trait UnionPlayerRefreshContext {
    fn faction_update_player_info(
        &self,
        faction_id: i32,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> Option<Vec<i32>>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionFactionPlayerRefreshReport {
    pub(crate) faction_id: i32,
    pub(crate) refreshed_player_ids: Vec<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionPlayerRefreshReport {
    pub(crate) factions: Vec<UnionFactionPlayerRefreshReport>,
}

/// Read-only faction callback для полных enemy/owned-city snapshot-ов.
pub(crate) trait UnionClientSnapshotContext {
    fn faction_update_enemy_snapshot(
        &self,
        faction_id: i32,
        game: &CGame,
    ) -> Option<Vec<FactionEnemyDelivery>>;

    fn faction_update_city_war_enemy_snapshot(
        &self,
        faction_id: i32,
        game: &CGame,
    ) -> Option<Vec<FactionEnemyDelivery>>;

    fn faction_update_owned_city_snapshot(
        &self,
        faction_id: i32,
        game: &CGame,
    ) -> Result<Option<Vec<FactionOwnedCityDelivery>>, FactionOwnedCityUpdateBuildError>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionEnemySnapshotFactionReport {
    pub(crate) faction_id: i32,
    pub(crate) deliveries: Vec<FactionEnemyDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionEnemySnapshotReport {
    pub(crate) factions: Vec<UnionEnemySnapshotFactionReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionOwnedCitySnapshotFactionReport {
    pub(crate) faction_id: i32,
    pub(crate) deliveries: Vec<FactionOwnedCityDelivery>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionOwnedCitySnapshotReport {
    pub(crate) factions: Vec<UnionOwnedCitySnapshotFactionReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnionOwnedCitySnapshotBlock {
    pub(crate) faction_id: i32,
    pub(crate) completed_factions: Vec<UnionOwnedCitySnapshotFactionReport>,
    pub(crate) source: FactionOwnedCityUpdateBuildError,
}

/// Поля `CUnion`, которые буквально копирует и читает save-цепочка.
pub(crate) struct CUnion {
    union_id: i32,
    name: Vec<u8>,
    master_id: i32,
    members: BTreeMap<i32, TagMemInfo>,
    established_time: TagTimeValue,
    change_data_type: i32,
}

impl CUnion {
    /// Создаёт достигнутую save-проекцию, не имитируя live `Initial`.
    pub(crate) fn from_reached_save_state(
        union_id: i32,
        name: Vec<u8>,
        master_id: i32,
        members: BTreeMap<i32, TagMemInfo>,
        established_time: TagTimeValue,
        change_data_type: i32,
    ) -> Self {
        Self {
            union_id,
            name,
            master_id,
            members,
            established_time,
            change_data_type,
        }
    }

    /// Воспроизводит nullable результат virtual `CloneSaveData`.
    pub(crate) fn clone_save_data(&self) -> Option<Self> {
        (self.change_data_type != 0).then(|| Self {
            union_id: self.union_id,
            name: self.name.clone(),
            master_id: self.master_id,
            members: self.members.clone(),
            established_time: self.established_time,
            change_data_type: self.change_data_type,
        })
    }

    /// Применяет точную bit-mask семантику virtual `SetChangeData`.
    pub(crate) fn set_change_data(&mut self, change_data_type: i32) {
        if change_data_type == 0 {
            self.change_data_type = 0;
        } else if self.change_data_type & change_data_type == 0 {
            self.change_data_type |= change_data_type;
        }
    }

    pub(crate) const fn union_id(&self) -> i32 {
        self.union_id
    }

    pub(crate) fn name(&self) -> &[u8] {
        &self.name
    }

    pub(crate) const fn master_id(&self) -> i32 {
        self.master_id
    }

    /// Возвращает byte-exact время учреждения; старый const-reference был Copy.
    pub(crate) const fn established_time(&self) -> TagTimeValue {
        self.established_time
    }

    /// Legacy union не менял title/job и всегда сообщал успех.
    pub(crate) const fn dub_and_set_job_level(
        &self,
        _manager_id: i32,
        _target_id: i32,
        _title: &[u8],
        _job_level: i32,
    ) -> bool {
        true
    }

    /// У union нет leave-word мутации; virtual всегда возвращал `false`.
    pub(crate) const fn edit_leave_word(
        &self,
        _player_id: i32,
        _word_id: i32,
        _operation: EOperator,
    ) -> bool {
        false
    }

    /// У union нет tax-операции; оба аргумента игнорировались.
    pub(crate) const fn operator_tax(&self, _player_id: i32, _operation: i32) -> bool {
        false
    }

    /// Virtual присутствовал в общем interface, но union не менял state.
    pub(crate) const fn set_contributor(
        &self,
        _requester_id: i32,
        _target_id: i32,
        _enabled: bool,
    ) {}

    /// Union-upgrade отсутствовал и всегда возвращал `false`.
    pub(crate) const fn upgrade(&self, _player_id: i32) -> bool {
        false
    }

    /// Разрешает header через master-faction, сохраняя legacy `0` на miss.
    pub(crate) fn player_header<Lookup, Block>(
        &self,
        master_player_id_by_faction: Lookup,
    ) -> Result<i32, Block>
    where
        Lookup: FnOnce(i32) -> Result<Option<i32>, Block>,
    {
        if self.master_id <= 0 {
            return Ok(0);
        }
        Ok(master_player_id_by_faction(self.master_id)?.unwrap_or(0))
    }

    pub(crate) const fn members(&self) -> &BTreeMap<i32, TagMemInfo> {
        &self.members
    }

    /// Возвращает новый снимок member-ID в исходном signed map-order.
    pub(crate) fn member_ids_snapshot(&self) -> Vec<i32> {
        self.members.keys().copied().collect()
    }

    /// Union не хранит отдельный standard enemy-set и всегда возвращал пустой.
    pub(crate) fn enemy_factions_snapshot(&self) -> BTreeSet<i32> {
        BTreeSet::new()
    }

    /// Folded city-war getter также всегда возвращал новый пустой set.
    pub(crate) fn city_war_enemy_factions_snapshot(&self) -> BTreeSet<i32> {
        BTreeSet::new()
    }

    /// Разрешает member faction-объекты и сохраняет pointer-identity dedupe.
    pub(crate) fn member_organizings<'a, T, Lookup>(&self, mut lookup: Lookup) -> Vec<&'a T>
    where
        T: ?Sized + 'a,
        Lookup: FnMut(i32) -> Option<&'a T>,
    {
        let mut output: Vec<&'a T> = Vec::new();
        for &member_id in self.members.keys() {
            if member_id < 1 {
                continue;
            }
            let Some(organizing) = lookup(member_id) else {
                continue;
            };
            if output
                .iter()
                .any(|current| std::ptr::eq(*current, organizing))
            {
                continue;
            }
            output.push(organizing);
        }
        output
    }

    /// Возвращает union ID только для существующего faction-member key.
    pub(crate) fn is_member(&self, faction_id: i32) -> i32 {
        if self.members.contains_key(&faction_id) {
            self.union_id
        } else {
            0
        }
    }

    /// Проверяет точное состояние `PST_Permit` одного faction-member права.
    pub(crate) fn is_using_purview(&self, faction_id: i32, purview: i32) -> bool {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return false;
        };
        if faction_id == 0 {
            return false;
        }
        self.members
            .get(&faction_id)
            .is_some_and(|member| member.purview[purview.index()] == EPurviewOwnState::Permit)
    }

    /// Переводит только `PST_No` в `PST_Permit`.
    pub(crate) fn set_member_purview(
        &mut self,
        faction_id: i32,
        purview: i32,
    ) -> MemberPurviewMutation {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return MemberPurviewMutation::InvalidPurview;
        };
        let Some(member) = self.members.get_mut(&faction_id) else {
            return MemberPurviewMutation::MemberNotFound;
        };
        let state = &mut member.purview[purview.index()];
        if *state != EPurviewOwnState::No {
            return MemberPurviewMutation::Unchanged;
        }
        *state = EPurviewOwnState::Permit;
        MemberPurviewMutation::Changed
    }

    /// Переводит любое не-`PST_No` состояние в `PST_No`.
    pub(crate) fn abolish_member_purview(
        &mut self,
        faction_id: i32,
        purview: i32,
    ) -> MemberPurviewMutation {
        let Some(purview) = EPurview::from_wire_value(purview) else {
            return MemberPurviewMutation::InvalidPurview;
        };
        let Some(member) = self.members.get_mut(&faction_id) else {
            return MemberPurviewMutation::MemberNotFound;
        };
        let state = &mut member.purview[purview.index()];
        if *state == EPurviewOwnState::No {
            return MemberPurviewMutation::Unchanged;
        }
        *state = EPurviewOwnState::No;
        MemberPurviewMutation::Changed
    }

    /// Проверяет право player-а через faction-master scan и union membership.
    pub(crate) fn check_operator_validate<Context>(
        &self,
        manager_player_id: i32,
        purview: i32,
        context: &Context,
    ) -> Result<bool, Context::Block>
    where
        Context: UnionOperatorValidationContext,
    {
        let manager_faction_id = context.faction_id_by_master_player(manager_player_id)?;
        if manager_faction_id == 0 || self.is_member(manager_faction_id) == 0 {
            return Ok(false);
        }
        Ok(self.is_using_purview(manager_faction_id, purview))
    }

    /// Проверяет manager-а относительно другой member-faction.
    pub(crate) fn check_operator_validate_target<Context>(
        &self,
        manager_player_id: i32,
        target_faction_id: i32,
        purview: i32,
        context: &Context,
    ) -> Result<bool, Context::Block>
    where
        Context: UnionOperatorValidationContext,
    {
        if manager_player_id == target_faction_id {
            return Ok(false);
        }
        let manager_faction_id = context.faction_id_by_master_player(manager_player_id)?;
        if manager_faction_id == 0
            || target_faction_id == 0
            || target_faction_id == self.master_id
            || self.is_member(manager_faction_id) == 0
            || self.is_member(target_faction_id) == 0
        {
            return Ok(false);
        }
        let manager_permitted = self.is_using_purview(manager_faction_id, purview);
        if !manager_permitted {
            return Ok(false);
        }
        Ok(manager_permitted != self.is_using_purview(target_faction_id, purview))
    }

    /// Делегирует owned-city predicate только найденной master-faction.
    pub(crate) fn is_owned_city<Context>(&self, region_id: i32, context: &Context) -> i32
    where
        Context: UnionMasterFactionQueryContext,
    {
        if self.master_id <= 0 {
            return 0;
        }
        context
            .faction_is_owned_city(self.master_id, region_id)
            .unwrap_or(0)
    }

    /// Делегирует исторический enemy predicate только master-faction.
    pub(crate) fn is_enemy_faction<Context>(&self, enemy_id: i32, context: &Context) -> i32
    where
        Context: UnionMasterFactionQueryContext,
    {
        if self.master_id <= 0 {
            return 0;
        }
        context
            .faction_is_enemy_faction(self.master_id, enemy_id)
            .unwrap_or(0)
    }

    /// Возвращает независимый owned-city snapshot вместо dangling reference.
    pub(crate) fn owned_cities_snapshot<Context>(&self, context: &Context) -> VecDeque<i32>
    where
        Context: UnionMasterFactionQueryContext,
    {
        if self.master_id <= 0 {
            return VecDeque::new();
        }
        context
            .faction_owned_cities(self.master_id)
            .unwrap_or_default()
    }

    pub(crate) fn has_enemy_faction<Context>(&self, context: &Context) -> bool
    where
        Context: UnionMasterFactionQueryContext,
    {
        self.master_id > 0
            && context
                .faction_has_enemy(self.master_id)
                .unwrap_or(false)
    }

    pub(crate) fn has_city_war_enemy_faction<Context>(&self, context: &Context) -> bool
    where
        Context: UnionMasterFactionQueryContext,
    {
        self.master_id > 0
            && context
                .faction_has_city_war_enemy(self.master_id)
                .unwrap_or(false)
    }

    /// Делегирует enemy-leader query только найденной master-faction.
    pub(crate) fn enemy_leader_organizing_id<Context>(&self, context: &Context) -> i32
    where
        Context: UnionMasterFactionQueryContext,
    {
        if self.master_id <= 0 {
            return 0;
        }
        context
            .faction_enemy_leader_organizing_id(self.master_id)
            .unwrap_or(0)
    }

    fn fan_out_owned_city_mutation<Context, Dispatch>(
        &self,
        context: &mut Context,
        mut dispatch: Dispatch,
    ) -> Result<UnionOwnedCityFanoutReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
        Dispatch: FnMut(
            &mut Context,
            i32,
        ) -> Result<bool, OwnedCityMutationBuildError>,
    {
        let mut invoked_faction_ids = Vec::new();
        for &faction_id in self.members.keys() {
            if faction_id <= 0 {
                continue;
            }
            match dispatch(context, faction_id) {
                Ok(true) => invoked_faction_ids.push(faction_id),
                Ok(false) => {}
                Err(source) => {
                    return Err(UnionOwnedCityMutationBlock {
                        faction_id,
                        completed_faction_ids: invoked_faction_ids,
                        source,
                    });
                }
            }
        }
        Ok(UnionOwnedCityFanoutReport {
            invoked_faction_ids,
        })
    }

    /// Добавляет region каждой найденной положительной member-faction.
    pub(crate) fn add_owned_city<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
        region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionOwnedCityFanoutReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        self.fan_out_owned_city_mutation(context, |context, faction_id| {
            context.faction_add_owned_city(faction_id, game, region_id, update_player)
        })
    }

    /// Добавляет исходный list каждой найденной положительной member-faction.
    pub(crate) fn add_owned_cities<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
        region_ids: &VecDeque<i32>,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionOwnedCityFanoutReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        self.fan_out_owned_city_mutation(context, |context, faction_id| {
            context.faction_add_owned_cities(faction_id, game, region_ids, update_player)
        })
    }

    /// Очищает owned-city state всех найденных member-фракций.
    pub(crate) fn clear_owned_cities<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionOwnedCityBooleanMutationReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        let fanout = self.fan_out_owned_city_mutation(context, |context, faction_id| {
            context.faction_clear_owned_cities(faction_id, game, update_player)
        })?;
        Ok(UnionOwnedCityBooleanMutationReport {
            legacy_result: true,
            invoked_faction_ids: fanout.invoked_faction_ids,
        })
    }

    /// Заменяет owned-city list только у найденной положительной master-faction.
    pub(crate) fn set_owned_cities<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
        region_ids: &VecDeque<i32>,
    ) -> Result<UnionOwnedCityFanoutReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        if self.master_id <= 0 {
            return Ok(UnionOwnedCityFanoutReport {
                invoked_faction_ids: Vec::new(),
            });
        }
        match context.faction_set_owned_cities(self.master_id, game, region_ids) {
            Ok(true) => Ok(UnionOwnedCityFanoutReport {
                invoked_faction_ids: vec![self.master_id],
            }),
            Ok(false) => Ok(UnionOwnedCityFanoutReport {
                invoked_faction_ids: Vec::new(),
            }),
            Err(source) => Err(UnionOwnedCityMutationBlock {
                faction_id: self.master_id,
                completed_faction_ids: Vec::new(),
                source,
            }),
        }
    }

    /// Сохраняет подтверждённый баг: аргумент игнорируется, master-list очищается.
    pub(crate) fn delete_owned_city<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
        _region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<UnionOwnedCityBooleanMutationReport, UnionOwnedCityMutationBlock>
    where
        Context: UnionOwnedCityMutationContext,
    {
        if self.master_id <= 0 {
            return Ok(UnionOwnedCityBooleanMutationReport {
                legacy_result: false,
                invoked_faction_ids: Vec::new(),
            });
        }
        match context.faction_clear_owned_cities(self.master_id, game, update_player) {
            Ok(found) => Ok(UnionOwnedCityBooleanMutationReport {
                legacy_result: found,
                invoked_faction_ids: found.then_some(self.master_id).into_iter().collect(),
            }),
            Err(source) => Err(UnionOwnedCityMutationBlock {
                faction_id: self.master_id,
                completed_faction_ids: Vec::new(),
                source,
            }),
        }
    }

    /// Очищает standard enemy-set каждой найденной member-faction.
    pub(crate) fn clear_enemy_factions<Context>(
        &self,
        context: &mut Context,
    ) -> UnionFactionFanoutReport
    where
        Context: UnionFactionStateMutationContext,
    {
        let mut invoked_faction_ids = Vec::new();
        for &faction_id in self.members.keys() {
            if faction_id > 0 && context.faction_clear_enemy_factions(faction_id) {
                invoked_faction_ids.push(faction_id);
            }
        }
        UnionFactionFanoutReport {
            invoked_faction_ids,
        }
    }

    fn fan_out_victor_mutation<Context, Dispatch>(
        &self,
        context: &mut Context,
        game: &CGame,
        mut dispatch: Dispatch,
    ) -> Result<UnionVictorFanoutReport, UnionVictorMutationBlock>
    where
        Context: UnionFactionStateMutationContext,
        Dispatch: FnMut(
            &mut Context,
            i32,
            &CGame,
        ) -> Result<Option<Vec<FactionPropertyDelivery>>, FactionInitialPropertyBlock>,
    {
        let mut factions = Vec::new();
        for &faction_id in self.members.keys() {
            if faction_id <= 0 {
                continue;
            }
            match dispatch(context, faction_id, game) {
                Ok(Some(deliveries)) => factions.push(UnionVictorFactionReport {
                    faction_id,
                    deliveries,
                }),
                Ok(None) => {}
                Err(source) => {
                    return Err(UnionVictorMutationBlock {
                        faction_id,
                        completed_factions: factions,
                        source,
                    });
                }
            }
        }
        Ok(UnionVictorFanoutReport { factions })
    }

    pub(crate) fn add_defence_victor_counts<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
    ) -> Result<UnionVictorFanoutReport, UnionVictorMutationBlock>
    where
        Context: UnionFactionStateMutationContext,
    {
        self.fan_out_victor_mutation(context, game, |context, faction_id, game| {
            context.faction_add_defence_victor_count(faction_id, game)
        })
    }

    pub(crate) fn add_offense_victor_counts<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
    ) -> Result<UnionVictorFanoutReport, UnionVictorMutationBlock>
    where
        Context: UnionFactionStateMutationContext,
    {
        self.fan_out_victor_mutation(context, game, |context, faction_id, game| {
            context.faction_add_offense_victor_count(faction_id, game)
        })
    }

    pub(crate) fn add_village_war_victor_counts<Context>(
        &self,
        context: &mut Context,
        game: &CGame,
    ) -> Result<UnionVictorFanoutReport, UnionVictorMutationBlock>
    where
        Context: UnionFactionStateMutationContext,
    {
        self.fan_out_victor_mutation(context, game, |context, faction_id, game| {
            context.faction_add_village_war_victor_count(faction_id, game)
        })
    }

    fn target_faction_ids(&self, faction_id: i32) -> Vec<i32> {
        if faction_id == 0 {
            self.members.keys().copied().collect()
        } else {
            vec![faction_id]
        }
    }

    /// Обновляет online player-ов одной faction либо всех member-фракций.
    pub(crate) fn update_player_faction_info<Context>(
        &self,
        faction_id: i32,
        context: &Context,
        game: &CGame,
        update_player: &mut dyn FnMut(i32),
    ) -> UnionPlayerRefreshReport
    where
        Context: UnionPlayerRefreshContext,
    {
        let mut factions = Vec::new();
        for target_faction_id in self.target_faction_ids(faction_id) {
            if target_faction_id <= 0 {
                continue;
            }
            let Some(refreshed_player_ids) = context.faction_update_player_info(
                target_faction_id,
                game,
                update_player,
            ) else {
                continue;
            };
            factions.push(UnionFactionPlayerRefreshReport {
                faction_id: target_faction_id,
                refreshed_player_ids,
            });
        }
        UnionPlayerRefreshReport { factions }
    }

    /// Публикует полный standard enemy snapshot выбранных faction-target-ов.
    pub(crate) fn update_enemy_factions_to_client<Context>(
        &self,
        faction_id: i32,
        _operation: EOperator,
        context: &Context,
        game: &CGame,
    ) -> UnionEnemySnapshotReport
    where
        Context: UnionClientSnapshotContext,
    {
        let mut factions = Vec::new();
        for target_faction_id in self.target_faction_ids(faction_id) {
            if target_faction_id <= 0 {
                continue;
            }
            let Some(deliveries) =
                context.faction_update_enemy_snapshot(target_faction_id, game)
            else {
                continue;
            };
            factions.push(UnionEnemySnapshotFactionReport {
                faction_id: target_faction_id,
                deliveries,
            });
        }
        UnionEnemySnapshotReport { factions }
    }

    /// Публикует полный city-war enemy snapshot выбранных faction-target-ов.
    pub(crate) fn update_city_war_enemy_factions_to_client<Context>(
        &self,
        faction_id: i32,
        _operation: EOperator,
        context: &Context,
        game: &CGame,
    ) -> UnionEnemySnapshotReport
    where
        Context: UnionClientSnapshotContext,
    {
        let mut factions = Vec::new();
        for target_faction_id in self.target_faction_ids(faction_id) {
            if target_faction_id <= 0 {
                continue;
            }
            let Some(deliveries) =
                context.faction_update_city_war_enemy_snapshot(target_faction_id, game)
            else {
                continue;
            };
            factions.push(UnionEnemySnapshotFactionReport {
                faction_id: target_faction_id,
                deliveries,
            });
        }
        UnionEnemySnapshotReport { factions }
    }

    /// Публикует полный owned-city snapshot выбранных faction-target-ов.
    pub(crate) fn update_owned_cities_to_client<Context>(
        &self,
        faction_id: i32,
        _operation: EOperator,
        context: &Context,
        game: &CGame,
    ) -> Result<UnionOwnedCitySnapshotReport, UnionOwnedCitySnapshotBlock>
    where
        Context: UnionClientSnapshotContext,
    {
        let mut factions = Vec::new();
        for target_faction_id in self.target_faction_ids(faction_id) {
            if target_faction_id <= 0 {
                continue;
            }
            match context.faction_update_owned_city_snapshot(target_faction_id, game) {
                Ok(Some(deliveries)) => factions.push(UnionOwnedCitySnapshotFactionReport {
                    faction_id: target_faction_id,
                    deliveries,
                }),
                Ok(None) => {}
                Err(source) => {
                    return Err(UnionOwnedCitySnapshotBlock {
                        faction_id: target_faction_id,
                        completed_factions: factions,
                        source,
                    });
                }
            }
        }
        Ok(UnionOwnedCitySnapshotReport { factions })
    }

    pub(crate) const fn change_data_type(&self) -> i32 {
        self.change_data_type
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.h

// ============================================================================
// FUNCTION: tagVilWarSetup::tagVilWarSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp
// RVA: 0x00069110
// ADDRESS: 00469110
// PROTOTYPE: undefined __thiscall tagVilWarSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00469565
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp
// RVA: 0x00069565
// ADDRESS: 00469565
// PROTOTYPE: undefined Catch@00469565()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004696f9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp
// RVA: 0x000696F9
// ADDRESS: 004696f9
// PROTOTYPE: undefined Catch@004696f9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagVilWarSetup::tagVilWarSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp
// RVA: 0x00069740
// ADDRESS: 00469740
// PROTOTYPE: undefined __thiscall tagVilWarSetup(tagVilWarSetup * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagVilWarSetup::operator=
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp
// RVA: 0x00069860
// ADDRESS: 00469860
// PROTOTYPE: tagVilWarSetup * __thiscall operator=(tagVilWarSetup * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00469ddf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp
// RVA: 0x00069DDF
// ADDRESS: 00469ddf
// PROTOTYPE: undefined Catch@00469ddf()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0046adba
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp
// RVA: 0x0006ADBA
// ADDRESS: 0046adba
// PROTOTYPE: undefined Catch@0046adba()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0046b7b4
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp
// RVA: 0x0006B7B4
// ADDRESS: 0046b7b4
// PROTOTYPE: undefined Catch@0046b7b4()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// IMPLEMENTED выше: CUnion::SetChangeData RVA 0x000C17D0.

// ============================================================================
// FUNCTION: CUnion::ClearApplyList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:785
// RVA: 0x000C1840
// ADDRESS: 004c1840
// PROTOTYPE: bool __thiscall ClearApplyList(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_bool___thiscall_CUnion::Invite(long,long)'::__l22::InviteJoinConfeder::InviteJoinConfeder
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:847
// RVA: 0x000C1870
// ADDRESS: 004c1870
// PROTOTYPE: undefined __thiscall InviteJoinConfeder(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_bool___thiscall_CUnion::ApplyForJoin(long,long,long)'::__l24::PlayerApplyForJoinConfeder::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:618
// RVA: 0x000C18A0
// ADDRESS: 004c18a0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::DubAndSetJobLvl
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:1171
// RVA: 0x000C18B0
// ADDRESS: 004c18b0
// PROTOTYPE: bool __thiscall DubAndSetJobLvl(long param_1, long param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::CheckOperValidate
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:471
// RVA: 0x000C18D0
// ADDRESS: 004c18d0
// PROTOTYPE: bool __thiscall CheckOperValidate(long param_1, long param_2, ePurview param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004c18db
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:490
// RVA: 0x000C18DB
// ADDRESS: 004c18db
// PROTOTYPE: undefined FUN_004c18db()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::CheckOperValidate
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:494
// RVA: 0x000C1970
// ADDRESS: 004c1970
// PROTOTYPE: bool __thiscall CheckOperValidate(long param_1, ePurview param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_bool___thiscall_CUnion::ApplyForJoin(long,long,long)'::__l24::PlayerApplyForJoinConfeder::PlayerApplyForJoinConfeder
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:615
// RVA: 0x000C19C0
// ADDRESS: 004c19c0
// PROTOTYPE: undefined __thiscall PlayerApplyForJoinConfeder(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::Save
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:83
// RVA: 0x000C1A00
// ADDRESS: 004c1a00
// PROTOTYPE: bool __thiscall Save(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::GetEnemyList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:321
// RVA: 0x000C1A60
// ADDRESS: 004c1a60
// PROTOTYPE: void __thiscall GetEnemyList(list<COrganizing*,std::allocator<COrganizing*>_> param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_bool___thiscall_CUnion::ApplyForJoin(long,long,long)'::__l24::PlayerApplyForJoinConfeder::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:621
// RVA: 0x000C1A70
// ADDRESS: 004c1a70
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::GetMemberList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:284
// RVA: 0x000C1BB0
// ADDRESS: 004c1bb0
// PROTOTYPE: void __thiscall GetMemberList(list<long,std::allocator<long>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::GetEnemyList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:308
// RVA: 0x000C1C50
// ADDRESS: 004c1c50
// PROTOTYPE: set<long,std::less<long>,std::allocator<long>_> __thiscall GetEnemyList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::IsUsingPV
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:425
// RVA: 0x000C1D00
// ADDRESS: 004c1d00
// PROTOTYPE: bool __thiscall IsUsingPV(long param_1, ePurview param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::SetMemPV
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:443
// RVA: 0x000C1D70
// ADDRESS: 004c1d70
// PROTOTYPE: void __thiscall SetMemPV(long param_1, ePurview param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::CUnion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.h:8
// RVA: 0x000C1E30
// ADDRESS: 004c1e30
// PROTOTYPE: undefined __thiscall CUnion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::EditLeaveWord
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.h:51
// RVA: 0x000C1EC0
// ADDRESS: 004c1ec0
// PROTOTYPE: bool __thiscall EditLeaveWord(long param_1, long param_2, eOperator param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::OperatorTax
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:1165
// RVA: 0x000C1ED0
// ADDRESS: 004c1ed0
// PROTOTYPE: bool __thiscall OperatorTax(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::GetEstablishedTime
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.h:125
// RVA: 0x000C1F00
// ADDRESS: 004c1f00
// PROTOTYPE: tagTime * __thiscall GetEstablishedTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::GetFigure
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.h:126
// RVA: 0x000C1F10
// ADDRESS: 004c1f10
// PROTOTYPE: uchar __thiscall GetFigure(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::SetControbuter
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.h:177
// RVA: 0x000C1F20
// ADDRESS: 004c1f20
// PROTOTYPE: void __thiscall SetControbuter(long param_1, long param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::Upgrade
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.h:57
// RVA: 0x000C1F30
// ADDRESS: 004c1f30
// PROTOTYPE: bool __thiscall Upgrade(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::~CUnion
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:31
// RVA: 0x000C1F40
// ADDRESS: 004c1f40
// PROTOTYPE: void __thiscall ~CUnion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::Initial
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:36
// RVA: 0x000C1FE0
// ADDRESS: 004c1fe0
// PROTOTYPE: bool __thiscall Initial(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::AddMembersToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:154
// RVA: 0x000C21D0
// ADDRESS: 004c21d0
// PROTOTYPE: bool __thiscall AddMembersToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::IsOwnedCity
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:193
// RVA: 0x000C2340
// ADDRESS: 004c2340
// PROTOTYPE: long __thiscall IsOwnedCity(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::IsEnemyFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:202
// RVA: 0x000C23A0
// ADDRESS: 004c23a0
// PROTOTYPE: long __thiscall IsEnemyFaction(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::AddOwnedCity
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:210
// RVA: 0x000C2400
// ADDRESS: 004c2400
// PROTOTYPE: void __thiscall AddOwnedCity(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::AddOwnedCity
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:224
// RVA: 0x000C2490
// ADDRESS: 004c2490
// PROTOTYPE: void __thiscall AddOwnedCity(list<long,std::allocator<long>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::DelOwnedCity
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:237
// RVA: 0x000C2510
// ADDRESS: 004c2510
// PROTOTYPE: bool __thiscall DelOwnedCity(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::ClearOwnedCity
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:250
// RVA: 0x000C2570
// ADDRESS: 004c2570
// PROTOTYPE: bool __thiscall ClearOwnedCity(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::SetOwnedCity
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:265
// RVA: 0x000C25F0
// ADDRESS: 004c25f0
// PROTOTYPE: void __thiscall SetOwnedCity(list<long,std::allocator<long>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::GetOwnedCities
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:274
// RVA: 0x000C2650
// ADDRESS: 004c2650
// PROTOTYPE: list<long,std::allocator<long>_> * __thiscall GetOwnedCities(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::GetMemberList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:296
// RVA: 0x000C2710
// ADDRESS: 004c2710
// PROTOTYPE: void __thiscall GetMemberList(list<COrganizing*,std::allocator<COrganizing*>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::IsHaveEnymyFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:334
// RVA: 0x000C2810
// ADDRESS: 004c2810
// PROTOTYPE: bool __thiscall IsHaveEnymyFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::IsHaveCityEnemyFaction
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:343
// RVA: 0x000C2870
// ADDRESS: 004c2870
// PROTOTYPE: bool __thiscall IsHaveCityEnemyFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::ClearEnemyFation
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:367
// RVA: 0x000C28D0
// ADDRESS: 004c28d0
// PROTOTYPE: void __thiscall ClearEnemyFation(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::GetEnemyLeaderOrgnizingID
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:383
// RVA: 0x000C2950
// ADDRESS: 004c2950
// PROTOTYPE: long __thiscall GetEnemyLeaderOrgnizingID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::AddDefenceVictorCounts
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:392
// RVA: 0x000C29B0
// ADDRESS: 004c29b0
// PROTOTYPE: void __thiscall AddDefenceVictorCounts(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::AddOffenseVictorCounts
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:402
// RVA: 0x000C2A30
// ADDRESS: 004c2a30
// PROTOTYPE: void __thiscall AddOffenseVictorCounts(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::AddVillageWarVictorCounts
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:413
// RVA: 0x000C2AB0
// ADDRESS: 004c2ab0
// PROTOTYPE: void __thiscall AddVillageWarVictorCounts(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::DelMember
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:536
// RVA: 0x000C2B30
// ADDRESS: 004c2b30
// PROTOTYPE: bool __thiscall DelMember(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::ApplyForJoin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:559
// RVA: 0x000C2B90
// ADDRESS: 004c2b90
// PROTOTYPE: bool __thiscall ApplyForJoin(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_bool___thiscall_CUnion::ApplyForJoin(long,long,long)'::__l24::PlayerApplyForJoinConfeder::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:636
// RVA: 0x000C2EA0
// ADDRESS: 004c2ea0
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::AddFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:720
// RVA: 0x000C3170
// ADDRESS: 004c3170
// PROTOTYPE: void __thiscall AddFaction(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::Invite
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:795
// RVA: 0x000C3660
// ADDRESS: 004c3660
// PROTOTYPE: bool __thiscall Invite(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_bool___thiscall_CUnion::Invite(long,long)'::__l22::InviteJoinConfeder::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:853
// RVA: 0x000C39B0
// ADDRESS: 004c39b0
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_bool___thiscall_CUnion::Invite(long,long)'::__l22::InviteJoinConfeder::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:871
// RVA: 0x000C3B10
// ADDRESS: 004c3b10
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::Exit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:914
// RVA: 0x000C3DD0
// ADDRESS: 004c3dd0
// PROTOTYPE: bool __thiscall Exit(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::Disband
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:966
// RVA: 0x000C43F0
// ADDRESS: 004c43f0
// PROTOTYPE: bool __thiscall Disband(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::FireOut
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:1010
// RVA: 0x000C49D0
// ADDRESS: 004c49d0
// PROTOTYPE: bool __thiscall FireOut(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::Demise
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:1061
// RVA: 0x000C5060
// ADDRESS: 004c5060
// PROTOTYPE: bool __thiscall Demise(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::UpdatePlayerFactionInfo
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:1198
// RVA: 0x000C5770
// ADDRESS: 004c5770
// PROTOTYPE: void __thiscall UpdatePlayerFactionInfo(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::UpdateMemberInfoToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:1217
// RVA: 0x000C5840
// ADDRESS: 004c5840
// PROTOTYPE: void __thiscall UpdateMemberInfoToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::DeleteOrgaToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:1305
// RVA: 0x000C5D20
// ADDRESS: 004c5d20
// PROTOTYPE: void __thiscall DeleteOrgaToClient(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::UpdateEnemyFactionToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:1355
// RVA: 0x000C5FF0
// ADDRESS: 004c5ff0
// PROTOTYPE: void __thiscall UpdateEnemyFactionToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::UpdateCityWarEnemyFactionToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:1374
// RVA: 0x000C60D0
// ADDRESS: 004c60d0
// PROTOTYPE: void __thiscall UpdateCityWarEnemyFactionToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::UpdateOwnedCityToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:1402
// RVA: 0x000C61B0
// ADDRESS: 004c61b0
// PROTOTYPE: void __thiscall UpdateOwnedCityToClient(long param_1, eOperator param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::SendInfoToAllMember
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:1420
// RVA: 0x000C6290
// ADDRESS: 004c6290
// PROTOTYPE: void __thiscall SendInfoToAllMember(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, long param_3, ulong param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::GetPlayerHeader
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:1431
// RVA: 0x000C6320
// ADDRESS: 004c6320
// PROTOTYPE: long __thiscall GetPlayerHeader(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED выше: `CUnion::CloneSaveData` RVA `0x000C6380`.

// ============================================================================
// FUNCTION: CUnion::CUnion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:26
// RVA: 0x000C64D0
// ADDRESS: 004c64d0
// PROTOTYPE: undefined __thiscall CUnion(long param_1, long param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:135
// RVA: 0x000C6590
// ADDRESS: 004c6590
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CUnion::DoJoin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp:680
// RVA: 0x000C66A0
// ADDRESS: 004c66a0
// PROTOTYPE: bool __thiscall DoJoin(long param_1, long param_2, long param_3, tagTime * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Unwind@0052ff60
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\union.cpp
// RVA: 0x0012FF60
// ADDRESS: 0052ff60
// PROTOTYPE: undefined Unwind@0052ff60()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//















// COMPONENT_VARIANT_END: WorldServer
