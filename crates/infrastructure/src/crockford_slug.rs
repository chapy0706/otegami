use rand::Rng;

use domain::ports::SlugGenerator;
use domain::value_objects::Slug;

/// Crockford Base32(大文字小文字を区別せず I/L/O/U を除く 32 文字)。
const ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Crockford Base32 系の slug を生成する具象。長さは設定値として受け取る。
pub struct CrockfordSlugGenerator {
    length: usize,
}

impl CrockfordSlugGenerator {
    pub fn new(length: usize) -> Self {
        Self { length }
    }
}

impl SlugGenerator for CrockfordSlugGenerator {
    fn generate(&self) -> Slug {
        let mut rng = rand::thread_rng();
        let s: String = (0..self.length)
            .map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char)
            .collect();
        // 生成器は常に正しい文字集合・長さで作るため、ここでの失敗は不変条件違反。
        Slug::parse(&s).expect("generated slug must be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 設定長の_slug_を生成する() {
        let gen = CrockfordSlugGenerator::new(6);
        let slug = gen.generate();
        assert_eq!(slug.as_str().chars().count(), 6);
    }

    #[test]
    fn 生成した_slug_は_crockford_文字集合に収まる() {
        let gen = CrockfordSlugGenerator::new(6);
        for _ in 0..100 {
            let slug = gen.generate();
            assert!(slug.as_str().bytes().all(|b| ALPHABET.contains(&b)));
        }
    }
}
