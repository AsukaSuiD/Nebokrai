#pragma once

#include "baseobject.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <functional>
#include <optional>
#include <span>
#include <string>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/region.cpp / region.h.
 * CLS-RGN v1, размеры cell/switch, defaults и walkable-mask `(byte0 & 7)==0
 * && u16@+2==0` подтверждены Nworldserver.exe/PDB и 872 resource fixtures.
 * Файл читается уже загруженным span: filesystem/resource backend не является
 * частью игрового region-контракта.
 */
class CRegion : public CBaseObject
{
public:
    using Cell = std::array<std::uint8_t, 4>;
    struct Switch {
        std::int32_t state{};
        std::int32_t regionId{};
        std::int32_t x{};
        std::int32_t y{};
        std::int32_t direction{};
    };

    CRegion();
    ~CRegion() override = default;

    bool LoadResource(std::string_view path, std::span<const std::uint8_t> source);
    bool New();
    bool AddToByteArray(std::vector<std::uint8_t>& output,
                        bool includeChild) const override;
    bool DecordFromByteArray(std::span<const std::uint8_t> input,
                             std::size_t& offset,
                             bool includeChild) override;

    void SetResourceID(std::int32_t value) noexcept { m_ResourceId = value; }
    void SetExpScale(float value) noexcept { m_ExpScale = value; }
    void SetCountry(std::uint8_t value) noexcept { m_Country = value; }
    void SetNotify(std::int32_t value) noexcept { m_Notify = value; }
    [[nodiscard]] std::optional<std::uint8_t> GetCountry() const noexcept { return m_Country; }
    [[nodiscard]] std::int32_t GetWidth() const noexcept { return m_Width; }
    [[nodiscard]] std::int32_t GetHeight() const noexcept { return m_Height; }
    [[nodiscard]] const std::vector<Cell>& GetCells() const noexcept { return m_Cells; }
    [[nodiscard]] const std::vector<Switch>& GetSwitches() const noexcept { return m_Switches; }
    [[nodiscard]] bool GetRandomPosInRange(std::int32_t& x,
                                           std::int32_t& y,
                                           std::int32_t startX,
                                           std::int32_t startY,
                                           std::int32_t width,
                                           std::int32_t height,
                                           const std::function<std::int32_t(std::int32_t)>& random) const;

protected:
    [[nodiscard]] bool IsWalkable(std::int32_t x, std::int32_t y) const noexcept;

    std::string m_FileName;
    std::int32_t m_RegionType{};
    std::int32_t m_ResourceId{};
    float m_ExpScale{1.0F};
    std::int32_t m_Width{};
    std::int32_t m_Height{};
    std::optional<std::uint8_t> m_Country;
    std::optional<std::int32_t> m_Notify;
    std::vector<Cell> m_Cells;
    std::vector<Switch> m_Switches;
};

static_assert(sizeof(CRegion::Cell) == 4);
static_assert(sizeof(CRegion::Switch) == 20);
