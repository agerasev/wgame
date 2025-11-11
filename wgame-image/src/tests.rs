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
