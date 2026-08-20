#pragma once

#include <cstdint>
#include <memory>
#include <mutex>
#include <string>

/*
 * Исходный владелец: loginserver/applogin/gasoperator.cpp / gasoperator.h
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Подтверждённые RVA: конструктор 0x004264B0, деструктор 0x00426510,
 * GetInstance 0x004265E0,
 * ReleaseInstance 0x00426610, GetIP 0x00421500.
 *
 * PDB-layout старого CGasOperator содержит шесть std::string и m_pReadBuf.
 * Фактический рабочий поток GAS использует у этого владельца только GetIP:
 * исходный IPv4
 * форматируется в порядке младшего байта к старшему как "%u.%u.%u.%u".
 * Singleton-lifetime сохраняется на уровне API; std::unique_ptr/mutex заменяют
 * только ручной глобальный new/delete и не добавляют игрового состояния.
 */
namespace Login
{
class CGasOperator
{
public:
    CGasOperator() = default;
    ~CGasOperator() = default;

    CGasOperator(const CGasOperator&) = delete;
    CGasOperator& operator=(const CGasOperator&) = delete;

    [[nodiscard]] static CGasOperator* GetInstance() noexcept;
    static void ReleaseInstance() noexcept;

    [[nodiscard]] std::string GetIP(std::uint32_t ip) const;

private:
    std::string m_strObjectPath;
    std::string m_strURL;
    std::string m_strObject;
    char* m_pReadBuf{};
    std::string m_strValue;
    std::string m_strGetObjectName;
    std::string m_strHost;

    static std::mutex s_InstanceMutex;
    static std::unique_ptr<CGasOperator> s_Instance;
};
}
