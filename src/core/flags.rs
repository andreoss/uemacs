pub const CONTROL: u32 = 0x1000_0000;

pub const META: u32 = 0x2000_0000;

macro_rules! bitflags_newtype {
    ($name:ident, $repr:ty, { $($flag:ident = $val:expr),* $(,)? }) => {
        #[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
        pub struct $name($repr);

        impl $name {
            pub const EMPTY: Self = Self(0);
            $(pub const $flag: Self = Self($val);)*

            #[must_use]
            pub const fn intersects(self, other: Self) -> bool {
                self.0 & other.0 != 0
            }
        }

        impl core::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }

        impl core::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl core::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self {
                Self(self.0 & rhs.0)
            }
        }

        impl core::ops::BitAndAssign for $name {
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }

        impl core::ops::Not for $name {
            type Output = Self;
            fn not(self) -> Self {
                Self(!self.0)
            }
        }

        impl core::ops::BitXorAssign for $name {
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0;
            }
        }
    };
}

bitflags_newtype!(WindowFlags, u8, {
    FORCE = 0x01,
    MOVED = 0x02,
    EDITED = 0x04,
    HARD = 0x08,
    MODE_LINE = 0x10,
});

bitflags_newtype!(BufferFlags, u8, {
    INVISIBLE = 0x01,
    CHANGED = 0x02,
    TRUNCATED = 0x04,
});

bitflags_newtype!(CmdFlags, u8, {
    LINE_MOVE = 0x01,
    KILL = 0x02,
});

bitflags_newtype!(Mode, u16, {
    WRAP = 0x0001,
    C_MODE = 0x0002,
    SPELL = 0x0004,
    EXACT = 0x0008,
    VIEW = 0x0010,
    OVERWRITE = 0x0020,
    MAGIC = 0x0040,

    AUTO_SAVE = 0x0100,
});

impl WindowFlags {
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl Mode {
    pub const fn bits(self) -> u16 {
        self.0
    }
}
