#pragma once

#include "moveshape.h"

#include <string>
#include <utility>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/npc.cpp / npc.h.
 * CMoveShape-base, object type 500 и собственный ordered script-list
 * подтверждены Nworldserver.exe/PDB. Старый MSVC list заменён vector: порядок
 * сценариев сохраняется, allocator/runtime machinery не переносится.
 */
class CNpc final : public CMoveShape
{
public:
    CNpc();

    [[nodiscard]] const std::vector<std::string>& Scripts() const noexcept { return m_Scripts; }
    void SetScripts(std::vector<std::string> scripts) { m_Scripts = std::move(scripts); }
    void AddScript(std::string script) { m_Scripts.push_back(std::move(script)); }

private:
    std::vector<std::string> m_Scripts;
};
