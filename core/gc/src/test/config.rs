mod miri {
    use super::super::run_test;
    use crate::{GcConfig, gc_config, set_gc_config};

    #[test]
    fn gc_config_roundtrip() {
        run_test(|| {
            let defaults = gc_config();
            assert_eq!(defaults.threshold(), 1_048_576);
            assert_eq!(defaults.used_space_percentage(), 70);

            let custom = GcConfig::new(8_192, 55);
            set_gc_config(custom);

            let current = gc_config();
            assert_eq!(current.threshold(), 8_192);
            assert_eq!(current.used_space_percentage(), 55);
        });
    }

    #[test]
    fn gc_config_normalization() {
        run_test(|| {
            let custom = GcConfig::new(0, 999);
            assert_eq!(custom.threshold(), 1);
            assert_eq!(custom.used_space_percentage(), 100);

            let custom = custom
                .with_threshold(0)
                .with_used_space_percentage(0);
            assert_eq!(custom.threshold(), 1);
            assert_eq!(custom.used_space_percentage(), 1);
        });
    }
}
