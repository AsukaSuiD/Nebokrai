//! Общие форматы ресурсов; выбор корня и публикация остаются у владельца роли.

mod catalog; // каталог индекса и пакетов CClientResource; LoadEx задаёт порядок загрузки.
mod cbattlefairyexpconfig; // CBattleFairyExpConfig: опыт fairy/battle fairy.
mod changebody; // CChangeBodyConf: ограничения смены тела.
mod char_code_filter; // фильтр допустимых однобайтовых кодов.
mod ciqing; // конфигурация CiQing.
mod contributesetup; // CContributeSetup: country contribution.
mod dakongxiangqian; // CDaKongXiangQian: правила вставки больших отверстий.
mod dupliregionsetup; // список дублирующих регионов.
mod emotion; // CEmotion: формат и таблица эмоций.
mod equipmentcomposelist; // две static таблицы преобразования экипировки.
mod fairyexpconf; // CFairyExpConf: опыт fairy.
mod filesinfo; // индекс .ril (LoadFolderInfo).
mod hitlevel; // формат таблицы HitLevel.
mod honorelimilateconfig; // HonorElimilateConfig: порог honor за убийство.
mod incrementshoplist; // CIncrementShopList: increment shop.
mod leitingsetup; // CThingSetup: ежедневные действия.
mod lingbao; // CLingBaoSetup: LingBao.
mod logsystem; // CLogSystem: настройки журнала.
mod globesetup; // CGlobeSetup: глобальный setup.
mod gmlist; // CGMList: операторы.
mod godsbattleconf; // Gods Battle configuration.
mod goodsdestructionconfig; // CGoodsDestroySetup: уничтожение предметов.
mod marker; // поиск маркера в потоке текстовых токенов (ReadTo).
mod monsterlist; // CMonsterList: формат обмена монстрами и их дропом.
mod newskillmonsterlist; // CNewSkillMonserConf: списки монстров новых навыков.
mod regionrouter; // CRegionRouter: маршрутизация между регионами.
mod package; // чтение индекса и распаковка .pak.
mod path; // нормализация путей ресурсов (CheckRFileStr).
mod playerlist; // определения персонажа и их передача World → Game.
mod preciousboxconf; // PreciousBoxConf: Precious Box.
mod prisonconf; // PrisonConf: тюремная конфигурация.
mod quest; // CQuestSystem: каталог определений заданий.
mod quest_text; // текстовые определения заданий (Quest.ini → QuestEx.ini).
mod quest_wire; // wire-передача каталога заданий.
mod regionsetup; // CRegionSetup: ограничения регионов.
mod rfile; // CRFile: курсор памяти/файла.
mod source; // выбор источника ресурса (rfOpen).
mod stringtable; // StringTable: таблица текстов.
mod stringtable_wire; // wire-передача таблицы текстов (MyStringTable).
mod synthesis; // CSynthesis: синтез.
mod taozhuangsetup; // CTaoZhuangSetup: конфигурация комплектов TaoZhuang.
mod tradelist; // CTradeList: торговые списки.
mod wordsfilter; // CWordsFilter: фильтр запрещённых слов.

pub use catalog::{ResourceCatalog, ResourceLoadError, ResourceLoadReport, ResourcePackageLoad};
pub use cbattlefairyexpconfig::{
    BattleFairyExpDecodeError, BattleFairyExpFileLoadError, BattleFairyExpLoadError,
    BattleFairyExpLoadReport, BattleFairyExpSerializeError, CBattleFairyExpConfig,
};
pub use changebody::{
    CChangeBodyConf, ChangeBodyDecodeError, ChangeBodyLoadError, ChangeBodySerializeError,
};
pub use char_code_filter::{CharCodeFilter, CharRange};
pub use ciqing::{
    CCiQingSetup, CiQingComposeNode, CiQingCountSection, CiQingDecodeError,
    CiQingImproveNode, CiQingMakeNode, CiQingSerializationBlock,
};
pub use contributesetup::{
    CContributeSetup, ContributeItem, ContributeSetupDecodeError,
    ContributeSetupFileLoadError, ContributeSetupFormatError, ContributeSetupSerializeError,
};
pub use dakongxiangqian::{
    CDaKongXiangQian, DaKongDecodeError, DaKongDeluxModify, DaKongExternalAttribute,
    DaKongInfo, DaKongLoadError, DaKongLoadReport, DaKongSerializeError,
};
pub use dupliregionsetup::{
    CDupliRegionSetup, DupliRegionDecodeError, DupliRegionEntry, DupliRegionSerializeError,
};
pub use emotion::{
    CEmotion, EmotionDecodeError, EmotionFormatError, EmotionSerializeError,
};
pub use equipmentcomposelist::{
    EquipmentComposeDecodeError, EquipmentComposeList, EquipmentComposeSection,
    EquipmentComposeSerializeError,
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
pub use logsystem::{
    CLogSystem, LOG_SETTINGS_LENGTH, LogSystemDecodeError, LogSystemLoadError,
    LogSystemLoadReport, LogSystemSerializeError,
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
pub use taozhuangsetup::{
    CTaoZhuangSetup, TaoZhuangAddItem, TaoZhuangCountSection, TaoZhuangDecodeError,
    TaoZhuangItem, TaoZhuangSerializationBlock, TaoZhuangStringField,
};
pub use tradelist::{
    CTradeList, Trade, TradeGoods, TradeListDecodeError, TradeListFileLoadError,
    TradeListFormatError, TradeListSerializeError,
};
pub use wordsfilter::{
    CWordsFilter, WordsFilterDecodeError, WordsFilterDecodeSection,
    WordsFilterSerializeError, WordsFilterSerializeSection,
};
