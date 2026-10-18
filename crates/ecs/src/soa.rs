//! Armazenamento contíguo (SoA) com máscara de presença e mapa slot→linha
//! (ENG-01-07-S01): iteração filtra componentes ausentes, ordem estável pelo
//! slot, e nenhum ponteiro interno é exposto ao script.

/// Array contíguo por componente, esparso por slot de entidade.
pub struct SoA<T> {
    data: Vec<T>,
    /// slot → linha em `data` (None = componente ausente).
    row_of: Vec<Option<u32>>,
    /// linha → slot (paralelo a `data`, mantém remoção O(1)).
    slot_of_row: Vec<u32>,
}

impl<T> Default for SoA<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SoA<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            row_of: Vec::new(),
            slot_of_row: Vec::new(),
        }
    }

    /// Garante espaço de slots (chamado quando o mundo cresce).
    pub fn grow(&mut self, slots: usize) {
        if self.row_of.len() < slots {
            self.row_of.resize(slots, None);
        }
    }

    /// Quantidade de componentes presentes (contiguidade: `data.len()`).
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Insere (ou substitui) o componente do slot. Devolve `false` se já havia.
    pub fn insert(&mut self, slot: u32, value: T) -> bool {
        self.grow(slot as usize + 1);
        match self.row_of[slot as usize] {
            Some(r) => {
                self.data[r as usize] = value;
                false
            }
            None => {
                self.row_of[slot as usize] = Some(self.data.len() as u32);
                self.slot_of_row.push(slot);
                self.data.push(value);
                true
            }
        }
    }

    pub fn remove(&mut self, slot: u32) -> Option<T> {
        let row = self.row_of.get_mut(slot as usize)?.take()? as usize;
        let removed = self.data.swap_remove(row);
        self.slot_of_row.swap_remove(row);
        // O swap moveu a última linha para `row`; corrige o mapa do movido.
        if row < self.data.len() {
            let moved = self.slot_of_row[row];
            self.row_of[moved as usize] = Some(row as u32);
        }
        Some(removed)
    }

    pub fn get(&self, slot: u32) -> Option<&T> {
        let row = self.row_of.get(slot as usize).copied().flatten()?;
        self.data.get(row as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_remove_keeps_rows_consistent() {
        let mut s: SoA<u32> = SoA::new();
        assert!(s.insert(0, 10));
        assert!(s.insert(2, 30));
        assert!(!s.insert(0, 11)); // substitui
        assert_eq!(s.get(0), Some(&11));
        assert_eq!(s.len(), 2);

        assert_eq!(s.remove(0), Some(11));
        assert_eq!(s.get(0), None);
        // O swap_remove moveu o slot 2; o mapa continua correto.
        assert_eq!(s.get(2), Some(&30));
        assert_eq!(s.len(), 1);
        assert_eq!(s.remove(2), Some(30));
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn removal_of_many_keeps_every_other_slot() {
        let mut s: SoA<u32> = SoA::new();
        for slot in 0..10u32 {
            s.insert(slot, slot);
        }
        for slot in [4u32, 0, 9, 7] {
            s.remove(slot);
        }
        for slot in 0..10u32 {
            let expected = if [4u32, 0, 9, 7].contains(&slot) {
                None
            } else {
                Some(&slot)
            };
            assert_eq!(s.get(slot), expected, "slot {slot}");
        }
    }
}
