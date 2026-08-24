//! Общий wire-реестр `CMonsterList`, подтверждённый парными World/Game
//! owners: World serializer — `worldserver.exe/.pdb`, Game decoder —
//! `gameserver.exe/.pdb`; исходник `server/setup/monsterlist.h/.cpp`.
//!
//! Wire пишет ordered monster map: 160-байтный scalar prefix, две C-строки и
//! шестибайтные skills; затем ordered drop map с именами и 32-байтными records.
//! Map keys задают порядок, но отдельно не передаются.
//!
//! `BTreeMap<Vec<u8>, _>` сохраняет byte-лексикографический порядок, `Vec` —
//! порядок skills/drops. Первый внутренний NUL завершает legacy-строку.
//! Game decode очищает оба registry до count и публикует только полностью
//! прочитанные records; generic STL/SSO storage заменён owned Rust containers.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MonsterSkill {
    pub(crate) id: u16,
    pub(crate) level: u16,
    pub(crate) odds: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct MonsterProperties {
    pub(crate) index: u32,
    pub(crate) picture_id: u32,
    pub(crate) picture_level: u32,
    pub(crate) name_color: u32,
    pub(crate) hp_bar_color: u32,
    pub(crate) sound_id: u32,
    pub(crate) tamable: u32,
    pub(crate) maximum_tame_attempt_count: u32,
    pub(crate) figure: u32,
    pub(crate) level: u32,
    pub(crate) experience: u32,
    pub(crate) yp: u32,
    pub(crate) maximum_hp: u32,
    pub(crate) minimum_attack: u32,
    pub(crate) maximum_attack: u32,
    pub(crate) yao_attack: u32,
    pub(crate) minimum_element: u32,
    pub(crate) maximum_element: u32,
    pub(crate) hit: u32,
    pub(crate) defence: u32,
    pub(crate) dodge: u32,
    pub(crate) attack_speed: u32,
    pub(crate) strike_out_time: u32,
    pub(crate) move_speed: u32,
    pub(crate) element_resistant: u32,
    pub(crate) soul_resistant: u32,
    pub(crate) hp_recover_speed: u32,
    pub(crate) farthest: u32,
    pub(crate) nearest: u32,
    pub(crate) fight_range: u32,
    pub(crate) guard_range: u32,
    pub(crate) chase_range: u32,
    pub(crate) ai: u32,
    pub(crate) race: u32,
    pub(crate) kind: u32,
    pub(crate) move_timer: u32,
    pub(crate) stop_frame: u32,
    pub(crate) ai_interval: u32,
    pub(crate) re_ank: u32,
    pub(crate) attack_avoid: u16,
    pub(crate) element_avoid: u16,
    pub(crate) original_name: Vec<u8>,
    pub(crate) name: Vec<u8>,
    pub(crate) skills: Vec<MonsterSkill>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct MonsterDrop {
    pub(crate) goods_index: i32,
    pub(crate) odds: i32,
    pub(crate) maximum_odds: i32,
    pub(crate) minimum_money: i32,
    pub(crate) maximum_money: i32,
    pub(crate) level: i32,
    pub(crate) level_attenuation: f32,
    pub(crate) level_attenuation_limit: f32,
    pub(crate) name: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct MonsterDropList {
    pub(crate) monster_original_name: Vec<u8>,
    pub(crate) drops: Vec<MonsterDrop>,
}

pub(crate) type MonsterRegistry = BTreeMap<Vec<u8>, MonsterProperties>;
pub(crate) type MonsterDropRegistry = BTreeMap<Vec<u8>, MonsterDropList>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MonsterListDecodeReport {
    pub(crate) monsters: usize,
    pub(crate) drop_groups: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MonsterListDecodeError {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    MissingStringTerminator {
        field: &'static str,
        offset: usize,
    },
}

/// Декодирует парный World wire в Game-owned registries. Оба map очищаются
/// до чтения count; уже опубликованные полные records и cursor не откатываются
/// при safe short-buffer, сохраняя порядок мутаций старого pointer-reader-а.
pub(crate) fn decode_monster_list(
    monsters: &mut MonsterRegistry,
    drop_goods: &mut MonsterDropRegistry,
    source: &[u8],
    cursor: &mut usize,
) -> Result<MonsterListDecodeReport, MonsterListDecodeError> {
    monsters.clear();
    drop_goods.clear();

    let monster_count = read_decode_i32(source, cursor, "monster count")?;
    let mut monster_index = 0;
    while monster_index < monster_count {
        let mut scalar = [0_u32; 39];
        for value in &mut scalar {
            *value = read_decode_u32(source, cursor, "monster scalar prefix")?;
        }
        let attack_avoid = read_decode_u16(source, cursor, "monster attack avoid")?;
        let element_avoid = read_decode_u16(source, cursor, "monster element avoid")?;
        let original_name = read_decode_string(source, cursor, "monster original name")?;
        let name = read_decode_string(source, cursor, "monster display name")?;
        let skill_count = read_decode_i32(source, cursor, "monster skill count")?;
        let mut skills = Vec::new();
        let mut skill_index = 0;
        while skill_index < skill_count {
            skills.push(MonsterSkill {
                id: read_decode_u16(source, cursor, "monster skill ID")?,
                level: read_decode_u16(source, cursor, "monster skill level")?,
                odds: read_decode_u16(source, cursor, "monster skill odds")?,
            });
            skill_index = skill_index.wrapping_add(1);
        }
        let monster = MonsterProperties {
            index: scalar[0],
            picture_id: scalar[1],
            picture_level: scalar[2],
            name_color: scalar[3],
            hp_bar_color: scalar[4],
            sound_id: scalar[5],
            tamable: scalar[6],
            maximum_tame_attempt_count: scalar[7],
            figure: scalar[8],
            level: scalar[9],
            experience: scalar[10],
            yp: scalar[11],
            maximum_hp: scalar[12],
            minimum_attack: scalar[13],
            maximum_attack: scalar[14],
            yao_attack: scalar[15],
            minimum_element: scalar[16],
            maximum_element: scalar[17],
            hit: scalar[18],
            defence: scalar[19],
            dodge: scalar[20],
            attack_speed: scalar[21],
            strike_out_time: scalar[22],
            move_speed: scalar[23],
            element_resistant: scalar[24],
            soul_resistant: scalar[25],
            hp_recover_speed: scalar[26],
            farthest: scalar[27],
            nearest: scalar[28],
            fight_range: scalar[29],
            guard_range: scalar[30],
            chase_range: scalar[31],
            ai: scalar[32],
            race: scalar[33],
            kind: scalar[34],
            move_timer: scalar[35],
            stop_frame: scalar[36],
            ai_interval: scalar[37],
            re_ank: scalar[38],
            attack_avoid,
            element_avoid,
            original_name: original_name.clone(),
            name,
            skills,
        };
        monsters.insert(original_name, monster);
        monster_index = monster_index.wrapping_add(1);
    }

    let drop_group_count = read_decode_i32(source, cursor, "drop group count")?;
    let mut group_index = 0;
    while group_index < drop_group_count {
        let monster_original_name = read_decode_string(source, cursor, "drop monster name")?;
        let drop_count = read_decode_i32(source, cursor, "drop count")?;
        let mut drops = Vec::new();
        let mut drop_index = 0;
        while drop_index < drop_count {
            let name = read_decode_string(source, cursor, "drop goods name")?;
            drops.push(MonsterDrop {
                goods_index: read_decode_i32(source, cursor, "drop goods index")?,
                odds: read_decode_i32(source, cursor, "drop odds")?,
                maximum_odds: read_decode_i32(source, cursor, "drop maximum odds")?,
                minimum_money: read_decode_i32(source, cursor, "drop minimum money")?,
                maximum_money: read_decode_i32(source, cursor, "drop maximum money")?,
                level: read_decode_i32(source, cursor, "drop level")?,
                level_attenuation: f32::from_bits(read_decode_u32(
                    source,
                    cursor,
                    "drop level attenuation",
                )?),
                level_attenuation_limit: f32::from_bits(read_decode_u32(
                    source,
                    cursor,
                    "drop attenuation limit",
                )?),
                name,
            });
            drop_index = drop_index.wrapping_add(1);
        }
        drop_goods.insert(
            monster_original_name.clone(),
            MonsterDropList {
                monster_original_name,
                drops,
            },
        );
        group_index = group_index.wrapping_add(1);
    }

    Ok(MonsterListDecodeReport {
        monsters: monsters.len(),
        drop_groups: drop_goods.len(),
    })
}

fn read_decode_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<Vec<u8>, MonsterListDecodeError> {
    let offset = *cursor;
    let Some(relative_end) = source
        .get(offset..)
        .and_then(|tail| tail.iter().position(|byte| *byte == 0))
    else {
        return Err(MonsterListDecodeError::MissingStringTerminator { field, offset });
    };
    let end = offset + relative_end;
    *cursor = end + 1;
    Ok(source[offset..end].to_vec())
}

fn read_decode_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, MonsterListDecodeError> {
    Ok(read_decode_u32(source, cursor, field)? as i32)
}

fn read_decode_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, MonsterListDecodeError> {
    let bytes = read_decode_bytes::<4>(source, cursor, field)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_decode_u16(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u16, MonsterListDecodeError> {
    let bytes = read_decode_bytes::<2>(source, cursor, field)?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_decode_bytes<const SIZE: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; SIZE], MonsterListDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(SIZE) else {
        return Err(MonsterListDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: SIZE,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(MonsterListDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: SIZE,
            available,
        });
    };
    *cursor = end;
    Ok(bytes.try_into().expect("размер wire scalar проверен"))
}

/// Читает World `data/monsterlist.ini`: поиск `*`, 160-byte scalar prefix,
/// string-table ID и variable skill tail. Registry очищается до чтения, как
/// process-global map оригинала; malformed хвост не создаёт текущую запись.
pub(crate) fn load_monster_list(
    monsters: &mut MonsterRegistry,
    source: &[u8],
    mut resolve_name: impl FnMut(&[u8]) -> Option<Vec<u8>>,
) -> Result<usize, MonsterListLoadError> {
    monsters.clear();
    let tokens: Vec<&[u8]> = source
        .split(|byte| byte.is_ascii_whitespace())
        .filter(|token| !token.is_empty())
        .collect();
    let mut next = 0;
    let mut record = 0;
    while let Some(relative) = tokens[next..].iter().position(|token| *token == b"*") {
        next += relative + 1;
        let mut read = |field: &'static str| {
            let value = tokens
                .get(next)
                .copied()
                .ok_or(MonsterListLoadError::Missing { record, field })?;
            next += 1;
            Ok::<_, MonsterListLoadError>(value)
        };
        let index = parse_monster_number::<u32>(read("index")?, record, "index")?;
        let original_name = read("original name")?.to_vec();
        let name_id = read("name string ID")?.to_vec();
        let mut scalar = [0_u32; 38];
        for value in &mut scalar {
            *value = parse_monster_number::<u32>(read("scalar field")?, record, "scalar field")?;
        }
        let attack_avoid =
            parse_monster_number::<u16>(read("attack avoid")?, record, "attack avoid")?;
        let element_avoid =
            parse_monster_number::<u16>(read("element avoid")?, record, "element avoid")?;
        if attack_avoid > 99 || element_avoid > 99 {
            return Err(MonsterListLoadError::AvoidanceOutsideRange {
                record,
                index,
                attack_avoid,
                element_avoid,
            });
        }
        let skill_count = parse_monster_number::<i32>(read("skill count")?, record, "skill count")?;
        if skill_count < 0 {
            return Err(MonsterListLoadError::NegativeSkillCount {
                record,
                count: skill_count,
            });
        }
        let mut skills = Vec::with_capacity(skill_count as usize);
        for _ in 0..skill_count {
            skills.push(MonsterSkill {
                id: parse_monster_number::<u16>(read("skill ID")?, record, "skill ID")?,
                level: parse_monster_number::<u16>(read("skill level")?, record, "skill level")?,
                odds: parse_monster_number::<u16>(read("skill odds")?, record, "skill odds")?,
            });
        }
        let monster = MonsterProperties {
            index,
            picture_id: scalar[0],
            picture_level: scalar[1],
            name_color: scalar[2],
            hp_bar_color: scalar[3],
            sound_id: scalar[4],
            tamable: scalar[5],
            maximum_tame_attempt_count: scalar[6],
            figure: scalar[7],
            level: scalar[8],
            experience: scalar[9],
            yp: scalar[10],
            maximum_hp: scalar[11],
            minimum_attack: scalar[12],
            maximum_attack: scalar[13],
            yao_attack: scalar[14],
            minimum_element: scalar[15],
            maximum_element: scalar[16],
            hit: scalar[17],
            defence: scalar[18],
            dodge: scalar[19],
            attack_speed: scalar[20],
            strike_out_time: scalar[21],
            move_speed: scalar[22],
            element_resistant: scalar[23],
            soul_resistant: scalar[24],
            hp_recover_speed: scalar[25],
            farthest: scalar[26],
            nearest: scalar[27],
            fight_range: scalar[28],
            guard_range: scalar[29],
            chase_range: scalar[30],
            ai: scalar[31],
            race: scalar[32],
            kind: scalar[33],
            move_timer: scalar[34],
            stop_frame: scalar[35],
            ai_interval: scalar[36],
            re_ank: scalar[37],
            attack_avoid,
            element_avoid,
            name: resolve_name(&name_id).unwrap_or_default(),
            original_name: original_name.clone(),
            skills,
        };
        monsters.insert(original_name, monster);
        record += 1;
    }
    Ok(monsters.len())
}

/// Читает section-based `data/dropgoodslist.ini` и связывает имена предметов
/// с живым GoodsFactory. Повторная секция заменяет предыдущий список монстра,
/// как `operator[]`-ветка точного World owner-а.
pub(crate) fn load_drop_goods_list(
    drops: &mut MonsterDropRegistry,
    source: &[u8],
    mut query_goods_id: impl FnMut(&[u8]) -> u32,
) -> Result<usize, MonsterListLoadError> {
    drops.clear();
    let mut current: Option<Vec<u8>> = None;
    for (line_index, line) in source.split(|byte| *byte == b'\n').enumerate() {
        let tokens: Vec<&[u8]> = line
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|token| !token.is_empty())
            .collect();
        let Some(&first) = tokens.first() else {
            continue;
        };
        if first == b">" {
            let name = tokens
                .get(1)
                .ok_or(MonsterListLoadError::MissingSectionName {
                    line: line_index + 1,
                })?;
            let name = name.to_vec();
            drops.insert(
                name.clone(),
                MonsterDropList {
                    monster_original_name: name.clone(),
                    drops: Vec::new(),
                },
            );
            current = Some(name);
            continue;
        }
        let Some(current_name) = current.as_ref() else {
            continue;
        };
        let values = &tokens[1..];
        let money = first.eq_ignore_ascii_case(b"MONEY");
        let implicit_zero_level = !money && values.len() == 3;
        if values.len() != 4 && !implicit_zero_level {
            return Err(MonsterListLoadError::DropFieldCount {
                line: line_index + 1,
                name: first.to_vec(),
                actual: values.len(),
            });
        }
        let mut drop = MonsterDrop {
            name: first.to_vec(),
            goods_index: query_goods_id(first) as i32,
            ..MonsterDrop::default()
        };
        if money {
            (drop.odds, drop.maximum_odds) =
                parse_pair(values[0], b'/').ok_or_else(|| MonsterListLoadError::DropValue {
                    line: line_index + 1,
                    name: first.to_vec(),
                })?;
            (drop.minimum_money, drop.maximum_money) =
                parse_pair(values[1], b'-').ok_or_else(|| MonsterListLoadError::DropValue {
                    line: line_index + 1,
                    name: first.to_vec(),
                })?;
        } else if !implicit_zero_level {
            drop.level =
                parse_bytes::<i32>(values[0]).ok_or_else(|| MonsterListLoadError::DropValue {
                    line: line_index + 1,
                    name: first.to_vec(),
                })?;
        }
        let odds_index = usize::from(!implicit_zero_level);
        let attenuation_index = if implicit_zero_level { 1 } else { 2 };
        let limit_index = if implicit_zero_level { 2 } else { 3 };
        if !money {
            (drop.odds, drop.maximum_odds) =
                parse_pair(values[odds_index], b'/').ok_or_else(|| {
                    MonsterListLoadError::DropValue {
                        line: line_index + 1,
                        name: first.to_vec(),
                    }
                })?;
        }
        drop.level_attenuation = parse_bytes::<f32>(values[attenuation_index])
            .filter(|value| value.is_finite())
            .ok_or_else(|| MonsterListLoadError::DropValue {
                line: line_index + 1,
                name: first.to_vec(),
            })?;
        drop.level_attenuation_limit = parse_bytes::<f32>(values[limit_index])
            .filter(|value| value.is_finite())
            .ok_or_else(|| MonsterListLoadError::DropValue {
                line: line_index + 1,
                name: first.to_vec(),
            })?;
        drops
            .get_mut(current_name)
            .expect("current section опубликован перед drop-записями")
            .drops
            .push(drop);
    }
    Ok(drops.len())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MonsterListLoadError {
    Missing {
        record: usize,
        field: &'static str,
    },
    Invalid {
        record: usize,
        field: &'static str,
        token: Vec<u8>,
    },
    AvoidanceOutsideRange {
        record: usize,
        index: u32,
        attack_avoid: u16,
        element_avoid: u16,
    },
    NegativeSkillCount {
        record: usize,
        count: i32,
    },
    MissingSectionName {
        line: usize,
    },
    DropFieldCount {
        line: usize,
        name: Vec<u8>,
        actual: usize,
    },
    DropValue {
        line: usize,
        name: Vec<u8>,
    },
}

impl fmt::Display for MonsterListLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ошибка формата MonsterList: {self:?}")
    }
}

impl Error for MonsterListLoadError {}

fn parse_monster_number<T: std::str::FromStr>(
    token: &[u8],
    record: usize,
    field: &'static str,
) -> Result<T, MonsterListLoadError> {
    parse_bytes(token).ok_or_else(|| MonsterListLoadError::Invalid {
        record,
        field,
        token: token.to_vec(),
    })
}

fn parse_bytes<T: std::str::FromStr>(token: &[u8]) -> Option<T> {
    std::str::from_utf8(token).ok()?.parse().ok()
}

fn parse_pair(token: &[u8], separator: u8) -> Option<(i32, i32)> {
    let index = token.iter().position(|byte| *byte == separator)?;
    Some((
        parse_bytes(&token[..index])?,
        parse_bytes(&token[index + 1..])?,
    ))
}

/// Ищет свойство по legacy C-строке original name.
///
/// Original owner сначала строил `std::string` до первого NUL и выполнял
/// `map::find`; отсутствующий ключ возвращал null. `Vec<u8>` исключает
/// lifetime/SSO-детали старой строки, а `Option` сохраняет именно этот
/// различимый результат, не превращая его в разыменование null.
pub(crate) fn get_monster_property_by_origin_name<'registry>(
    monsters: &'registry MonsterRegistry,
    origin_name: &[u8],
) -> Option<&'registry MonsterProperties> {
    let c_string = origin_name
        .split(|byte| *byte == b'\0')
        .next()
        .unwrap_or_default();
    monsters.get(c_string)
}

pub(crate) fn get_monster_property_by_origin_name_mut<'registry>(
    monsters: &'registry mut MonsterRegistry,
    origin_name: &[u8],
) -> Option<&'registry mut MonsterProperties> {
    let c_string = origin_name
        .split(|byte| *byte == b'\0')
        .next()
        .unwrap_or_default();
    monsters.get_mut(c_string)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MonsterListSerializeError {
    pub(crate) owner: &'static str,
    pub(crate) count: usize,
}

impl fmt::Display for MonsterListSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} содержит {} элементов вне 32-битного legacy-диапазона",
            self.owner, self.count
        )
    }
}

impl Error for MonsterListSerializeError {}

pub(crate) fn serialize_monster_list(
    monsters: &MonsterRegistry,
    drop_goods: &MonsterDropRegistry,
    destination: &mut Vec<u8>,
) -> Result<(), MonsterListSerializeError> {
    append_count(destination, "monster registry", monsters.len())?;
    for monster in monsters.values() {
        append_monster_prefix(destination, monster);
        append_legacy_string(destination, &monster.original_name);
        append_legacy_string(destination, &monster.name);
        append_count(destination, "monster skill list", monster.skills.len())?;
        for skill in &monster.skills {
            destination.extend_from_slice(&skill.id.to_le_bytes());
            destination.extend_from_slice(&skill.level.to_le_bytes());
            destination.extend_from_slice(&skill.odds.to_le_bytes());
        }
    }

    append_count(destination, "monster drop registry", drop_goods.len())?;
    for drop_list in drop_goods.values() {
        append_legacy_string(destination, &drop_list.monster_original_name);
        append_count(destination, "monster drop list", drop_list.drops.len())?;
        for drop in &drop_list.drops {
            append_legacy_string(destination, &drop.name);
            for value in [
                drop.goods_index,
                drop.odds,
                drop.maximum_odds,
                drop.minimum_money,
                drop.maximum_money,
                drop.level,
            ] {
                destination.extend_from_slice(&value.to_le_bytes());
            }
            destination.extend_from_slice(&drop.level_attenuation.to_bits().to_le_bytes());
            destination.extend_from_slice(&drop.level_attenuation_limit.to_bits().to_le_bytes());
        }
    }
    Ok(())
}

fn append_monster_prefix(destination: &mut Vec<u8>, monster: &MonsterProperties) {
    for value in [
        monster.index,
        monster.picture_id,
        monster.picture_level,
        monster.name_color,
        monster.hp_bar_color,
        monster.sound_id,
        monster.tamable,
        monster.maximum_tame_attempt_count,
        monster.figure,
        monster.level,
        monster.experience,
        monster.yp,
        monster.maximum_hp,
        monster.minimum_attack,
        monster.maximum_attack,
        monster.yao_attack,
        monster.minimum_element,
        monster.maximum_element,
        monster.hit,
        monster.defence,
        monster.dodge,
        monster.attack_speed,
        monster.strike_out_time,
        monster.move_speed,
        monster.element_resistant,
        monster.soul_resistant,
        monster.hp_recover_speed,
        monster.farthest,
        monster.nearest,
        monster.fight_range,
        monster.guard_range,
        monster.chase_range,
        monster.ai,
        monster.race,
        monster.kind,
        monster.move_timer,
        monster.stop_frame,
        monster.ai_interval,
        monster.re_ank,
    ] {
        destination.extend_from_slice(&value.to_le_bytes());
    }
    destination.extend_from_slice(&monster.attack_avoid.to_le_bytes());
    destination.extend_from_slice(&monster.element_avoid.to_le_bytes());
}

fn append_count(
    destination: &mut Vec<u8>,
    owner: &'static str,
    count: usize,
) -> Result<(), MonsterListSerializeError> {
    let count = u32::try_from(count).map_err(|_| MonsterListSerializeError { owner, count })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn append_legacy_string(destination: &mut Vec<u8>, value: &[u8]) {
    destination.extend_from_slice(value.split(|byte| *byte == 0).next().unwrap_or_default());
    destination.push(0);
}
