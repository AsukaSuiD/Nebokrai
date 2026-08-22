//! Генератор legacy valid-code `CValidCode` из `validcode.cpp` и `.h`.
//!
//! Восстановлены функции, спорные числовые выражения
//! `CodeToBitmap` и поля BMP — подтверждено точным EXE. Точная пара:
//! `validcode.ini` читается как исходная whitespace-последовательность:
//! byte-exact двухбайтовый набор символов, три signed параметра шума и список
//! font-файлов. Четыре вызова выбирают по одной двухбайтовой паре и образуют
//! восьмибайтовый ответ. Пары baseline-файла являются GBK; `encoding_rs`
//! заменяет Windows `MultiByteToWideChar(CP_ACP)` для получения Unicode glyph,
//! но сравниваемое с ответом клиента значение остаётся исходными байтами.
//!
//! `freetype-rs` с bundled FreeType заменяет те же `FT_Init_FreeType`,
//! `FT_New_Face`, `FT_Set_Char_Size`, `FT_Set_Transform` и `FT_Load_Char`.
//! Библиотека отвечает только за разбор TTF и raster glyph; выбор символов,
//! размеры, поворот, координаты, отсутствие alpha-blend, шум и итоговый wire
//! остаются здесь. Linux case-sensitive пути разрешаются ASCII-
//! нечувствительно внутри runtime-каталога, сохраняя поведение Windows для
//! исходных `ValidCode.ini` и имён шрифтов.
//!
//! Итог всегда имеет ровно `0x70B6` байт: packed 14-байтовый BMP file header,
//! 40-байтовый info header и `200 * 48 * 3` BGR pixels. Подтверждённые
//! странности оригинала сохранены: `bfSize = 0xF6` и `biSizeImage = 0xC0`,
//! хотя фактический payload больше; glyph bitmap проверяется только на
//! ненулевой байт и его grayscale intensity не смешивается с цветом.
//!
//! Внутренности `std::string/vector/ifstream`, ручное владение wide-buffer и
//! FreeType handles удалены: их эффекты выражены owned `Vec`, `Box`, RAII и
//! библиотечными объектами.

use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use encoding_rs::GBK;
use freetype::face::LoadFlag;
use freetype::{Library, Matrix, Vector};

/// Фактическая длина BMP, которую LoginServer добавлял после `0x70B6`.
pub(crate) const VALID_CODE_BITMAP_LEN: usize = 0x70B6;

const WIDTH: i32 = 200;
const HEIGHT: i32 = 48;
const PIXEL_OFFSET: usize = 54;
const PIXEL_COUNT: usize = WIDTH as usize * HEIGHT as usize;
#[allow(
    clippy::approx_constant,
    reason = "LoginServer хранит для поворота буквальное значение 6.28318"
)]
const LEGACY_ROTATION_CIRCLE: f64 = 6.28318;

#[derive(Clone, Copy)]
struct Bgr {
    blue: u8,
    green: u8,
    red: u8,
}

const BLUE: Bgr = Bgr {
    blue: 0xff,
    green: 0,
    red: 0,
};
const BLACK: Bgr = Bgr {
    blue: 0,
    green: 0,
    red: 0,
};

struct ValidCodeSetup {
    charset: Vec<u8>,
    foreground_lines: i32,
    background_lines: i32,
    noisy_dot_odds: i32,
    fonts: Vec<PathBuf>,
}

/// Ошибка подготовки исходного valid-code изображения.
#[derive(Debug)]
pub(crate) enum ValidCodeError {
    /// Не удалось прочитать runtime `ValidCode.ini` либо каталог шрифтов.
    Io(std::io::Error),
    /// Whitespace-структура `ValidCode.ini` не соответствует исходному reader.
    InvalidSetup(&'static str),
    /// Набор символов не состоит из двухбайтовых GBK-пар.
    InvalidCharset,
    /// Системный источник случайных значений Linux недоступен.
    Random(getrandom::Error),
    /// Bundled FreeType не смог открыть библиотеку, face или glyph.
    FreeType(String),
}

impl fmt::Display for ValidCodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(
                formatter,
                "не удалось прочитать valid-code ресурсы: {error}"
            ),
            Self::InvalidSetup(reason) => {
                write!(formatter, "повреждён исходный ValidCode.ini: {reason}")
            }
            Self::InvalidCharset => formatter
                .write_str("набор символов ValidCode.ini не состоит из двухбайтовых GBK-пар"),
            Self::Random(error) => {
                write!(
                    formatter,
                    "недоступен источник случайности valid-code: {error}"
                )
            }
            Self::FreeType(error) => write!(formatter, "ошибка FreeType valid-code: {error}"),
        }
    }
}

impl Error for ValidCodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Random(error) => Some(error),
            Self::InvalidSetup(_) | Self::InvalidCharset | Self::FreeType(_) => None,
        }
    }
}

impl From<std::io::Error> for ValidCodeError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<getrandom::Error> for ValidCodeError {
    fn from(error: getrandom::Error) -> Self {
        Self::Random(error)
    }
}

/// Owned-форма исходного `CValidCode`: ответ и packed BMP wire.
pub(crate) struct CValidCode {
    valid_code: Vec<u8>,
    bitmap: Box<[u8; VALID_CODE_BITMAP_LEN]>,
}

impl CValidCode {
    /// Загружает setup относительно текущего LoginServer runtime-каталога.
    pub(crate) fn generate(runtime_directory: &Path) -> Result<Self, ValidCodeError> {
        let setup = ValidCodeSetup::load(runtime_directory)?;
        let valid_code = generate_valid_code_string(&setup.charset)?;
        let mut bitmap = Box::new([0; VALID_CODE_BITMAP_LEN]);
        initialize_bitmap(&mut bitmap);
        code_to_bitmap(&setup, &valid_code, &mut bitmap)?;
        add_noise(&setup, &mut bitmap)?;
        Ok(Self { valid_code, bitmap })
    }

    /// Возвращает исходные восемь байт, которые должен прислать клиент.
    pub(crate) fn valid_code(&self) -> &[u8] {
        &self.valid_code
    }

    /// Возвращает packed BMP ровно исходной wire-длины `0x70B6`.
    pub(crate) fn bitmap(&self) -> &[u8; VALID_CODE_BITMAP_LEN] {
        &self.bitmap
    }
}

impl ValidCodeSetup {
    fn load(runtime_directory: &Path) -> Result<Self, ValidCodeError> {
        let setup_path = resolve_ascii_case(runtime_directory, "ValidCode.ini")?;
        let bytes = fs::read(setup_path)?;
        let mut tokens = bytes
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());

        next_token(&mut tokens, "метка набора символов")?;
        let charset = next_token(&mut tokens, "набор символов")?.to_vec();
        next_token(&mut tokens, "метка foreground noise")?;
        let foreground_lines = parse_i32(next_token(&mut tokens, "foreground noise")?)?;
        next_token(&mut tokens, "метка background noise")?;
        let background_lines = parse_i32(next_token(&mut tokens, "background noise")?)?;
        next_token(&mut tokens, "метка noisy dot odds")?;
        let noisy_dot_odds = parse_i32(next_token(&mut tokens, "noisy dot odds")?)?;
        next_token(&mut tokens, "слово Font")?;
        next_token(&mut tokens, "слово file")?;

        let fonts_directory = resolve_ascii_case(runtime_directory, "fonts")?;
        let mut fonts = Vec::new();
        for font in tokens {
            let name = std::str::from_utf8(font)
                .map_err(|_| ValidCodeError::InvalidSetup("не-ASCII имя font-файла"))?;
            fonts.push(resolve_ascii_case(&fonts_directory, name)?);
        }
        if fonts.is_empty() {
            return Err(ValidCodeError::InvalidSetup("список font-файлов пуст"));
        }
        if charset.is_empty() || charset.len() % 2 != 0 {
            return Err(ValidCodeError::InvalidCharset);
        }

        Ok(Self {
            charset,
            foreground_lines,
            background_lines,
            noisy_dot_odds,
            fonts,
        })
    }
}

fn next_token<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    name: &'static str,
) -> Result<&'a [u8], ValidCodeError> {
    tokens.next().ok_or(ValidCodeError::InvalidSetup(name))
}

fn parse_i32(bytes: &[u8]) -> Result<i32, ValidCodeError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| ValidCodeError::InvalidSetup("нечисловой параметр шума"))?;
    text.parse()
        .map_err(|_| ValidCodeError::InvalidSetup("нечисловой параметр шума"))
}

fn resolve_ascii_case(directory: &Path, requested: &str) -> Result<PathBuf, std::io::Error> {
    let direct = directory.join(requested);
    if direct.exists() {
        return Ok(direct);
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case(requested)
        {
            return Ok(entry.path());
        }
    }
    Ok(direct)
}

fn generate_valid_code_string(charset: &[u8]) -> Result<Vec<u8>, ValidCodeError> {
    let pair_count = charset.len() / 2;
    if pair_count == 0 {
        return Err(ValidCodeError::InvalidCharset);
    }
    let mut code = Vec::with_capacity(8);
    for _ in 0..4 {
        let index = random_below(pair_count as u32)? as usize * 2;
        code.extend_from_slice(&charset[index..index + 2]);
    }
    Ok(code)
}

fn initialize_bitmap(bitmap: &mut [u8; VALID_CODE_BITMAP_LEN]) {
    put_u16(bitmap, 0, 0x4D42);
    // а не фактическую длину 0x70B6; клиент русской ветки это поле терпел.
    put_u32(bitmap, 2, 0xF6);
    put_u16(bitmap, 6, 0);
    put_u16(bitmap, 8, 0);
    put_u32(bitmap, 10, PIXEL_OFFSET as u32);
    put_u32(bitmap, 14, 40);
    put_i32(bitmap, 18, WIDTH);
    put_i32(bitmap, 22, HEIGHT);
    put_u16(bitmap, 26, 1);
    put_u16(bitmap, 28, 24);
    put_u32(bitmap, 30, 0);
    put_u32(bitmap, 34, 0xC0);
    put_i32(bitmap, 38, 0);
    put_i32(bitmap, 42, 0);
    put_u32(bitmap, 46, 0);
    put_u32(bitmap, 50, 0);
}

fn code_to_bitmap(
    setup: &ValidCodeSetup,
    valid_code: &[u8],
    bitmap: &mut [u8; VALID_CODE_BITMAP_LEN],
) -> Result<(), ValidCodeError> {
    let (decoded, _, decode_failed) = GBK.decode(valid_code);
    if decode_failed {
        return Err(ValidCodeError::InvalidCharset);
    }
    let characters: Vec<char> = decoded.chars().collect();
    if characters.is_empty() {
        return Ok(());
    }

    let library = Library::init().map_err(freetype_error)?;
    let mut faces = Vec::with_capacity(setup.fonts.len());
    for path in &setup.fonts {
        faces.push(library.new_face(path, 0).map_err(freetype_error)?);
    }

    let mut x = ((random_below(4)? as i32 - 1) as f64 * 0.01 * WIDTH as f64) as i32;
    let noisy_character = random_below(characters.len() as u32)? as usize;
    for (index, character) in characters.into_iter().enumerate() {
        if index == noisy_character {
            let line_count = random_below(4)? as i32 + 7;
            for _ in 0..line_count {
                let x1 = random_below(38)? as i32 + 1 + x;
                let y1 = random_below(44)? as i32 + 1;
                let x2 = random_below(38)? as i32 + 1 + x;
                let y2 = random_below(44)? as i32 + 1;
                draw_wide_line(bitmap, x1, y1, x2, y2, BLUE, 3);
            }
            x += 38;
        }

        let face_index = random_below(faces.len() as u32)? as usize;
        let face = &faces[face_index];
        let point_size = ((random_below(8)? + 56) as f64 * 0.01 * HEIGHT as f64) as isize;
        face.set_char_size(point_size << 6, 0, 100, 0)
            .map_err(freetype_error)?;

        let angle = (random_below(60)? as f64 - 30.0) * (1.0 / 360.0) * LEGACY_ROTATION_CIRCLE;
        let mut transform = Matrix {
            xx: (angle.cos() * 65536.0) as i64,
            xy: (-angle.sin() * 65536.0) as i64,
            yx: (angle.sin() * 65536.0) as i64,
            yy: (angle.cos() * 65536.0) as i64,
        };
        let mut delta = Vector { x: 0, y: 0 };
        face.set_transform(&mut transform, &mut delta);
        face.load_char(character as usize, LoadFlag::RENDER)
            .map_err(freetype_error)?;

        let glyph = face.glyph();
        let glyph_bitmap = glyph.bitmap();
        let y = ((random_below(8)? as i32 - 3) as f64 * 0.01 * HEIGHT as f64) as i32;
        draw_bitmap(bitmap, &glyph_bitmap, x, y, BLUE);
        let advance = glyph.advance().x;
        x += ((random_below(4)? + 90) as f64 * 0.01 * advance as f64) as i32;
    }
    Ok(())
}

fn draw_bitmap(
    destination: &mut [u8; VALID_CODE_BITMAP_LEN],
    source: &freetype::Bitmap,
    mut x: i32,
    original_y: i32,
    color: Bgr,
) {
    let width = source.width().max(0);
    let rows = source.rows().max(0);
    let source_bytes = source.buffer();
    for column in 0..width {
        if (0..WIDTH).contains(&x) {
            for (y, row) in (original_y..).zip(0..rows) {
                // Оригинал индексировал FreeType buffer через width, не pitch.
                let source_index = (column + width * row) as usize;
                if (0..HEIGHT).contains(&y)
                    && source_bytes
                        .get(source_index)
                        .is_some_and(|value| *value != 0)
                {
                    set_pixel(destination, x, y, color);
                }
            }
        }
        x += 1;
    }
}

fn add_noise(
    setup: &ValidCodeSetup,
    bitmap: &mut [u8; VALID_CODE_BITMAP_LEN],
) -> Result<(), ValidCodeError> {
    for _ in 0..setup.foreground_lines.max(0) {
        let x1 = random_below(WIDTH as u32)? as i32;
        let y1 = random_below(HEIGHT as u32)? as i32;
        let x2 = random_below(WIDTH as u32)? as i32;
        let y2 = random_below(HEIGHT as u32)? as i32;
        draw_wide_line(bitmap, x1, y1, x2, y2, BLUE, 3);
    }

    let background_lines = setup.background_lines.max(0);
    let mut x1 = 0;
    let mut y1 = 0;
    for index in 0..background_lines {
        let partition = WIDTH / (background_lines + 1);
        let x2 =
            random_below(partition as u32)? as i32 + WIDTH * (index + 1) / (background_lines + 1);
        let y2 = if y1 == 0 { 44 } else { 0 };
        draw_wide_line(bitmap, x1, y1, x2, y2, BLACK, 2);
        x1 = x2;
        y1 = y2;
    }

    if setup.noisy_dot_odds > 0 {
        for index in 0..PIXEL_COUNT {
            if random_below(100)? as i32 >= setup.noisy_dot_odds {
                continue;
            }
            let offset = PIXEL_OFFSET + index * 3;
            let white =
                bitmap[offset] == 0xff && bitmap[offset + 1] == 0xff && bitmap[offset + 2] == 0xff;
            if white {
                bitmap[offset] = random_byte()?;
                bitmap[offset + 1] = random_byte()?;
                bitmap[offset + 2] = random_byte()?;
            } else {
                bitmap[offset] = (random_byte()? & 0x7f).wrapping_add(bitmap[offset] >> 1);
                bitmap[offset + 1] = (random_byte()? & 0x7f).wrapping_add(bitmap[offset + 1] >> 1);
                bitmap[offset + 2] = (random_byte()? & 0x7f).wrapping_add(bitmap[offset + 2] >> 1);
            }
        }
    }
    Ok(())
}

fn draw_wide_line(
    bitmap: &mut [u8; VALID_CODE_BITMAP_LEN],
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    color: Bgr,
    width: i32,
) {
    for offset in 0..width {
        if (x1 - x2).abs() < (y1 - y2).abs() {
            bresenham(bitmap, x1 + offset, y1, x2 + offset, y2, color);
        } else {
            bresenham(bitmap, x1, y1 + offset, x2, y2 + offset, color);
        }
    }
}

fn bresenham(
    bitmap: &mut [u8; VALID_CODE_BITMAP_LEN],
    x1: i32,
    y1: i32,
    mut x2: i32,
    mut y2: i32,
    color: Bgr,
) {
    if !(0..WIDTH).contains(&x1)
        || !(0..HEIGHT).contains(&y1)
        || !(0..WIDTH).contains(&x2)
        || !(0..HEIGHT).contains(&y2)
    {
        return;
    }

    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    if dx == 0 {
        let limit = y1.max(y2);
        if y1 < y2 {
            x2 = x1;
            y2 = y1;
        }
        while y2 < limit {
            set_pixel(bitmap, x2, y2, color);
            y2 += 1;
        }
        return;
    }

    let slope = (y2 - y1) as f64 / (x2 - x1) as f64;
    if !(-1.0..=1.0).contains(&slope) {
        let straight = dx * 2;
        let mut error = straight - dy;
        let diagonal = (dx - dy) * 2;
        let limit = y1.max(y2);
        if y1 < y2 {
            x2 = x1;
            y2 = y1;
        }
        set_pixel(bitmap, x2, y2, color);
        while y2 < limit {
            y2 += 1;
            let increment = if error >= 0 {
                x2 += if slope <= 0.0 { -1 } else { 1 };
                diagonal
            } else {
                straight
            };
            error += increment;
            set_pixel(bitmap, x2, y2, color);
        }
    } else {
        let straight = dy * 2;
        let diagonal = (dy - dx) * 2;
        let limit = x1.max(x2);
        if x1 < x2 {
            y2 = y1;
            x2 = x1;
        }
        set_pixel(bitmap, x2, y2, color);
        let mut error = straight - dx;
        while x2 < limit {
            x2 += 1;
            let increment = if error >= 0 {
                y2 += if slope <= 0.0 { -1 } else { 1 };
                diagonal
            } else {
                straight
            };
            error += increment;
            set_pixel(bitmap, x2, y2, color);
        }
    }
}

fn set_pixel(bitmap: &mut [u8; VALID_CODE_BITMAP_LEN], x: i32, y: i32, color: Bgr) {
    if !(0..WIDTH).contains(&x) || !(0..HEIGHT).contains(&y) {
        return;
    }
    let offset = PIXEL_OFFSET + ((HEIGHT - 1 - y) * WIDTH + x) as usize * 3;
    bitmap[offset] = color.blue;
    bitmap[offset + 1] = color.green;
    bitmap[offset + 2] = color.red;
}

fn random_below(limit: u32) -> Result<u32, ValidCodeError> {
    debug_assert_ne!(limit, 0);
    Ok(getrandom::u32()? % limit)
}

fn random_byte() -> Result<u8, ValidCodeError> {
    Ok(getrandom::u32()? as u8)
}

fn freetype_error(error: freetype::Error) -> ValidCodeError {
    ValidCodeError::FreeType(format!("{error:?}"))
}

fn put_u16(destination: &mut [u8], offset: usize, value: u16) {
    destination[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(destination: &mut [u8], offset: usize, value: u32) {
    destination[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_i32(destination: &mut [u8], offset: usize, value: i32) {
    destination[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
