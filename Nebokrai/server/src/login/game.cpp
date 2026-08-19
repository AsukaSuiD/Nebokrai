#include "game.h"

#include "loginqueue.h"
#include "../dbaccess/myadobase.h"
#include "../public/tools.h"
#include "../nets/netlogin/message.h"
#include "../nets/netlogin/mynetserver_client.h"
#include "../nets/netlogin/mynetserver_world.h"

#include <iconv.h>
#include <sql.h>
#include <sqlext.h>

#include <algorithm>
#include <array>
#include <bit>
#include <cerrno>
#include <cmath>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <limits>
#include <new>
#include <sstream>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace Login
{
namespace
{
constexpr std::int32_t kLoginResponseMessageType = 0x000AF501;
constexpr std::int32_t kPlayerBaseMessageType = 0x0004FB01;
constexpr std::int32_t kKickWorldAccountMessageType = 0x0004FB07;
constexpr std::size_t kLegacyAccountLogBufferSize = 0x200U;

std::span<const std::uint8_t> CStringBytes(const char* value)
{
    if (value == nullptr) {
        return {};
    }
    return {reinterpret_cast<const std::uint8_t*>(value), std::strlen(value)};
}

std::vector<std::uint8_t> OwnedCStringBytes(const char* value)
{
    const auto bytes = CStringBytes(value);
    return {bytes.begin(), bytes.end()};
}

std::string LegacyIpv4Text(std::uint32_t address)
{
    return std::to_string(address & 0xFFU) + '.' +
           std::to_string((address >> 8U) & 0xFFU) + '.' +
           std::to_string((address >> 16U) & 0xFFU) + '.' +
           std::to_string((address >> 24U) & 0xFFU);
}

struct ConvertedText
{
    std::optional<std::string> value;
    std::string error;
};

ConvertedText Windows1251ToUtf8(std::string_view source)
{
    if (source.empty()) {
        return ConvertedText{.value = std::string{}};
    }

    iconv_t converter = iconv_open("UTF-8", "WINDOWS-1251");
    if (converter == reinterpret_cast<iconv_t>(-1)) {
        return ConvertedText{.error = "iconv WINDOWS-1251 -> UTF-8 unavailable"};
    }
    struct Closer
    {
        iconv_t converter;
        ~Closer() { iconv_close(converter); }
    } closer{converter};

    std::string output(source.size() * 4U + 4U, '\0');
    char* input = const_cast<char*>(source.data());
    std::size_t inputLeft = source.size();
    char* destination = output.data();
    std::size_t destinationLeft = output.size();
    errno = 0;
    if (iconv(converter,
              &input,
              &inputLeft,
              &destination,
              &destinationLeft) == static_cast<std::size_t>(-1)) {
        return ConvertedText{
            .error = "iconv WINDOWS-1251 -> UTF-8 failed, errno=" +
                     std::to_string(errno)};
    }
    output.resize(output.size() - destinationLeft);
    return ConvertedText{.value = std::move(output)};
}

bool OdbcSucceeded(SQLRETURN result) noexcept
{
    return result == SQL_SUCCESS || result == SQL_SUCCESS_WITH_INFO;
}

std::string OdbcDiagnostic(SQLSMALLINT handleType,
                           SQLHANDLE handle,
                           std::string_view operation)
{
    SQLCHAR state[6]{};
    SQLINTEGER native = 0;
    SQLCHAR message[1024]{};
    SQLSMALLINT length = 0;
    const SQLRETURN result = SQLGetDiagRec(handleType,
                                            handle,
                                            1,
                                            state,
                                            &native,
                                            message,
                                            static_cast<SQLSMALLINT>(sizeof(message)),
                                            &length);
    std::string detail(operation);
    if (OdbcSucceeded(result)) {
        detail += ": SQLSTATE=";
        detail += reinterpret_cast<const char*>(state);
        detail += ", native=" + std::to_string(native);
        if (length > 0) {
            detail += ", ";
            const std::size_t copied = std::min<std::size_t>(
                static_cast<std::size_t>(length), sizeof(message) - 1U);
            detail.append(reinterpret_cast<const char*>(message), copied);
        }
    } else {
        detail += ": ODBC diagnostic unavailable";
    }
    return detail;
}

template <typename Value>
void ReadLabeled(std::istream& input, std::string& label, Value& value)
{
    input >> label >> value;
}

void ReadLegacySetupBody(std::istream& input, CGame::tagSetup& setup)
{
    std::string label;
    // VERIFIED_DECOMPILE 0x0040E690 setup.dat branch: legacy schema starts
    // directly with Client/World ports and ends at _bindPort.
    ReadLabeled(input, label, setup.dwListenPort_Client);
    ReadLabeled(input, label, setup.dwListenPort_World);
    ReadLabeled(input, label, setup.strSqlConType);
    ReadLabeled(input, label, setup.strSqlServerIP);
    ReadLabeled(input, label, setup.strSqlUserName);
    ReadLabeled(input, label, setup.strSqlPassWord);
    ReadLabeled(input, label, setup.strDBName);
    ReadLabeled(input, label, setup.bCheckNet);
    ReadLabeled(input, label, setup.dwMaxByteNum);
    ReadLabeled(input, label, setup.dwMaxMsgLen);
    ReadLabeled(input, label, setup.dwBanIPTime);
    ReadLabeled(input, label, setup.bCheckMsgCon);
    ReadLabeled(input, label, setup.lMaxConnectNum);
    ReadLabeled(input, label, setup.lMaxIOSendNum);
    ReadLabeled(input, label, setup.lMaxClientSendBuf);
    ReadLabeled(input, label, setup.bWorldCheckNet);
    ReadLabeled(input, label, setup.dwWorldMaxByteNum);
    ReadLabeled(input, label, setup.dwWorldMaxMsgLen);
    ReadLabeled(input, label, setup.dwWorldBanIPTime);
    ReadLabeled(input, label, setup.bWorldCheckMsgCon);
    ReadLabeled(input, label, setup.lWorldMaxConnectNum);
    ReadLabeled(input, label, setup.lWorldMaxIOSendNum);
    ReadLabeled(input, label, setup.lWorldMaxClientSendBuf);
    ReadLabeled(input, label, setup.dwRefeashInfoTime);
    ReadLabeled(input, label, setup.dwSaveInfoTime);
    ReadLabeled(input, label, setup.dwDoQueueInter);
    ReadLabeled(input, label, setup.dwSendMsgToQueInter);
    ReadLabeled(input, label, setup.dwWorldMaxPlayers);
    ReadLabeled(input, label, setup.fWorldBusyScale);
    ReadLabeled(input, label, setup.fWorldFullScale);
    ReadLabeled(input, label, setup.dwPingWorldServerTime);
    ReadLabeled(input, label, setup.dwPingWorldServerErrorTime);
    ReadLabeled(input, label, setup.bCheckForbidIP);
    ReadLabeled(input, label, setup.bCheckAllowIP);
    ReadLabeled(input, label, setup.bCheckBetweenIP);
    ReadLabeled(input, label, setup.dwServerInfoLogTime);
    ReadLabeled(input, label, setup.strServerInfoLogProvider);
    ReadLabeled(input, label, setup.strServerInfoLogUID);
    ReadLabeled(input, label, setup.strServerInfoLogPWD);
    ReadLabeled(input, label, setup.strServerInfoLogIP);
    ReadLabeled(input, label, setup.strServerInfoLogDB);
    ReadLabeled(input, label, setup.authTimeOut);
    ReadLabeled(input, label, setup._bindIP);
    ReadLabeled(input, label, setup._bindPort);
}

void ReadPlainSetup(std::istream& input, CGame::tagSetup& setup)
{
    std::string label;
    // Plain setup.ini has one newer prefix field, then the exact legacy body.
    ReadLabeled(input, label, setup._server_version);
    ReadLegacySetupBody(input, setup);

    // VERIFIED_DECOMPILE plaintext-only extension after _bindPort.
    ReadLabeled(input, label, setup.lforbitTime);
    ReadLabeled(input, label, setup.lforbitNum);
    ReadLabeled(input, label, setup.lMode);
    ReadLabeled(input, label, setup.strGasAreaUserID);
    ReadLabeled(input, label, setup.strGasAreaPasswd);
    ReadLabeled(input, label, setup._db_ip);
    ReadLabeled(input, label, setup._db_billing_name);
    ReadLabeled(input, label, setup._db_user);
    ReadLabeled(input, label, setup._db_psd);
    ReadLabeled(input, label, setup.m_lIsInsideUse);
    ReadLabeled(input, label, setup.m_strVerificationAddr);
    ReadLabeled(input, label, setup.m_lVerifiSignUpper);
}

std::optional<std::int32_t> LegacyScaledWorldLevel(std::uint32_t worldMax,
                                                    float scale)
{
    // VERIFIED_ASSEMBLY 0x0040EDA2 / 0x0040F402: MSVC converts unsigned dword
    // through signed FILD and adds exact 2^32 when the high bit is set.
    const long double product = static_cast<long double>(worldMax) *
                                static_cast<long double>(scale);
    if (!std::isfinite(product) ||
        product < static_cast<long double>(std::numeric_limits<std::int32_t>::min()) ||
        product > static_cast<long double>(std::numeric_limits<std::int32_t>::max())) {
        return std::nullopt;
    }
    return static_cast<std::int32_t>(product);
}
}

CGame::tagSetup::tagSetup()
{
    // VERIFIED_DIRECT tagSetup::tagSetup 0x0040C6E0. Fundamental fields not
    // mentioned here were not explicitly initialized by the original ctor.
    bCheckNet = true;
    dwMaxByteNum = 5000U;
    dwMaxMsgLen = 0x19000U;
    dwBanIPTime = 10U;
    bCheckMsgCon = true;
    dwRefeashInfoTime = 1000U;
    dwSaveInfoTime = 60000U;
    authTimeOut = 5000U;
    _bindIP = "0.0.0.0";
    _bindPort = 0U;
    lMode = 0;
    strGasAreaUserID.clear();
    strGasAreaPasswd.clear();
    _db_ip.clear();
    _db_billing_name.clear();
    _db_user.clear();
    _db_psd.clear();
    m_lIsInsideUse = 1;
}

bool CGame::LoadSetup()
{
    // VERIFIED_DIRECT 0x0040E690: plaintext setup.ini has priority. setup.dat
    // is consulted only when setup.ini cannot be opened.
    std::ifstream plain("setup.ini");
    if (plain.is_open()) {
        ReadPlainSetup(plain, m_Setup);
        if (plain.fail()) {
            // Direct code would continue with whatever remained in raw scalar
            // storage. Linux makes that invalid-input UB an explicit technical
            // boundary before those values feed world thresholds/timers.
            RecordTechnicalError("LoadSetup: incomplete setup.ini stream");
            return false;
        }
    } else {
        char setupDatName[] = "setup.dat";
        const int fileLength = GetFileLength(setupDatName);
        std::FILE* encodedFile = std::fopen(setupDatName, "rb");
        if (encodedFile == nullptr) {
            return false;
        }

        if (fileLength < 0 ||
            static_cast<std::size_t>(fileLength) >
                (std::numeric_limits<std::size_t>::max() - 2U) / 2U) {
            std::fclose(encodedFile);
            RecordTechnicalError("LoadSetup: setup.dat length is not representable");
            return false;
        }

        try {
            // VERIFIED_ASSEMBLY: encoded buffer = len+1, decoded = len*2+2;
            // both are zero-filled before fread/IniDecoder.
            std::vector<char> encoded(static_cast<std::size_t>(fileLength) + 1U, '\0');
            std::vector<char> decoded(static_cast<std::size_t>(fileLength) * 2U + 2U, '\0');

            // Direct owner ignores fread result. Because the source buffer was
            // pre-zeroed, a short read leaves zero bytes in the unread tail.
            static_cast<void>(std::fread(encoded.data(),
                                         static_cast<std::size_t>(fileLength),
                                         1U,
                                         encodedFile));
            std::fclose(encodedFile);
            encodedFile = nullptr;

            IniDecoder(encoded.data(), decoded.data(), fileLength);

            // Direct code inserts decoded C-string into stringstream then seekg(0),
            // so an embedded NUL truncates the parse even though decoded is 2*len+2.
            std::stringstream decodedStream;
            decodedStream << decoded.data();
            decodedStream.seekg(0, std::ios::beg);
            ReadLegacySetupBody(decodedStream, m_Setup);
            if (decodedStream.fail()) {
                RecordTechnicalError("LoadSetup: incomplete decoded setup.dat stream");
                return false;
            }
        } catch (const std::bad_alloc&) {
            if (encodedFile != nullptr) {
                std::fclose(encodedFile);
            }
            RecordTechnicalError("LoadSetup: setup.dat buffer allocation failed");
            return false;
        }
    }

    const auto busyLevel =
        LegacyScaledWorldLevel(m_Setup.dwWorldMaxPlayers, m_Setup.fWorldBusyScale);
    const auto fullLevel =
        LegacyScaledWorldLevel(m_Setup.dwWorldMaxPlayers, m_Setup.fWorldFullScale);
    if (!busyLevel || !fullLevel) {
        RecordTechnicalError("LoadSetup: world state threshold is outside legacy long range");
        return false;
    }

    // VERIFIED x87 post-parse sequence in both setup.ini and setup.dat branches.
    m_StateLvl[0] = -1;
    m_StateLvl[1] = *busyLevel;
    m_StateLvl[2] = *fullLevel;
    m_StateLvl[3] = std::bit_cast<std::int32_t>(m_Setup.dwWorldMaxPlayers);
    ChangeAllWorldSate();

    if (m_pLoginQueue != nullptr) {
        m_pLoginQueue->OnInitial(m_Setup.dwDoQueueInter,
                                 m_Setup.dwSendMsgToQueInter,
                                 m_Setup.dwWorldMaxPlayers);
    }

    // Direct owner only appends success AddLogText after this point.
    return true;
}

bool CGame::LoadSetupEx()
{
    // VERIFIED_DECOMPILE 0x0040D9E0: literal lowercase filename, обычный
    // whitespace stream; каждая подпись читается и отбрасывается перед value.
    // После успешного open исходник НЕ проверяет failbit и всегда возвращает
    // true, поэтому partial extraction намеренно оставляет уже прочитанные поля.
    std::ifstream input("setupex.ini");
    if (!input.is_open()) {
        return false;
    }

    std::string label;
    input >> label >> m_SetupEx.iAreaId
          >> label >> m_SetupEx.lClientMaxBlockConNum
          >> label >> m_SetupEx.lClientValidDelayRecDataTime
          >> label >> m_SetupEx.lWorldMaxBlockConNum
          >> label >> m_SetupEx.lWorldValidDelayRecDataTime
          >> label >> m_SetupEx.lQuestPlayerDataInterval
          >> label >> m_SetupEx.matrix_timeout
          >> label >> m_SetupEx.bValidCode
          >> label >> m_SetupEx.lValidCodeOvertime
          >> label >> m_SetupEx.iValidErrUpperLimit
          >> label >> m_SetupEx.dwValidErrStayTime;

    // Direct owner затем только AddLogText("load setupex.ini...ok!"). Общий
    // logger ещё не восстановлен; success-log не является state-machine side effect.
    return true;
}

bool CGame::ReLoadSetupEx()
{
    // VERIFIED_DECOMPILE 0x0040DC60.
    if (!LoadSetupEx()) {
        return false;
    }

    if (s_pNetServer_Client == nullptr || s_pNetServer_World == nullptr) {
        // EXE здесь напрямую разыменовывает оба owner-а. В Linux не повторяем
        // null-deref; это typed technical boundary, а не новый login result.
        RecordTechnicalError("ReLoadSetupEx: network server owner is missing");
        return false;
    }

    // Direct CServer +0x160/+0x164 = m_lMaxBlockConnetNum/m_lSendInterTime.
    s_pNetServer_Client->ConfigureAcceptLimitsAfterHost(
        m_SetupEx.lClientMaxBlockConNum,
        m_SetupEx.lClientValidDelayRecDataTime);
    s_pNetServer_World->ConfigureAcceptLimitsAfterHost(
        m_SetupEx.lWorldMaxBlockConNum,
        m_SetupEx.lWorldValidDelayRecDataTime);
    return true;
}

void CGame::ChangeAllWorldSate()
{
    // VERIFIED_ASSEMBLY 0x004075E0. Only worlds that already have an
    // s_listCdkey entry are recalculated; absent entries keep their old state.
    for (auto& [worldId, world] : m_listWorldInfo) {
        const auto accounts = s_listCdkey.find(worldId);
        if (accounts == s_listCdkey.end()) {
            continue;
        }

        const std::uint32_t rawCount =
            static_cast<std::uint32_t>(accounts->second.size());
        const std::int32_t signedCount = std::bit_cast<std::int32_t>(rawCount);

        std::int32_t state = 1;
        while (state <= 3 && signedCount >= m_StateLvl[static_cast<std::size_t>(state)]) {
            ++state;
        }
        if (state > 3) {
            state = 3;
        }
        world.lStateLvl = state;
    }
}

std::int32_t CGame::GetWorldIDByName(const char* worldName) const
{
    if (worldName == nullptr) {
        return -1;
    }
    for (const auto& [worldId, world] : m_listWorldInfo) {
        if (std::strcmp(worldName, world.strName.c_str()) == 0) {
            return world.lStateLvl == 0 ? -1 : worldId;
        }
    }
    return -1;
}

bool CGame::WorldServerIsOpenState(std::int32_t worldId) const
{
    const auto found = m_WorldInfoSetup.find(worldId);
    return found != m_WorldInfoSetup.end() && found->second.lStateLvl != 0;
}

void CGame::AddWorldInfoToMsg(LoginNet::CMessage& message, const char* account) const
{
    // VERIFIED_ASSEMBLY 0x004074E0: map size идёт short, затем для каждого
    // live world — long state и C-string name в map-key order.
    const auto lowCount = static_cast<std::uint16_t>(m_listWorldInfo.size());
    message.Base().Add(std::bit_cast<std::int16_t>(lowCount));

    for (const auto& [worldId, world] : m_listWorldInfo) {
        const auto setup = m_WorldInfoSetup.find(worldId);
        bool visible =
            setup != m_WorldInfoSetup.end() && setup->second.lStateLvl != 0;
        if (!visible && m_pLoginQueue != nullptr) {
            visible = m_pLoginQueue->IsInNoQueueList(CStringBytes(account));
        }
        message.Base().Add(visible ? world.lStateLvl : 0);
        message.Base().Add(world.strName.c_str());
    }
}

std::int32_t CGame::FindCdkey(const char* account) const
{
    if (account == nullptr) {
        return -1;
    }
    for (const auto& [worldId, accounts] : s_listCdkey) {
        if (std::find(accounts.begin(), accounts.end(), account) != accounts.end()) {
            return worldId;
        }
    }
    return -1;
}

bool CGame::L2W_PlayerBase_Send(const char* worldName, const char* account) const
{
    if (worldName == nullptr || account == nullptr || account[0] == '\0') {
        return false;
    }

    const std::int32_t worldId = GetWorldIDByName(worldName);
    if (worldId == -1) {
        return false;
    }

    if (!WorldServerIsOpenState(worldId)) {
        const bool noQueue =
            m_pLoginQueue != nullptr &&
            m_pLoginQueue->IsInNoQueueList(CStringBytes(account));
        if (!noQueue) {
            return false;
        }
    }

    LoginNet::CMessage message(kPlayerBaseMessageType);
    message.Base().Add(account);
    if (s_pNetServer_World != nullptr) {
        static_cast<void>(
            message.SendToWorldMap(s_pNetServer_World->CommandHandle(), worldId));
    }
    // Direct EXE возвращает true после логических проверок и не анализирует send.
    return true;
}

const char* CGame::GetLoginCdkeyWorldServer(const char* account) const
{
    if (account == nullptr) {
        return nullptr;
    }
    const auto found = m_LoginCdkeyWorld.find(account);
    return found == m_LoginCdkeyWorld.end() ? nullptr : found->second.c_str();
}

void CGame::SetLoginCdkeyWorldServer(const char* account, const char* worldServer)
{
    if (account == nullptr || worldServer == nullptr) {
        return;
    }
    m_LoginCdkeyWorld[account] = worldServer;
}

bool CGame::KickOut(const char* account) const
{
    if (account == nullptr) {
        return false;
    }

    if (GetLoginCdkeyWorldServer(account) != nullptr) {
        if (s_pNetServer_Client != nullptr) {
            static_cast<void>(
                s_pNetServer_Client->QuitClientByMapName(CStringBytes(account)));
        }
        return true;
    }

    const std::int32_t worldId = FindCdkey(account);
    if (worldId == -1) {
        return false;
    }

    LoginNet::CMessage message(kKickWorldAccountMessageType);
    message.Base().Add(account);
    if (s_pNetServer_World != nullptr) {
        static_cast<void>(
            message.SendToWorldMap(s_pNetServer_World->CommandHandle(), worldId));
    }
    return true;
}

void CGame::AccountEnterLog(const char* account, std::uint32_t ip)
{
    if (account == nullptr) {
        return;
    }

    std::array<char, 64> time{};
    static_cast<void>(CMyAdoBase::GetTimeString(time.data(), time.size()));
    if (time[0] == '\0') {
        RecordTechnicalError("AccountEnterLog: localtime conversion failed");
        return;
    }

    const std::string ipText = LegacyIpv4Text(ip);
    std::string sql = "INSERT INTO LogInfo(Account,AccountEnterTime,IP) VALUES('";
    sql += account;
    sql += "','";
    sql += time.data();
    sql += "','";
    sql += ipText;
    sql += "')";

    // Direct owner uses char[512] + sprintf. Reached login accounts are <=31
    // bytes, therefore normal behavior is byte-identical. Oversize external
    // input is an explicit technical boundary instead of reproducing stack OOB.
    if (sql.size() >= kLegacyAccountLogBufferSize) {
        RecordTechnicalError("AccountEnterLog: SQL exceeds legacy char[512]");
        return;
    }
    _acc_logs.Push(std::move(sql));
}

std::int32_t CGame::PrepareEnter(const char* account,
                                 std::uint32_t ip,
                                 std::int32_t socketId,
                                 const char* worldServer,
                                 bool matrix)
{
    if (account == nullptr || ip == 0U || socketId == 0 || worldServer == nullptr) {
        return 1;
    }

    if (worldServer[0] == '\0' && KickOut(account)) {
        // VERIFIED_ASSEMBLY 0x0041198C..0x004119B4: unlike old Linux donor,
        // original sends 0x08 to the NEW socket and returns immediately.
        LoginNet::CMessage response(kLoginResponseMessageType);
        response.Base().Add(static_cast<char>(0x08));
        if (s_pNetServer_Client != nullptr) {
            static_cast<void>(response.SendToClientSocket(
                s_pNetServer_Client->CommandHandle(), socketId));
        }
        return 1;
    }

    if (s_pNetServer_Client != nullptr) {
        static_cast<void>(
            s_pNetServer_Client->SetClientMapName(socketId, CStringBytes(account)));
    } else {
        RecordTechnicalError("PrepareEnter: client server owner is missing");
    }

    AccountEnterLog(account, ip);
    SetLoginCdkeyWorldServer(account, worldServer);

    if (worldServer[0] != '\0' || !matrix) {
        return 0;
    }

    if (m_pLoginQueue == nullptr) {
        RecordTechnicalError("PrepareEnter: login queue owner is missing for matrix_register");
        return 1;
    }

    TagPwdChecked checked(socketId,
                          ip,
                          OwnedCStringBytes(account),
                          OwnedCStringBytes(worldServer),
                          true);
    if (auto error = m_pLoginQueue->MatrixRegister(
            checked,
            [this](const LoginNet::CMessage& message, std::int32_t targetSocket) {
                if (s_pNetServer_Client != nullptr) {
                    static_cast<void>(message.SendToClientSocket(
                        s_pNetServer_Client->CommandHandle(), targetSocket));
                }
            })) {
        RecordTechnicalError("PrepareEnter/matrix_register: " + error->detail);
    }
    return 1;
}

void CGame::EnterGame(const char* account,
                      std::uint32_t ip,
                      std::int32_t socketId,
                      const char* worldServer) const
{
    if (account == nullptr || ip == 0U || socketId == 0 || worldServer == nullptr) {
        return;
    }

    if (worldServer[0] != '\0') {
        static_cast<void>(L2W_PlayerBase_Send(worldServer, account));
        return;
    }

    LoginNet::CMessage response(kLoginResponseMessageType);
    response.Base().Add(static_cast<char>(0x02));
    response.Base().Add(account);
    AddWorldInfoToMsg(response, account);
    if (s_pNetServer_Client != nullptr) {
        static_cast<void>(response.SendToClientSocket(
            s_pNetServer_Client->CommandHandle(), socketId));
    }
}

bool CGame::ExecuteProce(std::string userId,
                         std::string userIp,
                         char* passwordHex,
                         std::int32_t unusedResult)
{
    // VERIFIED_ASSEMBLY 0x00407AB0..0x0040822C: этот аргумент не читается.
    static_cast<void>(unusedResult);
    if (passwordHex == nullptr) {
        RecordTechnicalError("ExecuteProce: UserPwd pointer is null");
        return false;
    }

    const ConvertedText host = Windows1251ToUtf8(m_Setup._db_ip);
    const ConvertedText database = Windows1251ToUtf8(m_Setup._db_billing_name);
    const ConvertedText dbUser = Windows1251ToUtf8(m_Setup._db_user);
    const ConvertedText dbPassword = Windows1251ToUtf8(m_Setup._db_psd);
    const ConvertedText convertedUser = Windows1251ToUtf8(userId);
    const ConvertedText convertedIp = Windows1251ToUtf8(userIp);
    const ConvertedText convertedPassword = Windows1251ToUtf8(passwordHex);

    const ConvertedText* conversions[] = {
        &host, &database, &dbUser, &dbPassword,
        &convertedUser, &convertedIp, &convertedPassword,
    };
    for (const ConvertedText* conversion : conversions) {
        if (!conversion->value) {
            RecordTechnicalError("ExecuteProce: " + conversion->error);
            return false;
        }
    }

    // ADO/COM is Windows-only plumbing. The existing Linux server stack uses
    // unixODBC with Microsoft ODBC Driver 18; game semantics/procedure contract
    // remain in this original CGame owner.
    std::string connection = "DRIVER={ODBC Driver 18 for SQL Server};SERVER=";
    connection += *host.value;
    connection += ";DATABASE=";
    connection += *database.value;
    connection += ";UID=";
    connection += *dbUser.value;
    connection += ";PWD=";
    connection += *dbPassword.value;
    connection += ";Encrypt=no;TrustServerCertificate=yes;";

    struct Environment
    {
        SQLHENV handle{SQL_NULL_HENV};
        ~Environment()
        {
            if (handle != SQL_NULL_HENV) {
                SQLFreeHandle(SQL_HANDLE_ENV, handle);
            }
        }
    } environment;
    struct Connection
    {
        SQLHDBC handle{SQL_NULL_HDBC};
        ~Connection()
        {
            if (handle != SQL_NULL_HDBC) {
                SQLDisconnect(handle);
                SQLFreeHandle(SQL_HANDLE_DBC, handle);
            }
        }
    } db;
    struct Statement
    {
        SQLHSTMT handle{SQL_NULL_HSTMT};
        ~Statement()
        {
            if (handle != SQL_NULL_HSTMT) {
                SQLFreeHandle(SQL_HANDLE_STMT, handle);
            }
        }
    } statement;

    SQLRETURN result =
        SQLAllocHandle(SQL_HANDLE_ENV, SQL_NULL_HANDLE, &environment.handle);
    if (!OdbcSucceeded(result)) {
        RecordTechnicalError("ExecuteProce: SQLAllocHandle(ENV) failed");
        return false;
    }
    result = SQLSetEnvAttr(environment.handle,
                           SQL_ATTR_ODBC_VERSION,
                           reinterpret_cast<SQLPOINTER>(SQL_OV_ODBC3),
                           0);
    if (!OdbcSucceeded(result)) {
        RecordTechnicalError(
            "ExecuteProce: " +
            OdbcDiagnostic(SQL_HANDLE_ENV, environment.handle, "SQLSetEnvAttr"));
        return false;
    }
    result = SQLAllocHandle(SQL_HANDLE_DBC, environment.handle, &db.handle);
    if (!OdbcSucceeded(result)) {
        RecordTechnicalError(
            "ExecuteProce: " +
            OdbcDiagnostic(SQL_HANDLE_ENV, environment.handle, "SQLAllocHandle(DBC)"));
        return false;
    }
    result = SQLDriverConnect(
        db.handle,
        nullptr,
        reinterpret_cast<SQLCHAR*>(connection.data()),
        SQL_NTS,
        nullptr,
        0,
        nullptr,
        SQL_DRIVER_NOPROMPT);
    if (!OdbcSucceeded(result)) {
        RecordTechnicalError(
            "ExecuteProce: " +
            OdbcDiagnostic(SQL_HANDLE_DBC, db.handle, "SQLDriverConnect"));
        return false;
    }
    result = SQLAllocHandle(SQL_HANDLE_STMT, db.handle, &statement.handle);
    if (!OdbcSucceeded(result)) {
        RecordTechnicalError(
            "ExecuteProce: " +
            OdbcDiagnostic(SQL_HANDLE_DBC, db.handle, "SQLAllocHandle(STMT)"));
        return false;
    }

    // Original ADODB CommandType=4 (stored procedure), CommandText=getAccInfoEx.
    SQLCHAR procedure[] = "{CALL getAccInfoEx(?,?,?,?)}";
    result = SQLPrepare(statement.handle, procedure, SQL_NTS);
    if (!OdbcSucceeded(result)) {
        RecordTechnicalError(
            "ExecuteProce: " +
            OdbcDiagnostic(SQL_HANDLE_STMT, statement.handle, "SQLPrepare(getAccInfoEx)"));
        return false;
    }

    std::string parameterUser = *convertedUser.value;
    std::string parameterIp = *convertedIp.value;
    std::string parameterPassword = *convertedPassword.value;
    SQLLEN userLength = static_cast<SQLLEN>(parameterUser.size());
    SQLLEN ipLength = static_cast<SQLLEN>(parameterIp.size());
    SQLLEN passwordLength = static_cast<SQLLEN>(parameterPassword.size());
    SQLINTEGER procedureResult = 0;
    SQLLEN resultLength = 0;

    const auto bindText = [&](SQLUSMALLINT number,
                              std::string& value,
                              SQLULEN legacySize,
                              SQLLEN& length) -> bool {
        const SQLRETURN bound = SQLBindParameter(
            statement.handle,
            number,
            SQL_PARAM_INPUT,
            SQL_C_CHAR,
            SQL_VARCHAR,
            legacySize,
            0,
            value.data(),
            static_cast<SQLLEN>(value.size() + 1U),
            &length);
        if (!OdbcSucceeded(bound)) {
            RecordTechnicalError(
                "ExecuteProce: " +
                OdbcDiagnostic(SQL_HANDLE_STMT, statement.handle, "SQLBindParameter"));
            return false;
        }
        return true;
    };

    // VERIFIED_ASSEMBLY: @UserID adVarChar input size 0x20,
    // @UserIP size 0x18, @UserPwd size 0x40, затем @Result adInteger OUTPUT.
    if (!bindText(1, parameterUser, 0x20U, userLength) ||
        !bindText(2, parameterIp, 0x18U, ipLength) ||
        !bindText(3, parameterPassword, 0x40U, passwordLength)) {
        return false;
    }
    result = SQLBindParameter(statement.handle,
                              4,
                              SQL_PARAM_OUTPUT,
                              SQL_C_SLONG,
                              SQL_INTEGER,
                              0,
                              0,
                              &procedureResult,
                              sizeof(procedureResult),
                              &resultLength);
    if (!OdbcSucceeded(result)) {
        RecordTechnicalError(
            "ExecuteProce: " +
            OdbcDiagnostic(SQL_HANDLE_STMT, statement.handle, "SQLBindParameter(@Result)"));
        return false;
    }

    result = SQLExecute(statement.handle);
    if (!OdbcSucceeded(result) && result != SQL_NO_DATA) {
        RecordTechnicalError(
            "ExecuteProce: " +
            OdbcDiagnostic(SQL_HANDLE_STMT, statement.handle, "SQLExecute(getAccInfoEx)"));
        return false;
    }

    // Direct EXE не читает output @Result и на success тоже делает xor al,al.
    static_cast<void>(procedureResult);
    return false;
}

void CGame::RecordTechnicalError(std::string detail)
{
    std::lock_guard guard(m_TechnicalErrorMutex);
    m_TechnicalErrors.push_back(std::move(detail));
}

std::optional<std::string> CGame::PopTechnicalError()
{
    std::lock_guard guard(m_TechnicalErrorMutex);
    if (m_TechnicalErrors.empty()) {
        return std::nullopt;
    }
    std::string detail = std::move(m_TechnicalErrors.front());
    m_TechnicalErrors.pop_front();
    return detail;
}
}
