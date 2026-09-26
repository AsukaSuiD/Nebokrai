//! Мировые контентные каталоги-инстансы Realm (прежние transitional-поля
//! `CGame`); источник контракта — точная пара `Nworldserver.exe` +
//! `WorldServer.pdb`.
//!
//! Первичное состояние принадлежит `content`: [`WorldContentCatalogs`]
//! владеет инстансами и их жизненным циклом, форматы таблиц остаются в
//! Shared, а `CGame` хранит только composition handle и делегирует свой
//! прежний pub facade. Init/reload/Release-оркестрация (`app/world_game_init`,
//! `app/world_reload`) заполняет инстансы в доказанном исходном порядке и с
//! исходными частичными эффектами, поэтому поля публичны (прецедент —
//! `persistence::savedata::WorldDbData`).
//!
//! Механика таблиц различна и сохранена у orchestration-слоя: `string_table`,
//! `emotion`, `trade_list`, `hit_level_setup`, `words_filter`,
//! `quest_system`, `increment_shop_list`, `contribute_setup`, `prison_conf`,
//! `equipment_compose_list`, `tao_zhuang_setup`, `ci_qing_setup`,
//! `thing_setup` и `script_resources` мутируются in place их же loader-ами;
//! `dupli_region_setup` публикуется в `Init` и снимается в `Release`
//! (`Option`), reload-пути не имеет.
//!
//! Доказательства: docs/reconstruction/realm-services.md#world-контент-и-reload

use nebokrai_shared::resources::{
    CCiQingSetup, CContributeSetup, CDupliRegionSetup, CEmotion, CHitLevelSetup,
    CIncrementShopList, CTaoZhuangSetup, CThingSetup, CTradeList, CWordsFilter,
    EquipmentComposeList, MyStringTable, PrisonConf,
};

use crate::content::quests::QuestCatalog;
use crate::content::scripts::ScriptResources;

/// Действующие контентные каталоги одного мира.
///
/// Constructor-ное состояние — пустые таблицы и непубликованный
/// `dupli_region_setup`; обязательное наполнение выполняет `CGame::Init` и
/// reload-профили, а не этот конструктор.
pub struct WorldContentCatalogs {
    pub thing_setup: CThingSetup,
    pub emotion: CEmotion,
    pub string_table: MyStringTable,
    pub string_table_array: Vec<u8>,
    pub words_filter: CWordsFilter,
    pub dupli_region_setup: Option<CDupliRegionSetup>,
    pub equipment_compose_list: EquipmentComposeList,
    pub ci_qing_setup: CCiQingSetup,
    pub tao_zhuang_setup: CTaoZhuangSetup,
    pub hit_level_setup: CHitLevelSetup,
    pub trade_list: CTradeList,
    pub increment_shop_list: CIncrementShopList,
    pub prison_conf: PrisonConf,
    pub contribute_setup: CContributeSetup,
    pub quest_system: QuestCatalog,
    pub script_resources: ScriptResources,
}

impl WorldContentCatalogs {
    pub fn new() -> Self {
        Self {
            thing_setup: CThingSetup::new(),
            emotion: CEmotion::default(),
            string_table: MyStringTable::new(),
            string_table_array: Vec::new(),
            words_filter: CWordsFilter::new(),
            dupli_region_setup: None,
            equipment_compose_list: EquipmentComposeList::default(),
            ci_qing_setup: CCiQingSetup::default(),
            tao_zhuang_setup: CTaoZhuangSetup::default(),
            hit_level_setup: CHitLevelSetup::default(),
            trade_list: CTradeList::default(),
            increment_shop_list: CIncrementShopList::default(),
            prison_conf: PrisonConf::default(),
            contribute_setup: CContributeSetup::default(),
            quest_system: QuestCatalog::default(),
            script_resources: ScriptResources::default(),
        }
    }
}
