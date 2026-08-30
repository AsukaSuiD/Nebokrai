//! Concrete GameServer-владелец деревенского war-region `CServerVillageRegion`.
//!
//! Исходный owner PDB —
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\servervillageregion.cpp`,
//! точная пара GameServer. PDB подтверждает наследование `CServerWarRegion`,
//! ordered goods-list `+0x270` и `m_lFlagOwnerFacID +0x27C`.
//! Фазовые callbacks RVA `0x001D1310`, `0x001D13C0`,
//! `0x001D1590..0x001D16D0`, victory `0x001D12C0` и clear `0x001D1370`
//! имеют статус `IMPLEMENTED, VERIFIED_DISASSEMBLY`; membership `0x001D12F0`,
//! direct timeout-forwarding `0x001D14F0` и `AddNeedGood` `0x001D1900` —
//! `IMPLEMENTED`, decoder-forwarding `0x001D1280` и ownership query
//! `0x001D1980` — `IMPLEMENTED, VERIFIED_DISASSEMBLY`.
//!
//! Timeout намеренно игнорирует аргумент и шлёт текущие `(war, region,
//! flag-owner, 0)` как `0x60136`; snapshot исполняет канонический `CGame`
//! перед `GS0240` log. `OnFactionVictory` вызывает тот же доказанный no-op
//! `0x004A8750`, поэтому отдельного callback-а у него нет. End сначала
//! сбрасывает war-state и очищает contenders, после чего `CGame` удаляет
//! предметы в порядке `goods × players`; только затем owner ставит
//! 60-секундное вытеснение и обнуляет country/ownership/flag-owner.
//! Inherited `CServerWarRegion::AI` вызывается реальным `CGame::AI` через
//! Village adapter с canonical player/state/network effects.
//!
//! Конструктор `0x001D1850` создаёт base-region, пустой ordered goods-list и
//! нулевой flag owner; это точно выражено `Default` и `Vec::default`.
//! Деструктор `0x001D18C0` только уничтожает элементы списка и base-object;
//! автоматические `Drop` для `Vec` и полей сохраняют контракт без ручного кода.
//! Неизвестного поведения владельца не осталось; STL/SEH заменены безопасными
//! стандартными контейнерами Rust.

use super::organizingsystem::villagewarsys::CVillageWarSys;
use super::serverregion::ServerRegionDecodeError;
use super::skills::skillfactory::CSkillFactory;
use crate::setup::monsterlist::MonsterRegistry;
use super::serverwarregion::{
    CServerWarRegion, ContendState, WarContendContext, WarRegionContext, WarRegionDecodeContext,
    WarRegionDecodeError,
};

pub(crate) trait VillageOwnerContext {
    type Region: Copy;
    /// Ищет сначала `s_mapRegion`, а при miss либо null — `FindProxyRegion`.
    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region>;
    fn owned_city_faction(&mut self, region: Self::Region) -> i32;
}

pub(crate) trait VillageRegionContext: WarRegionContext {
    /// Пишет localized template в канал `war` с аргументом region name.
    fn write_war_log(&mut self, string_id: &'static str, region_name: &str);
    fn now_millis(&mut self) -> u32;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VillageTimeoutEffect {
    pub(crate) war_number: i32,
    pub(crate) region_id: i32,
    pub(crate) flag_owner_faction_id: i32,
    pub(crate) region_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VillageWarEndTargets {
    pub(crate) region_id: i32,
    pub(crate) player_ids: Vec<i32>,
    pub(crate) goods: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CServerVillageRegion {
    pub(crate) war: CServerWarRegion,
    pub(crate) goods: Vec<String>,
    pub(crate) flag_owner_faction_id: i32,
}

impl CServerVillageRegion {
    pub(crate) fn decord_from_byte_array<Context: WarRegionDecodeContext>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        area_width: i32,
        area_height: i32,
        monster_registry: &MonsterRegistry,
        skill_factory: &CSkillFactory,
        context: &mut Context,
    ) -> Result<bool, WarRegionDecodeError<ServerRegionDecodeError>> {
        let _ = self
            .war
            .decord_from_byte_array(
                source,
                cursor,
                include_child,
                area_width,
                area_height,
                monster_registry,
                skill_factory,
                context,
            )?;
        Ok(true)
    }

    pub(crate) fn on_contend_time_over<Context: WarContendContext>(
        &mut self,
        contender: &ContendState,
        context: &mut Context,
    ) -> Result<(), Context::MembershipError> {
        self.war.on_contend_time_over(contender, context)
    }

    pub(crate) fn add_need_good(&mut self, good_name: &str) {
        if !good_name.is_empty() {
            self.goods.push(good_name.to_owned());
        }
    }

    pub(crate) fn is_owner<Context: VillageOwnerContext>(
        &self,
        schedules: &CVillageWarSys,
        faction_id: i32,
        context: &mut Context,
    ) -> bool {
        let village_region_id = schedules.get_village_region_id_by_time(self.war.base.war_number);
        let Some(village_region) = context.find_region_then_proxy(village_region_id) else {
            return false;
        };
        faction_id != 0 && faction_id == context.owned_city_faction(village_region)
    }

    pub(crate) fn is_apply_war_faction(&self, schedules: &CVillageWarSys, faction_id: i32) -> bool {
        schedules.is_already_declar_for_war(self.war.base.war_number, faction_id)
    }

    pub(crate) fn on_faction_win_one_symbol(&mut self, faction_id: i32, symbol_id: i32) {
        if symbol_id == 0 {
            self.flag_owner_faction_id = faction_id;
        }
    }

    pub(crate) fn on_war_declare<Context: VillageRegionContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) {
        self.war.on_war_declare(war_number);
        self.flag_owner_faction_id = self.war.base.param.owned_faction_id;
        context.write_war_log("GS0238", &self.war.base.name);
    }

    pub(crate) fn on_war_start<Context: VillageRegionContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) {
        if self.war.base.war_number != war_number {
            return;
        }
        self.war.base.on_war_start(war_number);
        self.war.base.country = 0;
        context.write_war_log("GS0239", &self.war.base.name);
    }

    pub(crate) fn on_war_time_out(&self, _war_number: i32) -> VillageTimeoutEffect {
        VillageTimeoutEffect {
            war_number: self.war.base.war_number,
            region_id: self.war.base.id,
            flag_owner_faction_id: self.flag_owner_faction_id,
            region_name: self.war.base.name.clone(),
        }
    }

    pub(crate) fn begin_war_end<Context: WarRegionContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> Option<VillageWarEndTargets> {
        if self.war.base.war_number != war_number || self.war.base.city_state == 0 {
            return None;
        }
        self.war.on_war_end(war_number);
        self.war.clear_region(context);
        Some(VillageWarEndTargets {
            region_id: self.war.base.id,
            player_ids: self.war.base.registered_player_ids(),
            goods: self.goods.clone(),
        })
    }

    pub(crate) fn finish_war_end(&mut self, now_ms: u32) {
        self.war.base.start_clear_player_out_at(60_000, now_ms);
        self.war.base.country = 0;
        self.war.base.set_owned_city_org(0, 0);
        self.flag_owner_faction_id = 0;
    }
}
