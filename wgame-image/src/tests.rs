use euclid::default::Size2D;

use crate::{Image, prelude::*};

#[test]
fn slice() {
    let img = Image::<u8>::with_data(
        (3, 3),
        [
            0, 1, 2, //
            3, 4, 5, //
            6, 7, 8, //
        ],
    );

    let slice = img.slice((..2, ..2));

    assert_eq!(slice.size(), Size2D::new(2, 2));
    assert_eq!(slice.stride(), 3);
    assert_eq!(slice.data(), [0, 1, 2, 3, 4]);
    assert_eq!(
        slice.rows().map(|(_, row)| row).collect::<Vec<_>>(),
        [
            [0, 1], //
            [3, 4], //
        ]
    );
    assert_eq!(
        slice.pixels().map(|(_, pix)| *pix).collect::<Vec<_>>(),
        [
            0, 1, //
            3, 4, //
        ]
    );
}

#[test]
fn empty_slice() {
    let img = Image::<u8>::with_data(
        (2, 2),
        [
            0, 1, //
            2, 3, //
        ],
    );

    let slice = img.slice((1..1, 1..1));

    assert_eq!(slice.size(), Size2D::new(0, 0));
    assert_eq!(slice.data(), []);
    assert_eq!(slice.rows().next(), None);
    assert_eq!(slice.pixels().next(), None);
}

#[test]
fn slice_mut() {
    let mut img = Image::<u8>::with_data(
        (4, 4),
        [
            0, 1, 2, 3, //
            4, 5, 6, 7, //
            8, 9, 10, 11, //
            12, 13, 14, 15, //
        ],
    );

    let mut slice = img.slice_mut((1..3, 1..3));

    assert_eq!(slice.size(), Size2D::new(2, 2));
    assert_eq!(slice.stride(), 4);
    assert_eq!(slice.data(), [5, 6, 7, 8, 9, 10]);
    assert_eq!(
        slice.rows_mut().map(|(_, row)| row).collect::<Vec<_>>(),
        [
            [5, 6],  //
            [9, 10], //
        ]
    );
    assert_eq!(
        slice.pixels_mut().map(|(_, pix)| *pix).collect::<Vec<_>>(),
        [
            5, 6, //
            9, 10, //
        ]
    );

    for (_, pix) in slice.pixels_mut() {
        *pix += 10;
    }

    assert_eq!(
        img.data(),
        [
            0, 1, 2, 3, //
            4, 15, 16, 7, //
            8, 19, 20, 11, //
            12, 13, 14, 15, //
        ]
    );
}

#[test]
fn copy_from() {
    let mut img = Image::<u8>::with_data(
        (4, 4),
        [
            0, 1, 2, 3, //
            4, 5, 6, 7, //
            8, 9, 10, 11, //
            12, 13, 14, 15, //
        ],
    );

    let mut slice = img.slice_mut((1..3, 1..3));

    slice.copy_from(Image::with_data((2, 2), [15, 16, 19, 20]));

    assert_eq!(
        img.data(),
        [
            0, 1, 2, 3, //
            4, 15, 16, 7, //
            8, 19, 20, 11, //
            12, 13, 14, 15, //
        ]
    );
}

#[test]
fn atlas_growth_resize_and_drop_preserve_pixels() {
    let atlas = crate::Atlas::<u8>::with_size(Size2D::new(4, 4));
    let mut images = Vec::new();
    for value in 1..40u8 {
        let item = atlas.allocate((3, 3));
        item.update(|mut dst| dst.copy_from(Image::with_color((3, 3), value)));
        images.push((item, value));
        for (item, expected) in &images {
            item.with(|src| assert!(src.pixels().all(|(_, p)| p == expected)));
        }
    }
    let first = images[0].0.clone();
    first.resize((5, 5));
    first.with(|src| assert!(src.slice((..3, ..3)).pixels().all(|(_, p)| *p == 1)));
    drop(images);
    first.with(|src| assert!(src.slice((..3, ..3)).pixels().all(|(_, p)| *p == 1)));
}
#[test]
fn atlas_tracker_unions_partial_updates() {
    let mut atlas = crate::Atlas::<u8>::default();
    let tracker = std::rc::Rc::new(crate::atlas::Tracker::default());
    atlas.subscribe(std::rc::Rc::downgrade(&tracker));
    let item = atlas.allocate((4, 4));
    tracker.clear();
    item.update_part(|_| {}, euclid::rect(1, 1, 1, 1));
    item.update_part(|_| {}, euclid::rect(2, 2, 1, 1));
    assert_eq!(
        tracker.take_next(),
        Some(euclid::rect(
            item.rect().origin.x + 1,
            item.rect().origin.y + 1,
            2,
            2
        ))
    );
    assert_eq!(tracker.take_next(), None);
}

#[test]
fn atlas_drop_and_resize_abandon_rectangles_until_replacement() {
    let atlas = crate::Atlas::<u8>::with_size((16, 16).into());
    let first = atlas.allocate((4, 4));
    first.update(|mut dst| dst.copy_from(Image::with_color((4, 4), 7)));
    let abandoned = first.rect();
    drop(first);
    let second = atlas.allocate((4, 4));
    assert_eq!(atlas.generation(), 0);
    assert!(!abandoned.intersects(&second.rect()));
    second.update(|mut dst| dst.copy_from(Image::with_color((4, 4), 9)));
    let before_resize = second.rect();
    second.resize((5, 5));
    assert_eq!(atlas.generation(), 0);
    assert!(!before_resize.intersects(&second.rect()));
    second.with(|src| assert!(src.slice((..4, ..4)).pixels().all(|(_, p)| *p == 9)));
    atlas.with_data(|src| {
        assert!(src.slice(abandoned).pixels().all(|(_, p)| *p == 7));
        assert!(src.slice(before_resize).pixels().all(|(_, p)| *p == 9));
    });
}

#[test]
fn atlas_compacts_at_same_size_and_preserves_live_items() {
    let mut atlas = crate::Atlas::<u8>::with_size((16, 16).into());
    let tracker = std::rc::Rc::new(crate::atlas::Tracker::default());
    atlas.subscribe(std::rc::Rc::downgrade(&tracker));
    let live = atlas.allocate((8, 8));
    live.update(|mut dst| dst.copy_from(Image::with_color((8, 8), 42)));
    for _ in 0..3 {
        drop(atlas.allocate((8, 8)));
    }
    tracker.clear();
    let next = atlas.allocate((8, 8));
    assert_eq!(atlas.size(), Size2D::new(16, 16));
    assert_eq!(atlas.generation(), 1);
    assert!(!live.rect().intersects(&next.rect()));
    live.with(|src| assert!(src.pixels().all(|(_, p)| *p == 42)));
    assert_eq!(tracker.take_next(), Some(euclid::rect(0, 0, 16, 16)));
}

#[test]
fn atlas_compaction_falls_back_to_growth_for_long_items() {
    let atlas = crate::Atlas::<u8>::with_size((16, 16).into());
    drop(atlas.allocate((16, 16)));
    // Below half the area, but too wide for a same-size replacement.
    let wide = atlas.allocate((20, 1));
    assert_eq!(atlas.generation(), 1);
    assert!(atlas.size().width >= 20);
    assert_eq!(wide.size(), Size2D::new(20, 1));
}

#[test]
fn atlas_repeated_compaction_does_not_grow_for_dead_items() {
    let atlas = crate::Atlas::<u8>::with_size((16, 16).into());
    for _ in 0..40 {
        drop(atlas.allocate((8, 8)));
    }
    assert_eq!(atlas.size(), Size2D::new(16, 16));
    assert!(atlas.generation() > 1);
}

#[test]
fn atlas_resize_compacts_using_replacement_area_and_copies_pixels() {
    let atlas = crate::Atlas::<u8>::with_size((16, 16).into());
    let item = atlas.allocate((16, 16));
    item.update(|mut dst| dst.copy_from(Image::with_color((16, 16), 17)));
    let alias = item.clone();
    // The replaced allocation is excluded from the live-area calculation.
    item.resize((8, 8));
    assert_eq!(atlas.generation(), 1);
    assert_eq!(atlas.size(), Size2D::new(16, 16));
    assert_eq!(alias.size(), Size2D::new(8, 8));
    alias.with(|src| assert!(src.pixels().all(|(_, p)| *p == 17)));
}
