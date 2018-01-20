pub mod load;

pub use self::load::*;

use std::any::Any;
use std::fmt;
use module::SubInstrument;
use util::NOTES;

/// ModInstrument defines extra instrument fields used in Protracker instruments.
#[derive(Debug)]
pub struct ModInstrument {
    pub finetune: isize,
    pub smp_num : usize,
}

impl ModInstrument {
    pub fn new() -> Self {
        ModInstrument {
            finetune: 0,
            smp_num : 0,
        }
    }
}

impl SubInstrument for ModInstrument {
    fn as_any(&self) -> &Any {
        self
    }

    fn sample_num(&self) -> usize {
        self.smp_num
    }
}


/// ModEvent defines the event format used in Protracker patterns.
pub struct ModEvent {
    pub note : u8,
    pub ins  : u8,
    pub cmd  : u8,
    pub cmdlo: u8,
}

impl ModEvent {
    fn from_slice(b: &[u8]) -> Self {
        ModEvent {
            note : PeriodTable::period_to_note_all((((b[0] & 0x0f) as u16) << 8) | b[1] as u16),
            ins  : (b[0] & 0xf0) | ((b[2] & 0xf0) >> 4),
            cmd  : b[2] & 0x0f,
            cmdlo: b[3],
        }
    }
}

impl fmt::Display for ModEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let note = if self.note == 0 {
            "---".to_owned()
        } else {
            format!("{}{}", NOTES[self.note as usize % 12], self.note / 12)
        };

        let ins = if self.ins == 0 {
            "--".to_owned()
        } else {
            format!("{:02x}", self.ins)
        };

        write!(f, "{} {} {:02X}{:02X}", note, ins, self.cmd, self.cmdlo)
    }
}


/// Protracker period table.
pub struct PeriodTable;

impl PeriodTable {
    const MT_PERIOD_TABLE: &'static [u16] = &[
    // Tuning 0, Normal
        856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480, 453,
        428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226,
        214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 113,
    // Tuning 1
        850, 802, 757, 715, 674, 637, 601, 567, 535, 505, 477, 450,
        425, 401, 379, 357, 337, 318, 300, 284, 268, 253, 239, 225,
        213, 201, 189, 179, 169, 159, 150, 142, 134, 126, 119, 113,
    // Tuning 2
        844, 796, 752, 709, 670, 632, 597, 563, 532, 502, 474, 447,
        422, 398, 376, 355, 335, 316, 298, 282, 266, 251, 237, 224,
        211, 199, 188, 177, 167, 158, 149, 141, 133, 125, 118, 112,
    // Tuning 3
        838, 791, 746, 704, 665, 628, 592, 559, 528, 498, 470, 444,
        419, 395, 373, 352, 332, 314, 296, 280, 264, 249, 235, 222,
        209, 198, 187, 176, 166, 157, 148, 140, 132, 125, 118, 111,
    // Tuning 4
        832, 785, 741, 699, 660, 623, 588, 555, 524, 495, 467, 441,
        416, 392, 370, 350, 330, 312, 294, 278, 262, 247, 233, 220,
        208, 196, 185, 175, 165, 156, 147, 139, 131, 124, 117, 110,
    // Tuning 5
        826, 779, 736, 694, 655, 619, 584, 551, 520, 491, 463, 437,
        413, 390, 368, 347, 328, 309, 292, 276, 260, 245, 232, 219,
        206, 195, 184, 174, 164, 155, 146, 138, 130, 123, 116, 109,
    // Tuning 6
        820, 774, 730, 689, 651, 614, 580, 547, 516, 487, 460, 434,
        410, 387, 365, 345, 325, 307, 290, 274, 258, 244, 230, 217,
        205, 193, 183, 172, 163, 154, 145, 137, 129, 122, 115, 109,
    // Tuning 7
        814, 768, 725, 684, 646, 610, 575, 543, 513, 484, 457, 431,
        407, 384, 363, 342, 323, 305, 288, 272, 256, 242, 228, 216,
        204, 192, 181, 171, 161, 152, 144, 136, 128, 121, 114, 108,
    // Tuning -8
        907, 856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480,
        453, 428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240,
        226, 214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120,
    // Tuning -7
        900, 850, 802, 757, 715, 675, 636, 601, 567, 535, 505, 477,
        450, 425, 401, 379, 357, 337, 318, 300, 284, 268, 253, 238,
        225, 212, 200, 189, 179, 169, 159, 150, 142, 134, 126, 119,
    // Tuning -6
        894, 844, 796, 752, 709, 670, 632, 597, 563, 532, 502, 474,
        447, 422, 398, 376, 355, 335, 316, 298, 282, 266, 251, 237,
        223, 211, 199, 188, 177, 167, 158, 149, 141, 133, 125, 118,
    // Tuning -5
        887, 838, 791, 746, 704, 665, 628, 592, 559, 528, 498, 470,
        444, 419, 395, 373, 352, 332, 314, 296, 280, 264, 249, 235,
        222, 209, 198, 187, 176, 166, 157, 148, 140, 132, 125, 118,
    // Tuning -4
        881, 832, 785, 741, 699, 660, 623, 588, 555, 524, 494, 467,
        441, 416, 392, 370, 350, 330, 312, 294, 278, 262, 247, 233,
        220, 208, 196, 185, 175, 165, 156, 147, 139, 131, 123, 117,
    // Tuning -3
        875, 826, 779, 736, 694, 655, 619, 584, 551, 520, 491, 463,
        437, 413, 390, 368, 347, 328, 309, 292, 276, 260, 245, 232,
        219, 206, 195, 184, 174, 164, 155, 146, 138, 130, 123, 116,
    // Tuning -2
        868, 820, 774, 730, 689, 651, 614, 580, 547, 516, 487, 460,
        434, 410, 387, 365, 345, 325, 307, 290, 274, 258, 244, 230,
        217, 205, 193, 183, 172, 163, 154, 145, 137, 129, 122, 115,
    // Tuning -1
        862, 814, 768, 725, 684, 646, 610, 575, 543, 513, 484, 457,
        431, 407, 384, 363, 342, 323, 305, 288, 272, 256, 242, 228,
        216, 203, 192, 181, 171, 161, 152, 144, 136, 128, 121, 114
    ];

    fn finetune(mut fine: i8) -> usize {
        fine >>= 4;
        clamp!(fine, -8, 7);
        if fine < 0 {
           fine += 16;
        }
        fine as usize
    }

    pub fn note_to_period(mut note: u8, mut fine: i8) -> u16 {
        clamp!(note, 48, 83);
        note -= 48;
        Self::MT_PERIOD_TABLE[Self::finetune(fine) * 36 + note as usize]
    }

    pub fn period_to_note(period: u16, mut fine: i8) -> u8 {
        if period == 0 {
            return 0;
        }

        let ofs = Self::finetune(fine) * 36;
        let mut note = 48;
        for p in Self::MT_PERIOD_TABLE[ofs..ofs+36].iter() {
            if period >= *p {
               break;
            }
            note += 1;
        }
        note
    }

    fn period_to_note_all(period: u16) -> u8 {
        if period == 0 {
            return 0;
        }

        let mut note = 0;
        for p in Self::MT_PERIOD_TABLE[0..16*36].iter() {
            if period == *p {
               break;
            }
            note += 1;
        }
        48 + (note % 36)
    }
}
