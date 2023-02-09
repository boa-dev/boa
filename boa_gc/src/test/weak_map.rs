use super::run_test;
use crate::{force_collect, has_weak_maps, Gc, WeakMap};

#[test]
fn weak_map_basic() {
    run_test(|| {
        let key1 = Gc::new(String::from("key1"));
        let key2 = Gc::new(String::from("key2"));
        let key3 = Gc::new(String::from("key3"));

        assert!(!has_weak_maps());

        let mut map = WeakMap::new();

        assert!(has_weak_maps());

        map.insert(&key1, ());
        map.insert(&key2, ());
        map.insert(&key3, ());

        force_collect();
        assert!(has_weak_maps());

        assert!(map.contains_key(&key1));
        assert!(map.contains_key(&key2));
        assert!(map.contains_key(&key3));

        drop(key1);

        force_collect();
        assert!(has_weak_maps());

        assert!(map.contains_key(&key2));
        assert!(map.contains_key(&key3));

        drop(key2);

        force_collect();
        assert!(has_weak_maps());

        assert!(map.contains_key(&key3));
        assert!(has_weak_maps());

        drop(key3);

        assert!(has_weak_maps());

        force_collect();
        assert!(has_weak_maps());

        drop(map);

        force_collect();
        assert!(!has_weak_maps());
    });
}

#[test]
fn weak_map_multiple() {
    run_test(|| {
        let key1 = Gc::new(String::from("key1"));
        let key2 = Gc::new(String::from("key2"));
        let key3 = Gc::new(String::from("key3"));

        assert!(!has_weak_maps());

        let mut map_1 = WeakMap::new();
        let mut map_2 = WeakMap::new();

        assert!(has_weak_maps());

        map_1.insert(&key1, ());
        map_1.insert(&key2, ());
        map_2.insert(&key3, ());

        force_collect();
        assert!(has_weak_maps());

        assert!(map_1.contains_key(&key1));
        assert!(map_1.contains_key(&key2));
        assert!(!map_1.contains_key(&key3));
        assert!(!map_2.contains_key(&key1));
        assert!(!map_2.contains_key(&key2));
        assert!(map_2.contains_key(&key3));

        force_collect();
        assert!(has_weak_maps());

        drop(key1);
        drop(key2);

        force_collect();
        assert!(has_weak_maps());

        assert!(!map_1.contains_key(&key3));
        assert!(map_2.contains_key(&key3));

        drop(key3);

        force_collect();
        assert!(has_weak_maps());

        drop(map_1);

        force_collect();
        assert!(has_weak_maps());

        drop(map_2);

        force_collect();
        assert!(!has_weak_maps());
    });
}

#[test]
fn weak_map_key_live() {
    run_test(|| {
        let key = Gc::new(String::from("key"));
        let key_copy = key.clone();

        let mut map = WeakMap::new();

        map.insert(&key, ());

        assert!(map.contains_key(&key));
        assert!(map.contains_key(&key_copy));

        assert_eq!(map.remove(&key), Some(()));

        map.insert(&key, ());

        drop(key);

        force_collect();

        assert!(map.contains_key(&key_copy));
    });
}
