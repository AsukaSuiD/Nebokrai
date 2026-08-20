#pragma once

#include "../msgqueue.h"
#include "../serverclient.h"
#include "message.h"

#include <cstddef>
#include <cstdint>
#include <optional>

/*
 * Исходный владелец: nets/netauth/mynetserverclient_auth.cpp
 *
 * AuthServer EXE/PDB, подтверждённые RVA: конструктор 0x00015A10,
 * OnAccept 0x00015AD0,
 * OnClose 0x00015B60, OnReceive 0x00015C10. Производный receive accumulator
 * начинал с 0xA00000. OnAccept публикует 0xCF401, OnClose сначала 0xCF402 и
 * только затем выставляет общий close-state.
 *
 * OnReceive разбирает [len, crc(len), crc(normalized message), message]. CRC
 * длины проверяется до ожидания полного frame; content CRC считается уже по
 * CMessage после нормализации первого header-word. При CRC mismatch исходный
 * accumulator очищается. Unsafe signed/unsigned длины <28 и с sign bit остаются
 * отдельной неизвестностью, а не объявляются новым fail-closed поведением.
 */

enum class AuthReceiveStatus
{
    Ok,
    LengthChecksumMismatch,
    MessageChecksumMismatch,
    EnvelopeLengthReactionUnknown,
    CreateMessageFailed,
};

struct AuthReceiveResult
{
    AuthReceiveStatus status{AuthReceiveStatus::Ok};
    std::int32_t messages{};
    std::size_t pendingBytes{};
    std::uint32_t problematicLength{};
    std::optional<AuthCreateMessageError> createError;

    [[nodiscard]] bool Succeeded() const noexcept
    {
        return status == AuthReceiveStatus::Ok;
    }
};

class CMyNetServerClientAuth final : public CServerClient
{
public:
    CMyNetServerClientAuth(std::int32_t socketId,
                           std::uint32_t peerIPv4,
                           std::uint32_t nowMs);

    void OnAccept(CMsgQueue<CMessage>& messages);
    void OnClose(CMsgQueue<CMessage>& messages);
    [[nodiscard]] AuthReceiveResult OnReceive(std::uint32_t recvTimeMs,
                                              CMsgQueue<CMessage>& messages);
};
