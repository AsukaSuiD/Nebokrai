//! Передача каталога по server/setup/questsystem.cpp/.h.
//! Game DecordFromByteArray RVA 0x000627A0; порядок полей и частичных изменений сохранён.
//! Формат и ограничения: docs/gameplay/quests.md.

use std::{error::Error, fmt};

use super::quest::{CQuestSystem, QuestEntry};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuestSystemDecodeOutcome {
    pub declared: i32,
    pub decoded: usize,
    pub retained: usize,
}

impl CQuestSystem {
    /// Декодирует GameServer snapshot RVA `0x000627A0` с исходным порядком
    /// частичных side effects.
    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<QuestSystemDecodeOutcome, QuestSystemDecodeError> {
        self.max_quest_count = read_quest_i32(source, cursor, QuestWireField::MaxQuestCount)?;
        self.level_difference = read_quest_u32(source, cursor, QuestWireField::LevelDifference)?;
        self.player_login_script =
            read_quest_c_string(source, cursor, QuestWireField::PlayerLoginScript)?;
        self.player_level_up_script =
            read_quest_c_string(source, cursor, QuestWireField::PlayerLevelUpScript)?;
        self.player_died_script =
            read_quest_c_string(source, cursor, QuestWireField::PlayerDiedScript)?;

        self.quests.clear();
        let count = read_quest_i32(source, cursor, QuestWireField::QuestCount)?;
        let mut decoded = 0;
        for _ in 0..count.max(0) {
            let id = read_quest_u16(source, cursor, QuestWireField::QuestId)?;
            let quest = QuestEntry {
                id,
                old: read_quest_u32(source, cursor, QuestWireField::QuestOld)?,
                quest_type: read_quest_u32(source, cursor, QuestWireField::QuestType)?,
                level: read_quest_u32(source, cursor, QuestWireField::QuestLevel)?,
                difficulty: read_quest_u32(source, cursor, QuestWireField::QuestDifficulty)?,
                track: read_quest_u32(source, cursor, QuestWireField::QuestTrack)?,
                short_description: read_quest_c_string(
                    source,
                    cursor,
                    QuestWireField::QuestShortDescription,
                )?,
                name: read_quest_c_string(source, cursor, QuestWireField::QuestName)?,
                description: read_quest_c_string(source, cursor, QuestWireField::QuestDescription)?,
                abandon_script: read_quest_c_string(
                    source,
                    cursor,
                    QuestWireField::QuestAbandonScript,
                )?,
                complete_script: read_quest_c_string(
                    source,
                    cursor,
                    QuestWireField::QuestCompleteScript,
                )?,
                region_id: read_quest_i32(source, cursor, QuestWireField::QuestRegionId)?,
                tile_x: read_quest_i32(source, cursor, QuestWireField::QuestTileX)?,
                tile_y: read_quest_i32(source, cursor, QuestWireField::QuestTileY)?,
                effect_id: read_quest_i32(source, cursor, QuestWireField::QuestEffectId)?,
                display: read_quest_u8(source, cursor, QuestWireField::QuestDisplay)? != 0,
            };
            self.quests.insert(id, quest);
            decoded += 1;
        }
        Ok(QuestSystemDecodeOutcome {
            declared: count,
            decoded,
            retained: self.quests.len(),
        })
    }

    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), QuestSystemSerializationBlock> {
        let count = i32::try_from(self.quests.len()).map_err(|_| {
            QuestSystemSerializationBlock::QuestCountOutOfRange {
                count: self.quests.len(),
            }
        })?;
        let mut payload = Vec::new();
        payload.extend_from_slice(&self.max_quest_count.to_le_bytes());
        payload.extend_from_slice(&self.level_difference.to_le_bytes());
        write_quest_string(
            &mut payload,
            None,
            QuestStringField::PlayerLoginScript,
            &self.player_login_script,
        )?;
        write_quest_string(
            &mut payload,
            None,
            QuestStringField::PlayerLevelUpScript,
            &self.player_level_up_script,
        )?;
        write_quest_string(
            &mut payload,
            None,
            QuestStringField::PlayerDiedScript,
            &self.player_died_script,
        )?;
        payload.extend_from_slice(&count.to_le_bytes());
        for quest in self.quests.values() {
            payload.extend_from_slice(&quest.id.to_le_bytes());
            for value in [
                quest.old,
                quest.quest_type,
                quest.level,
                quest.difficulty,
                quest.track,
            ] {
                payload.extend_from_slice(&value.to_le_bytes());
            }
            for (field, value) in [
                (
                    QuestStringField::ShortDescription,
                    quest.short_description.as_slice(),
                ),
                (QuestStringField::Name, quest.name.as_slice()),
                (QuestStringField::Description, quest.description.as_slice()),
                (
                    QuestStringField::AbandonScript,
                    quest.abandon_script.as_slice(),
                ),
                (
                    QuestStringField::CompleteScript,
                    quest.complete_script.as_slice(),
                ),
            ] {
                write_quest_string(&mut payload, Some(quest.id), field, value)?;
            }
            payload.extend_from_slice(&quest.region_id.to_le_bytes());
            payload.extend_from_slice(&quest.tile_x.to_le_bytes());
            payload.extend_from_slice(&quest.tile_y.to_le_bytes());
            payload.extend_from_slice(&quest.effect_id.to_le_bytes());
            payload.push(u8::from(quest.display));
        }
        destination.extend_from_slice(&payload);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuestWireField {
    MaxQuestCount,
    LevelDifference,
    PlayerLoginScript,
    PlayerLevelUpScript,
    PlayerDiedScript,
    QuestCount,
    QuestId,
    QuestOld,
    QuestType,
    QuestLevel,
    QuestDifficulty,
    QuestTrack,
    QuestShortDescription,
    QuestName,
    QuestDescription,
    QuestAbandonScript,
    QuestCompleteScript,
    QuestRegionId,
    QuestTileX,
    QuestTileY,
    QuestEffectId,
    QuestDisplay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuestSystemDecodeError {
    UnexpectedEnd {
        field: QuestWireField,
        offset: usize,
        needed: usize,
        available: usize,
    },
    UnterminatedString {
        field: QuestWireField,
        offset: usize,
    },
}

impl fmt::Display for QuestSystemDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "QuestSystem snapshot обрывается на {field:?} в {offset}: нужно {needed}, доступно {available}"
            ),
            Self::UnterminatedString { field, offset } => write!(
                formatter,
                "QuestSystem snapshot содержит незавершённую строку {field:?} в {offset}"
            ),
        }
    }
}

impl Error for QuestSystemDecodeError {}

fn read_quest_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    size: usize,
    field: QuestWireField,
) -> Result<&'a [u8], QuestSystemDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(size)) else {
        return Err(QuestSystemDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: size,
            available,
        });
    };
    *cursor += size;
    Ok(bytes)
}

fn read_quest_i32(
    source: &[u8],
    cursor: &mut usize,
    field: QuestWireField,
) -> Result<i32, QuestSystemDecodeError> {
    Ok(i32::from_le_bytes(
        read_quest_bytes(source, cursor, 4, field)?
            .try_into()
            .expect("quest signed long уже проверен"),
    ))
}

fn read_quest_u32(
    source: &[u8],
    cursor: &mut usize,
    field: QuestWireField,
) -> Result<u32, QuestSystemDecodeError> {
    Ok(u32::from_le_bytes(
        read_quest_bytes(source, cursor, 4, field)?
            .try_into()
            .expect("quest unsigned long уже проверен"),
    ))
}

fn read_quest_u16(
    source: &[u8],
    cursor: &mut usize,
    field: QuestWireField,
) -> Result<u16, QuestSystemDecodeError> {
    Ok(u16::from_le_bytes(
        read_quest_bytes(source, cursor, 2, field)?
            .try_into()
            .expect("quest unsigned short уже проверен"),
    ))
}

fn read_quest_u8(
    source: &[u8],
    cursor: &mut usize,
    field: QuestWireField,
) -> Result<u8, QuestSystemDecodeError> {
    Ok(read_quest_bytes(source, cursor, 1, field)?[0])
}

fn read_quest_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: QuestWireField,
) -> Result<Vec<u8>, QuestSystemDecodeError> {
    let offset = *cursor;
    let tail = source
        .get(offset..)
        .ok_or(QuestSystemDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: 1,
            available: 0,
        })?;
    let Some(length) = tail.iter().position(|byte| *byte == 0) else {
        return Err(QuestSystemDecodeError::UnterminatedString { field, offset });
    };
    *cursor += length + 1;
    Ok(tail[..length].to_vec())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuestStringField {
    PlayerLoginScript,
    PlayerLevelUpScript,
    PlayerDiedScript,
    ShortDescription,
    Name,
    Description,
    AbandonScript,
    CompleteScript,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuestSystemSerializationBlock {
    QuestCountOutOfRange { count: usize },
    StringContainsNul {
        quest_id: Option<u16>,
        field: QuestStringField,
    },
}

impl fmt::Display for QuestSystemSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QuestCountOutOfRange { count } => write!(
                formatter,
                "QuestSystem содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::StringContainsNul { quest_id, field } => {
                let owner = quest_id
                    .map_or_else(|| "setup".to_owned(), |id| format!("quest {id}"));
                write!(
                    formatter,
                    "QuestSystem {owner}, поле {field:?} содержит внутренний NUL"
                )
            }
        }
    }
}

impl Error for QuestSystemSerializationBlock {}

fn write_quest_string(
    destination: &mut Vec<u8>,
    quest_id: Option<u16>,
    field: QuestStringField,
    value: &[u8],
) -> Result<(), QuestSystemSerializationBlock> {
    if value.contains(&0) {
        return Err(QuestSystemSerializationBlock::StringContainsNul {
            quest_id,
            field,
        });
    }
    destination.extend_from_slice(value);
    destination.push(0);
    Ok(())
}
