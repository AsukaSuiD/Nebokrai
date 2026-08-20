#pragma once

#include "moveshape.h"

#include <cstdint>
#include <functional>
#include <string>
#include <string_view>
#include <utility>

/*
 * Исходный владелец: WorldServer/appworld/monster.cpp / monster.h.
 * CMoveShape-base и object type 600 подтверждены exact PDB/disassembly.
 * GetFigure в оригинале разрешал figure через monster setup по original name;
 * process-global CMonsterList заменён внедряемым read-only resolver-ом.
 * AI/property-table, не достигнутые реконструкцией, здесь не выдумываются.
 */
class CMonster final : public CMoveShape
{
public:
    using FigureResolver = std::function<std::uint8_t(std::string_view)>;

    CMonster();
    explicit CMonster(std::string originalName);

    [[nodiscard]] std::uint8_t GetFigure() const noexcept override;
    [[nodiscard]] std::string_view GetOriginalName() const noexcept { return m_OriginalName; }
    void SetOriginalName(std::string value) { m_OriginalName = std::move(value); }
    void SetFigureResolver(FigureResolver resolver) { m_FigureResolver = std::move(resolver); }

private:
    std::string m_OriginalName;
    FigureResolver m_FigureResolver;
};
