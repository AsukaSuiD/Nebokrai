#include "textcodec.h"

#include <iconv.h>

#include <cerrno>
#include <utility>

namespace Nebokrai
{
TextConversionResult ConvertTextEncoding(
    std::span<const char> source,
    std::string_view sourceEncoding,
    std::string_view destinationEncoding)
{
    if (source.empty()) {
        return TextConversionResult{.value = std::string{}};
    }

    const std::string sourceName(sourceEncoding);
    const std::string destinationName(destinationEncoding);
    iconv_t converter = iconv_open(destinationName.c_str(), sourceName.c_str());
    if (converter == reinterpret_cast<iconv_t>(-1)) {
        return TextConversionResult{
            .error = "iconv не поддерживает преобразование " + sourceName +
                     " -> " + destinationName};
    }

    struct IconvCloser
    {
        iconv_t converter;
        ~IconvCloser() { iconv_close(converter); }
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
        return TextConversionResult{
            .error = "iconv не выполнил преобразование " + sourceName +
                     " -> " + destinationName + ", errno=" +
                     std::to_string(errno)};
    }

    output.resize(output.size() - destinationLeft);
    return TextConversionResult{.value = std::move(output)};
}

TextConversionResult Windows1251ToUtf8(std::string_view source)
{
    return ConvertTextEncoding(
        std::span<const char>(source.data(), source.size()),
        "WINDOWS-1251",
        "UTF-8");
}
}
