use crate::types::{ArchivePolicy, DownloadRecipe, MediaFingerprint};

/// P2 Extension Point: Best Available Archive Manager Domain Boundary
pub struct ArchiveManager {
    pub policy: ArchivePolicy,
}

impl ArchiveManager {
    pub fn new(policy: ArchivePolicy) -> Self {
        Self { policy }
    }

    pub fn generate_nfo_metadata(recipe: &DownloadRecipe, fingerprint: &MediaFingerprint) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
            <episodedetails>\n  \
              <title>{}</title>\n  \
              <uniqueid type=\"{}\">{}</uniqueid>\n  \
              <recipeId>{}</recipeId>\n  \
              <canonicalId>{}</canonicalId>\n  \
              <container>{}</container>\n  \
              <strategy>{:?}</strategy>\n  \
              <acquiredAt>{}</acquiredAt>\n\
            </episodedetails>",
            fingerprint.source.title,
            fingerprint.source.extractor,
            fingerprint.source.source_id,
            recipe.id,
            fingerprint.canonical_id,
            recipe.output_container,
            recipe.strategy,
            recipe.timestamp
        )
    }
}
