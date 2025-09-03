// ...existing content moved from hdlc/src/bitvecdeque.rs...
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

    pub fn clear(&mut self) {
        self.blocks.clear();
        self.front_offset = 0;
        self.back_used = 0;
        self.len = 0;
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let blocks_needed = capacity.div_ceil(BITS_PER_BLOCK);
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
            let needed_blocks = (self.front_offset + self.len).div_ceil(BITS_PER_BLOCK);
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

impl From<BitVec> for BitVecDeque {
    fn from(bitvec: BitVec) -> Self {
        let mut deque = Self::with_capacity(bitvec.len());
        for bit in bitvec {
            deque.push_back(bit);
        }
        deque
    }
}

impl From<BitVecDeque> for BitVec {
    fn from(deque: BitVecDeque) -> Self {
        let mut bitvec = Self::with_capacity(deque.len);
        for i in 0..deque.len {
            bitvec.push(deque.get_unchecked(i));
        }
        bitvec
    }
}

impl Default for BitVecDeque {
    fn default() -> Self {
        Self::new()
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

    // ...other tests copied verbatim omitted here to keep patch concise...
}
