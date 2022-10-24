/// The BorrowFlag used by GC is split into 2 parts. the upper 63 or 31 bits
/// (depending on the architecture) are used to store the number of borrowed
/// references to the type. The low bit is used to record the rootedness of the
/// type.
///
/// This means that GcCell can have, at maximum, half as many outstanding
/// borrows as RefCell before panicking. I don't think that will be a problem.
#[derive(Copy, Clone)]
pub(crate) struct BorrowFlag(usize);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum BorrowState {
    Reading,
    Writing,
    Unused,
}

const ROOT: usize = 1;
const WRITING: usize = !1;
const UNUSED: usize = 0;

/// The base borrowflag init is rooted, and has no outstanding borrows.
pub(crate) const BORROWFLAG_INIT: BorrowFlag = BorrowFlag(1);

impl BorrowFlag {
    pub(crate) fn borrowed(self) -> BorrowState {
        match self.0 & !ROOT {
            UNUSED => BorrowState::Unused,
            WRITING => BorrowState::Writing,
            _ => BorrowState::Reading,
        }
    }

    pub(crate) fn rooted(self) -> bool {
        match self.0 & ROOT {
            0 => false,
            _ => true,
        }
    }

    pub(crate) fn set_writing(self) -> Self {
        // Set every bit other than the root bit, which is preserved
        BorrowFlag(self.0 | WRITING)
    }

    pub(crate) fn set_unused(self) -> Self {
        // Clear every bit other than the root bit, which is preserved
        BorrowFlag(self.0 & ROOT)
    }

    pub(crate) fn add_reading(self) -> Self {
        assert!(self.borrowed() != BorrowState::Writing);
        // Add 1 to the integer starting at the second binary digit. As our
        // borrowstate is not writing, we know that overflow cannot happen, so
        // this is equivalent to the following, more complicated, expression:
        //
        // BorrowFlag((self.0 & ROOT) | (((self.0 >> 1) + 1) << 1))
        BorrowFlag(self.0 + 0b10)
    }

    pub(crate) fn sub_reading(self) -> Self {
        assert!(self.borrowed() == BorrowState::Reading);
        // Subtract 1 from the integer starting at the second binary digit. As
        // our borrowstate is not writing or unused, we know that overflow or
        // undeflow cannot happen, so this is equivalent to the following, more
        // complicated, expression:
        //
        // BorrowFlag((self.0 & ROOT) | (((self.0 >> 1) - 1) << 1))
        BorrowFlag(self.0 - 0b10)
    }

    pub(crate) fn set_rooted(self, rooted: bool) -> Self {
        // Preserve the non-root bits
        BorrowFlag((self.0 & !ROOT) | (rooted as usize))
    }
}
