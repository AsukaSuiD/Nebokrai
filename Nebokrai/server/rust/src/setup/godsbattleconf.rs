//! Общий Gods Battle configuration owner, подтверждённый WorldServer и
//! GameServer EXE/PDB.
//!
//! Wire последовательно содержит NPC, ordered base-money records, три vector-
//! секции, два XYD и die-back records; все counts signed. Map key задаёт
//! unsigned порядок и отдельно не передаётся. Game decoder очищает все
//! коллекции, кроме `m_vFactionRule`: исходный owner при повторном snapshot-е
//! дополняет именно эту vector-секцию.
//!
//! Loader читает шесть ресурсов по очереди и очищает секцию непосредственно
//! перед её открытием. Ошибка позднего файла сохраняет обновлённые ранние
//! секции и прежние ещё не начатые. `m_XYD[1..=2]` — те же поля, которые
//! изменяет `SetFactionXYD`, а не отдельный Shape state.
//! SZL calculation сохраняет first matching inclusive level band, base-money
//! lookup по уровню жертвы, единственную revise-запись и signed x86 wrapping
//! clamp перед wrapping addition.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::public::readwrite::read_to;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleFactionNpcName {
    pub(crate) faction: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) monsters: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleBaseMoney {
    pub(crate) money_level: u32,
    pub(crate) add: u32,
    pub(crate) subtract: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleReviseMoney {
    pub(crate) level_gap_revise: u32,
    pub(crate) money_level_gap_revise: u32,
    pub(crate) revise_min: u32,
    pub(crate) revise_max: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleSzlLevel {
    pub(crate) level: u32,
    pub(crate) min_szl: u32,
    pub(crate) max_szl: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleSzlCalculation {
    pub(crate) value: u32,
    pub(crate) killer_money_level: u32,
    pub(crate) victim_money_level: u32,
    pub(crate) revise_applied: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleFactionRule {
    pub(crate) faction: u32,
    pub(crate) country_a: u32,
    pub(crate) country_b: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleDieBackPosition {
    pub(crate) region: i32,
    pub(crate) destination_region: i32,
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
    pub(crate) faction: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CGodsBattleConf {
    npc_names: Vec<GodsBattleFactionNpcName>,
    base_money: BTreeMap<u32, GodsBattleBaseMoney>,
    revise_money: Vec<GodsBattleReviseMoney>,
    szl_levels: Vec<GodsBattleSzlLevel>,
    faction_rules: Vec<GodsBattleFactionRule>,
    xyd: [u32; 3],
    die_back_positions: Vec<GodsBattleDieBackPosition>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleLoadSection {
    NpcNames,
    BaseMoney,
    ReviseMoney,
    SzlLevels,
    FactionRules,
    DieBackPositions,
}

impl GodsBattleLoadSection {
    pub(crate) const fn path(self) -> &'static [u8] {
        match self {
            Self::NpcNames => b"data/gbNpc.ini",
            Self::BaseMoney => b"data/baseSZL.ini",
            Self::ReviseMoney => b"data/reviseSZL.ini",
            Self::SzlLevels => b"data/SZLlev.ini",
            Self::FactionRules => b"data/ZYFP.ini",
            Self::DieBackPositions => b"data/DiePos.ini",
        }
    }

    pub(crate) const fn missing_notice(self) -> (&'static [u8], &'static [u8]) {
        match self {
            Self::NpcNames => (b"error", b"Can't find file: data/gbNpc.ini "),
            Self::BaseMoney => (b"Error", b"Can't find file : data/baseSZL.ini"),
            Self::ReviseMoney => (b"Error", b"Can't find file: data/reviseSZL.ini"),
            Self::SzlLevels => (b"Error", b"Can't find file: data/SZLlev.ini"),
            Self::FactionRules => (b"Error", b"Can't find file: data/ZYFP.ini"),
            Self::DieBackPositions => (b"Error", b"Can't find file: data/DiePos.ini"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleLoadError {
    MissingResource {
        section: GodsBattleLoadSection,
    },
    UnexpectedEnd {
        section: GodsBattleLoadSection,
        field: &'static str,
    },
    InvalidNumber {
        section: GodsBattleLoadSection,
        field: &'static str,
        token: Vec<u8>,
    },
}

impl fmt::Display for GodsBattleLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingResource { section } => write!(
                formatter,
                "отсутствует GodsBattle resource {}",
                String::from_utf8_lossy(section.path())
            ),
            Self::UnexpectedEnd { section, field } => write!(
                formatter,
                "в GodsBattle resource {} отсутствует поле {field}",
                String::from_utf8_lossy(section.path())
            ),
            Self::InvalidNumber {
                section,
                field,
                token,
            } => write!(
                formatter,
                "поле {field} в GodsBattle resource {} не является числом: {}",
                String::from_utf8_lossy(section.path()),
                String::from_utf8_lossy(token)
            ),
        }
    }
}

impl Error for GodsBattleLoadError {}

fn gods_battle_tokens(source: &[u8]) -> impl Iterator<Item = &[u8]> {
    source
        .split(u8::is_ascii_whitespace)
        .filter(|token| !token.is_empty())
}

fn gods_battle_token<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    section: GodsBattleLoadSection,
    field: &'static str,
) -> Result<&'a [u8], GodsBattleLoadError> {
    tokens
        .next()
        .ok_or(GodsBattleLoadError::UnexpectedEnd { section, field })
}

fn gods_battle_u32<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    section: GodsBattleLoadSection,
    field: &'static str,
) -> Result<u32, GodsBattleLoadError> {
    let token = gods_battle_token(tokens, section, field)?;
    let text = std::str::from_utf8(token).map_err(|_| GodsBattleLoadError::InvalidNumber {
        section,
        field,
        token: token.to_vec(),
    })?;
    text.parse::<u32>()
        .map_err(|_| GodsBattleLoadError::InvalidNumber {
            section,
            field,
            token: token.to_vec(),
        })
}

fn gods_battle_i32<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    section: GodsBattleLoadSection,
    field: &'static str,
) -> Result<i32, GodsBattleLoadError> {
    let token = gods_battle_token(tokens, section, field)?;
    let text = std::str::from_utf8(token).map_err(|_| GodsBattleLoadError::InvalidNumber {
        section,
        field,
        token: token.to_vec(),
    })?;
    text.parse::<i32>()
        .map_err(|_| GodsBattleLoadError::InvalidNumber {
            section,
            field,
            token: token.to_vec(),
        })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleFactionXydUpdate {
    FactionA { previous: u32, current: u32 },
    FactionB { previous: u32, current: u32 },
    IgnoredFaction { faction: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleNpcFactionUpdate {
    pub(crate) record_index: usize,
    pub(crate) previous: i32,
    pub(crate) current: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleDecodeSection {
    NpcNames,
    BaseMoney,
    ReviseMoney,
    SzlLevels,
    FactionRules,
    FactionXyd,
    DieBackPositions,
}

impl fmt::Display for GodsBattleDecodeSection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NpcNames => "NPC-записи",
            Self::BaseMoney => "base-money записи",
            Self::ReviseMoney => "revise-money записи",
            Self::SzlLevels => "SZL-level записи",
            Self::FactionRules => "правила фракций",
            Self::FactionXyd => "faction XYD",
            Self::DieBackPositions => "точки возврата после смерти",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleNpcStringField {
    Name,
    Monsters,
}

impl fmt::Display for GodsBattleNpcStringField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Name => "имя NPC",
            Self::Monsters => "список монстров",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleDecodeError {
    Truncated {
        section: GodsBattleDecodeSection,
        record_index: Option<usize>,
        offset: usize,
        needed: usize,
        available: usize,
    },
    UnterminatedNpcString {
        record_index: usize,
        field: GodsBattleNpcStringField,
        offset: usize,
        available: usize,
    },
}

impl fmt::Display for GodsBattleDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated {
                section,
                record_index,
                offset,
                needed,
                available,
            } => {
                if let Some(record_index) = record_index {
                    write!(
                        formatter,
                        "GodsBattle {section} #{record_index} обрывается на {offset}: нужно {needed}, доступно {available}"
                    )
                } else {
                    write!(
                        formatter,
                        "GodsBattle {section} обрывается на {offset}: нужно {needed}, доступно {available}"
                    )
                }
            }
            Self::UnterminatedNpcString {
                record_index,
                field,
                offset,
                available,
            } => write!(
                formatter,
                "GodsBattle NPC #{record_index}: {field} с позиции {offset} не завершено NUL в доступных {available} байтах"
            ),
        }
    }
}

impl Error for GodsBattleDecodeError {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleDecodeReport {
    pub(crate) npc_names: usize,
    pub(crate) base_money: usize,
    pub(crate) revise_money: usize,
    pub(crate) szl_levels: usize,
    pub(crate) faction_rules: usize,
    pub(crate) die_back_positions: usize,
}

impl CGodsBattleConf {
    /// Воспроизводит `CGodsBattleMgr::DecordFromByteArray` GameServer RVA
    /// `0x000AB260`. Callbacks остаются внутри decoder-а, потому что warning
    /// следует сразу за revise-money, а die-back audit — только после полного
    /// чтения последней секции.
    pub(crate) fn decord_from_byte_array<ReviseMismatch, MissingDieBack>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        mut revise_money_mismatch: ReviseMismatch,
        mut missing_die_back_positions: MissingDieBack,
    ) -> Result<GodsBattleDecodeReport, GodsBattleDecodeError>
    where
        ReviseMismatch: FnMut(),
        MissingDieBack: FnMut(),
    {
        self.npc_names.clear();
        let npc_count =
            read_gods_battle_i32(source, cursor, GodsBattleDecodeSection::NpcNames, None)?;
        for record_index in 0..npc_count.max(0) as usize {
            let faction = read_gods_battle_i32(
                source,
                cursor,
                GodsBattleDecodeSection::NpcNames,
                Some(record_index),
            )?;
            let name = read_gods_battle_c_string(
                source,
                cursor,
                record_index,
                GodsBattleNpcStringField::Name,
            )?;
            let monsters = read_gods_battle_c_string(
                source,
                cursor,
                record_index,
                GodsBattleNpcStringField::Monsters,
            )?;
            self.npc_names.push(GodsBattleFactionNpcName {
                faction,
                name,
                monsters,
            });
        }

        self.base_money.clear();
        let base_money_count =
            read_gods_battle_i32(source, cursor, GodsBattleDecodeSection::BaseMoney, None)?;
        for record_index in 0..base_money_count.max(0) as usize {
            let money_level = read_gods_battle_u32(
                source,
                cursor,
                GodsBattleDecodeSection::BaseMoney,
                Some(record_index),
            )?;
            let add = read_gods_battle_u32(
                source,
                cursor,
                GodsBattleDecodeSection::BaseMoney,
                Some(record_index),
            )?;
            let subtract = read_gods_battle_u32(
                source,
                cursor,
                GodsBattleDecodeSection::BaseMoney,
                Some(record_index),
            )?;
            // `std::map::operator[]` заменяет значение duplicate key.
            self.base_money.insert(
                money_level,
                GodsBattleBaseMoney {
                    money_level,
                    add,
                    subtract,
                },
            );
        }

        self.revise_money.clear();
        let revise_money_count =
            read_gods_battle_i32(source, cursor, GodsBattleDecodeSection::ReviseMoney, None)?;
        for record_index in 0..revise_money_count.max(0) as usize {
            self.revise_money.push(GodsBattleReviseMoney {
                level_gap_revise: read_gods_battle_u32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::ReviseMoney,
                    Some(record_index),
                )?,
                money_level_gap_revise: read_gods_battle_u32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::ReviseMoney,
                    Some(record_index),
                )?,
                revise_min: read_gods_battle_u32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::ReviseMoney,
                    Some(record_index),
                )?,
                revise_max: read_gods_battle_u32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::ReviseMoney,
                    Some(record_index),
                )?,
            });
        }
        if self.revise_money.len() != 1 {
            revise_money_mismatch();
        }

        self.szl_levels.clear();
        let szl_level_count =
            read_gods_battle_i32(source, cursor, GodsBattleDecodeSection::SzlLevels, None)?;
        for record_index in 0..szl_level_count.max(0) as usize {
            self.szl_levels.push(GodsBattleSzlLevel {
                level: read_gods_battle_u32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::SzlLevels,
                    Some(record_index),
                )?,
                min_szl: read_gods_battle_u32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::SzlLevels,
                    Some(record_index),
                )?,
                max_szl: read_gods_battle_u32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::SzlLevels,
                    Some(record_index),
                )?,
            });
        }

        // В отличие от остальных vector-ов Game EXE не вызывает `_Tidy` для
        // `m_vFactionRule`: повторный snapshot дополняет прежние правила.
        let faction_rule_count =
            read_gods_battle_i32(source, cursor, GodsBattleDecodeSection::FactionRules, None)?;
        for record_index in 0..faction_rule_count.max(0) as usize {
            self.faction_rules.push(GodsBattleFactionRule {
                faction: read_gods_battle_u32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::FactionRules,
                    Some(record_index),
                )?,
                country_a: read_gods_battle_u32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::FactionRules,
                    Some(record_index),
                )?,
                country_b: read_gods_battle_u32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::FactionRules,
                    Some(record_index),
                )?,
            });
        }

        self.xyd[1] =
            read_gods_battle_u32(source, cursor, GodsBattleDecodeSection::FactionXyd, Some(1))?;
        self.xyd[2] =
            read_gods_battle_u32(source, cursor, GodsBattleDecodeSection::FactionXyd, Some(2))?;

        self.die_back_positions.clear();
        let die_back_count = read_gods_battle_i32(
            source,
            cursor,
            GodsBattleDecodeSection::DieBackPositions,
            None,
        )?;
        for record_index in 0..die_back_count.max(0) as usize {
            self.die_back_positions.push(GodsBattleDieBackPosition {
                region: read_gods_battle_i32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::DieBackPositions,
                    Some(record_index),
                )?,
                destination_region: read_gods_battle_i32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::DieBackPositions,
                    Some(record_index),
                )?,
                left: read_gods_battle_i32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::DieBackPositions,
                    Some(record_index),
                )?,
                top: read_gods_battle_i32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::DieBackPositions,
                    Some(record_index),
                )?,
                right: read_gods_battle_i32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::DieBackPositions,
                    Some(record_index),
                )?,
                bottom: read_gods_battle_i32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::DieBackPositions,
                    Some(record_index),
                )?,
                faction: read_gods_battle_u32(
                    source,
                    cursor,
                    GodsBattleDecodeSection::DieBackPositions,
                    Some(record_index),
                )?,
            });
        }
        if self.die_back_positions.is_empty() {
            missing_die_back_positions();
        }

        Ok(GodsBattleDecodeReport {
            npc_names: self.npc_names.len(),
            base_money: self.base_money.len(),
            revise_money: self.revise_money.len(),
            szl_levels: self.szl_levels.len(),
            faction_rules: self.faction_rules.len(),
            die_back_positions: self.die_back_positions.len(),
        })
    }

    /// Повторяет `LoadFile`, извлекая resource-байты только при достижении
    /// соответствующей секции.
    ///
    /// `resolve_npc_name` — единственная owner-зависимая часть: оригинал World
    /// сразу заменяет string-table ID локализованным именем, а отсутствие ID
    /// превращает имя в пустую C-строку. `m_XYD[1..=2]` этот loader не меняет.
    pub(crate) fn load_from_resources<ReadResource, ResolveNpcName>(
        &mut self,
        read_resource: &mut ReadResource,
        resolve_npc_name: &mut ResolveNpcName,
    ) -> Result<(), GodsBattleLoadError>
    where
        ReadResource: FnMut(GodsBattleLoadSection) -> Option<Vec<u8>>,
        ResolveNpcName: FnMut(&[u8]) -> Option<Vec<u8>>,
    {
        self.npc_names.clear();
        let source = read_resource(GodsBattleLoadSection::NpcNames).ok_or(
            GodsBattleLoadError::MissingResource {
                section: GodsBattleLoadSection::NpcNames,
            },
        )?;
        let mut tokens = gods_battle_tokens(&source);
        while read_to(&mut tokens, b"*") {
            let faction = gods_battle_i32(&mut tokens, GodsBattleLoadSection::NpcNames, "faction")?;
            let name_id = gods_battle_token(
                &mut tokens,
                GodsBattleLoadSection::NpcNames,
                "StringTable ID",
            )?;
            let monsters =
                gods_battle_token(&mut tokens, GodsBattleLoadSection::NpcNames, "monsters")?;
            self.npc_names.push(GodsBattleFactionNpcName {
                faction,
                name: resolve_npc_name(name_id).unwrap_or_default(),
                monsters: monsters.to_vec(),
            });
        }

        self.base_money.clear();
        let source = read_resource(GodsBattleLoadSection::BaseMoney).ok_or(
            GodsBattleLoadError::MissingResource {
                section: GodsBattleLoadSection::BaseMoney,
            },
        )?;
        let mut tokens = gods_battle_tokens(&source);
        while read_to(&mut tokens, b"*") {
            let money_level =
                gods_battle_u32(&mut tokens, GodsBattleLoadSection::BaseMoney, "money level")?;
            let add = gods_battle_u32(&mut tokens, GodsBattleLoadSection::BaseMoney, "add")?;
            let subtract =
                gods_battle_u32(&mut tokens, GodsBattleLoadSection::BaseMoney, "subtract")?;
            // `std::map::operator[]` оригинал owner-а заменяет duplicate key.
            self.base_money.insert(
                money_level,
                GodsBattleBaseMoney {
                    money_level,
                    add,
                    subtract,
                },
            );
        }

        self.revise_money.clear();
        let source = read_resource(GodsBattleLoadSection::ReviseMoney).ok_or(
            GodsBattleLoadError::MissingResource {
                section: GodsBattleLoadSection::ReviseMoney,
            },
        )?;
        let mut tokens = gods_battle_tokens(&source);
        while read_to(&mut tokens, b"*") {
            self.revise_money.push(GodsBattleReviseMoney {
                level_gap_revise: gods_battle_u32(
                    &mut tokens,
                    GodsBattleLoadSection::ReviseMoney,
                    "level gap revise",
                )?,
                money_level_gap_revise: gods_battle_u32(
                    &mut tokens,
                    GodsBattleLoadSection::ReviseMoney,
                    "money level gap revise",
                )?,
                revise_min: gods_battle_u32(
                    &mut tokens,
                    GodsBattleLoadSection::ReviseMoney,
                    "revise min",
                )?,
                revise_max: gods_battle_u32(
                    &mut tokens,
                    GodsBattleLoadSection::ReviseMoney,
                    "revise max",
                )?,
            });
        }

        self.szl_levels.clear();
        let source = read_resource(GodsBattleLoadSection::SzlLevels).ok_or(
            GodsBattleLoadError::MissingResource {
                section: GodsBattleLoadSection::SzlLevels,
            },
        )?;
        let mut tokens = gods_battle_tokens(&source);
        while read_to(&mut tokens, b"*") {
            self.szl_levels.push(GodsBattleSzlLevel {
                level: gods_battle_u32(&mut tokens, GodsBattleLoadSection::SzlLevels, "level")?,
                min_szl: gods_battle_u32(
                    &mut tokens,
                    GodsBattleLoadSection::SzlLevels,
                    "minimum SZL",
                )?,
                max_szl: gods_battle_u32(
                    &mut tokens,
                    GodsBattleLoadSection::SzlLevels,
                    "maximum SZL",
                )?,
            });
        }

        self.faction_rules.clear();
        let source = read_resource(GodsBattleLoadSection::FactionRules).ok_or(
            GodsBattleLoadError::MissingResource {
                section: GodsBattleLoadSection::FactionRules,
            },
        )?;
        let mut tokens = gods_battle_tokens(&source);
        while read_to(&mut tokens, b"*") {
            self.faction_rules.push(GodsBattleFactionRule {
                faction: gods_battle_u32(
                    &mut tokens,
                    GodsBattleLoadSection::FactionRules,
                    "faction",
                )?,
                country_a: gods_battle_u32(
                    &mut tokens,
                    GodsBattleLoadSection::FactionRules,
                    "country A",
                )?,
                country_b: gods_battle_u32(
                    &mut tokens,
                    GodsBattleLoadSection::FactionRules,
                    "country B",
                )?,
            });
        }

        self.die_back_positions.clear();
        let source = read_resource(GodsBattleLoadSection::DieBackPositions).ok_or(
            GodsBattleLoadError::MissingResource {
                section: GodsBattleLoadSection::DieBackPositions,
            },
        )?;
        let mut tokens = gods_battle_tokens(&source);
        while read_to(&mut tokens, b"*") {
            self.die_back_positions.push(GodsBattleDieBackPosition {
                region: gods_battle_i32(
                    &mut tokens,
                    GodsBattleLoadSection::DieBackPositions,
                    "region",
                )?,
                faction: gods_battle_u32(
                    &mut tokens,
                    GodsBattleLoadSection::DieBackPositions,
                    "faction",
                )?,
                destination_region: gods_battle_i32(
                    &mut tokens,
                    GodsBattleLoadSection::DieBackPositions,
                    "destination region",
                )?,
                left: gods_battle_i32(
                    &mut tokens,
                    GodsBattleLoadSection::DieBackPositions,
                    "left",
                )?,
                top: gods_battle_i32(&mut tokens, GodsBattleLoadSection::DieBackPositions, "top")?,
                right: gods_battle_i32(
                    &mut tokens,
                    GodsBattleLoadSection::DieBackPositions,
                    "right",
                )?,
                bottom: gods_battle_i32(
                    &mut tokens,
                    GodsBattleLoadSection::DieBackPositions,
                    "bottom",
                )?,
            });
        }
        Ok(())
    }

    pub(crate) fn push_npc(&mut self, npc: GodsBattleFactionNpcName) {
        self.npc_names.push(npc);
    }

    pub(crate) fn insert_base_money(
        &mut self,
        key: u32,
        value: GodsBattleBaseMoney,
    ) -> Option<GodsBattleBaseMoney> {
        self.base_money.insert(key, value)
    }

    pub(crate) fn push_revise_money(&mut self, value: GodsBattleReviseMoney) {
        self.revise_money.push(value);
    }

    pub(crate) fn push_szl_level(&mut self, value: GodsBattleSzlLevel) {
        self.szl_levels.push(value);
    }

    pub(crate) fn push_faction_rule(&mut self, value: GodsBattleFactionRule) {
        self.faction_rules.push(value);
    }

    pub(crate) fn faction_for_country(&self, country: u8) -> Option<u32> {
        self.faction_rules
            .iter()
            .find(|rule| {
                u32::from(country) == rule.country_a || u32::from(country) == rule.country_b
            })
            .map(|rule| rule.faction)
    }

    pub(crate) fn calculate_szl_gain(
        &self,
        killer_level: u8,
        killer_szl: u32,
        victim_level: u8,
        victim_szl: u32,
    ) -> GodsBattleSzlCalculation {
        self.calculate_szl_change(killer_level, killer_szl, victim_level, victim_szl, true)
    }

    pub(crate) fn calculate_szl_loss(
        &self,
        killer_level: u8,
        killer_szl: u32,
        victim_level: u8,
        victim_szl: u32,
    ) -> GodsBattleSzlCalculation {
        self.calculate_szl_change(killer_level, killer_szl, victim_level, victim_szl, false)
    }

    fn calculate_szl_change(
        &self,
        killer_level: u8,
        killer_szl: u32,
        victim_level: u8,
        victim_szl: u32,
        gain: bool,
    ) -> GodsBattleSzlCalculation {
        let killer_money_level = self.szl_level(killer_szl).unwrap_or(0);
        let victim_money_level = self.szl_level(victim_szl).unwrap_or(0);
        let mut value = self
            .base_money
            .get(&victim_money_level)
            .map(|money| if gain { money.add } else { money.subtract })
            .unwrap_or(0);
        let Some(revise) = self
            .revise_money
            .as_slice()
            .first()
            .filter(|_| self.revise_money.len() == 1)
        else {
            return GodsBattleSzlCalculation {
                value,
                killer_money_level,
                victim_money_level,
                revise_applied: false,
            };
        };
        let mut adjustment = i32::from(victim_level)
            .wrapping_sub(i32::from(killer_level))
            .wrapping_mul(revise.level_gap_revise as i32)
            .wrapping_add(
                (victim_money_level as i32)
                    .wrapping_sub(killer_money_level as i32)
                    .wrapping_mul(revise.money_level_gap_revise as i32),
            );
        if adjustment > revise.revise_max as i32 {
            adjustment = revise.revise_max as i32;
        }
        if adjustment < revise.revise_min as i32 {
            adjustment = revise.revise_min as i32;
        }
        value = value.wrapping_add(adjustment as u32);
        GodsBattleSzlCalculation {
            value,
            killer_money_level,
            victim_money_level,
            revise_applied: true,
        }
    }

    pub(crate) fn szl_level(&self, szl: u32) -> Option<u32> {
        self.szl_levels
            .iter()
            .find(|level| level.min_szl <= szl && szl <= level.max_szl)
            .map(|level| level.level)
    }

    pub(crate) fn set_xyd_from_db(&mut self, faction_one: u32, faction_two: u32) {
        self.xyd[1] = faction_one;
        self.xyd[2] = faction_two;
    }

    pub(crate) const fn faction_xyd(&self) -> (u32, u32) {
        (self.xyd[1], self.xyd[2])
    }

    pub(crate) fn set_faction_xyd(&mut self, faction: i32, xyd: u32) -> GodsBattleFactionXydUpdate {
        match faction {
            1 => {
                let previous = std::mem::replace(&mut self.xyd[1], xyd);
                GodsBattleFactionXydUpdate::FactionA {
                    previous,
                    current: xyd,
                }
            }
            2 => {
                let previous = std::mem::replace(&mut self.xyd[2], xyd);
                GodsBattleFactionXydUpdate::FactionB {
                    previous,
                    current: xyd,
                }
            }
            faction => GodsBattleFactionXydUpdate::IgnoredFaction { faction },
        }
    }

    pub(crate) fn set_npc_faction(
        &mut self,
        name: &[u8],
        faction: i32,
    ) -> Option<GodsBattleNpcFactionUpdate> {
        let (record_index, npc) = self
            .npc_names
            .iter_mut()
            .enumerate()
            .find(|(_, npc)| npc.name == name)?;
        let previous = std::mem::replace(&mut npc.faction, faction);
        Some(GodsBattleNpcFactionUpdate {
            record_index,
            previous,
            current: faction,
        })
    }

    pub(crate) fn npc_names(&self) -> &[GodsBattleFactionNpcName] {
        &self.npc_names
    }

    pub(crate) fn npc_by_name(&self, name: &[u8]) -> Option<&GodsBattleFactionNpcName> {
        self.npc_names.iter().find(|npc| npc.name == name)
    }

    pub(crate) fn npc_name_by_monster(&self, original_name: &[u8]) -> Option<&[u8]> {
        self.npc_names
            .iter()
            .find(|npc| {
                original_name.is_empty()
                    || npc
                        .monsters
                        .windows(original_name.len())
                        .any(|candidate| candidate == original_name)
            })
            .map(|npc| npc.name.as_slice())
    }

    pub(crate) fn push_die_back_position(&mut self, value: GodsBattleDieBackPosition) {
        self.die_back_positions.push(value);
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), GodsBattleSerializeError> {
        write_gods_battle_count(
            destination,
            GodsBattleCollection::NpcNames,
            self.npc_names.len(),
        )?;
        for (record_index, npc) in self.npc_names.iter().enumerate() {
            destination.extend_from_slice(&npc.faction.to_le_bytes());
            write_gods_battle_string(
                destination,
                record_index,
                GodsBattleStringField::NpcName,
                &npc.name,
            )?;
            write_gods_battle_string(
                destination,
                record_index,
                GodsBattleStringField::Monsters,
                &npc.monsters,
            )?;
        }

        write_gods_battle_count(
            destination,
            GodsBattleCollection::BaseMoney,
            self.base_money.len(),
        )?;
        for money in self.base_money.values() {
            destination.extend_from_slice(&money.money_level.to_le_bytes());
            destination.extend_from_slice(&money.add.to_le_bytes());
            destination.extend_from_slice(&money.subtract.to_le_bytes());
        }

        write_gods_battle_count(
            destination,
            GodsBattleCollection::ReviseMoney,
            self.revise_money.len(),
        )?;
        for revise in &self.revise_money {
            destination.extend_from_slice(&revise.level_gap_revise.to_le_bytes());
            destination.extend_from_slice(&revise.money_level_gap_revise.to_le_bytes());
            destination.extend_from_slice(&revise.revise_min.to_le_bytes());
            destination.extend_from_slice(&revise.revise_max.to_le_bytes());
        }

        write_gods_battle_count(
            destination,
            GodsBattleCollection::SzlLevels,
            self.szl_levels.len(),
        )?;
        for level in &self.szl_levels {
            destination.extend_from_slice(&level.level.to_le_bytes());
            destination.extend_from_slice(&level.min_szl.to_le_bytes());
            destination.extend_from_slice(&level.max_szl.to_le_bytes());
        }

        write_gods_battle_count(
            destination,
            GodsBattleCollection::FactionRules,
            self.faction_rules.len(),
        )?;
        for rule in &self.faction_rules {
            destination.extend_from_slice(&rule.faction.to_le_bytes());
            destination.extend_from_slice(&rule.country_a.to_le_bytes());
            destination.extend_from_slice(&rule.country_b.to_le_bytes());
        }

        destination.extend_from_slice(&self.xyd[1].to_le_bytes());
        destination.extend_from_slice(&self.xyd[2].to_le_bytes());

        write_gods_battle_count(
            destination,
            GodsBattleCollection::DieBackPositions,
            self.die_back_positions.len(),
        )?;
        for position in &self.die_back_positions {
            destination.extend_from_slice(&position.region.to_le_bytes());
            destination.extend_from_slice(&position.destination_region.to_le_bytes());
            destination.extend_from_slice(&position.left.to_le_bytes());
            destination.extend_from_slice(&position.top.to_le_bytes());
            destination.extend_from_slice(&position.right.to_le_bytes());
            destination.extend_from_slice(&position.bottom.to_le_bytes());
            destination.extend_from_slice(&position.faction.to_le_bytes());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleCollection {
    NpcNames,
    BaseMoney,
    ReviseMoney,
    SzlLevels,
    FactionRules,
    DieBackPositions,
}

impl fmt::Display for GodsBattleCollection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NpcNames => "NPC-записей",
            Self::BaseMoney => "base-money записей",
            Self::ReviseMoney => "revise-money записей",
            Self::SzlLevels => "SZL-level записей",
            Self::FactionRules => "правил фракций",
            Self::DieBackPositions => "точек возврата после смерти",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleStringField {
    NpcName,
    Monsters,
}

impl fmt::Display for GodsBattleStringField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NpcName => "имя NPC",
            Self::Monsters => "список монстров",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleSerializeError {
    CountOutOfRange {
        collection: GodsBattleCollection,
        count: usize,
    },
    StringContainsNul {
        record_index: usize,
        field: GodsBattleStringField,
    },
}

impl fmt::Display for GodsBattleSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange { collection, count } => write!(
                formatter,
                "GodsBattle содержит {count} {collection} вне signed 32-битного диапазона"
            ),
            Self::StringContainsNul {
                record_index,
                field,
            } => write!(
                formatter,
                "GodsBattle NPC #{record_index}: {field} содержит внутренний NUL"
            ),
        }
    }
}

impl Error for GodsBattleSerializeError {}

fn write_gods_battle_count(
    destination: &mut Vec<u8>,
    collection: GodsBattleCollection,
    count: usize,
) -> Result<(), GodsBattleSerializeError> {
    let count_i32 = i32::try_from(count)
        .map_err(|_| GodsBattleSerializeError::CountOutOfRange { collection, count })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

fn write_gods_battle_string(
    destination: &mut Vec<u8>,
    record_index: usize,
    field: GodsBattleStringField,
    value: &[u8],
) -> Result<(), GodsBattleSerializeError> {
    if value.contains(&0) {
        return Err(GodsBattleSerializeError::StringContainsNul {
            record_index,
            field,
        });
    }
    destination.extend_from_slice(value);
    destination.push(0);
    Ok(())
}

fn read_gods_battle_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    section: GodsBattleDecodeSection,
    record_index: Option<usize>,
) -> Result<[u8; N], GodsBattleDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(N)) else {
        return Err(GodsBattleDecodeError::Truncated {
            section,
            record_index,
            offset,
            needed: N,
            available,
        });
    };
    *cursor += N;
    Ok(bytes
        .try_into()
        .expect("GodsBattle wire slice имеет запрошенную длину"))
}

fn read_gods_battle_i32(
    source: &[u8],
    cursor: &mut usize,
    section: GodsBattleDecodeSection,
    record_index: Option<usize>,
) -> Result<i32, GodsBattleDecodeError> {
    Ok(i32::from_le_bytes(read_gods_battle_array(
        source,
        cursor,
        section,
        record_index,
    )?))
}

fn read_gods_battle_u32(
    source: &[u8],
    cursor: &mut usize,
    section: GodsBattleDecodeSection,
    record_index: Option<usize>,
) -> Result<u32, GodsBattleDecodeError> {
    Ok(u32::from_le_bytes(read_gods_battle_array(
        source,
        cursor,
        section,
        record_index,
    )?))
}

fn read_gods_battle_c_string(
    source: &[u8],
    cursor: &mut usize,
    record_index: usize,
    field: GodsBattleNpcStringField,
) -> Result<Vec<u8>, GodsBattleDecodeError> {
    let offset = *cursor;
    let Some(remaining) = source.get(offset..) else {
        return Err(GodsBattleDecodeError::UnterminatedNpcString {
            record_index,
            field,
            offset,
            available: 0,
        });
    };
    let Some(length) = remaining.iter().position(|&byte| byte == 0) else {
        return Err(GodsBattleDecodeError::UnterminatedNpcString {
            record_index,
            field,
            offset,
            available: remaining.len(),
        });
    };
    *cursor += length + 1;
    Ok(remaining[..length].to_vec())
}
