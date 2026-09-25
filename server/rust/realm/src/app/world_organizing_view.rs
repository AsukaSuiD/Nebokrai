//! Узкие organizing-view для обработчиков мировых сообщений Realm.
//!
//! Два уровня обязанностей живут здесь вместе: минимальный dyn
//! [`WorldOrganizingView`] для ветвей прочих диспетчеров и полный статический
//! [`WorldOrganizingDispatchView`] для диспетчера `organsysmessage` — он
//! не dyn-совместим из-за generic context/effects подписей контроллера и
//! потребляется handler-ами только через `impl WorldOrganizingDispatchView`.

use crate::activities::factionwarsys::CFactionWarSys;
use crate::app::world_game_view::WorldGameView;
use crate::content::organizing::{EOperator, TagTimeValue};
use crate::organizations::faction::{
    CFaction, FactionApplyForJoinEffects, FactionApplyForJoinOutcome, FactionContributorContext,
    FactionDisbandContext, FactionDoJoinEffects, FactionExperienceBlock,
    FactionInitialPropertyBlock, FactionOperationBlock, FactionOperationOutcome,
    FactionOrganizingInfoContext, FactionPermitBlock, FactionPermitUpdate,
    FactionSetParameterBlock, FactionSetParameterContext, FactionSetParameterOutcome,
    FactionTalkDelivery, FactionUploadIconBlock, FactionUploadIconContext,
    FactionUploadIconOutcome, OwnedCityAddOutcome, OwnedCityMutationBuildError,
};
use crate::organizations::organizingctrl::{
    ApplyFactionLookup, AttackCityEndBlock, AttackCityEndEffects, AttackCityEndReport,
    DeclareWarFactionPage, DeclareWarFactionPageBlock, FactionCountryCountBlock,
    FactionListPage, FactionListPageBlock, FactionMasterLookupBlock,
    FactionUnionMembershipLookupBlock, FreePlayerLookup, OrganizingConfederationDisbandBlock,
    OrganizingConfederationDisbandOutcome, OrganizingContributorBlock,
    OrganizingContributorOutcome, OrganizingDisbandBlock, OrganizingDisbandOutcome,
    OrganizingFactionApplicationBlock, OrganizingFactionDoJoinBlock,
    OrganizingFactionDoJoinOutcome, OrganizingFactionExperienceMutation,
    OrganizingFactionMemberStateOutcome, OrganizingLeaveWordBlock,
    OrganizingLeaveWordEditBlock, OrganizingLeaveWordEditOutcome,
    OrganizingLeaveWordEnableBlock, OrganizingLeaveWordEnableOutcome,
    OrganizingLeaveWordOutcome, OrganizingNameCountryBlock, OrganizingNameLookupBlock,
    OrganizingNameMatch, OrganizingNamedUnionApplicationBlock, OrganizingPronounceBlock,
    OrganizingPronounceOutcome, OrganizingUnionByMasterBlock, OrganizingUnionDemiseBlock,
    OrganizingUnionDemiseOutcome, OrganizingUnionExitBlock, OrganizingUnionExitOutcome,
    OrganizingUnionFireOutBlock, OrganizingUnionFireOutOutcome,
    RemovePersonFromApplyFactionListOutcome,
};
use crate::organizations::organizingparam::COrganizingParam;
use crate::organizations::union::{
    UnionApplyForJoinEffects, UnionApplyForJoinOutcome, UnionFireOutEffects,
};

/// Точки обратного вызова handler-ветвей мира в organizing-владельца.
/// Реализация живёт на realm `COrganizingCtrl` (`organizations/organizingctrl`)
/// и делегирует его inherent-методам и `CFaction::talk`; отдельный трейт,
/// а не `WorldGameView`, потому что organizing передаётся диспетчерам
/// отдельным owner-параметром.
pub trait WorldOrganizingView {
    fn is_free_player(&self, player_id: i32) -> FreePlayerLookup;

    /// Разговор фракции `0x7FA02`: `None` — фракция не найдена. Цикл по
    /// членам и wire-формат остаются у владельца (`CFaction::talk`).
    fn faction_talk(
        &self,
        game: &dyn WorldGameView,
        faction_id: i32,
        speaker_id: i32,
        first_text: &[u8],
        second_text: &[u8],
    ) -> Option<Vec<FactionTalkDelivery>>;
}

/// Точки обратного вызова `CWorldRegion::init_owner_relation` в organizing
/// и игру. Inherent `add_owned_city_to_faction` требует `&CGame` (обновление
/// owned-city wire и faction info), поэтому реализация живёт в адаптере-
/// мосте у единственного call-site инициализации, который связывает
/// `&mut COrganizingCtrl` и `&CGame`; отдельный трейт вместо методов
/// `WorldOrganizingView`, чтобы не перетаскивать `&CGame` в чужой контракт.
pub trait WorldRegionOwnerOrganizingView {
    fn has_faction(&self, faction_id: i32) -> bool;

    fn has_confederation(&self, union_id: i32) -> bool;

    fn add_owned_city_to_faction(
        &mut self,
        faction_id: i32,
        region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<Option<OwnedCityAddOutcome>, OwnedCityMutationBuildError>;

    fn country_by_faction(
        &self,
        faction_id: i32,
    ) -> Result<Option<u8>, FactionInitialPropertyBlock>;
}

/// Полный статический organizing-view диспетчера `organsysmessage`. Реализация
/// живёт у старого `COrganizingCtrl` и делегирует его одноимённым inherent-
/// методам; имена намеренно совпадают — inherent priority исключает рекурсию.
///
/// Трейт НЕ потребляется через `dyn`: часть inherent-методов контроллера
/// generic по context/effects (Realm-трейты `Faction*Context`,
/// `UnionApplyForJoinEffects`, `UnionFireOutEffects`, `AttackCityEndEffects`),
/// поэтому зеркальные методы трейта generic, а handler-ы принимают owner
/// как `impl WorldOrganizingDispatchView`. Game-доступ передаётся только
/// `&dyn WorldGameView`/`&mut dyn WorldGameView` — ни одна ветвь диспетчера
/// не требует конкретного `CGame`. `&self`-методы — чистые lookup-ы данных
/// контроллера.
pub trait WorldOrganizingDispatchView {
    fn is_free_player(&self, player_id: i32) -> FreePlayerLookup;

    /// `None` покрывает отсутствующий ключ и сохранённый null pointer —
    /// pointer-семантика остаётся у вызывающей ветви.
    fn faction_by_id(&self, faction_id: i32) -> Option<&CFaction>;

    fn faction_by_id_mut(&mut self, faction_id: i32) -> Option<&mut CFaction>;

    fn faction_id_by_master_player(
        &self,
        player_id: i32,
    ) -> Result<i32, FactionMasterLookupBlock>;

    fn union_id_by_master_player(
        &self,
        player_id: i32,
    ) -> Result<Option<i32>, OrganizingUnionByMasterBlock>;

    fn faction_by_player_in_apply_list(&self, player_id: i32) -> ApplyFactionLookup;

    fn faction_count_by_country(&self, country: u8) -> Result<i32, FactionCountryCountBlock>;

    fn faction_list_page(
        &self,
        requested_page: i32,
        country: u8,
    ) -> Result<FactionListPage, FactionListPageBlock>;

    fn declare_war_faction_count(&self) -> Result<i32, DeclareWarFactionPageBlock>;

    fn declare_war_faction_page(
        &self,
        source_faction_id: i32,
        requested_page: i32,
        faction_wars: &CFactionWarSys,
    ) -> Result<DeclareWarFactionPage, DeclareWarFactionPageBlock>;

    fn organizing_by_name(
        &self,
        requested_name: &[u8],
    ) -> Result<Option<OrganizingNameMatch>, OrganizingNameLookupBlock>;

    fn country_by_name_match(
        &self,
        matched: OrganizingNameMatch,
    ) -> Result<u8, OrganizingNameCountryBlock>;

    fn operate_faction_tax(
        &self,
        faction_id: i32,
        player_id: i32,
        region_id: i32,
    ) -> Result<
        Option<FactionOperationOutcome>,
        FactionOperationBlock<FactionUnionMembershipLookupBlock>,
    >;

    fn operate_faction_city_gate(
        &self,
        faction_id: i32,
        player_id: i32,
        region_id: i32,
    ) -> Result<
        Option<FactionOperationOutcome>,
        FactionOperationBlock<FactionUnionMembershipLookupBlock>,
    >;

    /// `None` — фракция не найдена; счётчик goods war ветви `0x60139`.
    fn set_faction_goods_war_count(&mut self, faction_id: i32, count: i32) -> Option<i32>;

    fn enable_leave_word_for_master<Context>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> Result<OrganizingLeaveWordEnableOutcome, OrganizingLeaveWordEnableBlock>
    where
        Context: FactionOrganizingInfoContext;

    /// Единственный `&mut` game-путь ветвей: выделение leave-word ID внутри
    /// `CFaction::leave_word` (`WorldGameView::allocate_leave_word_id`).
    fn leave_word_for_player(
        &mut self,
        game: &mut dyn WorldGameView,
        player_id: i32,
        content: &mut Vec<u8>,
        time: TagTimeValue,
    ) -> Result<OrganizingLeaveWordOutcome, OrganizingLeaveWordBlock>;

    fn edit_leave_word_for_player(
        &mut self,
        game: &dyn WorldGameView,
        player_id: i32,
        leave_word_id: i32,
        operator: EOperator,
    ) -> Result<OrganizingLeaveWordEditOutcome, OrganizingLeaveWordEditBlock>;

    fn pronounce_for_player(
        &mut self,
        game: &dyn WorldGameView,
        player_id: i32,
        content: &mut Vec<u8>,
        time: TagTimeValue,
    ) -> Result<OrganizingPronounceOutcome, OrganizingPronounceBlock>;

    fn remove_person_from_apply_faction_list(
        &mut self,
        game: &dyn WorldGameView,
        player_id: i32,
    ) -> RemovePersonFromApplyFactionListOutcome;

    /// Ветвь `0x60108` faction-пути: detached owner поверх controller-map;
    /// `second_parameter`/`third_parameter` — исходные нули update-формы.
    #[allow(clippy::too_many_arguments, reason = "точная форма inherent-делегации контроллера")]
    fn apply_for_faction_join_by_map_key<Effects>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        map_key: i32,
        player_id: i32,
        second_parameter: i32,
        third_parameter: i32,
        effects: &mut Effects,
    ) -> Result<FactionApplyForJoinOutcome, OrganizingFactionApplicationBlock>
    where
        Effects: FactionApplyForJoinEffects;

    /// Ветвь `0x60108` union-пути по уже найденному map-key без повторного
    /// master-player lookup ветки `0x60118`.
    fn apply_for_named_union_join<Effects>(
        &mut self,
        game: &dyn WorldGameView,
        map_key: i32,
        applicant_faction_id: i32,
        second_parameter: i32,
        third_parameter: i32,
        effects: &mut Effects,
    ) -> Result<
        UnionApplyForJoinOutcome<Effects::SessionReport>,
        OrganizingNamedUnionApplicationBlock<Effects::SessionBlock>,
    >
    where
        Effects: UnionApplyForJoinEffects;

    /// Ветвь `0x6010A`: два lookup-а faction и один local-time snapshot на
    /// исходной позиции, virtual `CFaction::DoJoin` через detached owner.
    fn do_faction_join_by_manager<Effects, GetLocalTime>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        manager_id: i32,
        applicant_id: i32,
        approve_flag: i32,
        get_local_time: GetLocalTime,
        effects: &mut Effects,
    ) -> Result<OrganizingFactionDoJoinOutcome, OrganizingFactionDoJoinBlock>
    where
        Effects: FactionDoJoinEffects,
        GetLocalTime: FnOnce() -> TagTimeValue;

    fn fire_out_union_by_master<Effects>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        manager_id: i32,
        target_faction_id: i32,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<OrganizingUnionFireOutOutcome, OrganizingUnionFireOutBlock>
    where
        Effects: UnionFireOutEffects;

    fn exit_union_by_player<Effects>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        player_id: i32,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<OrganizingUnionExitOutcome, OrganizingUnionExitBlock>
    where
        Effects: UnionFireOutEffects;

    fn demise_union_by_master<Effects>(
        &mut self,
        game: &dyn WorldGameView,
        old_master_player_id: i32,
        new_master_faction_id: i32,
        effects: &mut Effects,
        get_tick: &mut dyn FnMut() -> u32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<OrganizingUnionDemiseOutcome, OrganizingUnionDemiseBlock>
    where
        Effects: UnionFireOutEffects;

    fn disband_confederation<Effects>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        manager_id: i32,
        union_id: i32,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<OrganizingConfederationDisbandOutcome, OrganizingConfederationDisbandBlock>
    where
        Effects: UnionFireOutEffects;

    /// Synchronous prefix `0x60111` без player-flag/DB-log финализации —
    /// она остаётся за dispatch continuation старого пакета.
    fn disband_faction<Context>(
        &mut self,
        game: &dyn WorldGameView,
        player_id: i32,
        faction_id: i32,
        context: &mut Context,
    ) -> Result<OrganizingDisbandOutcome, OrganizingDisbandBlock>
    where
        Context: FactionDisbandContext;

    fn set_faction_parameter<Context>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        faction_id: i32,
        parameter: &[u8],
        value: i32,
        context: &mut Context,
    ) -> Result<Option<FactionSetParameterOutcome>, FactionSetParameterBlock>
    where
        Context: FactionSetParameterContext;

    /// Единственная операция ветвей без game-пути: иконка не публикует
    /// game-сообщений сверх контекста доставки.
    fn upload_faction_icon<Context>(
        &mut self,
        parameters: &COrganizingParam,
        faction_id: i32,
        player_id: i32,
        time: &TagTimeValue,
        context: &mut Context,
    ) -> Result<Option<FactionUploadIconOutcome>, FactionUploadIconBlock>
    where
        Context: FactionUploadIconContext;

    fn set_contributor_for_player<Context>(
        &mut self,
        game: &dyn WorldGameView,
        parameters: &COrganizingParam,
        requester_id: i32,
        target_id: i32,
        enabled: bool,
        context: &mut Context,
    ) -> Result<OrganizingContributorOutcome, OrganizingContributorBlock>
    where
        Context: FactionContributorContext;

    fn add_contributor_experience(
        &mut self,
        game: &dyn WorldGameView,
        faction_id: i32,
        player_id: i32,
        experience_delta: i32,
    ) -> Result<OrganizingFactionExperienceMutation, FactionExperienceBlock>;

    /// `read_value` — продолжение чтения operation-specific Long ветки
    /// `0x6012A` с той же wire-позиции.
    fn change_faction_member_state(
        &mut self,
        game: &dyn WorldGameView,
        faction_id: i32,
        player_id: i32,
        operation: i32,
        read_value: &mut dyn FnMut() -> i32,
    ) -> OrganizingFactionMemberStateOutcome;

    fn set_faction_admission_permit(
        &mut self,
        game: &dyn WorldGameView,
        faction_id: i32,
        player_id: i32,
        permit: bool,
    ) -> Result<Option<FactionPermitUpdate>, FactionPermitBlock>;

    #[allow(clippy::too_many_arguments, reason = "точная форма inherent-делегации контроллера ветки 0x60133")]
    fn on_attack_city_end<Effects>(
        &mut self,
        game: &dyn WorldGameView,
        result: i32,
        region_id: i32,
        attacker_player_id: i32,
        defender_faction_id: i32,
        effects: &mut Effects,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<AttackCityEndReport, AttackCityEndBlock>
    where
        Effects: AttackCityEndEffects;
}
