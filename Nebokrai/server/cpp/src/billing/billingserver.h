#pragma once

#include "game.h"

#include <filesystem>

namespace Billing
{
/* Точка процессного lifecycle поверх исходного CGame BillingServer. */
class BillingServer
{
public:
    [[nodiscard]] BillingGameResult Initialize(
        const std::filesystem::path& runtimeDirectory);
    [[nodiscard]] BillingGameResult RunTurn();
    [[nodiscard]] BillingGameResult Release();

private:
    CGame m_Game;
};
}
