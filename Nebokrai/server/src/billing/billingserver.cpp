#include "billingserver.h"

namespace Billing
{
BillingGameResult BillingServer::Initialize(
    const std::filesystem::path& runtimeDirectory)
{
    return m_Game.Initialize(runtimeDirectory);
}

BillingGameResult BillingServer::RunTurn()
{
    return m_Game.RunTurn();
}

BillingGameResult BillingServer::Release()
{
    return m_Game.Release();
}
}
