#pragma once

#include "acclogqueue.h"

#include <array>
#include <cstdint>
#include <deque>
#include <list>
#include <map>
#include <mutex>
#include <optional>
#include <string>

namespace LoginNet
{
class CMessage;
class CMyNetServerClient;
class CMyNetServerWorld;
}

namespace Login
{
class CLoginQueue;

/*
 * Owner: loginserver/game.cpp / game.h — доказанный CGame slice.
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Материализуются только уже доказанные поля и методы; полный 1024-byte PDB
 * layout не подменяется фиктивными полями и расширяется по мере recovery.
 */
class CGame
{
public:
    struct tagWorldInfo
    {
        std::string strName;
        std::int32_t lStateLvl{};
    };

    // Уже доказанный DB/GAS хвост исходного tagSetup. Остальные поля будут
    // добавлены только вместе с LoadSetup/ReLoadSetup owner.
    struct tagSetup
    {
        std::string _db_ip;
        std::string _db_billing_name;
        std::string _db_user;
        std::string _db_psd;
        std::int32_t m_lIsInsideUse{1};
        std::string m_strVerificationAddr;
        std::int32_t m_lVerifiSignUpper{};
    };

    [[nodiscard]] std::int32_t GetWorldIDByName(const char* worldName) const;
    [[nodiscard]] bool WorldServerIsOpenState(std::int32_t worldId) const;
    void AddWorldInfoToMsg(LoginNet::CMessage& message, const char* account) const;

    [[nodiscard]] std::int32_t FindCdkey(const char* account) const;
    [[nodiscard]] bool L2W_PlayerBase_Send(const char* worldName,
                                           const char* account) const;

    [[nodiscard]] const char* GetLoginCdkeyWorldServer(const char* account) const;
    void SetLoginCdkeyWorldServer(const char* account, const char* worldServer);
    [[nodiscard]] bool KickOut(const char* account) const;

    void AccountEnterLog(const char* account, std::uint32_t ip);
    [[nodiscard]] std::int32_t PrepareEnter(const char* account,
                                            std::uint32_t ip,
                                            std::int32_t socketId,
                                            const char* worldServer,
                                            bool matrix);
    void EnterGame(const char* account,
                   std::uint32_t ip,
                   std::int32_t socketId,
                   const char* worldServer) const;

    // Direct EXE: getAccInfoEx side effect через ADO; fourth int и @Result не
    // участвуют в state-machine, функция всегда возвращает false.
    [[nodiscard]] bool ExecuteProce(std::string userId,
                                    std::string userIp,
                                    char* passwordHex,
                                    std::int32_t unusedResult);

    [[nodiscard]] std::optional<std::string> PopTechnicalError();

private:
    void RecordTechnicalError(std::string detail);

    CLoginQueue* m_pLoginQueue{};
    LoginNet::CMyNetServerClient* s_pNetServer_Client{};
    LoginNet::CMyNetServerWorld* s_pNetServer_World{};

    std::map<std::string, std::string> m_LoginCdkeyWorld;
    std::map<std::int32_t, std::list<std::string>> s_listCdkey;
    std::array<std::int32_t, 4> m_StateLvl{};
    std::map<std::int32_t, tagWorldInfo> m_listWorldInfo;
    std::map<std::int32_t, tagWorldInfo> m_WorldInfoSetup;
    tagSetup m_Setup;
    AccLogQueue _acc_logs;

    mutable std::mutex m_TechnicalErrorMutex;
    std::deque<std::string> m_TechnicalErrors;
};
}
