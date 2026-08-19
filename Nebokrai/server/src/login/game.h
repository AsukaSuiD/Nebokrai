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
class AuthManager;
class CLoginQueue;

/*
 * Owner: loginserver/game.cpp / game.h — доказанный CGame slice.
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Материализуются только уже доказанные поля и методы; полный 1024-byte PDB
 * layout не подменяется фиктивными полями и расширяется по мере recovery.
 *
 * Setup owner: tagSetup ctor 0x0040C6E0, ChangeAllWorldSate 0x004075E0,
 * LoadSetupEx 0x0040D9E0, ReLoadSetupEx 0x0040DC60,
 * LoadSetup 0x0040E690, ReLoadSetup 0x0040F4E0,
 * LoadWorldSetup 0x00410F30, SetListWorldInfoBySetup 0x00411170,
 * ReLoadWorldSetup 0x00411FF0, UpdateWorldInfoToAllClient 0x00407860,
 * CGame ctor defaults 0x00414350.
 */
class CGame
{
public:
    struct tagWorldInfo
    {
        std::string strName;
        std::int32_t lStateLvl{};
    };

    // Полный PDB-field set исходного tagSetup, необходимый LoadSetup.
    // Constructor defaults восстановлены отдельно из tagSetup::tagSetup 0x40C6E0;
    // неинициализированные там scalar-поля намеренно не получают fake zero-default.
    struct tagSetup
    {
        tagSetup();

        std::int32_t _server_version;
        std::uint32_t dwListenPort_World;
        std::uint32_t dwListenPort_Client;
        std::string strSqlConType;
        std::string strSqlServerIP;
        std::string strSqlUserName;
        std::string strSqlPassWord;
        std::string strDBName;
        bool bCheckNet;
        std::uint32_t dwMaxByteNum;
        std::uint32_t dwMaxMsgLen;
        std::uint32_t dwBanIPTime;
        bool bCheckMsgCon;
        std::int32_t lMaxConnectNum;
        std::int32_t lMaxIOSendNum;
        std::int32_t lMaxClientSendBuf;
        bool bWorldCheckNet;
        std::uint32_t dwWorldMaxByteNum;
        std::uint32_t dwWorldMaxMsgLen;
        std::uint32_t dwWorldBanIPTime;
        bool bWorldCheckMsgCon;
        std::int32_t lWorldMaxConnectNum;
        std::int32_t lWorldMaxIOSendNum;
        std::int32_t lWorldMaxClientSendBuf;
        std::uint32_t dwRefeashInfoTime;
        std::uint32_t dwSaveInfoTime;
        std::uint32_t dwDoQueueInter;
        std::uint32_t dwSendMsgToQueInter;
        std::uint32_t dwWorldMaxPlayers;
        float fWorldBusyScale;
        float fWorldFullScale;
        std::uint32_t dwPingWorldServerTime;
        std::uint32_t dwPingWorldServerErrorTime;
        bool bCheckForbidIP;
        bool bCheckAllowIP;
        bool bCheckBetweenIP;
        std::uint32_t dwServerInfoLogTime;
        std::string strServerInfoLogProvider;
        std::string strServerInfoLogUID;
        std::string strServerInfoLogPWD;
        std::string strServerInfoLogIP;
        std::string strServerInfoLogDB;
        std::uint32_t authTimeOut;
        std::string _bindIP;
        std::uint16_t _bindPort;
        std::int32_t lforbitTime;
        std::int32_t lforbitNum;
        std::int32_t lMode;
        std::string strGasAreaUserID;
        std::string strGasAreaPasswd;
        std::string _db_ip;
        std::string _db_billing_name;
        std::string _db_user;
        std::string _db_psd;
        std::int32_t m_lIsInsideUse;
        std::string m_strVerificationAddr;
        std::int32_t m_lVerifiSignUpper;
    };

    struct tagSetupEx
    {
        // Direct CGame ctor leaves only iAreaId without an explicit assignment.
        std::int32_t iAreaId;
        std::int32_t lClientMaxBlockConNum{10};
        std::int32_t lClientValidDelayRecDataTime{4000};
        std::int32_t lWorldMaxBlockConNum{5};
        std::int32_t lWorldValidDelayRecDataTime{4000};
        std::int32_t lQuestPlayerDataInterval{3000};
        std::int32_t matrix_timeout{60000};
        std::int32_t bValidCode{};
        std::int32_t lValidCodeOvertime{60000};
        std::int32_t iValidErrUpperLimit{3};
        std::uint32_t dwValidErrStayTime{180000U};
    };

    [[nodiscard]] bool LoadSetup();
    // Direct EXE used global gAuthMgr. The Linux reconstruction keeps the same
    // timeout side effect but passes the already-explicit AuthManager owner.
    [[nodiscard]] bool ReLoadSetup(AuthManager& authManager);
    [[nodiscard]] bool LoadSetupEx();
    [[nodiscard]] bool ReLoadSetupEx();
    [[nodiscard]] bool LoadWorldSetup();
    void SetListWorldInfoBySetup();
    void UpdateWorldInfoToAllClient();
    [[nodiscard]] bool ReLoadWorldSetup();

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
    void ChangeAllWorldSate();
    void RecordTechnicalError(std::string detail);

    CLoginQueue* m_pLoginQueue{};
    LoginNet::CMyNetServerClient* s_pNetServer_Client{};
    LoginNet::CMyNetServerWorld* s_pNetServer_World{};

    std::map<std::string, std::string> m_LoginCdkeyWorld;
    std::map<std::int32_t, std::list<std::string>> s_listCdkey;
    std::array<std::int32_t, 4> m_StateLvl{-1, 10000, 20000, 30000};
    std::map<std::int32_t, tagWorldInfo> m_listWorldInfo;
    std::map<std::int32_t, tagWorldInfo> m_WorldInfoSetup;
    tagSetup m_Setup;
    tagSetupEx m_SetupEx;
    AccLogQueue _acc_logs;

    mutable std::mutex m_TechnicalErrorMutex;
    std::deque<std::string> m_TechnicalErrors;
};
}
