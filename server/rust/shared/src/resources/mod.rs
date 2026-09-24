//! Общие форматы ресурсов; выбор корня и публикация остаются у владельца роли.

mod catalog;
mod cbattlefairyexpconfig;
mod changebody;
mod contributesetup;
mod emotion;
mod fairyexpconf;
mod filesinfo;
mod hitlevel;
mod honorelimilateconfig;
mod incrementshoplist;
mod leitingsetup;
mod lingbao;
mod globesetup;
mod gmlist;
mod godsbattleconf;
mod goodsdestructionconfig;
mod marker;
mod monsterlist;
mod newskillmonsterlist;
mod regionrouter;
mod package;
mod path;
mod playerlist;
mod preciousboxconf;
mod prisonconf;
mod quest;
mod quest_text;
mod quest_wire;
mod regionsetup;
mod rfile;
mod source;
mod stringtable;
mod stringtable_wire;
mod synthesis;
mod tradelist;

pub use catalog::{ResourceCatalog, ResourceLoadError, ResourceLoadReport, ResourcePackageLoad};
pub use cbattlefairyexpconfig::{
    BattleFairyExpDecodeError, BattleFairyExpFileLoadError, BattleFairyExpLoadError,
    BattleFairyExpLoadReport, BattleFairyExpSerializeError, CBattleFairyExpConfig,
};
pub use changebody::{
    CChangeBodyConf, ChangeBodyDecodeError, ChangeBodyLoadError, ChangeBodySerializeError,
};
pub use contributesetup::{
    CContributeSetup, ContributeItem, ContributeSetupDecodeError,
    ContributeSetupFileLoadError, ContributeSetupFormatError, ContributeSetupSerializeError,
};
pub use emotion::{
    CEmotion, EmotionDecodeError, EmotionFormatError, EmotionSerializeError,
};
pub use fairyexpconf::{CFairyExpConf, FairyExpLoadError};
pub use filesinfo::{FileInfo, FilesInfo, FilesInfoParseError, PackFileInfo};
pub use incrementshoplist::{
    CIncrementShopList, IncrementShopDecodeError, IncrementShopGoodsQuery,
    IncrementShopGoodsResult, IncrementShopItem, IncrementShopLoadError,
    IncrementShopLoadReport, IncrementShopSerializeError, IncrementShopStringField,
};
pub use leitingsetup::{
    CThingSetup, LeiTingDailyThing, LeiTingLocalTime, LeiTingThingNode,
    ThingSetupCodecError, ThingSetupEmptyFile, ThingSetupFileLoadError,
    ThingSetupLoadReport, ThingSetupTextCutoff, ThingSetupTextCutoffReason,
    ThingSetupTextField,
};
pub use lingbao::{
    CLingBaoSetup, LingBaoDecodeError, LingBaoFirstNode, LingBaoLoadReport,
    LingBaoNodeInfo, LingBaoNodeSection, LingBaoSecondNode, LingBaoSerializationBlock,
    LingBaoThirdNode,
};
pub use hitlevel::{
    CHitLevelSetup, HitLevelDecodeError, HitLevelEntry, HitLevelFormatError,
    HitLevelSerializeError,
};
pub use honorelimilateconfig::{HonorElimilateConfig, HonorEliminateDecodeError};
pub use newskillmonsterlist::{
    NewSkillMonsterConf, NewSkillMonsterDecodeError, NewSkillMonsterFileLoadError,
    NewSkillMonsterLoadError, NewSkillMonsterLoadReport, NewSkillMonsterSerializeError,
};
pub use monsterlist::{
    MonsterDrop, MonsterDropList, MonsterDropRegistry, MonsterListDecodeError,
    MonsterListLoadError, MonsterListSerializeError, MonsterProperties, MonsterRegistry,
    MonsterSkill, decode_monster_list, get_monster_property_by_origin_index,
    get_monster_property_by_origin_name, get_monster_property_by_origin_name_mut,
    get_monster_property_by_picture_id, load_drop_goods_list, load_monster_list,
    serialize_monster_list,
};
pub use gmlist::{
    CGMList, GmInfo, GmListCollection, GmListDecodeError, GmListLoadError,
    GmListSerializationBlock,
};
pub use globesetup::{
    GLOBE_SETUP_BLOB_LENGTH, GlobePlayerPropertyCoefficients, GlobeRpGainPolicy,
    GlobeSetupDecodeError, GlobeSetupLoadError, GlobeSetupLoadReport, GlobeSetupSnapshot,
    GlobeStiffenSetup,
};
pub use godsbattleconf::{
    CGodsBattleConf, GodsBattleBaseMoney, GodsBattleCollection, GodsBattleDecodeError,
    GodsBattleDecodeSection, GodsBattleDieBackPosition, GodsBattleFactionNpcName,
    GodsBattleFactionRule, GodsBattleFactionXydUpdate, GodsBattleLoadError,
    GodsBattleLoadSection, GodsBattleNpcFactionUpdate, GodsBattleNpcStringField,
    GodsBattleReviseMoney, GodsBattleSerializeError, GodsBattleStringField,
    GodsBattleSzlCalculation, GodsBattleSzlLevel,
};
pub use goodsdestructionconfig::{
    GoodsDestroyDecodeError, GoodsDestroyFileLoadError, GoodsDestroyFormatError,
    GoodsDestroyList, GoodsDestroyLoadReport, GoodsDestroySerializeError, GoodsDestroySetup,
};
pub use marker::read_to_marker;
pub use package::{PackageArchive, PackageFileIndex, PackageReadError};
pub use path::{normalize_resource_path, resolve_resource_path};
pub use preciousboxconf::{
    PreciousBox, PreciousBoxConf, PreciousBoxCount, PreciousBoxDecodeError,
    PreciousBoxDecodeField, PreciousBoxItem, PreciousBoxLoadDiagnostic, PreciousBoxLoadError,
    PreciousBoxLoadReport, PreciousBoxOdds, PreciousBoxSerializeError,
};
pub use prisonconf::{
    PrisonConf, PrisonConfDecodeError, PrisonConfFileLoadError, PrisonConfFormatError,
    PrisonConfSerializeError, PrisonParam,
};
pub use playerlist::{
    CPlayerList, PlayerBaseProperties, PlayerBasePropertiesMap, PlayerCreationPropertiesLookup,
    PlayerListDecodeError, PlayerListFormatError, PlayerListLoadReport, PlayerListSerializeError,
    PlayerOriginEquipment, PlayerPropertiesUpgrade, PlayerPropertiesUpgradeLoadReport,
    PlayerPropertiesUpgradeMap,
};
pub use regionrouter::{
    RegionNextNode, RegionRoutePoint, RegionRouteStep, RegionRouter,
    RegionRouterChangeOutcome, RegionRouterDecodeError, RegionRouterDecodeField,
    RegionRouterLoadError, RegionRouterLoadReport, RegionRouterNode,
    RegionRouterSerializeError,
};
pub use quest::{CQuestSystem, QuestEntry};
pub use quest_text::{
    QuestSystemLoadCompletion, QuestSystemLoadReport, QuestTextError, QuestTextErrorKind,
};
pub use regionsetup::{
    CRegionSetup, RegionSetupDecodeError, RegionSetupEntry, RegionSetupLoadError,
    RegionSetupSerializeError,
};
pub use quest_wire::{
    QuestStringField, QuestSystemDecodeError, QuestSystemDecodeOutcome, QuestSystemSerializationBlock,
    QuestWireField,
};
pub use rfile::CRFile;
pub use source::{ResourceOpenError, ResourceSource, open_resource};
pub use stringtable::{StringTable, StringTableParseError, StringTableParseErrorKind};
pub use stringtable_wire::{MyStringTable, MyStringTableDecodeError, MyStringTableDecodeOutcome};
pub use synthesis::{
    CSynthesis, SynthesisCount, SynthesisDecodeError, SynthesisDecodeField,
    SynthesisFormula, SynthesisLoadError, SynthesisLoadReport, SynthesisRecipe,
    SynthesisSerializeError, SynthesisString,
};
pub use tradelist::{
    CTradeList, Trade, TradeGoods, TradeListDecodeError, TradeListFileLoadError,
    TradeListFormatError, TradeListSerializeError,
};
