//! Process-owned ресурсы WorldServer, общие для init, reload и main loop.
//!
//! StringTable и встроенные setup-владельцы остаются у единственного `CGame`;
//! здесь собраны исторические process-global owners, передаваемые ему ссылками.

use std::path::{Path, PathBuf};

use crate::public::clientresource::DefaultClientResourceOwner;
use crate::public::dakongxiangqian::CDaKongXiangQian;
use crate::setup::cbattlefairyexpconfig::CBattleFairyExpConfig;
use crate::setup::changebody::CChangeBodyConf;
use crate::setup::fairyexpconf::CFairyExpConf;
use crate::setup::gmlist::CGMList;
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::setup::goodsdestructionconfig::GoodsDestroySetup;
use crate::setup::honorelimilateconfig::HonorElimilateConfig;
use crate::setup::lingbao::CLingBaoSetup;
use crate::setup::logsystem::CLogSystem;
use crate::setup::monsterlist::{MonsterDropRegistry, MonsterRegistry};
use crate::setup::newskillmonsterlist::NewSkillMonsterConf;
use crate::setup::playerlist::CPlayerList;
use crate::setup::preciousboxconf::PreciousBoxConf;
use crate::setup::regionrouter::RegionRouter;
use crate::setup::regionsetup::CRegionSetup;
use crate::setup::synthesis::CSynthesis;
use crate::worldserver::appworld::goods::cbattlefairyproperty::CBattleFairyProperty;
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, GoodsNameIndex, GoodsOriginalNameIndex,
};
use crate::worldserver::appworld::worldregion::WorldRegionResourceContext;

use super::game::WorldReloadContext;

pub(crate) struct WorldProcessResources {
    runtime_directory: PathBuf,
    default_client_resource: DefaultClientResourceOwner,
    goods: GoodsBasePropertiesRegistry,
    goods_by_original_name: GoodsOriginalNameIndex,
    goods_by_name: GoodsNameIndex,
    monsters: MonsterRegistry,
    monster_drops: MonsterDropRegistry,
    log_system: CLogSystem,
    region_setup: CRegionSetup,
    gm_list: CGMList,
    globe_setup: GlobeSetupSnapshot,
    region_router: RegionRouter,
    player_list: CPlayerList,
    goods_destroy: GoodsDestroySetup,
    new_skill_monsters: NewSkillMonsterConf,
    battle_fairy_exp: CBattleFairyExpConfig,
    battle_fairy_property: CBattleFairyProperty,
    synthesis: CSynthesis,
    honor_eliminate: HonorElimilateConfig,
    fairy_exp: CFairyExpConf,
    da_kong_xiang_qian: CDaKongXiangQian,
    change_body: CChangeBodyConf,
    precious_box: PreciousBoxConf,
    ling_bao: CLingBaoSetup,
    region_monsters: i32,
    region_npcs: i32,
    log_lines: Vec<Vec<u8>>,
    operator_notices: Vec<(Vec<u8>, Vec<u8>)>,
}

impl WorldProcessResources {
    pub(crate) fn new(runtime_directory: PathBuf) -> Self {
        Self {
            runtime_directory,
            default_client_resource: Default::default(),
            goods: Default::default(),
            goods_by_original_name: Default::default(),
            goods_by_name: Default::default(),
            monsters: Default::default(),
            monster_drops: Default::default(),
            log_system: Default::default(),
            region_setup: Default::default(),
            gm_list: Default::default(),
            globe_setup: Default::default(),
            region_router: Default::default(),
            player_list: Default::default(),
            goods_destroy: Default::default(),
            new_skill_monsters: Default::default(),
            battle_fairy_exp: Default::default(),
            battle_fairy_property: CBattleFairyProperty::with_constructor_defaults(),
            synthesis: Default::default(),
            honor_eliminate: Default::default(),
            fairy_exp: Default::default(),
            da_kong_xiang_qian: Default::default(),
            change_body: Default::default(),
            precious_box: Default::default(),
            ling_bao: Default::default(),
            region_monsters: 0,
            region_npcs: 0,
            log_lines: Vec::new(),
            operator_notices: Vec::new(),
        }
    }

    pub(crate) fn install_default_client_resource(&mut self) {
        let _ = self
            .default_client_resource
            .replace_from_world_directory(&self.runtime_directory);
    }

    pub(crate) fn drain_log_lines(&mut self) -> impl Iterator<Item = Vec<u8>> + '_ {
        self.log_lines.drain(..)
    }

    pub(crate) fn drain_operator_notices(
        &mut self,
    ) -> impl Iterator<Item = (Vec<u8>, Vec<u8>)> + '_ {
        self.operator_notices.drain(..)
    }
}

impl WorldRegionResourceContext for WorldProcessResources {
    fn default_client_resource(&mut self) -> &mut DefaultClientResourceOwner {
        &mut self.default_client_resource
    }

    fn region_monster_num_scale(&mut self) -> f32 {
        self.globe_setup.monster_number_scale()
    }
}

impl WorldReloadContext for WorldProcessResources {
    fn runtime_directory(&self) -> &Path {
        &self.runtime_directory
    }

    fn goods_registries(
        &mut self,
    ) -> (
        &mut GoodsBasePropertiesRegistry,
        &mut GoodsOriginalNameIndex,
        &mut GoodsNameIndex,
    ) {
        (
            &mut self.goods,
            &mut self.goods_by_original_name,
            &mut self.goods_by_name,
        )
    }

    fn monster_registries(&mut self) -> (&mut MonsterRegistry, &mut MonsterDropRegistry) {
        (&mut self.monsters, &mut self.monster_drops)
    }

    fn log_system(&mut self) -> &mut CLogSystem { &mut self.log_system }
    fn region_setup(&mut self) -> &mut CRegionSetup { &mut self.region_setup }
    fn gm_list(&mut self) -> &mut CGMList { &mut self.gm_list }
    fn globe_setup(&mut self) -> &mut GlobeSetupSnapshot { &mut self.globe_setup }
    fn region_router(&mut self) -> &mut RegionRouter { &mut self.region_router }

    fn globe_setup_and_router(&mut self) -> (&GlobeSetupSnapshot, &RegionRouter) {
        (&self.globe_setup, &self.region_router)
    }

    fn player_list(&mut self) -> &mut CPlayerList { &mut self.player_list }
    fn goods_destroy_setup(&mut self) -> &mut GoodsDestroySetup { &mut self.goods_destroy }
    fn new_skill_monster_conf(&mut self) -> &mut NewSkillMonsterConf { &mut self.new_skill_monsters }
    fn battle_fairy_exp_config(&mut self) -> &mut CBattleFairyExpConfig { &mut self.battle_fairy_exp }
    fn battle_fairy_property(&mut self) -> &mut CBattleFairyProperty { &mut self.battle_fairy_property }
    fn synthesis(&mut self) -> &mut CSynthesis { &mut self.synthesis }
    fn honor_eliminate_config(&mut self) -> &mut HonorElimilateConfig { &mut self.honor_eliminate }
    fn fairy_exp_conf(&mut self) -> &mut CFairyExpConf { &mut self.fairy_exp }
    fn da_kong_xiang_qian(&mut self) -> &mut CDaKongXiangQian { &mut self.da_kong_xiang_qian }
    fn change_body_conf(&mut self) -> &mut CChangeBodyConf { &mut self.change_body }
    fn precious_box_conf(&mut self) -> &mut PreciousBoxConf { &mut self.precious_box }
    fn ling_bao_setup(&mut self) -> &mut CLingBaoSetup { &mut self.ling_bao }

    fn query_goods_id_by_original_name(&mut self, original_name: &[u8]) -> u32 {
        self.goods_by_original_name.get(original_name).copied().unwrap_or(0)
    }

    fn query_goods_name(&mut self, goods_id: u32) -> Option<Vec<u8>> {
        self.goods
            .get(&goods_id)
            .and_then(Option::as_ref)
            .map(|properties| properties.get_name().to_vec())
    }

    fn four_nation_country_names(&mut self) -> [Vec<u8>; 5] {
        std::array::from_fn(|index| {
            self.globe_setup
                .country_name(index as u8)
                .unwrap_or_default()
                .to_vec()
        })
    }

    fn add_log_text(&mut self, payload: &[u8]) { self.log_lines.push(payload.to_vec()); }

    fn notify_reload_operator(&mut self, title: &[u8], message: &[u8]) {
        self.operator_notices.push((title.to_vec(), message.to_vec()));
    }

    fn add_region_object_counts(&mut self, monsters: i32, npcs: i32) -> (i32, i32) {
        self.region_monsters = self.region_monsters.wrapping_add(monsters);
        self.region_npcs = self.region_npcs.wrapping_add(npcs);
        (self.region_monsters, self.region_npcs)
    }

    fn region_object_counts(&mut self) -> (i32, i32) {
        (self.region_monsters, self.region_npcs)
    }
}
