#pragma once

#include "game.h"

#include <filesystem>

namespace Misc
{
class MiscServer
{
public:
    [[nodiscard]] MiscGameResult Initialize(const std::filesystem::path& runtimeDirectory);
    [[nodiscard]] MiscGameResult RunTurn();
    [[nodiscard]] MiscGameResult Release();

private:
    CGame m_Game;
};
}
