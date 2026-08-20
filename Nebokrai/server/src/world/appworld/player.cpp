#include "player.h"

#include <algorithm>
#include <cmath>
#include <cstring>
#include <limits>
#include <type_traits>

namespace
{
constexpr std::size_t LevelOffset = 0x04;
constexpr std::size_t OccupationOffset = 0x0E;
constexpr std::size_t StrengthOffset = 0xBC;
constexpr std::size_t DexterityOffset = 0xC0;
constexpr std::size_t ConstitutionOffset = 0xC4;
constexpr std::size_t IntelligenceOffset = 0xC8;
constexpr std::size_t BaseMaximumHpOffset = 0xB0;
constexpr std::size_t BaseMaximumMpOffset = 0xB4;
constexpr std::size_t BaseMaximumYpOffset = 0xB8;
constexpr std::size_t BaseMaximumRpOffset = 0xBA;
constexpr std::size_t BaseMinimumAttackOffset = 0xCC;
constexpr std::size_t BaseMaximumAttackOffset = 0xD0;
constexpr std::size_t BaseHitOffset = 0xD4;
constexpr std::size_t BaseBurdenOffset = 0xD6;
constexpr std::size_t BaseCchOffset = 0xD8;
constexpr std::size_t BaseDefenseOffset = 0xDC;
constexpr std::size_t BaseDodgeOffset = 0xE0;
constexpr std::size_t BaseAttackSpeedOffset = 0xE2;
constexpr std::size_t BaseResistanceOffset = 0xE4;
constexpr std::size_t BaseHpRecoveryOffset = 0xE8;
constexpr std::size_t BaseMpRecoveryOffset = 0xEA;
constexpr std::size_t FosterCountOffset = 0x120;
constexpr std::size_t HatcherCountOffset = 0x124;
constexpr std::size_t LightningEnergyOffset = 0x17C;
constexpr std::size_t LightningFlagsOffset = 0x180;
constexpr std::size_t LightningUp60Offset = 0x184;
constexpr std::size_t LightningPillOffset = 0x186;
constexpr std::size_t LightningStampOffset = 0x188;

template <class T>
void Append(std::vector<std::uint8_t>& output, const T& value)
{
    static_assert(std::is_trivially_copyable_v<T>);
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    output.insert(output.end(), bytes, bytes + sizeof(T));
}

template <class T>
bool Read(std::span<const std::uint8_t> input, std::size_t& offset, T& value)
{
    static_assert(std::is_trivially_copyable_v<T>);
    if (offset > input.size() || input.size() - offset < sizeof(T)) {
        return false;
    }
    std::memcpy(&value, input.data() + offset, sizeof(T));
    offset += sizeof(T);
    return true;
}

template <std::size_t Size>
bool ReadArray(std::span<const std::uint8_t> input,
               std::size_t& offset,
               std::array<std::uint8_t, Size>& output)
{
    if (offset > input.size() || input.size() - offset < Size) {
        return false;
    }
    std::copy_n(input.data() + offset, Size, output.data());
    offset += Size;
    return true;
}

bool ReadCString(std::span<const std::uint8_t> input,
                 std::size_t& offset,
                 std::string& output,
                 std::size_t maximum)
{
    if (offset > input.size()) {
        return false;
    }
    const auto begin = input.begin() + static_cast<std::ptrdiff_t>(offset);
    const auto end = std::find(begin, input.end(), std::uint8_t{0});
    const auto length = static_cast<std::size_t>(std::distance(begin, end));
    if (end == input.end() || length > maximum) {
        return false;
    }
    output.assign(reinterpret_cast<const char*>(&*begin), length);
    offset += length + 1;
    return true;
}

bool AppendCString(std::vector<std::uint8_t>& output,
                   std::string_view value,
                   std::size_t maximum = std::numeric_limits<std::size_t>::max())
{
    if (value.size() > maximum || value.find('\0') != std::string_view::npos) {
        return false;
    }
    output.insert(output.end(), value.begin(), value.end());
    output.push_back(0);
    return true;
}

bool AppendFixedCString(std::vector<std::uint8_t>& output,
                        std::string_view value,
                        std::size_t size)
{
    if (value.size() >= size || value.find('\0') != std::string_view::npos) {
        return false;
    }
    const auto start = output.size();
    output.resize(start + size);
    std::copy(value.begin(), value.end(), output.begin() + static_cast<std::ptrdiff_t>(start));
    return true;
}

template <class Collection>
bool AppendCount(std::vector<std::uint8_t>& output, const Collection& collection)
{
    if (collection.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max())) {
        return false;
    }
    Append(output, static_cast<std::int32_t>(collection.size()));
    return true;
}

std::uint16_t ReadU16(const std::array<std::uint8_t, CPlayer::BasePropertyWireSize>& bytes,
                      std::size_t offset) noexcept
{
    std::uint16_t value{};
    std::memcpy(&value, bytes.data() + offset, sizeof(value));
    return value;
}

std::uint32_t ReadU32(const std::array<std::uint8_t, CPlayer::BasePropertyWireSize>& bytes,
                      std::size_t offset) noexcept
{
    std::uint32_t value{};
    std::memcpy(&value, bytes.data() + offset, sizeof(value));
    return value;
}

template <std::size_t Size, class T>
void Write(std::array<std::uint8_t, Size>& bytes, std::size_t offset, T value) noexcept
{
    std::memcpy(bytes.data() + offset, &value, sizeof(value));
}

std::uint32_t LegacyProduct(std::uint32_t value, float coefficient) noexcept
{
    return static_cast<std::uint32_t>(static_cast<std::int32_t>(
        std::nearbyint(static_cast<float>(value) * coefficient)));
}
}

CPlayer::CPlayer(const CGoodsFactory& goodsFactory)
    : m_GoodsFactory(goodsFactory),
      m_Hand(goodsFactory),
      m_Packet(goodsFactory),
      m_Equipment(goodsFactory),
      m_Wallet(goodsFactory, goodsFactory.GoldCoinIndex()),
      m_YuanBao(goodsFactory),
      m_JiFen(goodsFactory),
      m_Bank(goodsFactory),
      m_Depot(goodsFactory),
      m_Fairy(goodsFactory),
      m_BattleFairy(goodsFactory),
      m_AuctionGoods(goodsFactory),
      m_Auction(goodsFactory),
      m_AuctionWallet(goodsFactory, goodsFactory.GoldCoinIndex()),
      m_CiQing(goodsFactory),
      m_ComposeCiQing(goodsFactory)
{
    SetType(400);
    Write(m_BaseProperty, FosterCountOffset, std::uint32_t{1});
    Write(m_BaseProperty, HatcherCountOffset, std::uint32_t{1});
    m_Hand.SetGoodsAmountLimit(1);
    m_Packet.SetContainerVolume(8, 0x0C);
    m_AuctionGoods.SetContainerVolume(0x12);
    m_Auction.SetContainerVolume(2);
    m_Depot.SetContainerVolume(0xA1);
    m_Fairy.SetContainerVolume(0x0E);
    m_BattleFairy.SetContainerVolume(0x11);
    m_CiQing.SetContainerVolume(8);
    m_ComposeCiQing.SetContainerVolume(3);
}

std::uint8_t CPlayer::GetLevel() const noexcept
{
    return m_BaseProperty[LevelOffset];
}

bool CPlayer::AddToByteArray(std::vector<std::uint8_t>& output, bool includeChild) const
{
    if (!CShape::AddToByteArray(output, includeChild) || !includeChild) {
        return !includeChild;
    }
    return AddBeforeContainers(output) && AddContainers(output) && AddAfterContainers(output);
}

bool CPlayer::DecordFromByteArray(std::span<const std::uint8_t> input,
                                  std::size_t& offset,
                                  bool includeChild)
{
    m_NewSkills.clear();
    m_ExStates.clear();
    m_Friends.clear();
    m_DailyThings.clear();
    m_UncreatedPets.clear();
    m_UncreatedCarriage = {};
    if (!CShape::DecordFromByteArray(input, offset, includeChild) || !includeChild) {
        return !includeChild;
    }
    return DecodeBeforeContainers(input, offset) && DecodeContainers(input, offset) &&
           DecodeAfterContainers(input, offset);
}

bool CPlayer::AddBeforeContainers(std::vector<std::uint8_t>& output) const
{
    output.insert(output.end(), m_BaseProperty.begin(), m_BaseProperty.end());
    if (!AppendCString(output, m_Account) || !AppendCString(output, m_Title)) {
        return false;
    }
    output.insert(output.end(), m_Property.begin(), m_Property.end());
    Append(output, m_TeamId);
    if (!AppendCount(output, m_CiQingIds)) return false;
    for (const auto id : m_CiQingIds) Append(output, id);
    if (!AppendCount(output, m_NewSkills)) return false;
    for (const auto& skill : m_NewSkills) { Append(output, skill.id); Append(output, skill.level); }
    if (!AppendCount(output, m_ExStates)) return false;
    output.insert(output.end(), m_ExStates.begin(), m_ExStates.end());
    if (!AppendCount(output, m_Friends)) return false;
    for (const auto& friendValue : m_Friends) {
        if (!AppendCString(output, friendValue.name)) return false;
        output.push_back(static_cast<std::uint8_t>(friendValue.online));
    }
    Append(output, ReadU32(m_BaseProperty, LightningFlagsOffset));
    Append(output, ReadU32(m_BaseProperty, LightningEnergyOffset));
    Append(output, ReadU32(m_BaseProperty, LightningStampOffset));
    Append(output, ReadU16(m_BaseProperty, LightningUp60Offset));
    Append(output, ReadU16(m_BaseProperty, LightningPillOffset));
    if (!AppendCount(output, m_DailyThings)) return false;
    for (const auto& thing : m_DailyThings) {
        Append(output, thing.id); Append(output, thing.count);
        Append(output, thing.maximumCount); Append(output, thing.point);
    }
    return true;
}

bool CPlayer::DecodeBeforeContainers(std::span<const std::uint8_t> input, std::size_t& offset)
{
    if (!ReadArray(input, offset, m_BaseProperty) ||
        !ReadCString(input, offset, m_Account, 0x100) ||
        !ReadCString(input, offset, m_Title, 0x100) ||
        !ReadArray(input, offset, m_Property) || !Read(input, offset, m_TeamId)) return false;
    std::uint32_t ciQingCount{};
    if (!Read(input, offset, ciQingCount)) return false;
    m_CiQingIds.clear();
    for (std::uint32_t i{}; i < ciQingCount; ++i) { std::uint32_t id{}; if (!Read(input, offset, id)) return false; m_CiQingIds.insert(id); }
    std::int32_t skillCount{};
    if (!Read(input, offset, skillCount) || skillCount < 0) return false;
    for (std::int32_t i{}; i < skillCount; ++i) { Skill value; if (!Read(input, offset, value.id) || !Read(input, offset, value.level)) return false; m_NewSkills.push_back(value); }
    std::int32_t stateCount{};
    if (!Read(input, offset, stateCount) || stateCount < 0 || offset > input.size() || input.size() - offset < static_cast<std::size_t>(stateCount)) return false;
    m_ExStates.assign(input.begin() + static_cast<std::ptrdiff_t>(offset), input.begin() + static_cast<std::ptrdiff_t>(offset + stateCount));
    offset += static_cast<std::size_t>(stateCount);
    std::int32_t friendCount{};
    if (!Read(input, offset, friendCount) || friendCount < 0) return false;
    for (std::int32_t i{}; i < friendCount; ++i) { Friend value; std::uint8_t online{}; if (!ReadCString(input, offset, value.name, 0x94) || !Read(input, offset, online)) return false; value.online = online != 0; m_Friends.push_back(std::move(value)); }
    std::uint32_t flags{}, energy{}, stamp{}; std::uint16_t up60{}, pills{};
    if (!Read(input, offset, flags) || !Read(input, offset, energy) || !Read(input, offset, stamp) || !Read(input, offset, up60) || !Read(input, offset, pills)) return false;
    Write(m_BaseProperty, LightningFlagsOffset, flags); Write(m_BaseProperty, LightningEnergyOffset, energy); Write(m_BaseProperty, LightningStampOffset, stamp); Write(m_BaseProperty, LightningUp60Offset, up60); Write(m_BaseProperty, LightningPillOffset, pills);
    std::uint32_t thingCount{};
    if (!Read(input, offset, thingCount)) return false;
    for (std::uint32_t i{}; i < thingCount; ++i) { Thing value; if (!Read(input, offset, value.id) || !Read(input, offset, value.count) || !Read(input, offset, value.maximumCount) || !Read(input, offset, value.point)) return false; m_DailyThings.push_back(value); }
    return true;
}

bool CPlayer::AddContainers(std::vector<std::uint8_t>& output) const
{
    return m_Hand.Serialize(output) && m_Equipment.Serialize(output) &&
           m_Packet.Serialize(output) && m_AuctionGoods.Serialize(output) &&
           m_Auction.Serialize(output) && m_Wallet.Serialize(output) &&
           m_AuctionWallet.Serialize(output) && m_YuanBao.Serialize(output) &&
           m_JiFen.Serialize(output) && AppendCString(output, m_DepotPassword) &&
           m_Bank.Serialize(output) && m_Depot.Serialize(output) &&
           m_Fairy.Serialize(output) && m_BattleFairy.Serialize(output) &&
           m_CiQing.Serialize(output) && m_ComposeCiQing.Serialize(output);
}

bool CPlayer::DecodeContainers(std::span<const std::uint8_t> input, std::size_t& offset)
{
    m_Hand.Release(); m_Hand.SetGoodsAmountLimit(1);
    if (!m_Hand.Unserialize(input, offset)) return false;
    m_Equipment.Release(); if (!m_Equipment.Unserialize(input, offset)) return false;
    m_Packet.Release(); m_Packet.SetContainerVolume(8, 0x0C); if (!m_Packet.Unserialize(input, offset)) return false;
    m_AuctionGoods.Release(); m_AuctionGoods.SetContainerVolume(0x12); if (!m_AuctionGoods.Unserialize(input, offset)) return false;
    m_Auction.Release(); m_Auction.SetContainerVolume(2); if (!m_Auction.Unserialize(input, offset)) return false;
    m_Wallet.Release(); if (!m_Wallet.Unserialize(input, offset)) return false;
    m_AuctionWallet.Release(); if (!m_AuctionWallet.Unserialize(input, offset)) return false;
    m_YuanBao.Release(); if (!m_YuanBao.Unserialize(input, offset)) return false;
    m_JiFen.Release(); if (!m_JiFen.Unserialize(input, offset)) return false;
    if (!ReadCString(input, offset, m_DepotPassword, 0x6C)) return false;
    m_Bank.Release(); if (!m_Bank.Unserialize(input, offset)) return false;
    m_Depot.Release(); m_Depot.SetContainerVolume(0xA1); if (!m_Depot.Unserialize(input, offset)) return false;
    m_Fairy.Release(); m_Fairy.SetContainerVolume(0x0E); if (!m_Fairy.Unserialize(input, offset)) return false;
    m_BattleFairy.Release(); m_BattleFairy.SetContainerVolume(0x11); if (!m_BattleFairy.Unserialize(input, offset)) return false;
    m_CiQing.Release(); m_CiQing.SetContainerVolume(8); if (!m_CiQing.Unserialize(input, offset)) return false;
    m_ComposeCiQing.Release(); m_ComposeCiQing.SetContainerVolume(3); return m_ComposeCiQing.Unserialize(input, offset);
}

bool CPlayer::AddOrganizing(std::vector<std::uint8_t>& output) const
{
    Append(output, m_Organizing.factionId);
    if (m_Organizing.factionId <= 0) return true;
    Append(output, m_Organizing.factionLogoId); Append(output, m_Organizing.factionLevel);
    Append(output, m_Organizing.factionExperience); Append(output, m_Organizing.force);
    Append(output, static_cast<std::uint32_t>(m_Organizing.factionContribute));
    if (!AppendCString(output, m_Organizing.factionName) || !AppendCString(output, m_Organizing.factionTitle)) return false;
    Append(output, m_Organizing.factionMasterId); Append(output, m_Organizing.unionId); Append(output, m_Organizing.unionMasterId);
    if (!AppendCount(output, m_Organizing.enemyFactions)) return false;
    for (auto id : m_Organizing.enemyFactions) Append(output, id);
    if (!AppendCount(output, m_Organizing.cityWarEnemyFactions)) return false;
    for (auto id : m_Organizing.cityWarEnemyFactions) Append(output, id);
    if (!AppendCount(output, m_Organizing.ownedRegions)) return false;
    for (const auto& region : m_Organizing.ownedRegions) output.insert(output.end(), region.begin(), region.end());
    return true;
}

bool CPlayer::AddAfterContainers(std::vector<std::uint8_t>& output) const
{
    if (m_VariableData.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()) || !m_Country || !m_Contribute) return false;
    Append(output, m_VariableCount); Append(output, static_cast<std::int32_t>(m_VariableData.size()));
    output.insert(output.end(), m_VariableData.begin(), m_VariableData.end());
    Append(output, m_SilenceTime); output.push_back(static_cast<std::uint8_t>(IsGod()));
    Append(output, m_MurdererTime); Append(output, m_FightStateCount);
    if (!AppendCount(output, m_UncreatedPets)) return false;
    for (const auto& pet : m_UncreatedPets) { if (!AppendCString(output, pet.originalName)) return false; Append(output, pet.hp); Append(output, pet.level); Append(output, pet.experience); }
    if (!AppendCString(output, m_UncreatedCarriage.originalName) || !AppendCString(output, m_UncreatedCarriage.script)) return false;
    Append(output, m_UncreatedCarriage.hp); output.push_back(static_cast<std::uint8_t>(m_RecreateCarriage)); output.push_back(static_cast<std::uint8_t>(m_Login));
    Append(output, m_CityWarDiedStateTime);
    if (!AppendCount(output, m_Quests)) return false;
    for (const auto& [id, quest] : m_Quests) { (void)id; Append(output, quest.id); output.push_back(quest.complete); }
    output.push_back(*m_Country); Append(output, *m_Contribute);
    output.insert(output.end(), m_JjcData.begin(), m_JjcData.end()); output.push_back(static_cast<std::uint8_t>(m_JjcPkState));
    return AddOrganizing(output) && AppendFixedCString(output, m_SessionId, 0x40);
}

bool CPlayer::DecodeAfterContainers(std::span<const std::uint8_t> input, std::size_t& offset)
{
    std::int32_t variableLength{};
    if (!Read(input, offset, m_VariableCount) || !Read(input, offset, variableLength) || variableLength < 0 || offset > input.size() || input.size() - offset < static_cast<std::size_t>(variableLength)) return false;
    m_VariableData.assign(input.begin() + static_cast<std::ptrdiff_t>(offset), input.begin() + static_cast<std::ptrdiff_t>(offset + variableLength)); offset += static_cast<std::size_t>(variableLength);
    std::uint8_t god{}; std::int32_t ignoredFightCount{};
    if (!Read(input, offset, m_SilenceTime) || !Read(input, offset, god) || !Read(input, offset, m_MurdererTime) || !Read(input, offset, ignoredFightCount)) return false;
    SetGod(god != 0); m_FightStateCount = 2;
    std::int32_t petCount{}; if (!Read(input, offset, petCount) || petCount < 0) return false;
    for (std::int32_t i{}; i < petCount; ++i) { Pet pet; if (!ReadCString(input, offset, pet.originalName, 0x94) || !Read(input, offset, pet.hp) || !Read(input, offset, pet.level) || !Read(input, offset, pet.experience)) return false; m_UncreatedPets.push_back(std::move(pet)); }
    std::uint8_t recreate{}, login{};
    if (!ReadCString(input, offset, m_UncreatedCarriage.originalName, 0x94) || !ReadCString(input, offset, m_UncreatedCarriage.script, 0x94) || !Read(input, offset, m_UncreatedCarriage.hp) || !Read(input, offset, recreate) || !Read(input, offset, login) || !Read(input, offset, m_CityWarDiedStateTime)) return false;
    m_RecreateCarriage = recreate != 0; m_Login = login != 0;
    std::int32_t questCount{}; if (!Read(input, offset, questCount) || questCount < 0) return false; m_Quests.clear();
    for (std::int32_t i{}; i < questCount; ++i) { Quest quest; if (!Read(input, offset, quest.id) || !Read(input, offset, quest.complete)) return false; m_Quests[quest.id] = quest; }
    std::uint8_t country{}; std::int32_t contribute{}; if (!Read(input, offset, country) || !Read(input, offset, contribute) || !ReadArray(input, offset, m_JjcData)) return false;
    m_Country = country; m_Contribute = contribute; std::uint8_t jjc{}; if (!Read(input, offset, jjc)) return false; m_JjcPkState = jjc != 0;
    // Organization не декодируется в исходном clone-path: сразу следует fixed session ID.
    if (offset > input.size() || input.size() - offset < 0x40) return false;
    const auto block = input.subspan(offset, 0x40); const auto nul = std::find(block.begin(), block.end(), std::uint8_t{0});
    if (nul == block.end()) return false;
    m_SessionId.assign(reinterpret_cast<const char*>(block.data()), static_cast<std::size_t>(std::distance(block.begin(), nul))); offset += 0x40;
    return true;
}

bool CPlayer::UpdateProperty(const PropertyCoefficients& c) noexcept
{
    const auto occupation = static_cast<std::size_t>(m_BaseProperty[OccupationOffset]);
    if (occupation >= 3) return false;
    const auto strength = ReadU32(m_BaseProperty, StrengthOffset);
    const auto dexterity = ReadU32(m_BaseProperty, DexterityOffset);
    const auto constitution = ReadU32(m_BaseProperty, ConstitutionOffset);
    const auto intelligence = ReadU32(m_BaseProperty, IntelligenceOffset);
    Write(m_Property, 0x0C, strength); Write(m_Property, 0x10, dexterity); Write(m_Property, 0x14, constitution); Write(m_Property, 0x18, intelligence);
    Write(m_Property, 0x00, ReadU32(m_BaseProperty, BaseMaximumHpOffset) + LegacyProduct(constitution, c.constitutionToMaximumHp[occupation]));
    Write(m_Property, 0x04, ReadU32(m_BaseProperty, BaseMaximumMpOffset) + LegacyProduct(intelligence, c.intelligenceToMaximumMp[occupation]));
    Write(m_Property, 0x08, ReadU16(m_BaseProperty, BaseMaximumYpOffset)); Write(m_Property, 0x0A, ReadU16(m_BaseProperty, BaseMaximumRpOffset));
    Write(m_Property, 0x1C, ReadU32(m_BaseProperty, BaseMinimumAttackOffset) + LegacyProduct(dexterity, c.dexterityToMinimumAttack[occupation]));
    Write(m_Property, 0x20, ReadU32(m_BaseProperty, BaseMaximumAttackOffset) + LegacyProduct(strength, c.strengthToMaximumAttack[occupation]));
    Write(m_Property, 0x24, ReadU16(m_BaseProperty, BaseHitOffset));
    Write(m_Property, 0x26, static_cast<std::uint16_t>(ReadU16(m_BaseProperty, BaseBurdenOffset) + LegacyProduct(strength, c.strengthToBurden[occupation])));
    Write(m_Property, 0x28, ReadU16(m_BaseProperty, BaseCchOffset));
    Write(m_Property, 0x2C, ReadU32(m_BaseProperty, BaseDefenseOffset) + LegacyProduct(constitution, c.constitutionToDefense[occupation]));
    Write(m_Property, 0x30, ReadU16(m_BaseProperty, BaseDodgeOffset)); Write(m_Property, 0x32, ReadU16(m_BaseProperty, BaseAttackSpeedOffset));
    Write(m_Property, 0x34, ReadU32(m_BaseProperty, BaseResistanceOffset) + LegacyProduct(intelligence, c.intelligenceToResistance[occupation]));
    Write(m_Property, 0x38, ReadU16(m_BaseProperty, BaseHpRecoveryOffset)); Write(m_Property, 0x3A, ReadU16(m_BaseProperty, BaseMpRecoveryOffset));
    Write(m_Property, 0x48, LegacyProduct(intelligence, c.intelligenceToElement[occupation]));
    Write(m_Property, 0x4C, static_cast<std::uint16_t>(LegacyProduct(dexterity, c.dexterityToStiffness[occupation])));
    return true;
}
