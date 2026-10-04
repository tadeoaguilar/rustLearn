//! Bonus: the bowling kata, written test-first. The tests below are in the
//! order they were written; each one forced a small change to `score`.

/// Total score of a complete game of ten frames.
pub fn score(rolls: &[u32]) -> u32 {
    let mut total = 0;
    let mut i = 0;
    for _frame in 0..10 {
        let roll = |k: usize| rolls.get(k).copied().unwrap_or(0);
        if roll(i) == 10 {
            // strike: 10 + next two rolls
            total += 10 + roll(i + 1) + roll(i + 2);
            i += 1;
        } else if roll(i) + roll(i + 1) == 10 {
            // spare: 10 + next roll
            total += 10 + roll(i + 2);
            i += 2;
        } else {
            total += roll(i) + roll(i + 1);
            i += 2;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gutter_game() {
        assert_eq!(score(&[0; 20]), 0);
    }

    #[test]
    fn all_ones() {
        assert_eq!(score(&[1; 20]), 20);
    }

    #[test]
    fn one_spare() {
        let mut rolls = vec![5, 5, 3];
        rolls.resize(20, 0);
        assert_eq!(score(&rolls), 16);
    }

    #[test]
    fn one_strike() {
        let mut rolls = vec![10, 3, 4];
        rolls.resize(19, 0);
        assert_eq!(score(&rolls), 24);
    }

    #[test]
    fn perfect_game() {
        assert_eq!(score(&[10; 12]), 300);
    }

    #[test]
    fn all_spares_with_final_five() {
        assert_eq!(score(&[5; 21]), 150);
    }
}
