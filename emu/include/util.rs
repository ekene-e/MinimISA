/// Sign-extend from a variable-width storage format.
///
/// Performs sign extension of the value `x`, which is stored in a signed
/// n-bit format, into a 64-bit signed format.
///
/// # Arguments
///
/// * `x` - Value to sign-extend (unsigned 64-bit).
/// * `n` - Number of bits on which `x` is represented.
///
/// # Returns
///
/// Returns `x` in signed 64-bit format.
pub fn sign_extend(x: u64, n: u32) -> i64 {
    // Shift x left and right to perform sign extension
    let shift = 64 - n;
    ((x << shift) as i64) >> shift
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_extend() {
        assert_eq!(sign_extend(0b0111, 4), 7);  // Positive value remains positive
        assert_eq!(sign_extend(0b1111, 4), -1); // Negative value is correctly extended
        assert_eq!(sign_extend(0b1000, 4), -8); // Larger negative value is correctly extended
        assert_eq!(sign_extend(0b0000, 4), 0);  // Zero stays zero
    }
}
