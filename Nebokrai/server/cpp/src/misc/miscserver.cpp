#include "miscserver.h"

namespace Misc
{
MiscGameResult MiscServer::Initialize(const std::filesystem::path& runtimeDirectory)
{
    return m_Game.Initialize(runtimeDirectory);
}
MiscGameResult MiscServer::RunTurn() { return m_Game.RunTurn(); }
MiscGameResult MiscServer::Release() { return m_Game.Release(); }
}
