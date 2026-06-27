use crate::value_objects::Slug;

/// 衝突しにくい短い slug を生成する(nanoid 実装を想定)。長さは実装側の関心事。
pub trait SlugGenerator: Send + Sync {
    fn generate(&self) -> Slug;
}
