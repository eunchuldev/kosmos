/// Copied from https://github.com/osanshouo/hilbert-index/blob/master/src/lib.rs

// 基本格子における部分格子の数 2^D - 1
const fn max<const D: usize>() -> usize { !( {usize::MAX}<<D ) }

const fn gc(i: usize) -> usize { i^(i >> 1) }

// Gray code の逆変換.
#[inline]
fn gc_inv<const D: usize>(g: usize) -> usize { (1..D).fold(g, |i, j| i^(g>>j)) }

const fn g(i: usize) -> usize {
    (!i).trailing_zeros() as usize
}

const fn dmap<const D: usize>(i: usize) -> usize {
    if i == 0 { 0 } else if i&1 == 0 { g(i-1) % D } else { g(i) % D }
}

const fn emap(i: usize) -> usize {
    if i == 0 { 0 } else { gc(2*( (i-1)/2 )) }
}

// #[inline]
// #[allow(dead_code)]
// fn fmap<const D: usize>(i: usize) -> usize { emap(i)^(1 << dmap::<D>(i)) }

// D bit の範囲で右回転
const fn rotate_right<const D: usize>(b: usize, i: usize) -> usize {
    let i = i.rem_euclid(D);
    (b >> i)^(b << (D-i))&max::<D>()
}

// D bit の範囲で左回転
const fn rotate_left<const D: usize>(b: usize, i: usize) -> usize {
    let i = i.rem_euclid(D);
    max::<D>() & (b << i)^(b >> (D-i))
}

const fn t<const D: usize>(b: usize, e: usize, d: usize) -> usize { rotate_right::<D>(b^e, d+1) }

const fn t_inv<const D: usize>(b: usize, e: usize, d: usize) -> usize { rotate_left::<D>(b, d+1)^e }

#[inline]
fn reduce<const D: usize>(p: &[usize; D], i: usize) -> usize {
    p.iter().enumerate()
        .fold(0, |l, (k, p)| l^( ((p >> i)&1) << k))
}

pub fn to_hilbert<const D: usize>(x: [usize; D], level: usize) -> usize {
    let (mut h, mut e, mut d) = (0, 0, 0);
    for i in(0..level).rev() {
        let l = t::<D>(reduce(&x, i), e, d);
        let w = gc_inv::<D>(l);
        e = e^( rotate_left::<D>(emap(w), d+1) );
        d = ( d + dmap::<D>(w) + 1 )%D;
        h = (h << D) | w;
    }
    h
}

pub fn from_hilbert<const D: usize>(x: usize, level: usize) -> [usize; D] {
    let (mut e, mut d) = (0, 0);
    let mut p = [0; D];

    for i in (0..level).rev() {
        let w = (0..D).fold(0, |w, k| w^( ((x >> (i*D + k)) & 1 ) << k ));
        let l = t_inv::<D>(gc(w), e, d);
        for j in 0..D {
            p[j] = (p[j] << 1)|((l >> j)&1);
        }
        e = e^rotate_left::<D>( emap(w), d+1 );
        d = ( d + dmap::<D>(w) + 1 )%D;
    }
    p
}

#[cfg(test)]
mod tests {
    use super::{from_hilbert, to_hilbert};
}
