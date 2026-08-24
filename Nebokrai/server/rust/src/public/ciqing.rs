//! Конфигурация CiQing Miracle.
//!
//! `CCiQingSetup::ReadSetupFile/AddByteToArray` подтверждены точными
//! World/Game EXE/PDB, а `DeByteFromArray` —
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`. Исходный owner:
//! `e:\svn\fengyun_russia_dev\public\ciqing.cpp/.h`. Exact compose lookup и
//! `RandChoise` используют insertion order и отдельный `random(0x2711)`;
//! применение к player остаётся у GameServer owner-а.
//!
//! Wire содержит три insertion-order секции: make records по шесть `u32`,
//! compose records и improve records по три `u32`; все counts signed `i32`.
//! Compose точно передаёт `source_a` дважды, затем `source_b, money,
//! probability, crystal, result_count` и пары `probability/result`. Это не
//! исправлено как опечатка: оригинал World serializer и Game decoder совместно
//! подтверждают наблюдаемый positional quirk. Rust хранит named поля и `Vec`,
//! но пишет доказанный порядок явно little-endian. Declared result count
//! остаётся частью owner-state, тогда как wire, как оригинал, берёт реальный
//! размер result-vector. Невозможный signed count блокирует append до изменения
//! destination.
//!
//! Text-loader открывает точный `/data/ciqing.ini`: при отсутствующем ресурсе
//! прежнее состояние сохраняется, а после успешного open сначала очищаются все
//! три списка. Затем whitespace stream читает label/count и записи трёх секций;
//! original names разрешаются через уже загруженный `CGoodsFactory`, miss даёт
//! `0`. Loader возвращает `0` только при null path/open
//! failure и `1` после любого открытого stream. После stream fail MSVC оставлял
//! default values и продолжал declared loops; Rust сохраняет это для
//! представимых counts, но отклоняет count больше размера самого source, чтобы
//! не переносить конфигурационный DoS/OOM как часть поведения Miracle.
//! Game decoder очищает три vector-а, сохраняет только полные records и
//! безопасно трактует отрицательные counts как пустые секции вместо legacy
//! unbounded-read. Второй `source_a` wire scalar читается и намеренно
//! отбрасывается: парный serializer всегда дублирует первый, а named owner не
//! назначает несовместимому payload придуманную игровую семантику.

use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CiQingMakeNode {
    pub(crate) destination_base_index: u32,
    pub(crate) equipment_position: u32,
    pub(crate) source_a_base_index: u32,
    pub(crate) source_a_count: u32,
    pub(crate) source_b_base_index: u32,
    pub(crate) source_b_count: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CiQingComposeNode {
    pub(crate) source_a_base_index: u32,
    pub(crate) source_b_base_index: u32,
    pub(crate) compose_probability: u32,
    pub(crate) money: u32,
    pub(crate) crystal_count: u32,
    pub(crate) declared_result_count: u32,
    pub(crate) results: Vec<(u32, u32)>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CiQingImproveNode {
    pub(crate) level: u32,
    pub(crate) base_index: u32,
    pub(crate) probability: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CCiQingSetup {
    make: Vec<CiQingMakeNode>,
    compose: Vec<CiQingComposeNode>,
    improve: Vec<CiQingImproveNode>,
}

impl CCiQingSetup {
    /// Перечитывает три секции из уже выбранного caller-ом resource backend-а.
    pub(crate) fn read_setup_file(
        &mut self,
        source: Option<&[u8]>,
        mut query_goods_id: impl FnMut(&[u8]) -> u32,
    ) -> bool {
        let Some(source) = source else {
            return false;
        };
        self.clear();

        let mut input = CiQingTokenStream::new(source);
        let _make_label = input.read_bytes();
        let make_count = input.read_u32();
        if !count_fits_source(make_count, source) {
            return false;
        }
        for _ in 0..make_count {
            let destination = input.read_bytes();
            let equipment_position = input.read_u32();
            let source_a = input.read_bytes();
            let source_a_count = input.read_u32();
            let source_b = input.read_bytes();
            let source_b_count = input.read_u32();
            self.make.push(CiQingMakeNode {
                destination_base_index: query_goods_id(&destination),
                equipment_position,
                source_a_base_index: query_goods_id(&source_a),
                source_a_count,
                source_b_base_index: query_goods_id(&source_b),
                source_b_count,
            });
        }

        let _compose_label = input.read_bytes();
        let compose_count = input.read_u32();
        if !count_fits_source(compose_count, source) {
            return false;
        }
        for _ in 0..compose_count {
            let source_a = input.read_bytes();
            let source_b = input.read_bytes();
            let compose_probability = input.read_u32();
            let money = input.read_u32();
            let crystal_count = input.read_u32();
            let declared_result_count = input.read_u32();
            if !count_fits_source(declared_result_count, source) {
                return false;
            }
            let source_a_base_index = query_goods_id(&source_a);
            let source_b_base_index = query_goods_id(&source_b);
            let mut results = Vec::new();
            for _ in 0..declared_result_count {
                let probability = input.read_u32();
                let result = input.read_bytes();
                results.push((probability, query_goods_id(&result)));
            }
            self.compose.push(CiQingComposeNode {
                source_a_base_index,
                source_b_base_index,
                compose_probability,
                money,
                crystal_count,
                declared_result_count,
                results,
            });
        }

        let _improve_label = input.read_bytes();
        let improve_count = input.read_u32();
        if !count_fits_source(improve_count, source) {
            return false;
        }
        for _ in 0..improve_count {
            let level = input.read_u32();
            let base_name = input.read_bytes();
            let probability = input.read_u32();
            let base_index = query_goods_id(&base_name);
            self.improve.push(CiQingImproveNode {
                level,
                base_index,
                probability,
            });
        }
        true
    }

    pub(crate) fn clear(&mut self) {
        self.make.clear();
        self.compose.clear();
        self.improve.clear();
    }

    pub(crate) fn push_make(&mut self, node: CiQingMakeNode) {
        self.make.push(node);
    }

    pub(crate) fn push_compose(&mut self, node: CiQingComposeNode) {
        self.compose.push(node);
    }

    pub(crate) fn push_improve(&mut self, node: CiQingImproveNode) {
        self.improve.push(node);
    }

    pub(crate) fn make(&self) -> &[CiQingMakeNode] {
        &self.make
    }

    pub(crate) fn compose(&self) -> &[CiQingComposeNode] {
        &self.compose
    }

    pub(crate) fn compute_node(&self, source_a: u32, source_b: u32) -> Option<CiQingComposeNode> {
        self.compose
            .iter()
            .find(|node| {
                node.source_a_base_index == source_a && node.source_b_base_index == source_b
            })
            .cloned()
    }

    /// Exact `RandChoise`: отдельный `random(0x2711)`, затем signed
    /// cumulative subtraction и first result при остатке `< 1`.
    pub(crate) fn random_choice(
        node: &CiQingComposeNode,
        mut random: impl FnMut(i32) -> i32,
    ) -> u32 {
        let mut roll = i64::from(random(0x2711));
        for &(probability, result) in &node.results {
            roll -= i64::from(probability);
            if roll < 1 {
                return result;
            }
        }
        0
    }

    pub(crate) fn improve(&self) -> &[CiQingImproveNode] {
        &self.improve
    }

    pub(crate) fn add_byte_to_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), CiQingSerializationBlock> {
        let mut payload = Vec::new();
        write_ciqing_count(&mut payload, CiQingCountSection::Make, self.make.len())?;
        for node in &self.make {
            write_u32_fields(
                &mut payload,
                &[
                    node.destination_base_index,
                    node.equipment_position,
                    node.source_a_base_index,
                    node.source_a_count,
                    node.source_b_base_index,
                    node.source_b_count,
                ],
            );
        }

        write_ciqing_count(
            &mut payload,
            CiQingCountSection::Compose,
            self.compose.len(),
        )?;
        for (compose_index, node) in self.compose.iter().enumerate() {
            write_u32_fields(
                &mut payload,
                &[
                    node.source_a_base_index,
                    node.source_a_base_index,
                    node.source_b_base_index,
                    node.money,
                    node.compose_probability,
                    node.crystal_count,
                ],
            );
            write_ciqing_count(
                &mut payload,
                CiQingCountSection::ComposeResults { compose_index },
                node.results.len(),
            )?;
            for &(probability, result_base_index) in &node.results {
                write_u32_fields(&mut payload, &[probability, result_base_index]);
            }
        }

        write_ciqing_count(
            &mut payload,
            CiQingCountSection::Improve,
            self.improve.len(),
        )?;
        for node in &self.improve {
            write_u32_fields(
                &mut payload,
                &[node.level, node.base_index, node.probability],
            );
        }
        destination.extend_from_slice(&payload);
        Ok(())
    }

    /// Декодирует Game `DeByteFromArray`, продвигая общий startup cursor.
    pub(crate) fn de_byte_from_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<CiQingDecodeReport, CiQingDecodeError> {
        self.clear();

        let make_count = read_wire_i32(source, cursor, "make count")?;
        for _ in 0..make_count.max(0) {
            self.make.push(CiQingMakeNode {
                destination_base_index: read_wire_u32(
                    source,
                    cursor,
                    "make destination base index",
                )?,
                equipment_position: read_wire_u32(source, cursor, "make equipment position")?,
                source_a_base_index: read_wire_u32(source, cursor, "make source A base index")?,
                source_a_count: read_wire_u32(source, cursor, "make source A count")?,
                source_b_base_index: read_wire_u32(source, cursor, "make source B base index")?,
                source_b_count: read_wire_u32(source, cursor, "make source B count")?,
            });
        }

        let compose_count = read_wire_i32(source, cursor, "compose count")?;
        for _ in 0..compose_count.max(0) {
            let source_a_base_index = read_wire_u32(source, cursor, "compose source A base index")?;
            let _duplicated_source_a =
                read_wire_u32(source, cursor, "compose duplicated source A")?;
            let source_b_base_index = read_wire_u32(source, cursor, "compose source B base index")?;
            let money = read_wire_u32(source, cursor, "compose money")?;
            let compose_probability = read_wire_u32(source, cursor, "compose probability")?;
            let crystal_count = read_wire_u32(source, cursor, "compose crystal count")?;
            let result_count = read_wire_i32(source, cursor, "compose result count")?;
            let mut results = Vec::new();
            for _ in 0..result_count.max(0) {
                results.push((
                    read_wire_u32(source, cursor, "compose result probability")?,
                    read_wire_u32(source, cursor, "compose result base index")?,
                ));
            }
            self.compose.push(CiQingComposeNode {
                source_a_base_index,
                source_b_base_index,
                compose_probability,
                money,
                crystal_count,
                declared_result_count: result_count.max(0) as u32,
                results,
            });
        }

        let improve_count = read_wire_i32(source, cursor, "improve count")?;
        for _ in 0..improve_count.max(0) {
            self.improve.push(CiQingImproveNode {
                level: read_wire_u32(source, cursor, "improve level")?,
                base_index: read_wire_u32(source, cursor, "improve base index")?,
                probability: read_wire_u32(source, cursor, "improve probability")?,
            });
        }

        Ok(CiQingDecodeReport {
            make: self.make.len(),
            compose: self.compose.len(),
            improve: self.improve.len(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CiQingCountSection {
    Make,
    Compose,
    ComposeResults { compose_index: usize },
    Improve,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CiQingSerializationBlock {
    pub(crate) section: CiQingCountSection,
    pub(crate) count: usize,
}

impl fmt::Display for CiQingSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CCiQingSetup {:?} содержит {} записей вне signed 32-битного диапазона",
            self.section, self.count
        )
    }
}

impl Error for CiQingSerializationBlock {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CiQingDecodeReport {
    pub(crate) make: usize,
    pub(crate) compose: usize,
    pub(crate) improve: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CiQingDecodeError {
    pub(crate) field: &'static str,
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

impl fmt::Display for CiQingDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CiQing snapshot, поле {} на {}: нужно {}, доступно {}",
            self.field, self.offset, self.needed, self.available
        )
    }
}

impl Error for CiQingDecodeError {}

fn read_wire_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, CiQingDecodeError> {
    Ok(i32::from_le_bytes(read_wire_array(source, cursor, field)?))
}

fn read_wire_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, CiQingDecodeError> {
    Ok(u32::from_le_bytes(read_wire_array(source, cursor, field)?))
}

fn read_wire_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], CiQingDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(N)) else {
        return Err(CiQingDecodeError {
            field,
            offset,
            needed: N,
            available,
        });
    };
    *cursor += N;
    Ok(bytes.try_into().expect("размер CiQing scalar уже проверен"))
}

fn write_ciqing_count(
    destination: &mut Vec<u8>,
    section: CiQingCountSection,
    count: usize,
) -> Result<(), CiQingSerializationBlock> {
    let count = i32::try_from(count).map_err(|_| CiQingSerializationBlock { section, count })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn write_u32_fields(destination: &mut Vec<u8>, values: &[u32]) {
    for value in values {
        destination.extend_from_slice(&value.to_le_bytes());
    }
}

fn count_fits_source(count: u32, source: &[u8]) -> bool {
    usize::try_from(count).is_ok_and(|count| count <= source.len())
}

struct CiQingTokenStream<'a> {
    source: &'a [u8],
    position: usize,
    failed: bool,
}

impl<'a> CiQingTokenStream<'a> {
    fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            position: 0,
            failed: false,
        }
    }

    fn read_bytes(&mut self) -> Vec<u8> {
        if self.failed {
            return Vec::new();
        }
        while self
            .source
            .get(self.position)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.position += 1;
        }
        if self.position == self.source.len() {
            self.failed = true;
            return Vec::new();
        }
        let start = self.position;
        while self
            .source
            .get(self.position)
            .is_some_and(|byte| !byte.is_ascii_whitespace())
        {
            self.position += 1;
        }
        self.source[start..self.position].to_vec()
    }

    fn read_u32(&mut self) -> u32 {
        let token = self.read_bytes();
        if self.failed {
            return 0;
        }
        match parse_legacy_u32(&token) {
            Some(value) => value,
            None => {
                self.failed = true;
                0
            }
        }
    }
}

fn parse_legacy_u32(token: &[u8]) -> Option<u32> {
    let (negative, digits) = match token {
        [b'-', rest @ ..] => (true, rest),
        [b'+', rest @ ..] => (false, rest),
        _ => (false, token),
    };
    if digits.is_empty() {
        return None;
    }
    let mut value = 0u64;
    for &digit in digits {
        if !digit.is_ascii_digit() {
            return None;
        }
        value = value
            .checked_mul(10)?
            .checked_add(u64::from(digit - b'0'))?;
    }
    if negative {
        (value <= u64::from(u32::MAX) + 1).then(|| (value as u32).wrapping_neg())
    } else {
        u32::try_from(value).ok()
    }
}
