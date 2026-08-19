#pragma once

#include <cstddef>
#include <optional>
#include <string>

/*
 * Owner: loginserver/applogin/mywininet.cpp / mywininet.h
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * RVA: ctor 0x00425200, Init 0x00425250, Recv 0x004252D0,
 * Close 0x00425380, dtor 0x00425400, Send 0x00425440.
 *
 * WinInet является только платформенным HTTP transport. Linux-реконструкция
 * заменяет InternetCrackUrl/InternetOpen/InternetConnect/HttpOpenRequest/
 * HttpSendRequest/InternetReadFile на libcurl, сохраняя внешний контракт:
 * Init/Send возвращают 1 при успехе и 0 при ошибке, Recv даёт nullptr для
 * отсутствующего/пустого ответа, Close сбрасывает всё состояние. Запрос остаётся
 * POST, User-Agent "App", Accept "text/*" и Content-Type
 * application/x-www-form-urlencoded.
 *
 * Старый Recv имел единственный char[1024]. Ответы, которые не помещаются в
 * безопасные 1023 bytes + NUL, теперь считаются технической transport-ошибкой,
 * а не продолжают старое overwrite/OOB-поведение. WinInet также делал отдельный
 * retry ERROR_INTERNET_INVALID_CA с SECURITY_FLAG_IGNORE_UNKNOWN_CA. libcurl не
 * имеет backend-independent эквивалента именно этого одного WinInet-флага;
 * ослаблять всю TLS-проверку здесь намеренно нельзя, поэтому такой случай
 * возвращает Send=0 вместе с typed transport-границей; GAS не превращает её в
 * придуманный state 100/0x07.
 */
namespace Login
{
enum class MyWinInetErrorKind
{
    CurlGlobal,
    InvalidUrl,
    UrlTooLong,
    CurlInit,
    CurlOptions,
    Transfer,
    ResponseTooLarge,
    TlsPeerVerificationLegacyRetryBoundary,
};

struct MyWinInetError
{
    MyWinInetErrorKind kind{};
    std::string detail;
};

class CMyWinInet
{
public:
    CMyWinInet() = default;
    ~CMyWinInet();

    CMyWinInet(const CMyWinInet&) = delete;
    CMyWinInet& operator=(const CMyWinInet&) = delete;

    [[nodiscard]] int Init(const char* url);
    [[nodiscard]] int Send(const char* content);
    [[nodiscard]] char* Recv() noexcept;
    void Close() noexcept;

    [[nodiscard]] const std::optional<MyWinInetError>& LastError() const noexcept;

private:
    static std::size_t WriteResponse(char* data,
                                     std::size_t size,
                                     std::size_t count,
                                     void* owner) noexcept;
    void SetError(MyWinInetErrorKind kind, std::string detail);

    std::string m_Url;
    std::string m_Data;
    std::optional<MyWinInetError> m_LastError;
};
}
