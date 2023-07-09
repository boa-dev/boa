use bitflags::bitflags;
pub(crate) type SlotIndex = u32;

bitflags! {
    /// Attributes of a slot.
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct SlotAttributes: u8 {
        const WRITABLE = 0b0000_0001;
        const ENUMERABLE = 0b0000_0010;
        const CONFIGURABLE = 0b0000_0100;
        const GET = 0b0000_1000;
        const SET = 0b0001_0000;
    }
}

impl SlotAttributes {
    pub(crate) const fn is_accessor_descriptor(self) -> bool {
        self.contains(Self::GET) || self.contains(Self::SET)
    }

    pub(crate) const fn has_get(self) -> bool {
        self.contains(Self::GET)
    }
    pub(crate) const fn has_set(self) -> bool {
        self.contains(Self::SET)
    }

    /// Check if slot type width matches, this can only happens,
    /// if they are both accessors, or both data properties.
    pub(crate) const fn width_match(self, other: Self) -> bool {
        self.is_accessor_descriptor() == other.is_accessor_descriptor()
    }

    /// Get the width of the slot.
    pub(crate) fn width(self) -> u32 {
        // accessor take 2 positions in the storage to accomodate for the `get` and `set` fields.
        1 + u32::from(self.is_accessor_descriptor())
    }
}

/// Represents an [`u32`] index and it's slot attributes of an element in a object storage.
///
/// Slots can have different width depending on its attributes, accessors properties have width `2`,
/// while data properties have width `1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Slot {
    pub(crate) index: SlotIndex,
    pub(crate) attributes: SlotAttributes,
}

impl Slot {
    /// Get the width of the slot.
    pub(crate) fn width(self) -> u32 {
        self.attributes.width()
    }

    /// Calculate next slot from previous one.
    ///
    /// This is needed because slots do not have the same width.
    pub(crate) fn from_previous(
        previous_slot: Option<Self>,
        new_attributes: SlotAttributes,
    ) -> Self {
        // If there was no previous slot then return 0 as the index.
        let Some(previous_slot) = previous_slot else {
            return Self {
                index: 0,
                attributes: new_attributes,
            };
        };

        Self {
            index: previous_slot.index + previous_slot.width(),
            attributes: new_attributes,
        }
    }
}
