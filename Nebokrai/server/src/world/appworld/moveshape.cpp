#include "moveshape.h"

/*
 * Исходный владелец: WorldServer/appworld/moveshape.cpp.
 * Nworldserver.exe/PDB и Rust подтверждают append, а не replace: ненулевые
 * extended-state bytes добавляются в хвост; очистку перед player decode
 * выполняет сам player-owner. m_bIsGod по умолчанию false.
 */
void CMoveShape::SetExStates(std::span<const std::uint8_t> states)
{
    m_ExStates.insert(m_ExStates.end(), states.begin(), states.end());
}
