#pragma once

#include "region.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <span>
#include <string>
#include <functional>
#include <vector>

enum class RegionType : std::int32_t {
    Normal = 0, Village = 1, City = 2, Country = 3, Nation = 4, GodsBattle = 5,
};

struct RegionRect {
    std::int32_t left{};
    std::int32_t top{};
    std::int32_t right{};
    std::int32_t bottom{};
};

struct RegionParam {
    std::int32_t id{};
    std::int32_t maxTaxRate{};
    std::int32_t currentTaxRate{};
    std::uint32_t totalTax{};
    std::uint32_t todayTotalTax{};
    std::int32_t superiorRegionId{};
    std::int32_t turnInTaxRate{};
    std::int32_t ownedFactionId{};
    std::int32_t ownedUnionId{};
};

/*
 * Исходный владелец: WorldServer/appworld/worldregion.cpp / .h.
 * Полный World sender wire, отдельный proxy wire, selective param decoder и
 * девятиполевый DB snapshot подтверждены Nworldserver.exe/PDB и поздней Rust-
 * реконструкцией. Constructor не инициализировал setup; optional не позволяет
 * тихо отправить придуманные нули до загрузки regions/{id}.rs.
 */
class CWorldRegion : public CRegion
{
public:
    struct Setup {
        std::int32_t returnRegionId{};
        RegionRect returnPoint;
        std::int32_t recallWhenLost{};
        std::int32_t moveMonsterWhenRefresh{};
        std::int32_t use{};
    };
    struct NpcRecord { std::array<std::uint8_t, 0x24> header{}; std::string name; std::string script; };
    struct MonsterVariant { std::array<std::uint8_t, 0x22> legacyPrefix{}; std::string name; std::string script; };
    struct MonsterRecord { std::array<std::uint8_t, 0x24> header{}; std::vector<MonsterVariant> variants; };
    struct Weather { std::int32_t index{}; std::uint32_t fogColor{}; };
    struct WeatherOption { std::int32_t cumulativeOdds{}; std::vector<Weather> weather; };
    struct WeatherTime { std::int32_t time{}; std::vector<WeatherOption> options; };

    CWorldRegion() = default;
    ~CWorldRegion() override = default;

    bool AddToByteArray(std::vector<std::uint8_t>& output,
                        bool includeChild) const override;
    bool DecordFromByteArray(std::span<const std::uint8_t>,
                             std::size_t&,
                             bool) override { return true; }
    bool AddToByteArrayForProxy(std::vector<std::uint8_t>& output,
                                bool includeChild) const;
    bool DecordRegionParamFromByteArray(std::span<const std::uint8_t> input,
                                        std::size_t& offset);
    bool LoadSetup(std::string_view text);
    bool LoadNpcList(std::string_view text,
                     const std::function<std::string(std::string_view)>& resolveName);
    bool LoadMonsterList(std::string_view text, float countScale = 1.0F);
    bool LoadWeatherSetup(std::optional<std::string_view> text);
    bool LoadTaxParam(std::optional<std::string_view> text);

    void SetNoPk(bool value) noexcept { m_NoPk = value; }
    void SetNoContribute(bool value) noexcept { m_NoContribute = value; }
    void SetWarRegionType(RegionType value) noexcept { m_WarRegionType = value; }
    [[nodiscard]] bool IsNoPk() const noexcept { return m_NoPk; }
    [[nodiscard]] bool IsNoContribute() const noexcept { return m_NoContribute; }
    [[nodiscard]] RegionType GetWarRegionType() const noexcept { return m_WarRegionType; }
    void SetParamFromDB(std::int32_t ownedFactionId,
                        std::int32_t ownedUnionId,
                        std::int32_t currentTaxRate,
                        std::int32_t todayTotalTax,
                        std::int32_t totalTax) noexcept;
    void SetParamFromGame(std::int32_t currentTaxRate,
                          std::uint32_t todayTotalTax,
                          std::uint32_t totalTax) noexcept;
    void SetOwnedCityOrg(std::int32_t factionId, std::int32_t unionId) noexcept;
    [[nodiscard]] const RegionParam& GetParam() const noexcept { return m_Param; }
    [[nodiscard]] std::optional<Setup> GetSetup() const noexcept { return m_Setup; }
    [[nodiscard]] RegionParam GenerateSaveData() const noexcept { return m_Param; }
    [[nodiscard]] std::vector<NpcRecord>& Npcs() noexcept { return m_Npcs; }
    [[nodiscard]] std::vector<MonsterRecord>& Monsters() noexcept { return m_Monsters; }
    [[nodiscard]] std::vector<WeatherTime>& WeatherSetup() noexcept { return m_Weather; }

protected:
    std::optional<Setup> m_Setup;
    RegionType m_WarRegionType{RegionType::Normal};
    bool m_NoPk{};
    bool m_NoContribute{};
    std::vector<std::string> m_ForbiddenMakeGoods;
    std::vector<NpcRecord> m_Npcs;
    std::vector<MonsterRecord> m_Monsters;
    std::vector<WeatherTime> m_Weather;
    RegionParam m_Param{};
};

static_assert(sizeof(RegionRect) == 16);
static_assert(sizeof(RegionParam) == 36);
static_assert(sizeof(CWorldRegion::Setup) == 32);
