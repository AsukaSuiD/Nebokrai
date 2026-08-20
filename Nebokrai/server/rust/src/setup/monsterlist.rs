//! Реестр и wire-сериализатор списка монстров Miracle.
//!
//! Статус `CMonsterList::AddToByteArray` WorldServer RVA `0x0009C760`:
//! `IMPLEMENTED`; загрузчики и lookup-ы ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! PDB `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp:240`.
//!
//! Exact owner сначала пишет 32-битное число записей ordered map, для каждой
//! записи — непрерывный 160-байтовый scalar prefix, две NUL-terminated legacy
//! строки, 32-битное число skills и элементы по шесть байт. Затем тем же
//! способом идёт ordered drop-map: имя монстра, число drops, а у каждого drop
//! сначала имя и потом 32-байтовый scalar prefix. Ключи map в wire не входят;
//! они задают только порядок обхода.
//!
//! `BTreeMap<Vec<u8>, _>` заменяет `std::map<std::string, _>` и сохраняет
//! byte-лексикографический порядок. `Vec` заменяет list/vector и владеет
//! элементами без raw pointers. Строки остаются произвольными legacy-байтами:
//! первый внутренний NUL завершает их так же, как старый `char const*` helper.
//! Недостижимый для MSVC32 размер коллекции выражен typed-ошибкой вместо
//! усечения `size_t`; корректный wire от этого не меняется.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// Один точный шестибайтовый skill-элемент монстра.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MonsterSkill {
    pub(crate) id: u16,
    pub(crate) level: u16,
    pub(crate) odds: u16,
}

/// Доказанные поля `CMonsterList::tagMonster`, попадающие в setup wire.
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

/// Один 32-байтовый drop prefix с предшествующим legacy-именем.
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

/// Drop-набор одного original monster name.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct MonsterDropList {
    pub(crate) monster_original_name: Vec<u8>,
    pub(crate) drops: Vec<MonsterDrop>,
}

pub(crate) type MonsterRegistry = BTreeMap<Vec<u8>, MonsterProperties>;
pub(crate) type MonsterDropRegistry = BTreeMap<Vec<u8>, MonsterDropList>;

/// Безопасная граница только для состояния вне 32-битного owner-контракта.
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

/// Кодирует оба ordered registry в exact `CMonsterList::AddToByteArray` wire.
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

// Остальной сырой C++ ниже является комментарием, а не Rust-реализацией.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp

// ============================================================================
// FUNCTION: CMonsterList::GetProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp:337
// RVA: 0x0006D680
// ADDRESS: 0046d680
// PROTOTYPE: tagMonster * __cdecl GetProperty(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::GetPropertyByOrginIndex
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp:390
// RVA: 0x0006D6C0
// ADDRESS: 0046d6c0
// PROTOTYPE: tagMonster * __cdecl GetPropertyByOrginIndex(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp:284
// RVA: 0x00070200
// ADDRESS: 00470200
// PROTOTYPE: bool __cdecl DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::GetPropertyByOrginName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp:385
// RVA: 0x000706A0
// ADDRESS: 004706a0
// PROTOTYPE: tagMonster * __cdecl GetPropertyByOrginName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::GetDropGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp:403
// RVA: 0x000707C0
// ADDRESS: 004707c0
// PROTOTYPE: tagDropGoods * __cdecl GetDropGoods(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c3bd6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp
// RVA: 0x001C3BD6
// ADDRESS: 005c3bd6
// PROTOTYPE: undefined Catch@005c3bd6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c3cdc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp
// RVA: 0x001C3CDC
// ADDRESS: 005c3cdc
// PROTOTYPE: undefined Catch@005c3cdc()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c3db6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp
// RVA: 0x001C3DB6
// ADDRESS: 005c3db6
// PROTOTYPE: undefined Catch@005c3db6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp

// ============================================================================
// FUNCTION: Catch@0043cdd6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp
// RVA: 0x0003CDD6
// ADDRESS: 0043cdd6
// PROTOTYPE: undefined Catch@0043cdd6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0043ced8
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp
// RVA: 0x0003CED8
// ADDRESS: 0043ced8
// PROTOTYPE: undefined Catch@0043ced8()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0043d08c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp
// RVA: 0x0003D08C
// ADDRESS: 0043d08c
// PROTOTYPE: undefined Catch@0043d08c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0043d29a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp
// RVA: 0x0003D29A
// ADDRESS: 0043d29a
// PROTOTYPE: undefined Catch@0043d29a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0043d359
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp
// RVA: 0x0003D359
// ADDRESS: 0043d359
// PROTOTYPE: undefined Catch@0043d359()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0043d766
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp
// RVA: 0x0003D766
// ADDRESS: 0043d766
// PROTOTYPE: undefined Catch@0043d766()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp:240
// RVA: 0x0009C760
// ADDRESS: 0049c760
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::LoadMonsterList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp:38
// RVA: 0x0009F450
// ADDRESS: 0049f450
// PROTOTYPE: int __cdecl LoadMonsterList(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::LoadDropGoodsList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp:140
// RVA: 0x0009FB10
// ADDRESS: 0049fb10
// PROTOTYPE: bool __cdecl LoadDropGoodsList(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::GetPropertyByOrginName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp:385
// RVA: 0x000A0230
// ADDRESS: 004a0230
// PROTOTYPE: tagMonster * __cdecl GetPropertyByOrginName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052dd00
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp
// RVA: 0x0012DD00
// ADDRESS: 0052dd00
// PROTOTYPE: undefined Unwind@0052dd00()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052dd20
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp
// RVA: 0x0012DD20
// ADDRESS: 0052dd20
// PROTOTYPE: undefined Unwind@0052dd20()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052dd70
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\monsterlist.cpp
// RVA: 0x0012DD70
// ADDRESS: 0052dd70
// PROTOTYPE: undefined Unwind@0052dd70()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
