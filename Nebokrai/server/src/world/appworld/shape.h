#pragma once

#include "baseobject.h"

#include <cstddef>
#include <cstdint>
#include <span>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/shape.cpp / shape.h.
 * Поля, default speed=2000, tile-центр +0.5 и wire-порядок подтверждены
 * Nworldserver.exe/PDB и поздней Rust-реконструкцией. При чтении wire m_lPos
 * намеренно потребляется, но live m_lPos становится нулём — это особенность
 * оригинала, а не исправление. Небезопасный pointer decoder заменён span.
 */
class CShape : public CBaseObject
{
public:
    CShape() = default;
    ~CShape() override = default;

    bool AddToByteArray(std::vector<std::uint8_t>& output,
                        bool includeChild) const override;
    bool DecordFromByteArray(std::span<const std::uint8_t> input,
                             std::size_t& offset,
                             bool includeChild) override;

    [[nodiscard]] virtual std::uint8_t GetFigure() const noexcept { return 0; }
    [[nodiscard]] std::int32_t GetRegionID() const noexcept { return m_RegionId; }
    void SetRegionID(std::int32_t value) noexcept { m_RegionId = value; }
    [[nodiscard]] float GetPosX() const noexcept { return m_PosX; }
    [[nodiscard]] float GetPosY() const noexcept { return m_PosY; }
    void SetPosX(float value) noexcept { m_PosX = value; }
    void SetPosY(float value) noexcept { m_PosY = value; }
    void SetPosXY(float x, float y) noexcept { m_PosX = x; m_PosY = y; }
    [[nodiscard]] std::int32_t GetDir() const noexcept { return m_Direction; }
    void SetDir(std::int32_t value) noexcept { m_Direction = value; }
    [[nodiscard]] std::int32_t GetPos() const noexcept { return m_Position; }
    void SetPos(std::int32_t value) noexcept { m_Position = value; }
    [[nodiscard]] float GetSpeed() const noexcept { return m_Speed; }
    void SetSpeed(float value) noexcept { m_Speed = value; }
    [[nodiscard]] std::uint16_t GetState() const noexcept { return m_State; }
    void SetState(std::uint16_t value) noexcept { m_State = value; }
    [[nodiscard]] std::uint16_t GetAction() const noexcept { return m_Action; }
    void SetAction(std::uint16_t value) noexcept { m_Action = value; }
    [[nodiscard]] std::int32_t GetTileX() const noexcept;
    [[nodiscard]] std::int32_t GetTileY() const noexcept;
    void SetTileXY(std::int32_t tileX, std::int32_t tileY) noexcept;

protected:
    bool AddShapeToByteArray(std::vector<std::uint8_t>& output) const;
    bool DecordShapeFromByteArray(std::span<const std::uint8_t> input,
                                  std::size_t& offset);

    std::int32_t m_RegionId{};
    float m_PosX{};
    float m_PosY{};
    std::int32_t m_Direction{};
    std::int32_t m_Position{};
    float m_Speed{2000.0F};
    std::uint16_t m_State{};
    std::uint16_t m_Action{};
};

class CMoveShape : public CShape
{
public:
    CMoveShape() = default;
    ~CMoveShape() override = default;

    void SetExStates(std::span<const std::uint8_t> states);
    void ClearExStates() noexcept { m_ExStates.clear(); }
    [[nodiscard]] std::span<const std::uint8_t> GetExStates() const noexcept { return m_ExStates; }
    [[nodiscard]] bool IsGod() const noexcept { return m_IsGod; }
    void SetGod(bool value) noexcept { m_IsGod = value; }

protected:
    std::vector<std::uint8_t> m_ExStates;
    bool m_IsGod{};
};
