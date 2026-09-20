#pragma once

#include "../loginqueue.h"
#include "mywininet.h"

#include <cstddef>
#include <cstdint>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <vector>

namespace LoginNet
{
class CMessage;
}

/*
 * Исходный владелец: loginserver/applogin/gasthread.cpp / gasthread.h
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Подтверждённые RVA: конструктор 0x00420EB0, GetNickNameFromStrs 0x00420FA0,
 * AnalysisRet 0x00421080, MD5vec2str 0x004211C0,
 * FormContent 0x004212C0, CheckAcc 0x00421590, Run 0x00421720.
 *
 * Worker берёт ТОЛЬКО CLoginQueue::m_GasQueue/Pop и никогда не пишет в
 * m_GasQuest. После полного drain исходник Sleep(10) и повторяет цикл.
 * Win32 Thread/COM здесь не являются game semantics: Run остаётся тем же
 * blocking worker-loop, а sleep заменён std::this_thread::sleep_for.
 *
 * GAS-протокол остаётся собственным кодом Nebokrai, а не библиотечной моделью:
 * legacy response parser ищет literal "\"nickname\"" вручную и AnalysisRet
 * читает fixed state в response[10..11]. Его нельзя заменять JSON parser-ом.
 * Технические части заменены библиотеками: CMyWinInet использует libcurl,
 * FormContent вычисляет MD5 через OpenSSL EVP. Оба password/sign hex в исходном
 * EXE lowercase; только итоговый sign hash upper-case, если
 * Условие: m_lVerifiSignUpper == 1.
 *
 * Старый MD5vec2str безусловно читал первые 16 bytes vector. Короткий digest,
 * невозможный MD5 и переполнение старых 512/1024 sprintf-буферов остаются
 * typed technical boundary, а не превращаются в придуманный client failure.
 * Короткий непустой HTTP-response, напротив, не является unknown boundary:
 * единственный caller передаёт обнулённый CMyWinInet::m_strData[1024], поэтому
 * отсутствующие response[10]/[11] безопасно моделируются нулевыми байтами.
 */
namespace Login
{
enum class GasThreadErrorKind
{
    PasswordDigestTooShort,
    Md5Unavailable,
    SignSourceTooLong,
    FormContentTooLong,
    TransportCompatibilityBoundary,
    CheckAccInvariant,
    DatabaseOwnerMissing,
    LocalTimeUnavailable,
    VariantTimeConversionFailed,
    GasOperatorUnavailable,
};

struct GasThreadError
{
    GasThreadErrorKind kind{};
    std::vector<std::uint8_t> account;
    std::size_t actualLength{};
    std::string detail;
};

struct GasRunReport
{
    std::size_t processed{};
    std::size_t forwardedToEnter{};
    std::size_t gasRejected{};
    std::size_t authenticated{};
    std::size_t droppedUnknownState{};
    std::vector<GasThreadError> errors;
};

class IGasThreadContext : public ILoginQueueContext
{
public:
    ~IGasThreadContext() override = default;

    [[nodiscard]] virtual std::string VerificationAddress() const = 0;
    [[nodiscard]] virtual std::int32_t VerificationSignUpper() const noexcept = 0;

    virtual void AddWorldInfoToMsg(LoginNet::CMessage& message,
                                   std::span<const std::uint8_t> account) = 0;

    // Сигнатура из PDB: bool CGame::ExecuteProce(std::string, std::string, char*, int).
    [[nodiscard]] virtual bool ExecuteProce(std::string nickname,
                                            std::string clientIp,
                                            char* passwordHex,
                                            std::int32_t mode) = 0;
};

class CGasThread
{
public:
    CGasThread(CLoginQueue& queue, IGasThreadContext& context);

    [[nodiscard]] GasRunReport RunOnce();
    [[noreturn]] void Run();

private:
    struct CheckAccResult
    {
        std::optional<std::int32_t> state;
        std::optional<GasThreadError> error;
    };

    [[nodiscard]] std::int32_t AnalysisRet(std::string_view response,
                                           std::string& nickname) const;
    [[nodiscard]] std::int32_t GetNickNameFromStrs(std::string_view response,
                                                   std::string& nickname) const;
    [[nodiscard]] std::optional<std::string>
    MD5vec2str(std::span<const std::uint8_t> digest,
               GasThreadError& error,
               std::span<const std::uint8_t> account) const;
    [[nodiscard]] std::optional<std::string>
    FormContent(const CLoginQueue::QuestCdkey& quest, GasThreadError& error) const;
    [[nodiscard]] CheckAccResult CheckAcc(CLoginQueue::QuestCdkey& quest);
    [[nodiscard]] std::optional<GasThreadError>
    ProcessQuest(CLoginQueue::QuestCdkey& quest, GasRunReport& report);

    CLoginQueue& m_Queue;
    IGasThreadContext& m_Context;
    CMyWinInet m_myWinInet;
};
}
