#include "authmanager.h"

#include <algorithm>
#include <chrono>
#include <bit>
#include <utility>

#if defined(__linux__)
#include <time.h>
#endif

namespace Login
{
namespace
{
constexpr std::int32_t kAuthQuestMessageType = 0x000CF501;
constexpr std::int32_t kAuthResponseMessageType = 0x000CF601;
constexpr std::int32_t kAuthTimeoutResult = 4;

void AddLegacyString(CBaseMessage& message, std::span<const std::uint8_t> value)
{
    const auto terminator = std::find(value.begin(), value.end(), std::uint8_t{0});
    const std::size_t length =
        static_cast<std::size_t>(std::distance(value.begin(), terminator));
    if (length != 0U) {
        message.Add(value.data(), static_cast<std::int32_t>(length));
    }
    message.Add(std::uint8_t{0});
}
}

AuthQuest::AuthQuest(std::uint32_t clientIp,
                     std::int32_t clientSocketId,
                     std::vector<std::uint8_t> account,
                     std::vector<std::uint8_t> password,
                     std::uint32_t startTimeMs)
    : m_ClientIP(clientIp),
      m_ClientSocketID(clientSocketId),
      m_Account(std::move(account)),
      m_Password(std::move(password)),
      m_StartTimeMs(startTimeMs)
{
}

std::uint32_t AuthQuest::ClientIP() const noexcept { return m_ClientIP; }
std::int32_t AuthQuest::ClientSocketID() const noexcept { return m_ClientSocketID; }
std::span<const std::uint8_t> AuthQuest::Account() const noexcept { return m_Account; }
std::span<const std::uint8_t> AuthQuest::Password() const noexcept { return m_Password; }
std::uint32_t AuthQuest::StartTimeMs() const noexcept { return m_StartTimeMs; }

AuthManager::AuthManager() = default;

bool AuthManager::Init(std::uint32_t timeoutMs) noexcept
{
    m_TimeoutMs = timeoutMs;
    return true;
}

void AuthManager::SetTimeout(std::uint32_t timeoutMs) noexcept
{
    m_TimeoutMs = timeoutMs;
}

AddQuestResult AuthManager::AddQuest(AuthQuest quest,
                                     ClientSendQueue& sender,
                                     IAuthListener& listener)
{
    const auto duplicate = std::find_if(
        m_Pending.begin(), m_Pending.end(), [&](const AuthQuest& pending) {
            return std::equal(pending.Account().begin(), pending.Account().end(),
                              quest.Account().begin(), quest.Account().end());
        });
    if (duplicate != m_Pending.end()) {
        return {AddQuestStatus::Duplicate, std::int32_t{0}};
    }

    m_Pending.push_back(std::move(quest));
    AuthQuest& stored = m_Pending.back();
    auto sendResult = SendQuestMessage(stored, sender);
    listener.OnQuest(stored);
    return {AddQuestStatus::Added, std::move(sendResult)};
}

AddQuestResult AuthManager::AddQuest(std::uint32_t clientIp,
                                     std::int32_t clientSocketId,
                                     std::span<const std::uint8_t> account,
                                     std::span<const std::uint8_t> password,
                                     ClientSendQueue& sender,
                                     IAuthListener& listener)
{
    return AddQuest(
        AuthQuest(clientIp,
                  clientSocketId,
                  std::vector<std::uint8_t>(account.begin(), account.end()),
                  std::vector<std::uint8_t>(password.begin(), password.end()),
                  LegacyTickMs()),
        sender,
        listener);
}

void AuthManager::RemoveQuest(std::span<const std::uint8_t> account)
{
    const auto found = std::find_if(
        m_Pending.begin(), m_Pending.end(), [&](const AuthQuest& pending) {
            return std::equal(pending.Account().begin(), pending.Account().end(),
                              account.begin(), account.end());
        });
    if (found != m_Pending.end()) {
        m_Pending.erase(found);
    }
}

AuthRunResult AuthManager::Run(const LoginNet::AuthClientEventPublisher& publisher) const
{
    AuthRunResult result;
    if (!publisher) {
        return result;
    }

    const std::uint32_t now = LegacyTickMs();
    for (const AuthQuest& quest : m_Pending) {
        if (m_TimeoutMs < now - quest.StartTimeMs()) {
            auto message = std::make_unique<LoginNet::CMessage>(kAuthResponseMessageType);
            message->Base().Add(kAuthTimeoutResult);
            AddLegacyString(message->Base(), quest.Account());
            message->Base().Add(quest.ClientIP());
            message->Base().Add(quest.ClientSocketID());
            publisher.PublishMessage(std::move(message));
            ++result.publishedTimeouts;
        }
    }
    return result;
}

AuthResponseResult AuthManager::OnResponseAuth(LoginNet::CMessage& message,
                                               IAuthListener& listener)
{
    AuthResult response{
        .result = message.Base().GetLong(),
        .account = message.GetString(),
        .clientIp = std::bit_cast<std::uint32_t>(message.Base().GetLong()),
        .clientSocketId = message.Base().GetLong(),
    };

    const auto found = std::find_if(
        m_Pending.begin(), m_Pending.end(), [&](const AuthQuest& pending) {
            return std::equal(pending.Account().begin(), pending.Account().end(),
                              response.account.begin(), response.account.end());
        });
    if (found == m_Pending.end()) {
        return {AuthResponseStatus::InvalidResponse, std::move(response)};
    }

    RemoveQuest(response.account);
    listener.OnResponse(response);
    return {AuthResponseStatus::Response, std::move(response)};
}

std::size_t AuthManager::PendingCount() const noexcept
{
    return m_Pending.size();
}

std::uint32_t AuthManager::LegacyTickMs() noexcept
{
#if defined(__linux__) && defined(CLOCK_BOOTTIME)
    timespec now{};
    if (::clock_gettime(CLOCK_BOOTTIME, &now) == 0) {
        const std::uint64_t milliseconds =
            static_cast<std::uint64_t>(now.tv_sec) * 1000ULL +
            static_cast<std::uint64_t>(now.tv_nsec) / 1'000'000ULL;
        return static_cast<std::uint32_t>(milliseconds);
    }
#endif
    const auto now = std::chrono::steady_clock::now().time_since_epoch();
    return static_cast<std::uint32_t>(
        std::chrono::duration_cast<std::chrono::milliseconds>(now).count());
}

std::variant<std::int32_t, LoginNet::SendMessageError>
AuthManager::SendQuestMessage(const AuthQuest& quest, ClientSendQueue& sender) const
{
    LoginNet::CMessage message(kAuthQuestMessageType);
    AddLegacyString(message.Base(), quest.Account());
    AddLegacyString(message.Base(), quest.Password());
    message.Base().Add(quest.ClientIP());
    message.Base().Add(quest.ClientSocketID());
    return message.SendToAuth(sender);
}
}
