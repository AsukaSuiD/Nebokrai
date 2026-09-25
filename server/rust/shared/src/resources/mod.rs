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

pub use catalog::{ResourceCatalog, ResourceLoadError, ResourceLoadReport, ResourcePackageLoad}; // каталог ресурсов и отчёт его загрузки.
pub use cbattlefairyexpconfig::{
    BattleFairyExpDecodeError, BattleFairyExpFileLoadError, BattleFairyExpLoadError,
    BattleFairyExpLoadReport, BattleFairyExpSerializeError, CBattleFairyExpConfig,
}; // кодек опыта battle fairy и его отчёт.
pub use changebody::{
    CChangeBodyConf, ChangeBodyDecodeError, ChangeBodyLoadError, ChangeBodySerializeError,
}; // ограничения смены тела и их кодек-ошибки.
pub use char_code_filter::{CharCodeFilter, CharRange}; // фильтр однобайтовых кодов и диапазон.
pub use ciqing::{
    CCiQingSetup, CiQingComposeNode, CiQingCountSection, CiQingDecodeError,
    CiQingImproveNode, CiQingMakeNode, CiQingSerializationBlock,
}; // конфигурация CiQing: узлы compose/make/improve.
pub use contributesetup::{
    CContributeSetup, ContributeItem, ContributeSetupDecodeError,
    ContributeSetupFileLoadError, ContributeSetupFormatError, ContributeSetupSerializeError,
}; // country contribution и её кодек-ошибки.
pub use dakongxiangqian::{
    CDaKongXiangQian, DaKongDecodeError, DaKongDeluxModify, DaKongExternalAttribute,
    DaKongInfo, DaKongLoadError, DaKongLoadReport, DaKongSerializeError,
}; // правила больших отверстий и их атрибуты.
pub use dupliregionsetup::{
    CDupliRegionSetup, DupliRegionDecodeError, DupliRegionEntry, DupliRegionSerializeError,
}; // список дублирующих регионов и его записи.
pub use emotion::{
    CEmotion, EmotionDecodeError, EmotionFormatError, EmotionSerializeError,
}; // таблица эмоций и её форматные ошибки.
pub use equipmentcomposelist::{
    EquipmentComposeDecodeError, EquipmentComposeList, EquipmentComposeSection,
    EquipmentComposeSerializeError,
}; // таблицы преобразования экипировки и их секции.
pub use fairyexpconf::{CFairyExpConf, FairyExpLoadError}; // опыт fairy и ошибка загрузки.
pub use filesinfo::{FileInfo, FilesInfo, FilesInfoParseError, PackFileInfo}; // индекс .ril и его элементы.
pub use incrementshoplist::{
    CIncrementShopList, IncrementShopDecodeError, IncrementShopGoodsQuery,
    IncrementShopGoodsResult, IncrementShopItem, IncrementShopLoadError,
    IncrementShopLoadReport, IncrementShopSerializeError, IncrementShopStringField,
}; // increment shop: товары, запросы и кодек-ошибки.
pub use leitingsetup::{
    CThingSetup, LeiTingDailyThing, LeiTingLocalTime, LeiTingThingNode,
    ThingSetupCodecError, ThingSetupEmptyFile, ThingSetupFileLoadError,
    ThingSetupLoadReport, ThingSetupTextCutoff, ThingSetupTextCutoffReason,
    ThingSetupTextField,
}; // ежедневные действия LeiTing и их кодек.
pub use logsystem::{
    CLogSystem, LOG_SETTINGS_LENGTH, LogSystemDecodeError, LogSystemLoadError,
    LogSystemLoadReport, LogSystemSerializeError,
}; // настройки журнала, длина их blob-а и ошибки кодека.
pub use lingbao::{
    CLingBaoSetup, LingBaoDecodeError, LingBaoFirstNode, LingBaoLoadReport,
    LingBaoNodeInfo, LingBaoNodeSection, LingBaoSecondNode, LingBaoSerializationBlock,
    LingBaoThirdNode,
}; // конфигурация LingBao: узлы трёх уровней.
pub use hitlevel::{
    CHitLevelSetup, HitLevelDecodeError, HitLevelEntry, HitLevelFormatError,
    HitLevelSerializeError,
}; // таблица HitLevel и её записи.
pub use honorelimilateconfig::{HonorElimilateConfig, HonorEliminateDecodeError}; // порог honor за убийство и ошибка декодирования.
pub use newskillmonsterlist::{
    NewSkillMonsterConf, NewSkillMonsterDecodeError, NewSkillMonsterFileLoadError,
    NewSkillMonsterLoadError, NewSkillMonsterLoadReport, NewSkillMonsterSerializeError,
}; // списки монстров новых навыков и их кодек-ошибки.
pub use monsterlist::{
    MonsterDrop, MonsterDropList, MonsterDropRegistry, MonsterListDecodeError,
    MonsterListLoadError, MonsterListSerializeError, MonsterProperties, MonsterRegistry,
    MonsterSkill, decode_monster_list, get_monster_property_by_origin_index,
    get_monster_property_by_origin_name, get_monster_property_by_origin_name_mut,
    get_monster_property_by_picture_id, load_drop_goods_list, load_monster_list,
    serialize_monster_list,
}; // монстры и их дроп: реестры, кодек и free-функции загрузки.
pub use gmlist::{
    CGMList, GmInfo, GmListCollection, GmListDecodeError, GmListLoadError,
    GmListSerializationBlock,
}; // список операторов и его коллекция.
pub use globesetup::{
    GLOBE_SETUP_BLOB_LENGTH, GlobePlayerPropertyCoefficients, GlobeRpGainPolicy,
    GlobeSetupDecodeError, GlobeSetupLoadError, GlobeSetupLoadReport, GlobeSetupSnapshot,
    GlobeStiffenSetup,
}; // глобальный setup: коэффициенты, RP-политика и snapshot.
pub use godsbattleconf::{
    CGodsBattleConf, GodsBattleBaseMoney, GodsBattleCollection, GodsBattleDecodeError,
    GodsBattleDecodeSection, GodsBattleDieBackPosition, GodsBattleFactionNpcName,
    GodsBattleFactionRule, GodsBattleFactionXydUpdate, GodsBattleLoadError,
    GodsBattleLoadSection, GodsBattleNpcFactionUpdate, GodsBattleNpcStringField,
    GodsBattleReviseMoney, GodsBattleSerializeError, GodsBattleStringField,
    GodsBattleSzlCalculation, GodsBattleSzlLevel,
}; // конфигурация Gods Battle: правила фракций и деньги.
pub use goodsdestructionconfig::{
    GoodsDestroyDecodeError, GoodsDestroyFileLoadError, GoodsDestroyFormatError,
    GoodsDestroyList, GoodsDestroyLoadReport, GoodsDestroySerializeError, GoodsDestroySetup,
}; // список уничтожаемых предметов и его кодек-ошибки.
pub use marker::read_to_marker; // чтение потока токенов до маркера.
pub use package::{PackageArchive, PackageFileIndex, PackageReadError}; // индекс и чтение .pak-архива.
pub use path::{normalize_resource_path, resolve_resource_path}; // нормализация и разрешение путей ресурсов.
pub use preciousboxconf::{
    PreciousBox, PreciousBoxConf, PreciousBoxCount, PreciousBoxDecodeError,
    PreciousBoxDecodeField, PreciousBoxItem, PreciousBoxLoadDiagnostic, PreciousBoxLoadError,
    PreciousBoxLoadReport, PreciousBoxOdds, PreciousBoxSerializeError,
}; // Precious Box: предметы, шансы и диагностика загрузки.
pub use prisonconf::{
    PrisonConf, PrisonConfDecodeError, PrisonConfFileLoadError, PrisonConfFormatError,
    PrisonConfSerializeError, PrisonParam,
}; // тюремная конфигурация и её параметры.
pub use playerlist::{
    CPlayerList, PlayerBaseProperties, PlayerBasePropertiesMap, PlayerCreationPropertiesLookup,
    PlayerListDecodeError, PlayerListFormatError, PlayerListLoadReport, PlayerListSerializeError,
    PlayerOriginEquipment, PlayerPropertiesUpgrade, PlayerPropertiesUpgradeLoadReport,
    PlayerPropertiesUpgradeMap,
}; // определения персонажа и upgrade-таблица свойств.
pub use regionrouter::{
    RegionNextNode, RegionRoutePoint, RegionRouteStep, RegionRouter,
    RegionRouterChangeOutcome, RegionRouterDecodeError, RegionRouterDecodeField,
    RegionRouterLoadError, RegionRouterLoadReport, RegionRouterNode,
    RegionRouterSerializeError,
}; // маршрутизация регионов: узлы, шаги и исход смены.
pub use quest::{CQuestSystem, QuestEntry}; // каталог определений заданий и его запись.
pub use quest_text::{
    QuestSystemLoadCompletion, QuestSystemLoadReport, QuestTextError, QuestTextErrorKind,
}; // отчёт загрузки текстовых определений заданий.
pub use regionsetup::{
    CRegionSetup, RegionSetupDecodeError, RegionSetupEntry, RegionSetupLoadError,
    RegionSetupSerializeError,
}; // ограничения регионов и их записи.
pub use quest_wire::{
    QuestStringField, QuestSystemDecodeError, QuestSystemDecodeOutcome, QuestSystemSerializationBlock,
    QuestWireField,
}; // wire-кодек каталога заданий.
pub use rfile::CRFile; // курсор памяти/файла CRFile.
pub use source::{ResourceOpenError, ResourceSource, open_resource}; // выбор источника ресурса и ошибка его открытия.
pub use stringtable::{StringTable, StringTableParseError, StringTableParseErrorKind}; // таблица текстов и ошибки её разбора.
pub use stringtable_wire::{MyStringTable, MyStringTableDecodeError, MyStringTableDecodeOutcome}; // wire-кодек таблицы текстов.
pub use synthesis::{
    CSynthesis, SynthesisCount, SynthesisDecodeError, SynthesisDecodeField,
    SynthesisFormula, SynthesisLoadError, SynthesisLoadReport, SynthesisRecipe,
    SynthesisSerializeError, SynthesisString,
}; // синтез: рецепты, формулы и кодек-ошибки.
pub use taozhuangsetup::{
    CTaoZhuangSetup, TaoZhuangAddItem, TaoZhuangCountSection, TaoZhuangDecodeError,
    TaoZhuangItem, TaoZhuangSerializationBlock, TaoZhuangStringField,
}; // комплекты TaoZhuang и их предметы.
pub use tradelist::{
    CTradeList, Trade, TradeGoods, TradeListDecodeError, TradeListFileLoadError,
    TradeListFormatError, TradeListSerializeError,
}; // торговые списки и их кодек-ошибки.
pub use wordsfilter::{
    CWordsFilter, WordsFilterDecodeError, WordsFilterDecodeSection,
    WordsFilterSerializeError, WordsFilterSerializeSection,
}; // фильтр запрещённых слов и его кодек-ошибки.
