#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::filters::{Filter, ReleaseChannel};
    use chrono::Utc;

    fn create_test_metadata(title: &str, filename: &str, game_versions: Vec<String>) -> Metadata {
        Metadata::new(
            title.to_string(),
            "Test description".to_string(),
            filename.to_string(),
            Utc::now(),
            Some(game_versions),
        )
    }

    #[tokio::test]
    async fn test_game_version_strict_filter() {
        let metadata1 = create_test_metadata("mod1", "mod1.zip", vec!["3.10.0".to_string()]);
        let metadata2 = create_test_metadata("mod2", "mod2.zip", vec!["3.11.0".to_string()]);
        let metadata3 = create_test_metadata("mod3", "mod3.zip", vec!["3.10.0".to_string(), "3.11.0".to_string()]);

        let filter = Filter::GameVersionStrict(vec!["3.10.0".to_string()]);

        assert!(filter.matches(&metadata1).await.unwrap());
        assert!(!filter.matches(&metadata2).await.unwrap());
        assert!(filter.matches(&metadata3).await.unwrap());
    }

    #[tokio::test]
    async fn test_release_channel_filter() {
        let mut metadata_alpha = create_test_metadata("alpha", "alpha.zip", vec![]);
        metadata_alpha.channel = ReleaseChannel::Alpha;

        let mut metadata_beta = create_test_metadata("beta", "beta.zip", vec![]);
        metadata_beta.channel = ReleaseChannel::Beta;

        let mut metadata_release = create_test_metadata("release", "release.zip", vec![]);
        metadata_release.channel = ReleaseChannel::Release;

        let filter_alpha = Filter::ReleaseChannel(ReleaseChannel::Alpha);
        let filter_beta = Filter::ReleaseChannel(ReleaseChannel::Beta);
        let filter_release = Filter::ReleaseChannel(ReleaseChannel::Release);

        // Alpha filter accepts everything
        assert!(filter_alpha.matches(&metadata_alpha).await.unwrap());
        assert!(filter_alpha.matches(&metadata_beta).await.unwrap());
        assert!(filter_alpha.matches(&metadata_release).await.unwrap());

        // Beta filter accepts beta and release
        assert!(!filter_beta.matches(&metadata_alpha).await.unwrap());
        assert!(filter_beta.matches(&metadata_beta).await.unwrap());
        assert!(filter_beta.matches(&metadata_release).await.unwrap());

        // Release filter accepts only release
        assert!(!filter_release.matches(&metadata_alpha).await.unwrap());
        assert!(!filter_release.matches(&metadata_beta).await.unwrap());
        assert!(filter_release.matches(&metadata_release).await.unwrap());
    }

    #[tokio::test]
    async fn test_filename_filter() {
        let metadata1 = create_test_metadata("mod1", "special-mod.zip", vec![]);
        let metadata2 = create_test_metadata("mod2", "regular-mod.zip", vec![]);

        let filter = Filter::Filename("special.*".to_string());

        assert!(filter.matches(&metadata1).await.unwrap());
        assert!(!filter.matches(&metadata2).await.unwrap());
    }

    #[tokio::test]
    async fn test_select_latest_with_filters() {
        let metadata1 = create_test_metadata("mod1", "mod1.zip", vec!["3.10.0".to_string()]);
        let metadata2 = create_test_metadata("mod2", "mod2.zip", vec!["3.11.0".to_string()]);
        let metadata3 = create_test_metadata("mod3", "mod3.zip", vec!["3.10.0".to_string()]);

        let candidates = vec![&metadata1, &metadata2, &metadata3];
        let filters = vec![Filter::GameVersionStrict(vec!["3.10.0".to_string()])];

        let result = select_latest(candidates.into_iter(), filters).await.unwrap();
        
        // Should return the first candidate that matches (metadata1)
        assert_eq!(result.title, "mod1");
    }

    #[tokio::test]
    async fn test_select_latest_no_matches() {
        let metadata1 = create_test_metadata("mod1", "mod1.zip", vec!["3.11.0".to_string()]);
        let metadata2 = create_test_metadata("mod2", "mod2.zip", vec!["3.11.0".to_string()]);

        let candidates = vec![&metadata1, &metadata2];
        let filters = vec![Filter::GameVersionStrict(vec!["3.10.0".to_string()])];

        let result = select_latest(candidates.into_iter(), filters).await;
        
        assert!(matches!(result, Err(Error::FilterEmpty(_))));
    }

    #[tokio::test]
    async fn test_select_latest_empty_candidates() {
        let candidates: Vec<&Metadata> = vec![];
        let filters = vec![Filter::GameVersionStrict(vec!["3.10.0".to_string()])];

        let result = select_latest(candidates.into_iter(), filters).await;
        
        assert!(matches!(result, Err(Error::NoCompatibleFiles)));
    }
}
