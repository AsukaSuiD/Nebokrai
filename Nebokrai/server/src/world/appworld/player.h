#pragma once

#include "moveshape.h"
#include "container/camountlimitgoodscontainer.h"
#include "container/cbank.h"
#include "container/cbattlefairycontainer.h"
#include "container/cdepot.h"
#include "container/cequipmentcontainer.h"
#include "container/cfairycontainer.h"
#include "container/cjifen.h"
#include "container/cvolumelimitgoodscontainer.h"
#include "container/cwallet.h"
#include "container/cyuanbao.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <deque>
#include <map>
#include <optional>
#include <set>
#include <span>
#include <string>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/player.cpp / player.h.
 * Каркас, object type 400, размеры tagBaseProperty/tagProperty, порядок
 * пятнадцати container-owner-ов и clone-wire подтверждены Nworldserver.exe,
 * PDB и поздней Rust-реконструкцией. Старый огромный ABI-класс намеренно не
 * копируется: здесь только достигнутое игровое состояние и безопасный codec.
 * Неинициализированные в оригинальном constructor country/contribute остаются
 * optional до decode. Владение товарами делегировано уже восстановленным
 * контейнерам; сетевой buffer заменён span/vector без изменения wire-порядка.
 */
class CPlayer final : public CMoveShape
{
public:
    static constexpr std::size_t BasePropertyWireSize = 0x194;
    static constexpr std::size_t PropertyWireSize = 0x9C;

    struct PropertyCoefficients {
        std::array<float, 3> strengthToMaximumAttack{};
        std::array<float, 3> strengthToBurden{};
        std::array<float, 3> dexterityToMinimumAttack{};
        std::array<float, 3> dexterityToStiffness{};
        std::array<float, 3> constitutionToMaximumHp{};
        std::array<float, 3> constitutionToDefense{};
        std::array<float, 3> intelligenceToElement{};
        std::array<float, 3> intelligenceToMaximumMp{};
        std::array<float, 3> intelligenceToResistance{};
    };

    struct Skill { std::uint16_t id{}; std::uint16_t level{}; };
    struct Friend { std::string name; bool online{}; };
    struct Thing {
        std::uint16_t id{};
        std::uint16_t count{};
        std::uint16_t maximumCount{};
        std::uint16_t point{};
    };
    struct Quest { std::uint16_t id{}; std::uint8_t complete{}; };
    struct Pet {
        std::string originalName;
        std::uint32_t hp{};
        std::uint32_t level{};
        std::uint32_t experience{};
    };
    struct Carriage {
        std::string originalName;
        std::string script;
        std::uint32_t hp{};
    };
    struct OrganizingState {
        std::int32_t factionId{};
        std::int32_t factionLogoId{};
        std::uint16_t factionLevel{};
        std::int32_t factionExperience{};
        std::int32_t force{};
        bool factionContribute{};
        std::string factionName;
        std::string factionTitle;
        std::int32_t factionMasterId{};
        std::int32_t unionId{};
        std::int32_t unionMasterId{};
        std::set<std::int32_t> enemyFactions;
        std::set<std::int32_t> cityWarEnemyFactions;
        std::deque<std::array<std::uint8_t, 8>> ownedRegions;
    };

    explicit CPlayer(const CGoodsFactory& goodsFactory);

    bool AddToByteArray(std::vector<std::uint8_t>& output,
                        bool includeChild) const override;
    bool DecordFromByteArray(std::span<const std::uint8_t> input,
                             std::size_t& offset,
                             bool includeChild) override;
    bool UpdateProperty(const PropertyCoefficients& coefficients) noexcept;

    [[nodiscard]] std::uint8_t GetLevel() const noexcept;
    [[nodiscard]] std::string_view GetAccount() const noexcept { return m_Account; }
    [[nodiscard]] std::string_view GetTitle() const noexcept { return m_Title; }
    [[nodiscard]] std::optional<std::uint8_t> GetCountry() const noexcept { return m_Country; }
    [[nodiscard]] std::optional<std::int32_t> GetContribute() const noexcept { return m_Contribute; }
    [[nodiscard]] std::int32_t GetTeamID() const noexcept { return m_TeamId; }
    [[nodiscard]] std::int32_t GetFactionID() const noexcept { return m_Organizing.factionId; }
    [[nodiscard]] bool HasFactionData() const noexcept { return m_FactionDataReceived; }
    void SetFactionDataReceived(bool value) noexcept { m_FactionDataReceived = value; }
    void SetCountry(std::uint8_t value) noexcept { m_Country = value; }
    void SetContribute(std::int32_t value) noexcept { m_Contribute = value; }

    [[nodiscard]] std::span<std::uint8_t, BasePropertyWireSize> BaseProperty() noexcept
    { return m_BaseProperty; }
    [[nodiscard]] std::span<const std::uint8_t, BasePropertyWireSize> BaseProperty() const noexcept
    { return m_BaseProperty; }
    [[nodiscard]] std::span<const std::uint8_t, PropertyWireSize> Property() const noexcept
    { return m_Property; }
    [[nodiscard]] OrganizingState& Organizing() noexcept { return m_Organizing; }
    [[nodiscard]] const OrganizingState& Organizing() const noexcept { return m_Organizing; }

    [[nodiscard]] CAmountLimitGoodsContainer& Hand() noexcept { return m_Hand; }
    [[nodiscard]] CEquipmentContainer& Equipment() noexcept { return m_Equipment; }
    [[nodiscard]] CVolumeLimitGoodsContainer& Packet() noexcept { return m_Packet; }
    [[nodiscard]] CWallet& Wallet() noexcept { return m_Wallet; }
    [[nodiscard]] CYuanBao& YuanBao() noexcept { return m_YuanBao; }
    [[nodiscard]] CJiFen& JiFen() noexcept { return m_JiFen; }
    [[nodiscard]] CBank& Bank() noexcept { return m_Bank; }
    [[nodiscard]] CDepot& Depot() noexcept { return m_Depot; }

private:
    bool AddBeforeContainers(std::vector<std::uint8_t>& output) const;
    bool DecodeBeforeContainers(std::span<const std::uint8_t> input, std::size_t& offset);
    bool AddContainers(std::vector<std::uint8_t>& output) const;
    bool DecodeContainers(std::span<const std::uint8_t> input, std::size_t& offset);
    bool AddAfterContainers(std::vector<std::uint8_t>& output) const;
    bool DecodeAfterContainers(std::span<const std::uint8_t> input, std::size_t& offset);
    bool AddOrganizing(std::vector<std::uint8_t>& output) const;

    const CGoodsFactory& m_GoodsFactory;
    CAmountLimitGoodsContainer m_Hand;
    CVolumeLimitGoodsContainer m_Packet;
    CEquipmentContainer m_Equipment;
    CWallet m_Wallet;
    CYuanBao m_YuanBao;
    CJiFen m_JiFen;
    CBank m_Bank;
    CDepot m_Depot;
    CFairyContainer m_Fairy;
    CBattleFairyContainer m_BattleFairy;
    CVolumeLimitGoodsContainer m_AuctionGoods;
    CVolumeLimitGoodsContainer m_Auction;
    CWallet m_AuctionWallet;
    CVolumeLimitGoodsContainer m_CiQing;
    CVolumeLimitGoodsContainer m_ComposeCiQing;

    std::array<std::uint8_t, BasePropertyWireSize> m_BaseProperty{};
    std::string m_Account;
    std::string m_Title;
    std::array<std::uint8_t, PropertyWireSize> m_Property{};
    std::int32_t m_TeamId{};
    std::set<std::uint32_t> m_CiQingIds;
    std::deque<Skill> m_NewSkills;
    std::deque<Friend> m_Friends;
    std::deque<Thing> m_DailyThings;
    std::map<std::uint16_t, Quest> m_Quests;
    std::string m_DepotPassword;
    std::int32_t m_VariableCount{};
    std::vector<std::uint8_t> m_VariableData;
    std::int32_t m_SilenceTime{};
    std::uint32_t m_MurdererTime{};
    std::int32_t m_FightStateCount{};
    std::vector<Pet> m_UncreatedPets;
    Carriage m_UncreatedCarriage;
    bool m_RecreateCarriage{};
    bool m_Login{};
    std::int32_t m_CityWarDiedStateTime{};
    std::optional<std::uint8_t> m_Country;
    std::optional<std::int32_t> m_Contribute;
    std::array<std::uint8_t, 0x10> m_JjcData{};
    bool m_JjcPkState{};
    std::string m_SessionId;
    OrganizingState m_Organizing;
    bool m_FactionDataReceived{};
};
