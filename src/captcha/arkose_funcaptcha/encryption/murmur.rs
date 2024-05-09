// ArkoseLabs modified murmurhash3 related funcs
// Reversed by Dort, and ofc bebebebebebebe
fn x64_add(t: [u32; 2], r: [u32; 2]) -> [u32; 2] {
    let t = [t[0].wrapping_shr(16), t[0] & 0xFFFF, t[1].wrapping_shr(16), t[1] & 0xFFFF];
    let r = [r[0].wrapping_shr(16), r[0] & 0xFFFF, r[1].wrapping_shr(16), r[1] & 0xFFFF];
    let mut e = [0u32; 4];

    e[3] += t[3] + r[3];
    e[2] += e[3].wrapping_shr(16);
    e[3] &= 0xFFFF;
    e[2] += t[2] + r[2];
    e[1] += e[2].wrapping_shr(16);
    e[2] &= 0xFFFF;
    e[1] += t[1] + r[1];
    e[0] += e[1].wrapping_shr(16);
    e[1] &= 0xFFFF;
    e[0] += t[0] + r[0];
    e[0] &= 0xFFFF;

    return [(e[0] << 16) | e[1], (e[2] << 16) | e[3]];
}
fn x64_multiply(t: [u32; 2], r: [u32; 2]) -> [u32; 2] {
    let t = [t[0].wrapping_shr(16), 65535 & t[0], t[1].wrapping_shr(16), 65535 & t[1]];
    let r =  [r[0].wrapping_shr(16), 65535 & r[0], r[1].wrapping_shr(16), 65535 & r[1]];
    let mut e: [u32; 4] = [0, 0, 0, 0];
    e[3] += t[3] * r[3];
    e[2] += e[3].wrapping_shr(16);
    e[3] &= 65535;
    e[2] += t[2] * r[3];
    e[1] += e[2].wrapping_shr(16);
    e[2] &= 65535;
    e[2] += t[3] * r[2];
    e[1] += e[2].wrapping_shr(16);
    e[2] &= 65535;
    e[1] += t[1] * r[3];
    e[0] += e[1].wrapping_shr(16);
    e[1] &= 65535;
    e[1] += t[2] * r[2];
    e[0] += e[1].wrapping_shr(16);
    e[1] &= 65535;
    e[1] += t[3] * r[1];
    e[0] += e[1].wrapping_shr(16);
    e[1] &= 65535;
    e[0] = e[0].wrapping_add((t[0] as u64 * r[3] as u64 + t[1] as u64 * r[2] as u64 + t[2] as u64 * r[1] as u64 + t[3] as u64 * r[0] as u64) as u32);
    e[0] &= 65535;

    return [(e[0] << 16) | e[1], (e[2] << 16) | e[3]];
}
fn x64_rotl(t: [u32; 2], r: u32) -> [u32; 2] {
    let r = r % 64;

    if r == 32 {
        [t[1], t[0]]
    } else if r < 32 {
        [
            (t[0] << r) | (t[1].wrapping_shr(32 - r)),
            (t[1] << r) | (t[0].wrapping_shr(32 - r)),
        ]
    } else {
        let r = r - 32;
        [
            (t[1] << r) | (t[0].wrapping_shr(32 - r)),
            (t[0] << r) | (t[1].wrapping_shr(32 - r)),
        ]
    }
}
fn x64_left_shift(t: [u32; 2], r: u32) -> [u32; 2] {
    let r = r % 64;

    return if r == 0 {
        t
    } else if r < 32 {
        [(t[0] << r) | (t[1].wrapping_shr(32 - r)), t[1] << r]
    } else {
        [t[1] << (r - 32), 0]
    }
}
fn x64_xor(t: [u32; 2], r: [u32; 2]) -> [u32; 2] {
    return [t[0] ^ r[0], t[1] ^ r[1]];
}
fn x64_fmix(mut t: [u32; 2]) -> [u32; 2] {
    t = x64_xor(t, [0, t[0].wrapping_shr(1)]);
    t = x64_multiply(t, [4283543511, 3981806797]);
    t = x64_xor(t, [0, t[0].wrapping_shr(1)]);
    t = x64_multiply(t, [3301882366, 444984403]);
    t = x64_xor(t, [0, t[0].wrapping_shr(1)]);
    return t;
}
fn get_byte_at_index(input_string: &str, index: usize) -> u32 {
    let bytes = input_string.as_bytes();

    if index < bytes.len() {
        bytes[index] as u32
    } else {
        0
    }
}
pub fn x64hash128(t: &str, r: u32) -> String {
    let e = t.len() % 16;
    let o = t.len() - e;
    let mut x: [u32; 2] = [0, r];
    let mut c: [u32; 2] = [0, r];
    let mut h: [u32; 2];
    let mut a: [u32; 2];
    let d: [u32; 2] = [2277735313, 289559509];
    let i: [u32; 2] = [1291169091, 658871167];

    let mut l = 0;
    while l < o {
        let h0 = (t.as_bytes()[l + 4] as u32)
            | ((t.as_bytes()[l + 5] as u32) << 8)
            | ((t.as_bytes()[l + 6] as u32) << 16)
            | ((t.as_bytes()[l + 7] as u32) << 24);

        let h1 = (t.as_bytes()[l] as u32)
            | ((t.as_bytes()[l + 1] as u32) << 8)
            | ((t.as_bytes()[l + 2] as u32) << 16)
            | ((t.as_bytes()[l + 3] as u32) << 24);

        let mut h = [h0, h1];
        let a0 = (255 & t.as_bytes()[l + 12] as u32)
            | ((255 & t.as_bytes()[l + 13] as u32) << 8)
            | ((255 & t.as_bytes()[l + 14] as u32) << 16)
            | ((255 & t.as_bytes()[l + 15] as u32) << 24);

        let a1 = (255 & t.as_bytes()[l + 8] as u32)
            | ((255 & t.as_bytes()[l + 9] as u32) << 8)
            | ((255 & t.as_bytes()[l + 10] as u32) << 16)
            | ((255 & t.as_bytes()[l + 11] as u32) << 24);

        let mut a = [a0, a1];

        h = x64_multiply(h, d);
        h = x64_rotl(h, 31);
        h = x64_multiply(h, i);
        x = x64_xor(x, h);
        x = x64_rotl(x, 27);
        x = x64_add(x, c);
        x = x64_add(x64_multiply(x, [0, 5]), [0, 1390208809]);
        a = x64_multiply(a, i);
        a = x64_rotl(a, 33);
        a = x64_multiply(a, d);
        c = x64_xor(c, a);
        c = x64_rotl(c, 31);
        c = x64_add(c, x);
        c = x64_add(x64_multiply(c, [0, 5]), [0, 944331445]);
        l += 16;
    };
    h = [0, 0];
    a = [0, 0];
    let mut current_e = e;
    while current_e >= 1 {
        match current_e {
            15 => {
                a = x64_xor(a, x64_left_shift([0, get_byte_at_index(t, l + 14)], 48));
            }
            14 => {
                a = x64_xor(a, x64_left_shift([0, get_byte_at_index(t, l + 13)], 40));
            }
            13 => {
                a = x64_xor(a, x64_left_shift([0, get_byte_at_index(t, l + 12)], 32));
            }
            12 => {
                a = x64_xor(a, x64_left_shift([0, get_byte_at_index(t, l + 11)], 24));
            }
            11 => {
                a = x64_xor(a, x64_left_shift([0, get_byte_at_index(t, l + 10)], 16));
            }
            10 => {
                a = x64_xor(a, x64_left_shift([0, get_byte_at_index(t, l + 9)], 8));
            }
            9 => {
                a = x64_xor(a, [0, get_byte_at_index(t, l + 8)]);
                a = x64_multiply(a, i);
                a = x64_rotl(a, 33);
                a = x64_multiply(a, d);
                c = x64_xor(c, a);
            }
            8 => {
                h = x64_xor(h, x64_left_shift([0, get_byte_at_index(t, l + 7)], 56));
            }
            7 => {
                h = x64_xor(h, x64_left_shift([0, get_byte_at_index(t, l + 6)], 48));
            }
            6 => {
                h = x64_xor(h, x64_left_shift([0, get_byte_at_index(t, l + 5)], 40));
            }
            5 => {
                h = x64_xor(h, x64_left_shift([0, get_byte_at_index(t, l + 4)], 32));
            }
            4 => {
                h = x64_xor(h, x64_left_shift([0, get_byte_at_index(t, l + 3)], 24));
            }
            3 => {
                h = x64_xor(h, x64_left_shift([0, get_byte_at_index(t, l + 2)], 16));
            }
            2 => {
                h = x64_xor(h, x64_left_shift([0, get_byte_at_index(t, l + 1)], 8));
            }
            1 => {
                h = x64_xor(h, [0, get_byte_at_index(t, l)]);
                h = x64_multiply(h, d);
                h = x64_rotl(h, 31);
                h = x64_multiply(h, i);
                x = x64_xor(x, h);
            }
            _ => {}
        }
        current_e -= 1;
    }
    x = x64_xor(x, [0, t.len().try_into().unwrap()]);
    c = x64_xor(c, [0, t.len().try_into().unwrap()]);
    x = x64_add(x, c);
    c = x64_add(c, x);
    x = x64_fmix(x);
    c = x64_fmix(c);
    x = x64_add(x, c);
    c = x64_add(c, x);
    return format!("{:08x}{:08x}{:08x}{:08x}", x[0], x[1], c[0], c[1]);
}
