mod lcs_table;

use self::lcs_table::LcsTable;
use std::cmp::{min, max};

// finds the longest common subsequences
// outputs the edit distance and a string containing
// all chars both inputs have in common
#[cfg_attr(feature = "cargo-clippy", allow(many_single_char_names))]
pub fn lcs(orig: &str, edit: &str, split: &str) -> (usize, String) {

    // make list by custom splits
    let x: Vec<&str> = orig.split(split).collect();
    let y: Vec<&str> = edit.split(split).collect();

    let (head, x_trunc, y_trunc, tail) = split_common_parts(&x, &y);

    let lcs = LcsTable::from(x_trunc, y_trunc);

    let mut dist = 0;
    let mut chunks = Vec::with_capacity(min(x.len(), y.len()));

    chunks.extend(head);

    for r in lcs.iter() {
        println!("{:?}: {:?}", r.0, r.1);

        use lcs::lcs_table::Step::*;
        match r.0 {
            Both => {
                chunks.push(*r.1);
            }
            _ => {
                dist += 1;
            }
        }
    }

    chunks.extend(tail);

    (dist, chunks.join(split))
}

/// Decompose `x` and `y` into a partition
///   `(head, x_truncated, y_truncated, tail)`
///
/// where
///   `x == [head, x_truncated, tail].concat()`
///   `y == [head, y_truncated, tail].concat()`
fn split_common_parts<'a>(x: &'a[&'a str], y: &'a [&'a str]) -> (&'a [&'a str], &'a [&'a str], &'a [&'a str], &'a [&'a str]) {
    println!();
    println!("x={:?}, y={:?}", x, y);

    let len = min(x.len(), y.len());

    // `start` is the position where `x` and `y` start to differ
    let start = match (0..len).find(|i| x[*i] != y[*i]) {
        Some(i) => i,
        None => len,
    };

    // `end_x` is the position in `x`, where
    //         the tail of `x` is equal to the corresponding tail of `y`
    let end_x_lower_bound = start + max(0, x.len()-len);
    let end_x_upper_bound = x.len();
    let delta_yx = y.len().wrapping_sub(x.len());

    let end_x = match (end_x_lower_bound..end_x_upper_bound).rev().find(|i| {
        x[*i] != y[(*i).wrapping_add(delta_yx)]
    }) {
        Some(i) => i+1,
        None => end_x_lower_bound,
    };

    let (x_trunc, tail) = x.split_at(end_x);
    let (head, x_trunc) = x_trunc.split_at(start);

    return (head, x_trunc, &y[start..end_x.wrapping_add(delta_yx)], tail);
}

#[test]
fn test_lcs_split_common_parts() {
    macro_rules! inner_as_ref {
        {$( $x:expr ),*}  => {
            ($( AsRef::as_ref(&$x) ),*)
        }
    }

    assert_eq!(split_common_parts(&["a"], &["a"]),
        inner_as_ref!(["a"], [], [], []));
    assert_eq!(split_common_parts(&["b"], &["c"]),
        inner_as_ref!([], ["b"], ["c"], []));
    assert_eq!(split_common_parts(&["a", "b"], &["a", "c"]),
        inner_as_ref!(["a"], ["b"], ["c"], []));
    assert_eq!(split_common_parts(&["b", "d"], &["c", "d"]),
        inner_as_ref!([], ["b"], ["c"], ["d"]));
    assert_eq!(split_common_parts(&["a", "b", "d"], &["a", "c", "d"]),
        inner_as_ref!(["a"], ["b"], ["c"], ["d"]));
    assert_eq!(split_common_parts(&["a1", "a2", "b1", "b2", "d1", "d2"],
                                  &["a1", "a2", "c1", "c2", "d1", "d2"]),
        inner_as_ref!(["a1", "a2"], ["b1", "b2"], ["c1", "c2"], ["d1", "d2"]));
}

#[test]
fn test_lcs() {
    assert_eq!(lcs("AGCAT", "GAC", ""), (4, "AC".to_string()));

    assert_eq!(lcs("a b : g", "b a : b b : g g", " "), (4, "a b : g".to_string()));

    assert_eq!(lcs("test", "test", ""), (0, "test".to_string()));

    assert_eq!(
        lcs(
            "The quick brown fox jumps over the lazy dog",
            "The quick brown dog leaps over the lazy cat",
            "\n",
        ),
        (2, "".to_string())
    );

    assert_eq!(lcs("test", "tost", ""), (2, "tst".to_string()));

    assert_eq!(lcs("test", "test", " "), (0, "test".to_string()));

    assert_eq!(
        lcs(
            "The quick brown fox jumps over the lazy dog",
            "The quick brown dog leaps over the lazy cat",
            "",
        ),
        (16, "The quick brown o ps over the lazy ".to_string())
    );
    assert_eq!(
        lcs(
            "The quick brown fox jumps over the lazy dog",
            "The quick brown dog leaps over the lazy cat",
            " ",
        ),
        (6, "The quick brown over the lazy".to_string())
    );

    assert_eq!(
        lcs(
            "The quick brown fox jumps over the lazy dog",
            "The quick brown fox jumps over the lazy dog",
            "\n",
        ),
        (0, "The quick brown fox jumps over the lazy dog".to_string())
    );
}
