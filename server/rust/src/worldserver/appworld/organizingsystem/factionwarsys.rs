//! Войны фракций `CFactionWarSys` из `factionwarsys.cpp/.h`, подтверждённые
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Реестр war types и список enemy pairs сохраняют числовой и insertion order.
//! `Run` использует strict gate `elapsed > 59999`, уменьшает оставшееся время,
//! затем отдельным проходом завершает копии истёкших войн.
//!
//! `StopFactionWar` удаляет registry pair до двух faction callbacks, после
//! чего обновляет обе стороны, рассылает localized notice и пишет war-log.
//! `OnPlayerDied` сохраняет master/union gates, повторно раскрывает обе стороны
//! и удаляет каждую попарную связь до callbacks.
//!
//! Save generation передаёт owned копии в общий DB snapshot. `VecDeque`,
//! `BTreeMap` и owned строки заменяют STL, pointer lists и stack formatting,
//! сохраняя порядок и отсутствие транзакционного rollback.

use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fmt;

use crate::dbaccess::worlddb::rsenemyfactions::{
    EnemyFactionLoadSnapshot, EnemyFactionSaveSnapshot, EnemyFactionsLoadOutcome,
};
use crate::worldserver::appworld::organizingsystem::faction::FactionEnemyMutationBlock;
use crate::public::readwrite::read_to;
use crate::worldserver::worldserver::game::CGame;

#[derive(Clone, Copy, Debug)]
pub(crate) struct EnemyFactionState {
    faction_id_1: i32,
    faction_id_2: i32,
    disband_time: u32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FactionWarType {
    war_type: i32,
    fight_time_ms: i32,
    money: i32,
}

impl FactionWarType {
    pub(crate) const fn new(war_type: i32, fight_time_ms: i32, money: i32) -> Self {
        Self {
            war_type,
            fight_time_ms,
            money,
        }
    }

    pub(crate) const fn fight_time_ms(self) -> i32 {
        self.fight_time_ms
    }
}

/// Наблюдаемый результат одного `CFactionWarSys::LoadIni`.
///
/// Старый метод всегда завершался истинным result: отсутствие файла только
/// очищало map и писало literal в общий лог. Неудачная formatted-extraction
/// оставляла уже применённый prefix; безопасный owner не читает остающиеся
/// неинициализированные stack-значения и останавливает только parser.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionWarIniLoadCompletion {
    FileMissing,
    FormatStopped,
    Loaded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionWarIniLoadReport {
    pub(crate) completion: FactionWarIniLoadCompletion,
    pub(crate) parsed_records: u32,
    pub(crate) legacy_result: bool,
}

#[derive(Debug)]
pub(crate) struct FactionWarInitializationReport {
    pub(crate) enemy_faction_load_succeeded: bool,
    pub(crate) loaded_relation_records: usize,
    pub(crate) applied_relation_records: usize,
    pub(crate) ini: FactionWarIniLoadReport,
    pub(crate) legacy_result: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionWarInitializationBlock {
    EnemyRelation(FactionEnemyMutationBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FactionWarStopBlock<ContextBlock> {
    Context(ContextBlock),
}

impl<ContextBlock: fmt::Display> fmt::Display for FactionWarStopBlock<ContextBlock> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Context(block) => {
                write!(formatter, "контекст остановки войны заблокирован: {block}")
            }
        }
    }
}

impl<ContextBlock: Error + 'static> Error for FactionWarStopBlock<ContextBlock> {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionWarStopOutcome {
    NotEnemy,
    RegistryOnlyMissingRoot,
    Completed {
        first_side_size: usize,
        second_side_size: usize,
        relation_pairs: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionWarFormatArgument<'a> {
    Text(&'a [u8]),
    Signed(i32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionWarFactionSnapshot {
    pub(crate) faction_id: i32,
    pub(crate) superior_organizing_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionWarDeclarationOutcome {
    WarTypeNotFound,
    MasterRequired,
    FactionNotFound,
    SameUnion,
    PlayerOffline,
    InsufficientFunds { required_money: i32 },
    Declared {
        source_faction_id: i32,
        target_faction_id: i32,
        relation_pairs: usize,
        required_money: i32,
    },
}

impl FactionWarDeclarationOutcome {
    pub(crate) const fn legacy_result(self) -> bool {
        matches!(self, Self::Declared { .. })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FactionWarDeclarationBlock<ContextBlock> {
    Context(ContextBlock),
}

pub(crate) trait FactionWarDeclarationContext {
    type Block;

    fn faction_id_by_master_player(&self, player_id: i32) -> Result<i32, Self::Block>;

    fn faction_snapshot(
        &self,
        faction_id: i32,
    ) -> Result<Option<FactionWarFactionSnapshot>, Self::Block>;

    fn faction_side(&self, root_faction_id: i32) -> Result<Vec<i32>, Self::Block>;

    fn player_money(&self, player_id: i32) -> Option<u32>;

    fn add_enemy_organizing(
        &mut self,
        faction_id: i32,
        enemy_id: i32,
    ) -> Result<(), Self::Block>;

    fn update_enemy_faction(&mut self, faction_id: i32) -> Result<(), Self::Block>;

    fn organizing_name(&self, faction_id: i32) -> Result<Vec<u8>, Self::Block>;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionWarFormatArgument<'_>],
    ) -> Vec<u8>;

    fn send_player_info(
        &mut self,
        player_id: i32,
        first_text: &[u8],
        second_text: &[u8],
    );

    fn send_orga_info_to_all(&mut self, info: &[u8], kind: u32, color: u32);

    fn put_war_log(&mut self, info: &[u8]);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionWarRunReport {
    pub(crate) now_ms: u32,
    pub(crate) elapsed_ms: u32,
    pub(crate) gate_opened: bool,
    pub(crate) decremented_relations: usize,
    pub(crate) expired_relations: usize,
    pub(crate) completed_stops: usize,
    pub(crate) registry_only_stops: usize,
    pub(crate) skipped_non_enemy_stops: usize,
}

pub(crate) trait FactionWarStopContext {
    type Block;

    fn faction_exists(&self, faction_id: i32) -> bool;

 /// Возвращает старый `COrganizing*` list-order одной стороны.
 ///
 /// Реализация обязана воспроизвести `IsFreeFaction`: свободная фракция
 /// даёт один root ID; найденный союз — virtual `GetOrganizingList`; null
 /// union даёт пустой список. Неизвестная null/UB граница возвращается как
 /// `Block`, а не получает придуманного продолжения.
    fn faction_side(&self, root_faction_id: i32) -> Result<Vec<i32>, Self::Block>;

    fn del_enemy_organizing(
        &mut self,
        faction_id: i32,
        enemy_id: i32,
    ) -> Result<(), Self::Block>;

    fn update_enemy_faction(&mut self, faction_id: i32) -> Result<(), Self::Block>;

    fn organizing_name(&self, faction_id: i32) -> Result<Vec<u8>, Self::Block>;

    fn format_world_string(&mut self, string_id: &[u8], arguments: &[&[u8]]) -> Vec<u8>;

    fn send_orga_info_to_all(&mut self, info: &[u8], kind: u32, color: u32);

    fn put_war_log(&mut self, info: &[u8]);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionWarPlayerDiedOutcome {
    DefeatedPlayerNotMaster,
    DefeatedFactionNotUnionMaster {
        defeated_faction_id: i32,
        union_id: i32,
    },
    VictorPlayerWithoutFaction {
        defeated_faction_id: i32,
    },
    MissingRootFaction {
        defeated_faction_id: i32,
        victor_faction_id: i32,
    },
    NotEnemy {
        defeated_faction_id: i32,
        victor_faction_id: i32,
    },
    Completed {
        defeated_faction_id: i32,
        victor_faction_id: i32,
        defeated_side_size: usize,
        victor_side_size: usize,
        relation_pairs: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FactionWarPlayerDiedBlock<ContextBlock> {
    Context(ContextBlock),
}

pub(crate) trait FactionWarPlayerDiedContext {
    type Block;

    fn faction_id_by_master_player(&self, player_id: i32) -> Result<i32, Self::Block>;
    fn union_id_by_faction(&self, faction_id: i32) -> Result<i32, Self::Block>;
    fn union_id_by_master_faction(&self, faction_id: i32) -> Result<i32, Self::Block>;
    fn faction_id_by_player(&self, player_id: i32) -> Result<i32, Self::Block>;
    fn faction_exists(&self, faction_id: i32) -> bool;
    fn faction_side(&self, root_faction_id: i32) -> Result<Vec<i32>, Self::Block>;
    fn del_enemy_organizing(
        &mut self,
        faction_id: i32,
        enemy_id: i32,
    ) -> Result<(), Self::Block>;
    fn update_enemy_faction(&mut self, faction_id: i32) -> Result<(), Self::Block>;
    fn organizing_name(&self, faction_id: i32) -> Result<Vec<u8>, Self::Block>;
    fn format_world_string(&mut self, string_id: &[u8], arguments: &[&[u8]]) -> Vec<u8>;
    fn send_orga_info_to_all(&mut self, info: &[u8], kind: u32, color: u32);
    fn put_war_log(&mut self, info: &[u8]);
}

pub(crate) struct CFactionWarSys {
    faction_wars: BTreeMap<i32, FactionWarType>,
    enemy_factions: VecDeque<EnemyFactionState>,
    start_time_ms: u32,
}

impl CFactionWarSys {
    pub(crate) const fn new(start_time_ms: u32) -> Self {
        Self {
            faction_wars: BTreeMap::new(),
            enemy_factions: VecDeque::new(),
            start_time_ms,
        }
    }

 /// Очищает live relation-list до самостоятельного DB open.
 ///
 /// Это отдельная операция, поскольку `CRsEnemyFactions::LoadAll…`
 /// выполнял clear *до* `CreateCn/OpenCn`; ошибка подключения оставляла
 /// именно пустой список, а не предыдущий snapshot.
    pub(crate) fn clear_enemy_factions_for_load(&mut self) {
        self.enemy_factions.clear();
    }

 /// Завершает `CFactionWarSys::Initialize` после самостоятельного DB read.
 ///
 /// Caller обязан вызвать `clear_enemy_factions_for_load` перед запуском
 /// DB-owner-а. Каждая строка сначала проходит unordered upsert, затем
 /// итоговый live list в list-order передаётся `SetEnemyFactionRelation`.
 /// Только после всех этих effects перечитывается `FactionWarSys.ini`.
    pub(crate) fn initialize_from_loaded_relations<ApplyRelation>(
        &mut self,
        load: EnemyFactionsLoadOutcome,
        ini_source: Option<&[u8]>,
        mut apply_relation: ApplyRelation,
    ) -> Result<FactionWarInitializationReport, FactionWarInitializationBlock>
    where
        ApplyRelation: FnMut(i32, i32) -> Result<(), FactionEnemyMutationBlock>,
    {
        let (enemy_faction_load_succeeded, loaded_relations) = match load {
            EnemyFactionsLoadOutcome::ReturnedTrue(relations) => (true, relations),
            EnemyFactionsLoadOutcome::ReturnedFalse(relations) => (false, relations),
        };
        for EnemyFactionLoadSnapshot {
            faction_id_1,
            faction_id_2,
            disband_time,
        } in &loaded_relations
        {
            self.add_one_enemy_faction(*faction_id_1, *faction_id_2, *disband_time);
        }
        let mut applied_relation_records = 0_usize;
        for relation in &self.enemy_factions {
            apply_relation(relation.faction_id_1, relation.faction_id_2)
                .map_err(FactionWarInitializationBlock::EnemyRelation)?;
            applied_relation_records = applied_relation_records.wrapping_add(1);
        }
        let ini = self.load_ini_from_resource(ini_source);
        Ok(FactionWarInitializationReport {
            enemy_faction_load_succeeded,
            loaded_relation_records: loaded_relations.len(),
            applied_relation_records,
            ini,
            legacy_result: true,
        })
    }

 /// Загружает package-resource `data/FactionWarSys.ini` в map-order.
 ///
 /// `ReadTo("#")` находит marker среди whitespace-токенов, после чего
 /// три signed значения образуют `lType`, минуты и `lMoney`. Минуты
 /// умножаются тем же 32-bit wrapping-product, с которым MSVC записывал
 /// `lFightTime`; повторный тип заменяет map-value. Отсутствующий resource
 /// очищает map и возвращает literal старого лога через report, поэтому
 /// caller сохраняет его исходную позицию в общем log owner-е.
    pub(crate) fn load_ini_from_resource(
        &mut self,
        source: Option<&[u8]>,
    ) -> FactionWarIniLoadReport {
        self.faction_wars.clear();
        let Some(source) = source else {
            return FactionWarIniLoadReport {
                completion: FactionWarIniLoadCompletion::FileMissing,
                parsed_records: 0,
                legacy_result: true,
            };
        };

        let mut records = 0_u32;
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());
        while read_to(&mut tokens, b"#") {
            let Some(war_type) = next_signed_long(&mut tokens) else {
                return FactionWarIniLoadReport {
                    completion: FactionWarIniLoadCompletion::FormatStopped,
                    parsed_records: records,
                    legacy_result: true,
                };
            };
            let Some(fight_minutes) = next_signed_long(&mut tokens) else {
                return FactionWarIniLoadReport {
                    completion: FactionWarIniLoadCompletion::FormatStopped,
                    parsed_records: records,
                    legacy_result: true,
                };
            };
            let Some(money) = next_signed_long(&mut tokens) else {
                return FactionWarIniLoadReport {
                    completion: FactionWarIniLoadCompletion::FormatStopped,
                    parsed_records: records,
                    legacy_result: true,
                };
            };
            self.insert_faction_war_type(FactionWarType::new(
                war_type,
                fight_minutes.wrapping_mul(60_000),
                money,
            ));
            records = records.wrapping_add(1);
        }

        FactionWarIniLoadReport {
            completion: FactionWarIniLoadCompletion::Loaded,
            parsed_records: records,
            legacy_result: true,
        }
    }

    pub(crate) fn insert_faction_war_type(&mut self, war_type: FactionWarType) {
        self.faction_wars.insert(war_type.war_type, war_type);
    }

    pub(crate) fn get_dec_war_money_by_type(&self, war_type: i32) -> i32 {
        self.faction_wars.get(&war_type).map_or(0, |war| war.money)
    }

    pub(crate) fn is_enemy_relation(&self, faction_id_1: i32, faction_id_2: i32) -> bool {
        self.enemy_factions.iter().any(|enemy| {
            (enemy.faction_id_1 == faction_id_1 && enemy.faction_id_2 == faction_id_2)
                || (enemy.faction_id_1 == faction_id_2 && enemy.faction_id_2 == faction_id_1)
        })
    }

    pub(crate) fn clear_enemy_faction(&mut self, faction_id_1: i32, faction_id_2: i32) {
        let Some(position) = self.enemy_factions.iter().position(|enemy| {
            (enemy.faction_id_1 == faction_id_1 && enemy.faction_id_2 == faction_id_2)
                || (enemy.faction_id_1 == faction_id_2 && enemy.faction_id_2 == faction_id_1)
        }) else {
            return;
        };
        self.enemy_factions.remove(position);
    }

    pub(crate) fn add_one_enemy_faction(
        &mut self,
        faction_id_1: i32,
        faction_id_2: i32,
        disband_time: u32,
    ) {
        if let Some(enemy) = self.enemy_factions.iter_mut().find(|enemy| {
            (enemy.faction_id_1 == faction_id_1 && enemy.faction_id_2 == faction_id_2)
                || (enemy.faction_id_1 == faction_id_2 && enemy.faction_id_2 == faction_id_1)
        }) {
            enemy.disband_time = disband_time;
            return;
        }
        self.enemy_factions.push_back(EnemyFactionState {
            faction_id_1,
            faction_id_2,
            disband_time,
        });
    }

 /// Объявляет войну в точном порядке.
 ///
 /// Registry relation обновляется до двух faction-callback-ов для каждой
 /// пары; затем стороны обновляются и их имена собираются в map-order.
 /// Деньги здесь только проверяются: фактическое списание инициирует caller
 /// через успешный ответ `0x7FE19`, как и в исходном WorldServer.
    pub(crate) fn dig_up_the_hatchet<Context>(
        &mut self,
        player_id: i32,
        target_faction_id: i32,
        war_type: i32,
        declaration_time: crate::public::date::TagTime,
        context: &mut Context,
    ) -> Result<FactionWarDeclarationOutcome, FactionWarDeclarationBlock<Context::Block>>
    where
        Context: FactionWarDeclarationContext,
    {
        let Some(war) = self.faction_wars.get(&war_type).copied() else {
            return Ok(FactionWarDeclarationOutcome::WarTypeNotFound);
        };

        let source_faction_id = context
            .faction_id_by_master_player(player_id)
            .map_err(FactionWarDeclarationBlock::Context)?;
        if source_faction_id == 0 {
            let first = context.world_string(b"WS0123");
            let second = context.world_string(b"WS0121");
            context.send_player_info(player_id, &first, &second);
            return Ok(FactionWarDeclarationOutcome::MasterRequired);
        }

        let source = context
            .faction_snapshot(source_faction_id)
            .map_err(FactionWarDeclarationBlock::Context)?;
        let target = context
            .faction_snapshot(target_faction_id)
            .map_err(FactionWarDeclarationBlock::Context)?;
        let (Some(source), Some(target)) = (source, target) else {
            return Ok(FactionWarDeclarationOutcome::FactionNotFound);
        };

        if source.superior_organizing_id != 0
            && source.superior_organizing_id == target.superior_organizing_id
        {
            let first = context.world_string(b"WS0230");
            let second = context.world_string(b"WS0121");
            context.send_player_info(player_id, &first, &second);
            return Ok(FactionWarDeclarationOutcome::SameUnion);
        }

        let Some(player_money) = context.player_money(player_id) else {
            return Ok(FactionWarDeclarationOutcome::PlayerOffline);
        };
        if player_money < war.money as u32 {
            let first = context.format_world_string(
                b"WS0231",
                &[FactionWarFormatArgument::Signed(war.money)],
            );
            let second = context.world_string(b"WS0121");
            context.send_player_info(player_id, &first, &second);
            return Ok(FactionWarDeclarationOutcome::InsufficientFunds {
                required_money: war.money,
            });
        }

        let source_side = context
            .faction_side(source.faction_id)
            .map_err(FactionWarDeclarationBlock::Context)?;
        let target_side = context
            .faction_side(target.faction_id)
            .map_err(FactionWarDeclarationBlock::Context)?;

        for &source_id in &source_side {
            for &target_id in &target_side {
                self.add_one_enemy_faction(source_id, target_id, war.fight_time_ms as u32);
                context
                    .add_enemy_organizing(source_id, target_id)
                    .map_err(FactionWarDeclarationBlock::Context)?;
                context
                    .add_enemy_organizing(target_id, source_id)
                    .map_err(FactionWarDeclarationBlock::Context)?;
            }
        }

        let source_names = collect_declaration_side_names(context, &source_side)
            .map_err(FactionWarDeclarationBlock::Context)?;
        let target_names = collect_declaration_side_names(context, &target_side)
            .map_err(FactionWarDeclarationBlock::Context)?;
        let time_text = declaration_time.get_format_string();
        let notice = context.format_world_string(
            b"WS0232",
            &[
                FactionWarFormatArgument::Text(time_text.as_bytes()),
                FactionWarFormatArgument::Text(&source_names),
                FactionWarFormatArgument::Text(&target_names),
            ],
        );
        context.send_orga_info_to_all(&notice, 0xFFFF_FE92, 0xFFFF_0000);
        context.put_war_log(&notice);

        Ok(FactionWarDeclarationOutcome::Declared {
            source_faction_id,
            target_faction_id,
            relation_pairs: source_side.len().saturating_mul(target_side.len()),
            required_money: war.money,
        })
    }

    pub(crate) fn stop_faction_war<Context: FactionWarStopContext>(
        &mut self,
        faction_id_1: i32,
        faction_id_2: i32,
        context: &mut Context,
    ) -> Result<FactionWarStopOutcome, FactionWarStopBlock<Context::Block>> {
        if !self.is_enemy_relation(faction_id_1, faction_id_2) {
            return Ok(FactionWarStopOutcome::NotEnemy);
        }
        let first_root_exists = context.faction_exists(faction_id_1);
        let second_root_exists = context.faction_exists(faction_id_2);
        if !first_root_exists || !second_root_exists {
            self.clear_enemy_faction(faction_id_1, faction_id_2);
            return Ok(FactionWarStopOutcome::RegistryOnlyMissingRoot);
        }

        let first_side = context
            .faction_side(faction_id_1)
            .map_err(FactionWarStopBlock::Context)?;
        let second_side = context
            .faction_side(faction_id_2)
            .map_err(FactionWarStopBlock::Context)?;

        for &first_id in &first_side {
            for &second_id in &second_side {
                self.clear_enemy_faction(first_id, second_id);
                context
                    .del_enemy_organizing(first_id, second_id)
                    .map_err(FactionWarStopBlock::Context)?;
                context
                    .del_enemy_organizing(second_id, first_id)
                    .map_err(FactionWarStopBlock::Context)?;
            }
        }

        let mut first_names = Vec::new();
        for &faction_id in &first_side {
            context
                .update_enemy_faction(faction_id)
                .map_err(FactionWarStopBlock::Context)?;
            first_names.extend_from_slice(
                &context
                    .organizing_name(faction_id)
                    .map_err(FactionWarStopBlock::Context)?,
            );
            first_names.push(b',');
        }
        let mut second_names = Vec::new();
        for &faction_id in &second_side {
            context
                .update_enemy_faction(faction_id)
                .map_err(FactionWarStopBlock::Context)?;
            second_names.extend_from_slice(
                &context
                    .organizing_name(faction_id)
                    .map_err(FactionWarStopBlock::Context)?,
            );
            second_names.push(b',');
        }

        let notice = context.format_world_string(b"WS0234", &[&second_names, &first_names]);
        context.send_orga_info_to_all(&notice, 0xFFFF_FE92, 0xFFFF_0000);
        context.put_war_log(&notice);

        Ok(FactionWarStopOutcome::Completed {
            first_side_size: first_side.len(),
            second_side_size: second_side.len(),
            relation_pairs: first_side.len().saturating_mul(second_side.len()),
        })
    }

    pub(crate) fn on_player_died<Context>(
        &mut self,
        defeated_master_player_id: i32,
        victor_player_id: i32,
        context: &mut Context,
    ) -> Result<FactionWarPlayerDiedOutcome, FactionWarPlayerDiedBlock<Context::Block>>
    where
        Context: FactionWarPlayerDiedContext,
    {
        let defeated_faction_id = context
            .faction_id_by_master_player(defeated_master_player_id)
            .map_err(FactionWarPlayerDiedBlock::Context)?;
        if defeated_faction_id == 0 {
            return Ok(FactionWarPlayerDiedOutcome::DefeatedPlayerNotMaster);
        }

        let union_id = context
            .union_id_by_faction(defeated_faction_id)
            .map_err(FactionWarPlayerDiedBlock::Context)?;
        if union_id > 0 {
            let master_union_id = context
                .union_id_by_master_faction(defeated_faction_id)
                .map_err(FactionWarPlayerDiedBlock::Context)?;
            if master_union_id == 0 {
                return Ok(FactionWarPlayerDiedOutcome::DefeatedFactionNotUnionMaster {
                    defeated_faction_id,
                    union_id,
                });
            }
        }

        let victor_faction_id = context
            .faction_id_by_player(victor_player_id)
            .map_err(FactionWarPlayerDiedBlock::Context)?;
        if victor_faction_id == 0 {
            return Ok(FactionWarPlayerDiedOutcome::VictorPlayerWithoutFaction {
                defeated_faction_id,
            });
        }
        if !context.faction_exists(defeated_faction_id)
            || !context.faction_exists(victor_faction_id)
        {
            return Ok(FactionWarPlayerDiedOutcome::MissingRootFaction {
                defeated_faction_id,
                victor_faction_id,
            });
        }
        if !self.is_enemy_relation(defeated_faction_id, victor_faction_id) {
            return Ok(FactionWarPlayerDiedOutcome::NotEnemy {
                defeated_faction_id,
                victor_faction_id,
            });
        }

        let defeated_side = context
            .faction_side(defeated_faction_id)
            .map_err(FactionWarPlayerDiedBlock::Context)?;
        let victor_side = context
            .faction_side(victor_faction_id)
            .map_err(FactionWarPlayerDiedBlock::Context)?;
        for &defeated_id in &defeated_side {
            for &victor_id in &victor_side {
                self.clear_enemy_faction(defeated_id, victor_id);
                context
                    .del_enemy_organizing(defeated_id, victor_id)
                    .map_err(FactionWarPlayerDiedBlock::Context)?;
                context
                    .del_enemy_organizing(victor_id, defeated_id)
                    .map_err(FactionWarPlayerDiedBlock::Context)?;
            }
        }

        let defeated_names = collect_player_died_side_names(context, &defeated_side)
            .map_err(FactionWarPlayerDiedBlock::Context)?;
        let victor_names = collect_player_died_side_names(context, &victor_side)
            .map_err(FactionWarPlayerDiedBlock::Context)?;
        let notice =
            context.format_world_string(b"WS0233", &[&victor_names, &defeated_names]);
        context.send_orga_info_to_all(&notice, 0xFFFF_FE92, 0xFFFF_0000);
        context.put_war_log(&notice);

        Ok(FactionWarPlayerDiedOutcome::Completed {
            defeated_faction_id,
            victor_faction_id,
            defeated_side_size: defeated_side.len(),
            victor_side_size: victor_side.len(),
            relation_pairs: defeated_side.len().saturating_mul(victor_side.len()),
        })
    }

    pub(crate) fn run<Context, GetTick>(
        &mut self,
        context: &mut Context,
        mut get_tick: GetTick,
    ) -> Result<FactionWarRunReport, FactionWarStopBlock<Context::Block>>
    where
        Context: FactionWarStopContext,
        GetTick: FnMut() -> u32,
    {
        let now_ms = get_tick();
        let elapsed_ms = now_ms.wrapping_sub(self.start_time_ms);
        if elapsed_ms <= 59_999 {
            return Ok(FactionWarRunReport {
                now_ms,
                elapsed_ms,
                gate_opened: false,
                decremented_relations: 0,
                expired_relations: 0,
                completed_stops: 0,
                registry_only_stops: 0,
                skipped_non_enemy_stops: 0,
            });
        }

        self.start_time_ms = now_ms;
        let mut expired = VecDeque::new();
        let mut decremented_relations = 0;
        for enemy in &mut self.enemy_factions {
            if elapsed_ms < enemy.disband_time {
                enemy.disband_time = enemy.disband_time.wrapping_sub(elapsed_ms);
                decremented_relations += 1;
            } else {
                expired.push_back(*enemy);
            }
        }

        let expired_relations = expired.len();
        let mut completed_stops = 0;
        let mut registry_only_stops = 0;
        let mut skipped_non_enemy_stops = 0;
        for enemy in expired {
            match self.stop_faction_war(enemy.faction_id_1, enemy.faction_id_2, context)? {
                FactionWarStopOutcome::NotEnemy => skipped_non_enemy_stops += 1,
                FactionWarStopOutcome::RegistryOnlyMissingRoot => registry_only_stops += 1,
                FactionWarStopOutcome::Completed { .. } => completed_stops += 1,
            }
        }

        Ok(FactionWarRunReport {
            now_ms,
            elapsed_ms,
            gate_opened: true,
            decremented_relations,
            expired_relations,
            completed_stops,
            registry_only_stops,
            skipped_non_enemy_stops,
        })
    }

    pub(crate) fn generate_save_data(&self, game: &CGame) {
        let snapshot = self
            .enemy_factions
            .iter()
            .map(|enemy| {
                Some(EnemyFactionSaveSnapshot {
                    faction_id_1: enemy.faction_id_1,
                    faction_id_2: enemy.faction_id_2,
                    disband_time: enemy.disband_time,
                })
            })
            .collect();
        game.set_enemy_factions(snapshot);
    }
}

/// Один formatted `long` extraction после найденного marker-а `#`.
///
/// При неуспехе MSVC stream оставлял destination неинициализированным; его
/// последующее использование не имеет подтверждённого результата. Вызывающий
/// останавливает parser после уже опубликованных map-records.
fn next_signed_long<'a>(tokens: &mut impl Iterator<Item = &'a [u8]>) -> Option<i32> {
    let token = tokens.next()?;
    std::str::from_utf8(token).ok()?.parse().ok()
}

fn collect_declaration_side_names<Context>(
    context: &mut Context,
    faction_ids: &[i32],
) -> Result<Vec<u8>, Context::Block>
where
    Context: FactionWarDeclarationContext,
{
    let mut names = Vec::new();
    for (index, &faction_id) in faction_ids.iter().enumerate() {
        context.update_enemy_faction(faction_id)?;
        if index != 0 {
            names.push(b',');
        }
        names.extend_from_slice(&context.organizing_name(faction_id)?);
    }
    Ok(names)
}

fn collect_player_died_side_names<Context>(
    context: &mut Context,
    faction_ids: &[i32],
) -> Result<Vec<u8>, Context::Block>
where
    Context: FactionWarPlayerDiedContext,
{
    let mut names = Vec::new();
    for &faction_id in faction_ids {
        context.update_enemy_faction(faction_id)?;
        names.extend_from_slice(&context.organizing_name(faction_id)?);
        names.push(b',');
    }
    let _ = names.pop();
    Ok(names)
}
