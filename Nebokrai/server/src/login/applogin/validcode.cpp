#include "validcode.h"

#include <ft2build.h>
#include FT_FREETYPE_H

#include <iconv.h>

#include <algorithm>
#include <array>
#include <cerrno>
#include <charconv>
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <exception>
#include <fstream>
#include <iterator>
#include <limits>
#include <memory>
#include <random>
#include <string>
#include <string_view>
#include <system_error>
#include <utility>

#if defined(__linux__)
#include <sys/random.h>
#endif

namespace Login
{
namespace
{
constexpr std::int32_t kWidth = 200;
constexpr std::int32_t kHeight = 48;
constexpr std::size_t kPixelOffset = 54U;
constexpr std::size_t kPixelCount =
    static_cast<std::size_t>(kWidth) * static_cast<std::size_t>(kHeight);
constexpr double kLegacyRotationCircle = 6.28318;

struct Bgr
{
    std::uint8_t blue{};
    std::uint8_t green{};
    std::uint8_t red{};
};

constexpr Bgr kBlue{0xFFU, 0U, 0U};
constexpr Bgr kBlack{0U, 0U, 0U};

struct ValidCodeSetup
{
    std::vector<std::uint8_t> charset;
    std::int32_t foregroundLines{};
    std::int32_t backgroundLines{};
    std::int32_t noisyDotOdds{};
    std::vector<std::filesystem::path> fonts;
};

struct FreeTypeOwners
{
    FT_Library library{};
    std::vector<FT_Face> faces;

    ~FreeTypeOwners()
    {
        for (FT_Face face : faces) {
            if (face != nullptr) {
                static_cast<void>(FT_Done_Face(face));
            }
        }
        if (library != nullptr) {
            static_cast<void>(FT_Done_FreeType(library));
        }
    }

    FreeTypeOwners() = default;
    FreeTypeOwners(const FreeTypeOwners&) = delete;
    FreeTypeOwners& operator=(const FreeTypeOwners&) = delete;
};

ValidCodeError Error(ValidCodeErrorKind kind, std::string detail)
{
    return ValidCodeError{kind, std::move(detail)};
}

char LowerAscii(char value) noexcept
{
    return value >= 'A' && value <= 'Z'
               ? static_cast<char>(value - 'A' + 'a')
               : value;
}

bool EqualAsciiCaseInsensitive(std::string_view left, std::string_view right)
{
    if (left.size() != right.size()) {
        return false;
    }
    for (std::size_t index = 0; index < left.size(); ++index) {
        if (LowerAscii(left[index]) != LowerAscii(right[index])) {
            return false;
        }
    }
    return true;
}

std::optional<ValidCodeError>
ResolveAsciiCase(const std::filesystem::path& directory,
                 std::string_view requested,
                 std::filesystem::path& resolved)
{
    const std::filesystem::path direct = directory / std::string(requested);
    std::error_code fileError;
    if (std::filesystem::exists(direct, fileError)) {
        resolved = direct;
        return std::nullopt;
    }

    fileError.clear();
    std::filesystem::directory_iterator iterator(directory, fileError);
    if (fileError) {
        return Error(ValidCodeErrorKind::Io,
                     "не удалось прочитать каталог " + directory.string() +
                         ": " + fileError.message());
    }
    for (const auto& entry : iterator) {
        const std::string name = entry.path().filename().string();
        if (EqualAsciiCaseInsensitive(name, requested)) {
            resolved = entry.path();
            return std::nullopt;
        }
    }

    // Поздняя реконструкция возвращала прямой путь; фактическая ошибка
    // возникает затем у read/FreeType, а не придумывается здесь.
    resolved = direct;
    return std::nullopt;
}

std::optional<ValidCodeError>
ReadFile(const std::filesystem::path& path, std::vector<std::uint8_t>& bytes)
{
    std::ifstream input(path, std::ios::binary);
    if (!input.is_open()) {
        return Error(ValidCodeErrorKind::Io,
                     "не удалось открыть " + path.string());
    }

    const std::vector<char> raw((std::istreambuf_iterator<char>(input)),
                                std::istreambuf_iterator<char>());
    if (input.bad()) {
        return Error(ValidCodeErrorKind::Io,
                     "ошибка чтения " + path.string());
    }
    bytes.assign(raw.begin(), raw.end());
    return std::nullopt;
}

bool IsAsciiWhitespace(std::uint8_t byte) noexcept
{
    return byte == 0x20U || (0x09U <= byte && byte <= 0x0DU);
}

std::vector<std::vector<std::uint8_t>>
SplitWhitespace(std::span<const std::uint8_t> bytes)
{
    std::vector<std::vector<std::uint8_t>> tokens;
    std::size_t position = 0;
    while (position < bytes.size()) {
        while (position < bytes.size() && IsAsciiWhitespace(bytes[position])) {
            ++position;
        }
        const std::size_t begin = position;
        while (position < bytes.size() && !IsAsciiWhitespace(bytes[position])) {
            ++position;
        }
        if (begin != position) {
            tokens.emplace_back(bytes.begin() + static_cast<std::ptrdiff_t>(begin),
                                bytes.begin() + static_cast<std::ptrdiff_t>(position));
        }
    }
    return tokens;
}

std::optional<ValidCodeError>
ParseI32(std::span<const std::uint8_t> token, std::int32_t& value)
{
    if (token.empty()) {
        return Error(ValidCodeErrorKind::InvalidSetup,
                     "пустой числовой параметр ValidCode.ini");
    }
    const char* begin = reinterpret_cast<const char*>(token.data());
    const char* end = begin + token.size();
    const auto parsed = std::from_chars(begin, end, value);
    if (parsed.ec != std::errc{} || parsed.ptr != end) {
        return Error(ValidCodeErrorKind::InvalidSetup,
                     "нечисловой параметр шума ValidCode.ini");
    }
    return std::nullopt;
}

std::optional<ValidCodeError>
LoadSetup(const std::filesystem::path& runtimeDirectory, ValidCodeSetup& setup)
{
    std::filesystem::path setupPath;
    if (auto error = ResolveAsciiCase(runtimeDirectory, "ValidCode.ini", setupPath)) {
        return error;
    }

    std::vector<std::uint8_t> bytes;
    if (auto error = ReadFile(setupPath, bytes)) {
        return error;
    }
    const auto tokens = SplitWhitespace(bytes);
    if (tokens.size() < 10U) {
        return Error(ValidCodeErrorKind::InvalidSetup,
                     "неполная whitespace-структура ValidCode.ini");
    }

    setup.charset = tokens[1];
    if (auto error = ParseI32(tokens[3], setup.foregroundLines)) {
        return error;
    }
    if (auto error = ParseI32(tokens[5], setup.backgroundLines)) {
        return error;
    }
    if (auto error = ParseI32(tokens[7], setup.noisyDotOdds)) {
        return error;
    }

    std::filesystem::path fontsDirectory;
    if (auto error = ResolveAsciiCase(runtimeDirectory, "fonts", fontsDirectory)) {
        return error;
    }
    for (std::size_t index = 10U; index < tokens.size(); ++index) {
        if (std::any_of(tokens[index].begin(), tokens[index].end(),
                        [](std::uint8_t byte) { return byte > 0x7FU; })) {
            return Error(ValidCodeErrorKind::InvalidSetup,
                         "не-ASCII имя font-файла");
        }
        const std::string fontName(tokens[index].begin(), tokens[index].end());
        std::filesystem::path fontPath;
        if (auto error = ResolveAsciiCase(fontsDirectory, fontName, fontPath)) {
            return error;
        }
        setup.fonts.push_back(std::move(fontPath));
    }

    if (setup.fonts.empty()) {
        return Error(ValidCodeErrorKind::InvalidSetup,
                     "список font-файлов пуст");
    }
    if (setup.charset.empty() || (setup.charset.size() & 1U) != 0U) {
        return Error(ValidCodeErrorKind::InvalidCharset,
                     "charset не состоит из двухбайтовых пар");
    }
    return std::nullopt;
}

std::optional<ValidCodeError> RandomWord(std::uint32_t& value)
{
#if defined(__linux__)
    auto* output = reinterpret_cast<std::uint8_t*>(&value);
    std::size_t filled = 0;
    while (filled < sizeof(value)) {
        const ssize_t received =
            ::getrandom(output + filled, sizeof(value) - filled, 0);
        if (received > 0) {
            filled += static_cast<std::size_t>(received);
            continue;
        }
        if (received < 0 && errno == EINTR) {
            continue;
        }
        return Error(ValidCodeErrorKind::Random,
                     "getrandom не вернул 32-битное значение");
    }
    return std::nullopt;
#else
    try {
        static thread_local std::random_device source;
        value = static_cast<std::uint32_t>(source());
        return std::nullopt;
    } catch (const std::exception& exception) {
        return Error(ValidCodeErrorKind::Random, exception.what());
    }
#endif
}

std::optional<ValidCodeError>
RandomBelow(std::uint32_t limit, std::uint32_t& value)
{
    if (limit == 0U) {
        return Error(ValidCodeErrorKind::InvalidSetup,
                     "нулевой modulus valid-code RNG");
    }
    std::uint32_t random = 0;
    if (auto error = RandomWord(random)) {
        return error;
    }
    value = random % limit;
    return std::nullopt;
}

std::optional<ValidCodeError> RandomByte(std::uint8_t& value)
{
    std::uint32_t random = 0;
    if (auto error = RandomWord(random)) {
        return error;
    }
    value = static_cast<std::uint8_t>(random);
    return std::nullopt;
}

std::optional<ValidCodeError>
GenerateValidCodeString(const ValidCodeSetup& setup,
                        std::array<std::uint8_t, kValidCodeLength>& code)
{
    const std::size_t pairCount = setup.charset.size() / 2U;
    if (pairCount == 0U || pairCount > std::numeric_limits<std::uint32_t>::max()) {
        return Error(ValidCodeErrorKind::InvalidCharset,
                     "недопустимое число двухбайтовых символов");
    }

    for (std::size_t pair = 0; pair < 4U; ++pair) {
        std::uint32_t selected = 0;
        if (auto error = RandomBelow(static_cast<std::uint32_t>(pairCount), selected)) {
            return error;
        }
        const std::size_t source = static_cast<std::size_t>(selected) * 2U;
        code[pair * 2U] = setup.charset[source];
        code[pair * 2U + 1U] = setup.charset[source + 1U];
    }
    return std::nullopt;
}

void PutU16(std::span<std::uint8_t> destination,
            std::size_t offset,
            std::uint16_t value)
{
    destination[offset] = static_cast<std::uint8_t>(value);
    destination[offset + 1U] = static_cast<std::uint8_t>(value >> 8U);
}

void PutU32(std::span<std::uint8_t> destination,
            std::size_t offset,
            std::uint32_t value)
{
    for (std::size_t index = 0; index < 4U; ++index) {
        destination[offset + index] =
            static_cast<std::uint8_t>(value >> (index * 8U));
    }
}

void PutI32(std::span<std::uint8_t> destination,
            std::size_t offset,
            std::int32_t value)
{
    PutU32(destination, offset, static_cast<std::uint32_t>(value));
}

void InitializeBitmap(std::array<std::uint8_t, kValidCodeBitmapLength>& bitmap)
{
    bitmap.fill(0U);
    PutU16(bitmap, 0U, 0x4D42U);
    PutU32(bitmap, 2U, 0xF6U);
    PutU16(bitmap, 6U, 0U);
    PutU16(bitmap, 8U, 0U);
    PutU32(bitmap, 10U, static_cast<std::uint32_t>(kPixelOffset));
    PutU32(bitmap, 14U, 40U);
    PutI32(bitmap, 18U, kWidth);
    PutI32(bitmap, 22U, kHeight);
    PutU16(bitmap, 26U, 1U);
    PutU16(bitmap, 28U, 24U);
    PutU32(bitmap, 30U, 0U);
    PutU32(bitmap, 34U, 0xC0U);
    PutI32(bitmap, 38U, 0);
    PutI32(bitmap, 42U, 0);
    PutU32(bitmap, 46U, 0U);
    PutU32(bitmap, 50U, 0U);
}

void SetPixel(std::array<std::uint8_t, kValidCodeBitmapLength>& bitmap,
              std::int32_t x,
              std::int32_t y,
              Bgr color)
{
    if (x < 0 || x >= kWidth || y < 0 || y >= kHeight) {
        return;
    }
    const std::size_t pixel =
        static_cast<std::size_t>((kHeight - 1 - y) * kWidth + x);
    const std::size_t offset = kPixelOffset + pixel * 3U;
    bitmap[offset] = color.blue;
    bitmap[offset + 1U] = color.green;
    bitmap[offset + 2U] = color.red;
}

void Bresenham(std::array<std::uint8_t, kValidCodeBitmapLength>& bitmap,
               std::int32_t x1,
               std::int32_t y1,
               std::int32_t x2,
               std::int32_t y2,
               Bgr color)
{
    if (x1 < 0 || x1 >= kWidth || y1 < 0 || y1 >= kHeight ||
        x2 < 0 || x2 >= kWidth || y2 < 0 || y2 >= kHeight) {
        return;
    }

    const std::int32_t dx = std::abs(x2 - x1);
    const std::int32_t dy = std::abs(y2 - y1);
    if (dx == 0) {
        const std::int32_t limit = std::max(y1, y2);
        if (y1 < y2) {
            x2 = x1;
            y2 = y1;
        }
        while (y2 < limit) {
            SetPixel(bitmap, x2, y2, color);
            ++y2;
        }
        return;
    }

    const double slope = static_cast<double>(y2 - y1) /
                         static_cast<double>(x2 - x1);
    if (slope < -1.0 || slope > 1.0) {
        const std::int32_t straight = dx * 2;
        std::int32_t error = straight - dy;
        const std::int32_t diagonal = (dx - dy) * 2;
        const std::int32_t limit = std::max(y1, y2);
        if (y1 < y2) {
            x2 = x1;
            y2 = y1;
        }
        SetPixel(bitmap, x2, y2, color);
        while (y2 < limit) {
            ++y2;
            const std::int32_t increment = error >= 0
                                               ? (x2 += slope <= 0.0 ? -1 : 1,
                                                  diagonal)
                                               : straight;
            error += increment;
            SetPixel(bitmap, x2, y2, color);
        }
        return;
    }

    const std::int32_t straight = dy * 2;
    const std::int32_t diagonal = (dy - dx) * 2;
    const std::int32_t limit = std::max(x1, x2);
    if (x1 < x2) {
        y2 = y1;
        x2 = x1;
    }
    SetPixel(bitmap, x2, y2, color);
    std::int32_t error = straight - dx;
    while (x2 < limit) {
        ++x2;
        const std::int32_t increment = error >= 0
                                           ? (y2 += slope <= 0.0 ? -1 : 1,
                                              diagonal)
                                           : straight;
        error += increment;
        SetPixel(bitmap, x2, y2, color);
    }
}

void DrawWideLine(std::array<std::uint8_t, kValidCodeBitmapLength>& bitmap,
                  std::int32_t x1,
                  std::int32_t y1,
                  std::int32_t x2,
                  std::int32_t y2,
                  Bgr color,
                  std::int32_t width)
{
    for (std::int32_t offset = 0; offset < width; ++offset) {
        if (std::abs(x1 - x2) < std::abs(y1 - y2)) {
            Bresenham(bitmap, x1 + offset, y1, x2 + offset, y2, color);
        } else {
            Bresenham(bitmap, x1, y1 + offset, x2, y2 + offset, color);
        }
    }
}

void DrawBitmap(std::array<std::uint8_t, kValidCodeBitmapLength>& destination,
                const FT_Bitmap& source,
                std::int32_t originalX,
                std::int32_t originalY,
                Bgr color)
{
    const std::int32_t width = static_cast<std::int32_t>(source.width);
    const std::int32_t rows = static_cast<std::int32_t>(source.rows);
    if (source.buffer == nullptr || width <= 0 || rows <= 0) {
        return;
    }
    const std::size_t available =
        static_cast<std::size_t>(std::abs(source.pitch)) *
        static_cast<std::size_t>(rows);

    std::int32_t x = originalX;
    for (std::int32_t column = 0; column < width; ++column, ++x) {
        if (x < 0 || x >= kWidth) {
            continue;
        }
        for (std::int32_t row = 0, y = originalY; row < rows; ++row, ++y) {
            const std::size_t sourceIndex =
                static_cast<std::size_t>(column + width * row);
            if (y >= 0 && y < kHeight && sourceIndex < available &&
                source.buffer[sourceIndex] != 0U) {
                SetPixel(destination, x, y, color);
            }
        }
    }
}

std::optional<ValidCodeError>
DecodeGbk(std::span<const std::uint8_t> bytes,
          std::vector<std::uint32_t>& codePoints)
{
    iconv_t converter = ::iconv_open("UTF-32LE", "GBK");
    if (converter == reinterpret_cast<iconv_t>(-1)) {
        return Error(ValidCodeErrorKind::Encoding,
                     "iconv не поддерживает GBK -> UTF-32LE");
    }

    std::vector<char> input(bytes.begin(), bytes.end());
    std::vector<char> output((bytes.size() + 1U) * sizeof(std::uint32_t) * 2U, 0);
    char* inputPtr = input.data();
    std::size_t inputBytes = input.size();
    char* outputPtr = output.data();
    std::size_t outputBytes = output.size();
    const std::size_t result =
        ::iconv(converter, &inputPtr, &inputBytes, &outputPtr, &outputBytes);
    static_cast<void>(::iconv_close(converter));
    if (result == static_cast<std::size_t>(-1) || inputBytes != 0U) {
        return Error(ValidCodeErrorKind::InvalidCharset,
                     "valid-code bytes не декодируются как GBK");
    }

    const std::size_t used = output.size() - outputBytes;
    for (std::size_t offset = 0; offset + 4U <= used; offset += 4U) {
        const auto* value = reinterpret_cast<const std::uint8_t*>(output.data() + offset);
        const std::uint32_t codePoint =
            static_cast<std::uint32_t>(value[0]) |
            (static_cast<std::uint32_t>(value[1]) << 8U) |
            (static_cast<std::uint32_t>(value[2]) << 16U) |
            (static_cast<std::uint32_t>(value[3]) << 24U);
        if (codePoint != 0U) {
            codePoints.push_back(codePoint);
        }
    }
    return std::nullopt;
}

std::optional<ValidCodeError>
CodeToBitmap(const ValidCodeSetup& setup,
             std::span<const std::uint8_t> validCode,
             std::array<std::uint8_t, kValidCodeBitmapLength>& bitmap)
{
    std::vector<std::uint32_t> characters;
    if (auto error = DecodeGbk(validCode, characters)) {
        return error;
    }
    if (characters.empty()) {
        return std::nullopt;
    }

    FreeTypeOwners freeType;
    FT_Error ftError = FT_Init_FreeType(&freeType.library);
    if (ftError != 0) {
        return Error(ValidCodeErrorKind::FreeType,
                     "FT_Init_FreeType: " + std::to_string(ftError));
    }
    for (const auto& path : setup.fonts) {
        FT_Face face = nullptr;
        ftError = FT_New_Face(freeType.library, path.string().c_str(), 0, &face);
        if (ftError != 0 || face == nullptr) {
            return Error(ValidCodeErrorKind::FreeType,
                         "FT_New_Face(" + path.string() + "): " +
                             std::to_string(ftError));
        }
        freeType.faces.push_back(face);
    }

    std::uint32_t random = 0;
    if (auto error = RandomBelow(4U, random)) {
        return error;
    }
    std::int32_t x = static_cast<std::int32_t>(
        static_cast<double>(static_cast<std::int32_t>(random) - 1) * 0.01 *
        static_cast<double>(kWidth));

    std::uint32_t noisyCharacter = 0;
    if (characters.size() > std::numeric_limits<std::uint32_t>::max()) {
        return Error(ValidCodeErrorKind::InvalidCharset,
                     "слишком много декодированных glyph");
    }
    if (auto error = RandomBelow(static_cast<std::uint32_t>(characters.size()),
                                 noisyCharacter)) {
        return error;
    }

    for (std::size_t index = 0; index < characters.size(); ++index) {
        if (index == static_cast<std::size_t>(noisyCharacter)) {
            if (auto error = RandomBelow(4U, random)) {
                return error;
            }
            const std::int32_t lineCount = static_cast<std::int32_t>(random) + 7;
            for (std::int32_t line = 0; line < lineCount; ++line) {
                std::uint32_t x1Random = 0;
                std::uint32_t y1Random = 0;
                std::uint32_t x2Random = 0;
                std::uint32_t y2Random = 0;
                if (auto error = RandomBelow(38U, x1Random)) return error;
                if (auto error = RandomBelow(44U, y1Random)) return error;
                if (auto error = RandomBelow(38U, x2Random)) return error;
                if (auto error = RandomBelow(44U, y2Random)) return error;
                DrawWideLine(bitmap,
                             static_cast<std::int32_t>(x1Random) + 1 + x,
                             static_cast<std::int32_t>(y1Random) + 1,
                             static_cast<std::int32_t>(x2Random) + 1 + x,
                             static_cast<std::int32_t>(y2Random) + 1,
                             kBlue,
                             3);
            }
            x += 38;
        }

        if (freeType.faces.size() > std::numeric_limits<std::uint32_t>::max()) {
            return Error(ValidCodeErrorKind::InvalidSetup,
                         "слишком много font-файлов");
        }
        if (auto error = RandomBelow(static_cast<std::uint32_t>(freeType.faces.size()),
                                     random)) {
            return error;
        }
        FT_Face face = freeType.faces[static_cast<std::size_t>(random)];

        if (auto error = RandomBelow(8U, random)) {
            return error;
        }
        const FT_F26Dot6 pointSize = static_cast<FT_F26Dot6>(
            static_cast<double>(random + 56U) * 0.01 *
            static_cast<double>(kHeight));
        ftError = FT_Set_Char_Size(face, pointSize << 6, 0, 100, 0);
        if (ftError != 0) {
            return Error(ValidCodeErrorKind::FreeType,
                         "FT_Set_Char_Size: " + std::to_string(ftError));
        }

        if (auto error = RandomBelow(60U, random)) {
            return error;
        }
        const double angle =
            (static_cast<double>(static_cast<std::int32_t>(random) - 30) *
             (1.0 / 360.0)) *
            kLegacyRotationCircle;
        FT_Matrix matrix{
            static_cast<FT_Fixed>(std::cos(angle) * 65536.0),
            static_cast<FT_Fixed>(-std::sin(angle) * 65536.0),
            static_cast<FT_Fixed>(std::sin(angle) * 65536.0),
            static_cast<FT_Fixed>(std::cos(angle) * 65536.0),
        };
        FT_Vector delta{0, 0};
        FT_Set_Transform(face, &matrix, &delta);
        ftError = FT_Load_Char(face,
                               static_cast<FT_ULong>(characters[index]),
                               FT_LOAD_RENDER);
        if (ftError != 0) {
            return Error(ValidCodeErrorKind::FreeType,
                         "FT_Load_Char: " + std::to_string(ftError));
        }

        if (auto error = RandomBelow(8U, random)) {
            return error;
        }
        const std::int32_t y = static_cast<std::int32_t>(
            static_cast<double>(static_cast<std::int32_t>(random) - 3) * 0.01 *
            static_cast<double>(kHeight));
        DrawBitmap(bitmap, face->glyph->bitmap, x, y, kBlue);

        if (auto error = RandomBelow(4U, random)) {
            return error;
        }
        const double advance = static_cast<double>(face->glyph->advance.x);
        x += static_cast<std::int32_t>(
            static_cast<double>(random + 90U) * 0.01 * advance);
    }
    return std::nullopt;
}

std::optional<ValidCodeError>
AddNoise(const ValidCodeSetup& setup,
         std::array<std::uint8_t, kValidCodeBitmapLength>& bitmap)
{
    for (std::int32_t line = 0; line < std::max(setup.foregroundLines, 0); ++line) {
        std::uint32_t x1 = 0;
        std::uint32_t y1 = 0;
        std::uint32_t x2 = 0;
        std::uint32_t y2 = 0;
        if (auto error = RandomBelow(static_cast<std::uint32_t>(kWidth), x1)) return error;
        if (auto error = RandomBelow(static_cast<std::uint32_t>(kHeight), y1)) return error;
        if (auto error = RandomBelow(static_cast<std::uint32_t>(kWidth), x2)) return error;
        if (auto error = RandomBelow(static_cast<std::uint32_t>(kHeight), y2)) return error;
        DrawWideLine(bitmap,
                     static_cast<std::int32_t>(x1),
                     static_cast<std::int32_t>(y1),
                     static_cast<std::int32_t>(x2),
                     static_cast<std::int32_t>(y2),
                     kBlue,
                     3);
    }

    const std::int32_t backgroundLines = std::max(setup.backgroundLines, 0);
    std::int32_t x1 = 0;
    std::int32_t y1 = 0;
    for (std::int32_t index = 0; index < backgroundLines; ++index) {
        const std::int32_t divisor = backgroundLines + 1;
        const std::int32_t partition = kWidth / divisor;
        std::uint32_t random = 0;
        if (auto error = RandomBelow(static_cast<std::uint32_t>(partition), random)) {
            return error;
        }
        const std::int32_t x2 =
            static_cast<std::int32_t>(random) +
            kWidth * (index + 1) / divisor;
        const std::int32_t y2 = y1 == 0 ? 44 : 0;
        DrawWideLine(bitmap, x1, y1, x2, y2, kBlack, 2);
        x1 = x2;
        y1 = y2;
    }

    if (setup.noisyDotOdds <= 0) {
        return std::nullopt;
    }

    for (std::size_t index = 0; index < kPixelCount; ++index) {
        std::uint32_t odds = 0;
        if (auto error = RandomBelow(100U, odds)) {
            return error;
        }
        if (static_cast<std::int32_t>(odds) >= setup.noisyDotOdds) {
            continue;
        }

        const std::size_t offset = kPixelOffset + index * 3U;
        const bool white = bitmap[offset] == 0xFFU &&
                           bitmap[offset + 1U] == 0xFFU &&
                           bitmap[offset + 2U] == 0xFFU;
        std::uint8_t random = 0;
        if (white) {
            if (auto error = RandomByte(random)) return error;
            bitmap[offset] = random;
            if (auto error = RandomByte(random)) return error;
            bitmap[offset + 1U] = random;
            if (auto error = RandomByte(random)) return error;
            bitmap[offset + 2U] = random;
        } else {
            if (auto error = RandomByte(random)) return error;
            bitmap[offset] = static_cast<std::uint8_t>(
                (random & 0x7FU) + (bitmap[offset] >> 1U));
            if (auto error = RandomByte(random)) return error;
            bitmap[offset + 1U] = static_cast<std::uint8_t>(
                (random & 0x7FU) + (bitmap[offset + 1U] >> 1U));
            if (auto error = RandomByte(random)) return error;
            bitmap[offset + 2U] = static_cast<std::uint8_t>(
                (random & 0x7FU) + (bitmap[offset + 2U] >> 1U));
        }
    }
    return std::nullopt;
}
}

std::optional<ValidCodeError>
CValidCode::Generate(const std::filesystem::path& runtimeDirectory,
                     CValidCode& output)
{
    ValidCodeSetup setup;
    if (auto error = LoadSetup(runtimeDirectory, setup)) {
        return error;
    }

    CValidCode generated;
    if (auto error = GenerateValidCodeString(setup, generated.m_ValidCode)) {
        return error;
    }
    InitializeBitmap(generated.m_Bitmap);
    if (auto error = CodeToBitmap(setup, generated.m_ValidCode, generated.m_Bitmap)) {
        return error;
    }
    if (auto error = AddNoise(setup, generated.m_Bitmap)) {
        return error;
    }

    output = std::move(generated);
    return std::nullopt;
}

std::span<const std::uint8_t> CValidCode::ValidCode() const noexcept
{
    return m_ValidCode;
}

std::span<const std::uint8_t, kValidCodeBitmapLength>
CValidCode::Bitmap() const noexcept
{
    return std::span<const std::uint8_t, kValidCodeBitmapLength>(m_Bitmap);
}
}
