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
