use bitvec::prelude::*;
use std::collections::VecDeque;
use std::ops::{Bound, Range, RangeBounds};

// Tipo de bloque para almacenar bits - usando usize para mejor rendimiento
type BitBlock = BitArray<[usize; 8]>; // 512 bits por bloque (8 * 64)
const BITS_PER_BLOCK: usize = 512;

#[derive(Debug, Clone)]
pub struct BitVecDeque {
    blocks: VecDeque<BitBlock>,
    front_offset: usize, // Offset desde el inicio del primer bloque
    back_used: usize,    // Bits usados en el último bloque
    len: usize,          // Número total de bits
}

pub struct BitDrain<'a> {
    deque: &'a mut BitVecDeque,
    range: Range<usize>,
    drained: usize,
}

impl<'a> Iterator for BitDrain<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.drained >= self.range.len() {
            return None;
        }

        let index = self.range.start;
        let bit = self.deque.get_unchecked(index);

        // Remover el bit en la posición actual
        self.deque.remove_at(index);
        self.drained += 1;

        Some(bit)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.range.len() - self.drained;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for BitDrain<'a> {}

impl BitVecDeque {
    pub fn new() -> Self {
        Self {
            blocks: VecDeque::new(),
            front_offset: 0,
            back_used: 0,
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let blocks_needed = (capacity + BITS_PER_BLOCK - 1) / BITS_PER_BLOCK;
        Self {
            blocks: VecDeque::with_capacity(blocks_needed),
            front_offset: 0,
            back_used: 0,
            len: 0,
        }
    }

    pub fn push_back(&mut self, bit: bool) {
        // Si no hay bloques o el último está lleno
        if self.blocks.is_empty() || self.back_used >= BITS_PER_BLOCK {
            self.blocks.push_back(BitBlock::ZERO);
            self.back_used = 0;
        }

        // Establecer el bit en el último bloque
        if let Some(block) = self.blocks.back_mut() {
            block.set(self.back_used, bit);
            self.back_used += 1;
            self.len += 1;
        }
    }

    pub fn push_front(&mut self, bit: bool) {
        // Caso especial: primer elemento
        if self.len == 0 {
            self.blocks.push_back(BitBlock::ZERO);
            self.blocks.back_mut().unwrap().set(0, bit);
            self.front_offset = 0;
            self.back_used = 1;
            self.len = 1;
            return;
        }

        // Si no hay espacio al frente, crear nuevo bloque
        if self.front_offset == 0 {
            self.blocks.push_front(BitBlock::ZERO);
            self.front_offset = BITS_PER_BLOCK;
        }

        self.front_offset -= 1;
        if let Some(block) = self.blocks.front_mut() {
            block.set(self.front_offset, bit);
        }
        self.len += 1;
    }

    pub fn pop_back(&mut self) -> Option<bool> {
        if self.len == 0 {
            return None;
        }

        // Si el último bloque está vacío, usar el anterior
        if self.back_used == 0 {
            if self.blocks.len() > 1 {
                self.blocks.pop_back();
                self.back_used = BITS_PER_BLOCK;
            } else {
                return None;
            }
        }

        self.back_used -= 1;
        let bit = self.blocks.back().unwrap()[self.back_used];
        self.len -= 1;

        // Limpiar si está vacío
        if self.len == 0 {
            self.blocks.clear();
            self.front_offset = 0;
            self.back_used = 0;
        }

        Some(bit)
    }

    pub fn pop_front(&mut self) -> Option<bool> {
        if self.len == 0 {
            return None;
        }

        let bit = self.blocks.front().unwrap()[self.front_offset];
        self.front_offset += 1;
        self.len -= 1;

        // Si completamos el primer bloque, removerlo
        if self.front_offset >= BITS_PER_BLOCK && self.blocks.len() > 1 {
            self.blocks.pop_front();
            self.front_offset = 0;
        }

        // Limpiar si está vacío
        if self.len == 0 {
            self.blocks.clear();
            self.front_offset = 0;
            self.back_used = 0;
        }

        Some(bit)
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        if index >= self.len {
            return None;
        }
        Some(self.get_unchecked(index))
    }

    fn get_unchecked(&self, index: usize) -> bool {
        let bit_position = self.front_offset + index;
        let block_idx = bit_position / BITS_PER_BLOCK;
        let bit_offset = bit_position % BITS_PER_BLOCK;

        self.blocks[block_idx][bit_offset]
    }

    fn remove_at(&mut self, index: usize) {
        if index >= self.len {
            return;
        }

        // Versión optimizada: mover bits en lugar de reconstruir todo
        for i in index..self.len - 1 {
            let next_bit = self.get_unchecked(i + 1);
            self.set_unchecked(i, next_bit);
        }

        // Decrementar longitud
        self.len -= 1;

        // Ajustar back_used
        if self.back_used > 0 {
            self.back_used -= 1;
        } else if self.blocks.len() > 1 {
            self.blocks.pop_back();
            self.back_used = BITS_PER_BLOCK - 1;
        }

        // Limpiar si está vacío
        if self.len == 0 {
            self.blocks.clear();
            self.front_offset = 0;
            self.back_used = 0;
        }
    }

    fn set_unchecked(&mut self, index: usize, bit: bool) {
        let bit_position = self.front_offset + index;
        let block_idx = bit_position / BITS_PER_BLOCK;
        let bit_offset = bit_position % BITS_PER_BLOCK;

        if let Some(block) = self.blocks.get_mut(block_idx) {
            block.set(bit_offset, bit);
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    // El método drain principal
    pub fn drain<R>(&mut self, range: R) -> BitDrain<'_>
    where
        R: RangeBounds<usize>,
    {
        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(&n) => n + 1,
            Bound::Excluded(&n) => n,
            Bound::Unbounded => self.len,
        };

        assert!(start <= end, "start must be <= end");
        assert!(end <= self.len, "end must be <= len");

        BitDrain {
            deque: self,
            range: start..end,
            drained: 0,
        }
    }

    // Método drain optimizado para rangos grandes
    pub fn drain_range(&mut self, start: usize, end: usize) -> Vec<bool> {
        assert!(start <= end && end <= self.len);

        let mut result = Vec::with_capacity(end - start);

        // Extraer bits en el rango
        for i in start..end {
            result.push(self.get_unchecked(i));
        }

        // Mover bits después del rango hacia adelante
        for i in end..self.len {
            let bit = self.get_unchecked(i);
            self.set_unchecked(i - (end - start), bit);
        }

        // Ajustar longitud
        self.len -= end - start;

        // Recalcular back_used
        if self.len == 0 {
            self.blocks.clear();
            self.front_offset = 0;
            self.back_used = 0;
        } else {
            let last_bit_pos = self.front_offset + self.len - 1;
            self.back_used = (last_bit_pos % BITS_PER_BLOCK) + 1;

            // Remover bloques vacíos al final
            let needed_blocks =
                (self.front_offset + self.len + BITS_PER_BLOCK - 1) / BITS_PER_BLOCK;
            while self.blocks.len() > needed_blocks {
                self.blocks.pop_back();
            }
        }

        result
    }

    // Métodos útiles para testing y conversión
    pub fn to_vec(&self) -> Vec<bool> {
        (0..self.len).map(|i| self.get_unchecked(i)).collect()
    }

    pub fn from_bits<I: IntoIterator<Item = bool>>(bits: I) -> Self {
        let mut deque = Self::new();
        for bit in bits {
            deque.push_back(bit);
        }
        deque
    }

    // Método para crear desde BitVec
    pub fn from_bitvec(bitvec: BitVec) -> Self {
        let mut deque = Self::with_capacity(bitvec.len());
        for bit in bitvec {
            deque.push_back(bit);
        }
        deque
    }

    // Convertir a BitVec
    pub fn to_bitvec(&self) -> BitVec {
        let mut bitvec = BitVec::with_capacity(self.len);
        for i in 0..self.len {
            bitvec.push(self.get_unchecked(i));
        }
        bitvec
    }

    // Métodos adicionales útiles con bitvec
    pub fn append_bitvec(&mut self, other: &BitVec) {
        for bit in other {
            self.push_back(*bit);
        }
    }

    pub fn prepend_bitvec(&mut self, other: &BitVec) {
        // Insertar en orden reverso para mantener el orden
        for bit in other.iter().rev() {
            self.push_front(*bit);
        }
    }

    // Método para obtener un slice de bits como BitVec
    pub fn slice_to_bitvec(&self, start: usize, end: usize) -> BitVec {
        assert!(start <= end && end <= self.len);
        let mut result = BitVec::with_capacity(end - start);
        for i in start..end {
            result.push(self.get_unchecked(i));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let deque = BitVecDeque::new();
        assert_eq!(deque.len(), 0);
        assert!(deque.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let deque = BitVecDeque::with_capacity(1000);
        assert_eq!(deque.len(), 0);
        assert!(deque.is_empty());
    }

    #[test]
    fn test_push_back() {
        let mut deque = BitVecDeque::new();
        deque.push_back(true);
        deque.push_back(false);
        deque.push_back(true);

        assert_eq!(deque.len(), 3);
        assert_eq!(deque.get(0), Some(true));
        assert_eq!(deque.get(1), Some(false));
        assert_eq!(deque.get(2), Some(true));
    }

    #[test]
    fn test_push_front() {
        let mut deque = BitVecDeque::new();
        deque.push_front(true);
        deque.push_front(false);
        deque.push_front(true);

        assert_eq!(deque.len(), 3);
        assert_eq!(deque.get(0), Some(true));
        assert_eq!(deque.get(1), Some(false));
        assert_eq!(deque.get(2), Some(true));
    }

    #[test]
    fn test_pop_back() {
        let mut deque = BitVecDeque::from_bits([true, false, true]);

        assert_eq!(deque.pop_back(), Some(true));
        assert_eq!(deque.pop_back(), Some(false));
        assert_eq!(deque.pop_back(), Some(true));
        assert_eq!(deque.pop_back(), None);
        assert!(deque.is_empty());
    }

    #[test]
    fn test_pop_front() {
        let mut deque = BitVecDeque::from_bits([true, false, true]);

        assert_eq!(deque.pop_front(), Some(true));
        assert_eq!(deque.pop_front(), Some(false));
        assert_eq!(deque.pop_front(), Some(true));
        assert_eq!(deque.pop_front(), None);
        assert!(deque.is_empty());
    }

    #[test]
    fn test_get() {
        let deque = BitVecDeque::from_bits([true, false, true, false]);

        assert_eq!(deque.get(0), Some(true));
        assert_eq!(deque.get(1), Some(false));
        assert_eq!(deque.get(2), Some(true));
        assert_eq!(deque.get(3), Some(false));
        assert_eq!(deque.get(4), None);
    }

    #[test]
    fn test_mixed_operations() {
        let mut deque = BitVecDeque::new();

        deque.push_back(true);
        deque.push_front(false);
        deque.push_back(true);
        deque.push_front(false);

        // Deque should be: [false, false, true, true]
        assert_eq!(deque.len(), 4);
        assert_eq!(deque.to_vec(), vec![false, false, true, true]);

        assert_eq!(deque.pop_front(), Some(false));
        assert_eq!(deque.pop_back(), Some(true));

        // Deque should be: [false, true]
        assert_eq!(deque.to_vec(), vec![false, true]);
    }

    #[test]
    fn test_large_sequence() {
        let mut deque = BitVecDeque::new();
        let test_pattern: Vec<bool> = (0..1000).map(|i| i % 3 == 0).collect();

        // Push all bits
        for &bit in &test_pattern {
            deque.push_back(bit);
        }

        assert_eq!(deque.len(), 1000);
        assert_eq!(deque.to_vec(), test_pattern);

        // Pop all bits from front
        for expected_bit in test_pattern {
            assert_eq!(deque.pop_front(), Some(expected_bit));
        }

        assert!(deque.is_empty());
    }

    #[test]
    fn test_drain_middle() {
        let mut deque =
            BitVecDeque::from_bits([true, false, true, true, false, false, true, false]);

        let drained: Vec<bool> = deque.drain(2..5).collect();

        assert_eq!(drained, vec![true, true, false]);
        assert_eq!(deque.to_vec(), vec![true, false, false, true, false]);
    }

    #[test]
    fn test_drain_beginning() {
        let mut deque = BitVecDeque::from_bits([true, false, true, false]);

        let drained: Vec<bool> = deque.drain(..2).collect();

        assert_eq!(drained, vec![true, false]);
        assert_eq!(deque.to_vec(), vec![true, false]);
    }

    #[test]
    fn test_drain_end() {
        let mut deque = BitVecDeque::from_bits([true, false, true, false]);

        let drained: Vec<bool> = deque.drain(2..).collect();

        assert_eq!(drained, vec![true, false]);
        assert_eq!(deque.to_vec(), vec![true, false]);
    }

    #[test]
    fn test_drain_all() {
        let mut deque = BitVecDeque::from_bits([true, false, true]);

        let drained: Vec<bool> = deque.drain(..).collect();

        assert_eq!(drained, vec![true, false, true]);
        assert!(deque.is_empty());
    }

    #[test]
    fn test_drain_range() {
        let mut deque = BitVecDeque::from_bits([true, false, true, true, false, false, true]);

        let drained = deque.drain_range(1, 4);

        assert_eq!(drained, vec![false, true, true]);
        assert_eq!(deque.to_vec(), vec![true, false, false, true]);
    }

    #[test]
    fn test_from_bitvec() {
        let bitvec = bitvec![1, 0, 1, 0, 1];
        let deque = BitVecDeque::from_bitvec(bitvec);

        assert_eq!(deque.to_vec(), vec![true, false, true, false, true]);
    }

    #[test]
    fn test_to_bitvec() {
        let deque = BitVecDeque::from_bits([true, false, true, false]);
        let bitvec = deque.to_bitvec();

        assert_eq!(bitvec.len(), 4);
        assert_eq!(bitvec[0], true);
        assert_eq!(bitvec[1], false);
        assert_eq!(bitvec[2], true);
        assert_eq!(bitvec[3], false);
    }

    #[test]
    fn test_append_bitvec() {
        let mut deque = BitVecDeque::from_bits([true, false]);
        let bitvec = bitvec![1, 0, 1];

        deque.append_bitvec(&bitvec);

        assert_eq!(deque.to_vec(), vec![true, false, true, false, true]);
    }

    #[test]
    fn test_prepend_bitvec() {
        let mut deque = BitVecDeque::from_bits([true, false]);
        let bitvec = bitvec![1, 0, 1];

        deque.prepend_bitvec(&bitvec);

        assert_eq!(deque.to_vec(), vec![true, false, true, true, false]);
    }

    #[test]
    fn test_slice_to_bitvec() {
        let deque = BitVecDeque::from_bits([true, false, true, false, true]);
        let slice = deque.slice_to_bitvec(1, 4);

        assert_eq!(slice.len(), 3);
        assert_eq!(slice[0], false);
        assert_eq!(slice[1], true);
        assert_eq!(slice[2], false);
    }

    #[test]
    fn test_empty_operations() {
        let mut deque = BitVecDeque::new();

        assert_eq!(deque.pop_front(), None);
        assert_eq!(deque.pop_back(), None);
        assert_eq!(deque.get(0), None);

        let drained: Vec<bool> = deque.drain(..).collect();
        assert!(drained.is_empty());
    }

    #[test]
    fn test_single_element() {
        let mut deque = BitVecDeque::new();
        deque.push_back(true);

        assert_eq!(deque.len(), 1);
        assert_eq!(deque.get(0), Some(true));
        assert_eq!(deque.pop_back(), Some(true));
        assert!(deque.is_empty());
    }

    #[test]
    fn test_block_boundaries() {
        let mut deque = BitVecDeque::new();

        // Add more than one block worth of bits
        for i in 0..(BITS_PER_BLOCK + 100) {
            deque.push_back(i % 2 == 0);
        }

        assert_eq!(deque.len(), BITS_PER_BLOCK + 100);

        // Verify all bits are correct
        for i in 0..(BITS_PER_BLOCK + 100) {
            assert_eq!(deque.get(i), Some(i % 2 == 0));
        }

        // Remove from front
        for i in 0..50 {
            assert_eq!(deque.pop_front(), Some(i % 2 == 0));
        }

        assert_eq!(deque.len(), BITS_PER_BLOCK + 50);
    }

    #[test]
    fn test_remove_at_internal() {
        let mut deque = BitVecDeque::from_bits([true, false, true, false, true]);

        deque.remove_at(2); // Remove the middle 'true'

        assert_eq!(deque.to_vec(), vec![true, false, false, true]);
        assert_eq!(deque.len(), 4);
    }

    #[test]
    fn test_set_unchecked_internal() {
        let mut deque = BitVecDeque::from_bits([true, false, true]);

        deque.set_unchecked(1, true); // Change middle to true

        assert_eq!(deque.to_vec(), vec![true, true, true]);
    }

    #[test]
    #[should_panic(expected = "start must be <= end")]
    fn test_drain_invalid_range() {
        let mut deque = BitVecDeque::from_bits([true, false, true]);
        let _: Vec<bool> = deque.drain(2..1).collect();
    }

    #[test]
    #[should_panic(expected = "end must be <= len")]
    fn test_drain_out_of_bounds() {
        let mut deque = BitVecDeque::from_bits([true, false]);
        let _: Vec<bool> = deque.drain(0..5).collect();
    }

    #[test]
    fn test_drain_iterator_properties() {
        let mut deque = BitVecDeque::from_bits([true, false, true, false]);
        let mut drain = deque.drain(1..3);

        assert_eq!(drain.size_hint(), (2, Some(2)));
        assert_eq!(drain.len(), 2);

        assert_eq!(drain.next(), Some(false));
        assert_eq!(drain.size_hint(), (1, Some(1)));
        assert_eq!(drain.len(), 1);

        assert_eq!(drain.next(), Some(true));
        assert_eq!(drain.size_hint(), (0, Some(0)));
        assert_eq!(drain.len(), 0);

        assert_eq!(drain.next(), None);
    }
}
