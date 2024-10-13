#[cfg(target_pointer_width = "32")]
type Uword = u32;
#[cfg(target_pointer_width = "32")]
type Sword = i32;
#[cfg(target_pointer_width = "32")]
type Doubleword = u64;

#[cfg(target_pointer_width = "64")]
type Uword = u64;
#[cfg(target_pointer_width = "64")]
type Sword = i64;
#[cfg(target_pointer_width = "64")]
type Doubleword = u128;
