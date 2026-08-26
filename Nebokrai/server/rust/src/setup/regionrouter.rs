//! Общая маршрутизация `CRegionRouter`, подтверждённая WorldServer и
//! GameServer EXE/PDB.
//!
//! Wire пишет ordered regions, их entry/exit/range и ordered next-region точки;
//! map key задаёт порядок, но в payload идёт ID из value. `sendSelf` не читается.
//!
//! Поиск маршрута выбирает минимальное число hops, при равенстве — меньший
//! signed region key. Для промежуточных узлов выдаются entry и exit-to-next,
//! для конечного — entry и заданные X/Y; отсутствующая точка даёт `(0, 0)`.
//!
//! Loader очищает owner до открытия и читает позиционный whitespace-формат.
//! Duplicate keys сохраняют первое значение. Ошибка оставляет разобранный
//! префикс; `BinaryHeap` и `BTreeMap` заменяют линейный MSVC tree search.
//! Game decoder также очищает owner первым, читает unsigned region count и
//! signed transition counts и сохраняет first-wins обоих `std::map::insert`.

use std::cmp::Reverse;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionRoutePoint {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionNextNode {
    pub(crate) next_region_id: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionRouterNode {
    pub(crate) region_id: i32,
    pub(crate) entry: RegionRoutePoint,
    pub(crate) exit: RegionRoutePoint,
    pub(crate) exit_range: i32,
    pub(crate) next: BTreeMap<i32, RegionNextNode>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionRouter {
    nodes: BTreeMap<i32, RegionRouterNode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionRouteStep {
    pub(crate) region_id: i32,
    pub(crate) point: RegionRoutePoint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionRouterChangeOutcome {
    RegionNotFound {
        from_region_found: bool,
        to_region_found: bool,
    },
    Complete(Vec<RegionRouteStep>),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionRouterLoadReport {
    pub(crate) declared_regions: u32,
    pub(crate) inserted_regions: u32,
    pub(crate) duplicate_regions: u32,
    pub(crate) duplicate_transitions: u32,
    pub(crate) trailing_tokens: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegionRouterDecodeField {
    RegionCount,
    RegionId,
    EntryX,
    EntryY,
    ExitX,
    ExitY,
    ExitRange,
    TransitionCount,
    NextRegionId,
    TransitionX,
    TransitionY,
}

impl fmt::Display for RegionRouterDecodeField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::RegionCount => "число регионов",
            Self::RegionId => "ID региона",
            Self::EntryX => "entry X",
            Self::EntryY => "entry Y",
            Self::ExitX => "exit X",
            Self::ExitY => "exit Y",
            Self::ExitRange => "exit range",
            Self::TransitionCount => "число переходов",
            Self::NextRegionId => "ID следующего региона",
            Self::TransitionX => "transition X",
            Self::TransitionY => "transition Y",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionRouterDecodeError {
    pub(crate) field: RegionRouterDecodeField,
    pub(crate) region_index: Option<u32>,
    pub(crate) transition_index: Option<usize>,
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

impl fmt::Display for RegionRouterDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "RegionRouter поле {}", self.field)?;
        if let Some(region_index) = self.region_index {
            write!(formatter, " региона #{region_index}")?;
        }
        if let Some(transition_index) = self.transition_index {
            write!(formatter, " перехода #{transition_index}")?;
        }
        write!(
            formatter,
            " обрывается на {}: нужно {}, доступно {}",
            self.offset, self.needed, self.available
        )
    }
}

impl Error for RegionRouterDecodeError {}

#[derive(Debug)]
pub(crate) enum RegionRouterLoadError {
    Io(io::Error),
    MissingToken { field: &'static str },
    InvalidInteger { field: &'static str, token: Vec<u8> },
}

impl PartialEq for RegionRouterLoadError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(left), Self::Io(right)) => {
                left.kind() == right.kind() && left.to_string() == right.to_string()
            }
            (Self::MissingToken { field: left }, Self::MissingToken { field: right }) => {
                left == right
            }
            (
                Self::InvalidInteger {
                    field: left_field,
                    token: left_token,
                },
                Self::InvalidInteger {
                    field: right_field,
                    token: right_token,
                },
            ) => left_field == right_field && left_token == right_token,
            _ => false,
        }
    }
}

impl Eq for RegionRouterLoadError {}

impl RegionRouter {
    pub(crate) fn insert_node(
        &mut self,
        map_region_id: i32,
        node: RegionRouterNode,
    ) -> Option<RegionRouterNode> {
        self.nodes.insert(map_region_id, node)
    }

    pub(crate) fn clear(&mut self) {
        self.nodes.clear();
    }

    /// Декодирует GameServer snapshot. Оба исходных `std::map::insert`
    /// сохраняют первое значение duplicate key; текущий router очищается до
    /// чтения count, а незавершённый node не публикуется.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), RegionRouterDecodeError> {
        self.clear();
        let declared_regions = read_router_u32(
            source,
            cursor,
            RegionRouterDecodeField::RegionCount,
            None,
            None,
        )?;
        let mut duplicate_regions = 0usize;
        let mut duplicate_transitions = 0usize;

        for region_index in 0..declared_regions {
            let region_id = read_router_i32(
                source,
                cursor,
                RegionRouterDecodeField::RegionId,
                Some(region_index),
                None,
            )?;
            let entry = RegionRoutePoint {
                x: read_router_i32(
                    source,
                    cursor,
                    RegionRouterDecodeField::EntryX,
                    Some(region_index),
                    None,
                )?,
                y: read_router_i32(
                    source,
                    cursor,
                    RegionRouterDecodeField::EntryY,
                    Some(region_index),
                    None,
                )?,
            };
            let exit = RegionRoutePoint {
                x: read_router_i32(
                    source,
                    cursor,
                    RegionRouterDecodeField::ExitX,
                    Some(region_index),
                    None,
                )?,
                y: read_router_i32(
                    source,
                    cursor,
                    RegionRouterDecodeField::ExitY,
                    Some(region_index),
                    None,
                )?,
            };
            let exit_range = read_router_i32(
                source,
                cursor,
                RegionRouterDecodeField::ExitRange,
                Some(region_index),
                None,
            )?;
            let transition_count = read_router_i32(
                source,
                cursor,
                RegionRouterDecodeField::TransitionCount,
                Some(region_index),
                None,
            )?;
            let mut next = BTreeMap::new();
            for transition_index in 0..transition_count.max(0) as usize {
                let next_region_id = read_router_i32(
                    source,
                    cursor,
                    RegionRouterDecodeField::NextRegionId,
                    Some(region_index),
                    Some(transition_index),
                )?;
                let transition = RegionNextNode {
                    next_region_id,
                    x: read_router_i32(
                        source,
                        cursor,
                        RegionRouterDecodeField::TransitionX,
                        Some(region_index),
                        Some(transition_index),
                    )?,
                    y: read_router_i32(
                        source,
                        cursor,
                        RegionRouterDecodeField::TransitionY,
                        Some(region_index),
                        Some(transition_index),
                    )?,
                };
                match next.entry(next_region_id) {
                    Entry::Vacant(entry) => {
                        entry.insert(transition);
                    }
                    Entry::Occupied(_) => duplicate_transitions += 1,
                }
            }

            let node = RegionRouterNode {
                region_id,
                entry,
                exit,
                exit_range,
                next,
            };
            match self.nodes.entry(region_id) {
                Entry::Vacant(entry) => {
                    entry.insert(node);
                }
                Entry::Occupied(_) => duplicate_regions += 1,
            }
        }

        tracing::trace!(declared_regions, regions = self.nodes.len(), transitions = self.nodes.values().map(|node| node.next.len()).sum::<usize>(), duplicate_regions, duplicate_transitions, "маршрутизатор регионов декодирован");
        Ok(())
    }

    pub(crate) fn load_router_setup(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<RegionRouterLoadReport, RegionRouterLoadError> {
        self.clear();
        let bytes = fs::read(path).map_err(RegionRouterLoadError::Io)?;
        self.load_router_setup_bytes(&bytes)
    }

    pub(crate) fn load_router_setup_bytes(
        &mut self,
        bytes: &[u8],
    ) -> Result<RegionRouterLoadReport, RegionRouterLoadError> {
        self.clear();
        let mut tokens = RegionRouterTokens::new(&bytes);
        tokens.skip("region count label")?;
        let declared_regions = tokens.read_u32("region count")?;
        let mut report = RegionRouterLoadReport {
            declared_regions,
            ..RegionRouterLoadReport::default()
        };

        for _ in 0..declared_regions {
            tokens.skip("region ID label")?;
            let region_id = tokens.read_i32("region ID")?;
            tokens.skip("entry point label")?;
            let entry = RegionRoutePoint {
                x: tokens.read_i32("entry X")?,
                y: tokens.read_i32("entry Y")?,
            };
            tokens.skip("exit point label")?;
            let exit = RegionRoutePoint {
                x: tokens.read_i32("exit X")?,
                y: tokens.read_i32("exit Y")?,
            };
            tokens.skip("exit range label")?;
            let exit_range = tokens.read_i32("exit range")?;
            tokens.skip("transition count label")?;
            let transition_count = tokens.read_u32("transition count")?;
            let mut next = BTreeMap::new();
            for _ in 0..transition_count {
                let next_region_id = tokens.read_i32("next region ID")?;
                let transition = RegionNextNode {
                    next_region_id,
                    x: tokens.read_i32("transition X")?,
                    y: tokens.read_i32("transition Y")?,
                };
                match next.entry(next_region_id) {
                    Entry::Vacant(entry) => {
                        entry.insert(transition);
                    }
                    Entry::Occupied(_) => {
                        report.duplicate_transitions += 1;
                    }
                }
            }

            let node = RegionRouterNode {
                region_id,
                entry,
                exit,
                exit_range,
                next,
            };
            match self.nodes.entry(region_id) {
                Entry::Vacant(entry) => {
                    entry.insert(node);
                    report.inserted_regions += 1;
                }
                Entry::Occupied(_) => {
                    report.duplicate_regions += 1;
                }
            }
        }
        report.trailing_tokens = tokens.remaining();
        Ok(report)
    }

    /// Exact GameServer `IsConectRegion`: разрешается только direct edge,
    /// текущая точка должна попадать в квадрат `lOutRange` вокруг transition,
    /// а success возвращает `pOut` destination node. Сравнение через `f64`
    /// точно для разности двух `i32` и сохраняет исходный floating `ABS`, не
    /// допуская signed overflow.
    pub(crate) fn connected_region_destination(
        &self,
        from_region: i32,
        to_region: i32,
        current: RegionRoutePoint,
    ) -> Option<RegionRoutePoint> {
        if from_region == to_region {
            return None;
        }
        let source = self.nodes.get(&from_region)?;
        let destination = self.nodes.get(&to_region)?;
        let transition = source.next.get(&to_region)?;
        let range = f64::from(source.exit_range);
        let x_distance = (f64::from(transition.x) - f64::from(current.x)).abs();
        let y_distance = (f64::from(transition.y) - f64::from(current.y)).abs();
        (x_distance <= range && y_distance <= range).then_some(destination.exit)
    }

    /// Восстанавливает оригинал World `ChageRegionRouter` с детерминированным
    /// signed-key tie-break исходного `std::map`.
    pub(crate) fn change_region_router(
        &self,
        from_region: i32,
        to_region: i32,
        target: RegionRoutePoint,
    ) -> RegionRouterChangeOutcome {
        let from_region_found = self.nodes.contains_key(&from_region);
        let to_region_found = self.nodes.contains_key(&to_region);
        if !from_region_found || !to_region_found {
            return RegionRouterChangeOutcome::RegionNotFound {
                from_region_found,
                to_region_found,
            };
        }
        if from_region == to_region {
            return RegionRouterChangeOutcome::Complete(vec![RegionRouteStep {
                region_id: to_region,
                point: target,
            }]);
        }

        const INITIAL_ROUTE_VALUE: i32 = 20_000_000;

        let mut distance = self
            .nodes
            .keys()
            .copied()
            .map(|region_id| (region_id, INITIAL_ROUTE_VALUE))
            .collect::<BTreeMap<_, _>>();
        let mut predecessor = BTreeMap::<i32, i32>::new();
        let mut settled = BTreeSet::<i32>::new();
        let mut pending = BinaryHeap::<Reverse<(i32, i32)>>::new();
        distance.insert(from_region, 0);
        pending.push(Reverse((0, from_region)));

        while let Some(Reverse((current_distance, current_region))) = pending.pop() {
            if settled.contains(&current_region)
                || distance.get(&current_region).copied() != Some(current_distance)
            {
                continue;
            }
            settled.insert(current_region);

            let current_node = self
                .nodes
                .get(&current_region)
                .expect("pending содержит только ключи RegionRouter");
            // Машина сравнивает destination с `stRouterNode::lRegionId`, хотя
            // путь и lookup-и строит по map key. Несовпадение полей сохраняем.
            if current_node.region_id == to_region {
                break;
            }

            let next_distance = current_distance + 1;
            for next_region in current_node.next.keys().copied() {
                let Some(known_distance) = distance.get_mut(&next_region) else {
                    continue;
                };
                if next_distance < *known_distance {
                    *known_distance = next_distance;
                    predecessor.insert(next_region, current_region);
                    pending.push(Reverse((next_distance, next_region)));
                }
            }
        }

        if !settled.contains(&to_region) {
            return RegionRouterChangeOutcome::Complete(Vec::new());
        }

        let mut reverse_path = vec![to_region];
        let mut current_region = to_region;
        while current_region != from_region {
            let Some(previous_region) = predecessor.get(&current_region).copied() else {
                return RegionRouterChangeOutcome::Complete(Vec::new());
            };
            reverse_path.push(previous_region);
            current_region = previous_region;
        }
        reverse_path.reverse();

        let mut route = Vec::new();
        for (index, region_id) in reverse_path.iter().copied().enumerate() {
            let node = self
                .nodes
                .get(&region_id)
                .expect("reconstructed path содержит только ключи RegionRouter");
            if index == 0 {
                let next_region = reverse_path[index + 1];
                route.push(RegionRouteStep {
                    region_id,
                    point: node.next_point(next_region),
                });
                continue;
            }

            route.push(RegionRouteStep {
                region_id,
                point: node.entry,
            });
            let point = reverse_path
                .get(index + 1)
                .copied()
                .map(|next_region| node.next_point(next_region))
                .unwrap_or(target);
            route.push(RegionRouteStep { region_id, point });
        }
        RegionRouterChangeOutcome::Complete(route)
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), RegionRouterSerializeError> {
        write_count(destination, self.nodes.len(), None)?;
        for node in self.nodes.values() {
            destination.extend_from_slice(&node.region_id.to_le_bytes());
            destination.extend_from_slice(&node.entry.x.to_le_bytes());
            destination.extend_from_slice(&node.entry.y.to_le_bytes());
            destination.extend_from_slice(&node.exit.x.to_le_bytes());
            destination.extend_from_slice(&node.exit.y.to_le_bytes());
            destination.extend_from_slice(&node.exit_range.to_le_bytes());
            write_count(destination, node.next.len(), Some(node.region_id))?;
            for next in node.next.values() {
                destination.extend_from_slice(&next.next_region_id.to_le_bytes());
                destination.extend_from_slice(&next.x.to_le_bytes());
                destination.extend_from_slice(&next.y.to_le_bytes());
            }
        }
        Ok(())
    }
}

impl RegionRouterNode {
    fn next_point(&self, next_region: i32) -> RegionRoutePoint {
        self.next
            .get(&next_region)
            .map(|next| RegionRoutePoint {
                x: next.x,
                y: next.y,
            })
            .unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionRouterSerializeError {
    pub(crate) region_id: Option<i32>,
    pub(crate) count: usize,
}

impl fmt::Display for RegionRouterSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.region_id {
            Some(region_id) => write!(
                formatter,
                "region {region_id} содержит {} переходов вне signed 32-битного диапазона",
                self.count
            ),
            None => write!(
                formatter,
                "RegionRouter содержит {} регионов вне signed 32-битного диапазона",
                self.count
            ),
        }
    }
}

impl Error for RegionRouterSerializeError {}

impl fmt::Display for RegionRouterLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "не удалось прочитать RegionRouter: {error}"),
            Self::MissingToken { field } => {
                write!(formatter, "в RegionRouter отсутствует token поля {field}")
            }
            Self::InvalidInteger { field, token } => write!(
                formatter,
                "в RegionRouter поле {field} содержит нецелое значение {:?}",
                String::from_utf8_lossy(token)
            ),
        }
    }
}

impl Error for RegionRouterLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::MissingToken { .. } | Self::InvalidInteger { .. } => None,
        }
    }
}

struct RegionRouterTokens<'a> {
    tokens: Vec<&'a [u8]>,
    cursor: usize,
}

impl<'a> RegionRouterTokens<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            tokens: bytes
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty())
                .collect(),
            cursor: 0,
        }
    }

    fn skip(&mut self, field: &'static str) -> Result<(), RegionRouterLoadError> {
        self.next(field).map(|_| ())
    }

    fn read_i32(&mut self, field: &'static str) -> Result<i32, RegionRouterLoadError> {
        self.read_integer(field)
    }

    fn read_u32(&mut self, field: &'static str) -> Result<u32, RegionRouterLoadError> {
        self.read_integer(field)
    }

    fn read_integer<T>(&mut self, field: &'static str) -> Result<T, RegionRouterLoadError>
    where
        T: std::str::FromStr,
    {
        let token = self.next(field)?;
        let value = std::str::from_utf8(token)
            .ok()
            .and_then(|value| value.parse().ok());
        value.ok_or_else(|| RegionRouterLoadError::InvalidInteger {
            field,
            token: token.to_vec(),
        })
    }

    fn next(&mut self, field: &'static str) -> Result<&'a [u8], RegionRouterLoadError> {
        let token = self
            .tokens
            .get(self.cursor)
            .copied()
            .ok_or(RegionRouterLoadError::MissingToken { field })?;
        self.cursor += 1;
        Ok(token)
    }

    fn remaining(&self) -> usize {
        self.tokens.len().saturating_sub(self.cursor)
    }
}

fn write_count(
    destination: &mut Vec<u8>,
    count: usize,
    region_id: Option<i32>,
) -> Result<(), RegionRouterSerializeError> {
    let count_i32 =
        i32::try_from(count).map_err(|_| RegionRouterSerializeError { region_id, count })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

fn read_router_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: RegionRouterDecodeField,
    region_index: Option<u32>,
    transition_index: Option<usize>,
) -> Result<[u8; N], RegionRouterDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(N)) else {
        return Err(RegionRouterDecodeError {
            field,
            region_index,
            transition_index,
            offset,
            needed: N,
            available,
        });
    };
    *cursor += N;
    Ok(bytes
        .try_into()
        .expect("RegionRouter wire slice имеет запрошенную длину"))
}

fn read_router_i32(
    source: &[u8],
    cursor: &mut usize,
    field: RegionRouterDecodeField,
    region_index: Option<u32>,
    transition_index: Option<usize>,
) -> Result<i32, RegionRouterDecodeError> {
    Ok(i32::from_le_bytes(read_router_array(
        source,
        cursor,
        field,
        region_index,
        transition_index,
    )?))
}

fn read_router_u32(
    source: &[u8],
    cursor: &mut usize,
    field: RegionRouterDecodeField,
    region_index: Option<u32>,
    transition_index: Option<usize>,
) -> Result<u32, RegionRouterDecodeError> {
    Ok(u32::from_le_bytes(read_router_array(
        source,
        cursor,
        field,
        region_index,
        transition_index,
    )?))
}
