//! Конфигурация синтеза исторического Miracle.
//!
//! Статус World `CSynthesis::AddToByteArray` RVA `0x0008D560`:
//! `IMPLEMENTED`; XML loader, static queries и Game decoder ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:283`.
//!
//! Wire сначала содержит ordered broadcast map: signed count, затем
//! `u16 tag + C-string`. После него идут signed recipe count и vector recipes:
//! `u32 index + u16 probability + u16 type + u32 goods + i32 coins +
//! i32 prestige + u16 broadcast_tag + C-string key + signed formula count`,
//! затем пары `u32 goods + u32 amount`.
//!
//! Порядок `probability -> type` намеренно обратен layout исходного
//! `tagSynthesis`, где type лежит раньше probability; точный Game decoder также
//! ждёт wire-порядок. Rust не воспроизводит layout/сырой `char*`, но сохраняет
//! этот compatibility quirk явно. `BTreeMap`, `Vec` и owned bytes заменяют
//! только MSVC containers и ручной lifetime.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SynthesisFormula {
    pub(crate) goods_index: u32,
    pub(crate) amount: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SynthesisRecipe {
    pub(crate) synthesis_index: u32,
    pub(crate) synthesis_type: u16,
    pub(crate) probability: u16,
    pub(crate) goods_index: u32,
    pub(crate) coins: i32,
    pub(crate) prestige: i32,
    pub(crate) broadcast_tag: u16,
    pub(crate) key: Vec<u8>,
    pub(crate) formulas: Vec<SynthesisFormula>,
}

/// Safe owner двух исходных static containers `CSynthesis`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CSynthesis {
    broadcasts: BTreeMap<u16, Vec<u8>>,
    recipes: Vec<SynthesisRecipe>,
}

impl CSynthesis {
    /// Сохраняет исходную map assignment семантику duplicate tag-а.
    pub(crate) fn insert_broadcast(&mut self, tag: u16, text: Vec<u8>) -> Option<Vec<u8>> {
        self.broadcasts.insert(tag, text)
    }

    pub(crate) fn push_recipe(&mut self, recipe: SynthesisRecipe) {
        self.recipes.push(recipe);
    }

    pub(crate) fn clear(&mut self) {
        self.broadcasts.clear();
        self.recipes.clear();
    }

    /// Дописывает exact broadcast + recipe wire.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), SynthesisSerializeError> {
        write_count(
            destination,
            self.broadcasts.len(),
            SynthesisCount::Broadcasts,
        )?;
        for (&tag, text) in &self.broadcasts {
            ensure_c_string(text, SynthesisString::Broadcast { tag })?;
            destination.extend_from_slice(&tag.to_le_bytes());
            destination.extend_from_slice(text);
            destination.push(0);
        }

        write_count(
            destination,
            self.recipes.len(),
            SynthesisCount::Recipes,
        )?;
        for recipe in &self.recipes {
            ensure_c_string(
                &recipe.key,
                SynthesisString::RecipeKey {
                    synthesis_index: recipe.synthesis_index,
                },
            )?;
            destination.extend_from_slice(&recipe.synthesis_index.to_le_bytes());
            // Compatibility quirk: EXE отправляет probability раньше type.
            destination.extend_from_slice(&recipe.probability.to_le_bytes());
            destination.extend_from_slice(&recipe.synthesis_type.to_le_bytes());
            destination.extend_from_slice(&recipe.goods_index.to_le_bytes());
            destination.extend_from_slice(&recipe.coins.to_le_bytes());
            destination.extend_from_slice(&recipe.prestige.to_le_bytes());
            destination.extend_from_slice(&recipe.broadcast_tag.to_le_bytes());
            destination.extend_from_slice(&recipe.key);
            destination.push(0);
            write_count(
                destination,
                recipe.formulas.len(),
                SynthesisCount::Formulas {
                    synthesis_index: recipe.synthesis_index,
                },
            )?;
            for formula in &recipe.formulas {
                destination.extend_from_slice(&formula.goods_index.to_le_bytes());
                destination.extend_from_slice(&formula.amount.to_le_bytes());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SynthesisCount {
    Broadcasts,
    Recipes,
    Formulas { synthesis_index: u32 },
}

impl fmt::Display for SynthesisCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Broadcasts => formatter.write_str("broadcast-записей синтеза"),
            Self::Recipes => formatter.write_str("рецептов синтеза"),
            Self::Formulas { synthesis_index } => {
                write!(formatter, "формул рецепта {synthesis_index}")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SynthesisString {
    Broadcast { tag: u16 },
    RecipeKey { synthesis_index: u32 },
}

impl fmt::Display for SynthesisString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Broadcast { tag } => write!(formatter, "broadcast-текст с tag {tag}"),
            Self::RecipeKey { synthesis_index } => {
                write!(formatter, "ключ рецепта {synthesis_index}")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SynthesisSerializeError {
    CountOutOfRange {
        field: SynthesisCount,
        count: usize,
    },
    StringContainsNul {
        field: SynthesisString,
    },
}

impl fmt::Display for SynthesisSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange { field, count } => write!(
                formatter,
                "количество {field} ({count}) не помещается в signed 32-битный диапазон"
            ),
            Self::StringContainsNul { field } => write!(formatter, "{field} содержит внутренний NUL"),
        }
    }
}

impl Error for SynthesisSerializeError {}

fn write_count(
    destination: &mut Vec<u8>,
    count: usize,
    field: SynthesisCount,
) -> Result<(), SynthesisSerializeError> {
    let count_i32 = i32::try_from(count)
        .map_err(|_| SynthesisSerializeError::CountOutOfRange { field, count })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

fn ensure_c_string(value: &[u8], field: SynthesisString) -> Result<(), SynthesisSerializeError> {
    if value.contains(&0) {
        return Err(SynthesisSerializeError::StringContainsNul { field });
    }
    Ok(())
}

// Сырой C++ ниже сохранён как локальная документация XML loader-а, static
// queries и Game decoder-а, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp

// ============================================================================
// FUNCTION: Catch@005c0171
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x001C0171
// ADDRESS: 005c0171
// PROTOTYPE: undefined Catch@005c0171()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::CheckType
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:367
// RVA: 0x001C1070
// ADDRESS: 005c1070
// PROTOTYPE: int __cdecl CheckType(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetProbability
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:408
// RVA: 0x001C1120
// ADDRESS: 005c1120
// PROTOTYPE: ushort __cdecl GetProbability(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetGoodsIndex
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:418
// RVA: 0x001C1150
// ADDRESS: 005c1150
// PROTOTYPE: ulong __cdecl GetGoodsIndex(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetCoins
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:439
// RVA: 0x001C1180
// ADDRESS: 005c1180
// PROTOTYPE: long __cdecl GetCoins(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetPrestige
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:450
// RVA: 0x001C11B0
// ADDRESS: 005c11b0
// PROTOTYPE: long __cdecl GetPrestige(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetKey
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:460
// RVA: 0x001C11E0
// ADDRESS: 005c11e0
// PROTOTYPE: char * __cdecl GetKey(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetBroadcastTag
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:469
// RVA: 0x001C1210
// ADDRESS: 005c1210
// PROTOTYPE: ushort __cdecl GetBroadcastTag(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetBroadcast
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:536
// RVA: 0x001C1310
// ADDRESS: 005c1310
// PROTOTYPE: basic_string<char,std::char_traits<char>,std::allocator<char>_> __cdecl GetBroadcast(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetFormula
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:479
// RVA: 0x001C26B0
// ADDRESS: 005c26b0
// PROTOTYPE: long __cdecl GetFormula(ulong param_1, vector<CSynthesis::tagFormula,std::allocator<CSynthesis::tagFormula>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::GetInitForm
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:389
// RVA: 0x001C2BF0
// ADDRESS: 005c2bf0
// PROTOTYPE: bool __cdecl GetInitForm(ushort param_1, vector<CSynthesis::tagNameAndIndex,std::allocator<CSynthesis::tagNameAndIndex>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:323
// RVA: 0x001C31B0
// ADDRESS: 005c31b0
// PROTOTYPE: bool __cdecl DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp

// ============================================================================
// FUNCTION: Catch@00488321
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00088321
// ADDRESS: 00488321
// PROTOTYPE: undefined Catch@00488321()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004887a1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x000887A1
// ADDRESS: 004887a1
// PROTOTYPE: undefined Catch@004887a1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00488831
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00088831
// ADDRESS: 00488831
// PROTOTYPE: undefined Catch@00488831()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangAddItem::stTaoZhuangAddItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x000888D0
// ADDRESS: 004888d0
// PROTOTYPE: undefined __thiscall stTaoZhuangAddItem(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangAddItem::~stTaoZhuangAddItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00088960
// ADDRESS: 00488960
// PROTOTYPE: void __thiscall ~stTaoZhuangAddItem(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangAddItem::stTaoZhuangAddItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00088A10
// ADDRESS: 00488a10
// PROTOTYPE: undefined __thiscall stTaoZhuangAddItem(stTaoZhuangAddItem * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00488af1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00088AF1
// ADDRESS: 00488af1
// PROTOTYPE: undefined Catch@00488af1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00488ba6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00088BA6
// ADDRESS: 00488ba6
// PROTOTYPE: undefined Catch@00488ba6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004891f1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x000891F1
// ADDRESS: 004891f1
// PROTOTYPE: undefined Catch@004891f1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004893f1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x000893F1
// ADDRESS: 004893f1
// PROTOTYPE: undefined Catch@004893f1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangItemNode::stTaoZhuangItemNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00089410
// ADDRESS: 00489410
// PROTOTYPE: undefined __thiscall stTaoZhuangItemNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangItemNode::~stTaoZhuangItemNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x000894C0
// ADDRESS: 004894c0
// PROTOTYPE: void __thiscall ~stTaoZhuangItemNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::stTaoZhuangItemNode::stTaoZhuangItemNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x000895D0
// ADDRESS: 004895d0
// PROTOTYPE: undefined __thiscall stTaoZhuangItemNode(stTaoZhuangItemNode * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0048973f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x0008973F
// ADDRESS: 0048973f
// PROTOTYPE: undefined Catch@0048973f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:283
// RVA: 0x0008D560
// ADDRESS: 0048d560
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::LoadSynthesisList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp:34
// RVA: 0x0008EE50
// ADDRESS: 0048ee50
// PROTOTYPE: int __cdecl LoadSynthesisList(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// FUNCTION: Unwind@005314a0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x001314A0
// ADDRESS: 005314a0
// PROTOTYPE: undefined Unwind@005314a0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//









// ============================================================================
// FUNCTION: Unwind@00531560
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\synthesis.cpp
// RVA: 0x00131560
// ADDRESS: 00531560
// PROTOTYPE: undefined Unwind@00531560()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//








// COMPONENT_VARIANT_END: WorldServer
