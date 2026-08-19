#pragma once

#include <cstdint>
#include <deque>
#include <functional>
#include <mutex>
#include <span>
#include <vector>

/*
 * Owner: loginserver/loginqueue.cpp / loginqueue.h
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Исходный путь PDB:
 * d:\complite_version\fengyun_russia\trunk\server\loginserver\loginserver\loginqueue.cpp
 *
 * Полный owner в поздней реконструкции уже восстановлен; в этой материализации
 * закрыта password/Auth-stage часть, которой прямо пользуется AuthHandler:
 * TagPwdChecked и PushBackPwdChecked. Exact account duplicate ищется под тем же
 * lock, CGame::KickOut вызывается до удаления прежнего объекта и всё ещё под
 * lock, затем новый объект всегда добавляется в хвост.
 *
 * Остальные коллекции CLoginQueue (quest/login/player/valid-code/matrix) не
 * получают пустых полей или заглушек: они будут добавляться вместе с их уже
 * восстановленными message-path, сохраняя единый state-owner.
 */
namespace Login
{
class TagPwdChecked
{
public:
    TagPwdChecked(std::int32_t socketId,
                  std::uint32_t clientIp,
                  std::vector<std::uint8_t> account,
                  std::vector<std::uint8_t> worldServer,
                  bool hasMatrix);

    [[nodiscard]] std::int32_t SocketID() const noexcept;
    [[nodiscard]] std::uint32_t ClientIP() const noexcept;
    [[nodiscard]] std::span<const std::uint8_t> Account() const noexcept;
    [[nodiscard]] std::span<const std::uint8_t> WorldServer() const noexcept;
    [[nodiscard]] bool HasMatrix() const noexcept;

private:
    std::uint32_t m_ClientIP{};
    std::int32_t m_SocketID{};
    std::vector<std::uint8_t> m_Account;
    std::vector<std::uint8_t> m_WorldServer;
    bool m_HasMatrix{};
};

class CLoginQueue
{
public:
    using KickOutCallback = std::function<void(std::span<const std::uint8_t>)>;

    CLoginQueue() = default;

    void PushBackPwdChecked(TagPwdChecked checked, const KickOutCallback& kickOut);
    [[nodiscard]] std::size_t PendingPwdChecked() const;

private:
    mutable std::mutex m_PwdCheckedMutex;
    std::deque<TagPwdChecked> m_PwdChecked;
};
}
