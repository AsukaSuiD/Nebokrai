#include "mywininet.h"

#include <curl/curl.h>

#include <limits>
#include <string_view>
#include <utility>

namespace Login
{
namespace
{
constexpr std::size_t kLegacyUrlBufferSize = 0x100U;
constexpr std::size_t kLegacyResponseBufferSize = 0x400U;

struct CurlGlobalState
{
    CurlGlobalState() noexcept : result(curl_global_init(CURL_GLOBAL_DEFAULT)) {}
    ~CurlGlobalState()
    {
        if (result == CURLE_OK) {
            curl_global_cleanup();
        }
    }

    CURLcode result;
};

CurlGlobalState& CurlGlobal()
{
    static CurlGlobalState state;
    return state;
}

bool IsHttpScheme(std::string_view scheme)
{
    return scheme == "http" || scheme == "https";
}
}

CMyWinInet::~CMyWinInet()
{
    Close();
}

int CMyWinInet::Init(const char* url)
{
    Close();
    if (CurlGlobal().result != CURLE_OK) {
        SetError(MyWinInetErrorKind::CurlGlobal,
                 curl_easy_strerror(CurlGlobal().result));
        return 0;
    }
    if (url == nullptr) {
        SetError(MyWinInetErrorKind::InvalidUrl, "verification URL is null");
        return 0;
    }

    const std::string_view candidate(url);
    if (candidate.size() >= kLegacyUrlBufferSize) {
        SetError(MyWinInetErrorKind::UrlTooLong,
                 "verification URL does not fit legacy char[256]");
        return 0;
    }

    CURLU* parsed = curl_url();
    if (parsed == nullptr) {
        SetError(MyWinInetErrorKind::CurlInit, "curl_url allocation failed");
        return 0;
    }

    const CURLUcode setResult =
        curl_url_set(parsed, CURLUPART_URL, candidate.data(), 0U);
    char* scheme = nullptr;
    const CURLUcode schemeResult = setResult == CURLUE_OK
        ? curl_url_get(parsed, CURLUPART_SCHEME, &scheme, 0U)
        : setResult;

    const bool accepted =
        setResult == CURLUE_OK && schemeResult == CURLUE_OK && scheme != nullptr &&
        IsHttpScheme(scheme);
    if (!accepted) {
        const CURLUcode error = setResult != CURLUE_OK ? setResult : schemeResult;
        SetError(MyWinInetErrorKind::InvalidUrl,
                 error == CURLUE_OK ? "verification URL is not http/https"
                                    : curl_url_strerror(error));
    }

    if (scheme != nullptr) {
        curl_free(scheme);
    }
    curl_url_cleanup(parsed);
    if (!accepted) {
        return 0;
    }

    m_Url.assign(candidate);
    return 1;
}

int CMyWinInet::Send(const char* content)
{
    m_Data.clear();
    m_LastError.reset();
    if (m_Url.empty() || content == nullptr) {
        SetError(MyWinInetErrorKind::InvalidUrl,
                 m_Url.empty() ? "Init did not provide verification URL"
                               : "POST content is null");
        return 0;
    }

    CURL* curl = curl_easy_init();
    if (curl == nullptr) {
        SetError(MyWinInetErrorKind::CurlInit, "curl_easy_init failed");
        return 0;
    }

    curl_slist* headers = curl_slist_append(nullptr, "Accept: text/*");
    if (headers == nullptr) {
        curl_easy_cleanup(curl);
        SetError(MyWinInetErrorKind::CurlOptions,
                 "curl_slist_append failed for Accept header");
        return 0;
    }
    curl_slist* extended = curl_slist_append(
        headers, "Content-Type:application/x-www-form-urlencoded");
    if (extended == nullptr) {
        curl_slist_free_all(headers);
        curl_easy_cleanup(curl);
        SetError(MyWinInetErrorKind::CurlOptions,
                 "curl_slist_append failed for Content-Type header");
        return 0;
    }
    headers = extended;

    const std::string_view body(content);
    CURLcode optionResult = CURLE_OK;
    const auto set = [&](CURLoption option, auto value) {
        if (optionResult == CURLE_OK) {
            optionResult = curl_easy_setopt(curl, option, value);
        }
    };

    set(CURLOPT_URL, m_Url.c_str());
    set(CURLOPT_POST, 1L);
    set(CURLOPT_COPYPOSTFIELDS, content);
    set(CURLOPT_POSTFIELDSIZE, static_cast<long>(body.size()));
    set(CURLOPT_HTTPHEADER, headers);
    set(CURLOPT_USERAGENT, "App");
    set(CURLOPT_FOLLOWLOCATION, 1L);
    set(CURLOPT_NOSIGNAL, 1L);
    set(CURLOPT_WRITEFUNCTION, &CMyWinInet::WriteResponse);
    set(CURLOPT_WRITEDATA, this);

    if (optionResult != CURLE_OK) {
        curl_slist_free_all(headers);
        curl_easy_cleanup(curl);
        SetError(MyWinInetErrorKind::CurlOptions,
                 curl_easy_strerror(optionResult));
        return 0;
    }

    const CURLcode transferResult = curl_easy_perform(curl);
    curl_slist_free_all(headers);
    curl_easy_cleanup(curl);
    if (transferResult != CURLE_OK) {
        if (!m_LastError ||
            m_LastError->kind != MyWinInetErrorKind::ResponseTooLarge) {
            if (transferResult == CURLE_PEER_FAILED_VERIFICATION) {
                SetError(MyWinInetErrorKind::TlsPeerVerificationLegacyRetryBoundary,
                         curl_easy_strerror(transferResult));
            } else {
                SetError(MyWinInetErrorKind::Transfer,
                         curl_easy_strerror(transferResult));
            }
        }
        m_Data.clear();
        return 0;
    }
    return 1;
}

char* CMyWinInet::Recv() noexcept
{
    return m_Data.empty() ? nullptr : m_Data.data();
}

void CMyWinInet::Close() noexcept
{
    m_Url.clear();
    m_Data.clear();
    m_LastError.reset();
}

const std::optional<MyWinInetError>& CMyWinInet::LastError() const noexcept
{
    return m_LastError;
}

std::size_t CMyWinInet::WriteResponse(char* data,
                                      std::size_t size,
                                      std::size_t count,
                                      void* owner) noexcept
{
    auto* self = static_cast<CMyWinInet*>(owner);
    if (self == nullptr || data == nullptr) {
        return 0U;
    }
    if (size != 0U && count > std::numeric_limits<std::size_t>::max() / size) {
        self->SetError(MyWinInetErrorKind::ResponseTooLarge,
                       "libcurl response chunk size overflow");
        return 0U;
    }

    const std::size_t bytes = size * count;
    if (bytes > (kLegacyResponseBufferSize - 1U) - self->m_Data.size()) {
        self->SetError(MyWinInetErrorKind::ResponseTooLarge,
                       "verification response exceeds safe legacy char[1024]");
        return 0U;
    }
    self->m_Data.append(data, bytes);
    return bytes;
}

void CMyWinInet::SetError(MyWinInetErrorKind kind, std::string detail)
{
    m_LastError = MyWinInetError{kind, std::move(detail)};
}
}
