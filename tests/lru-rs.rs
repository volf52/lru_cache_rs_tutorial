use lru_rs::LRUCache;

extern crate quickcheck;

#[macro_use(quickcheck)]
extern crate quickcheck_macros;

type TestCache = LRUCache<i32, 4>;

#[test]
fn test_empty() {
    let mut cache = TestCache::default();

    assert_eq!(cache.len(), 0);
    assert_eq!(cache.items(), []);
}

#[test]
fn test_insert() {
    let mut cache = TestCache::default();

    cache.insert(1);
    assert_eq!(cache.len(), 1);

    cache.insert(2);
    assert_eq!(cache.len(), 2);

    cache.insert(3);
    assert_eq!(cache.len(), 3);

    cache.insert(4);
    assert_eq!(cache.len(), 4);

    assert_eq!(
        cache.items(),
        [4, 3, 2, 1],
        "Ordered from most to least recent"
    );

    cache.insert(5);
    assert_eq!(cache.len(), 4);

    assert_eq!(
        cache.items(),
        [5, 4, 3, 2],
        "Ordered from most to least recent"
    );

    cache.insert(6);
    cache.insert(7);
    cache.insert(8);
    cache.insert(9);

    assert_eq!(cache.len(), 4);
    assert_eq!(
        cache.items(),
        [9, 8, 7, 6],
        "Least-recently-used item evicted"
    );
}

#[test]
fn test_lookup() {
    let mut cache = TestCache::default();
    cache.insert(1);
    cache.insert(2);
    cache.insert(3);
    cache.insert(4);

    let result = cache.lookup(|x| if *x == 5 { Some(()) } else { None });
    assert_eq!(result, None, "Cache miss.");
    assert_eq!(cache.items(), [4, 3, 2, 1], "Order not changed.");

    // Cache hit
    let result = cache.lookup(|x| if *x == 3 { Some(*x * 2) } else { None });
    assert_eq!(result, Some(6), "Cache hit.");
    assert_eq!(cache.items(), [3, 4, 2, 1], "Matching item moved to front.");
}

#[test]
fn test_clear() {
    let mut cache = TestCache::default();
    cache.insert(1);
    cache.clear();

    assert_eq!(cache.len(), 0);
    assert_eq!(cache.items(), [], "All items evicted");

    cache.insert(1);
    cache.insert(2);
    cache.insert(3);
    cache.insert(4);
    assert_eq!(cache.items(), [4, 3, 2, 1]);
    cache.clear();
    assert_eq!(cache.items(), [], "All items evicted again");
}

#[quickcheck]
fn test_touch(num: i16) {
    let first: i32 = num.into();
    let second = first + 1;
    let third = first + 2;
    let fourth = first + 3;

    let mut cache = TestCache::default();

    cache.insert(first);
    cache.insert(second);
    cache.insert(third);
    cache.insert(fourth);

    cache.touch(|x| *x == fourth + 1);

    assert_eq!(
        cache.items(),
        [fourth, third, second, first],
        "Nothing is touched."
    );

    cache.touch(|x| *x == second);

    assert_eq!(
        cache.items(),
        [second, fourth, third, first],
        "Touched item is moved to front."
    );
}

#[quickcheck]
fn test_fetch(num: i16) {
    let first: i32 = num.into();
    let second = first + 1;
    let third = first + 2;
    let fourth = first + 3;

    let mut cache = TestCache::default();

    cache.insert(first);
    cache.insert(second);
    cache.insert(third);
    cache.insert(fourth);

    cache.fetch(|x| *x == fourth + 1);

    assert_eq!(
        cache.items(),
        [fourth, third, second, first],
        "Nothing is touched."
    );

    cache.fetch(|x| *x == second);

    assert_eq!(
        cache.items(),
        [second, fourth, third, first],
        "Fetched item is moved to front."
    );
}

#[quickcheck]
fn test_front(num: i16) {
    let first: i32 = num.into();
    let second = first + 1;

    let mut cache = TestCache::default();

    assert_eq!(cache.front(), None, "Nothing is in the front.");

    cache.insert(first);
    cache.insert(second);

    assert_eq!(
        cache.front(),
        Some(&second),
        "The last inserted item should be in the front."
    );

    cache.touch(|x| *x == first);

    assert_eq!(
        cache.front(),
        Some(&first),
        "Touched item should be in the front."
    );
}
