use core::{cmp, convert::TryFrom, slice, str};

const MIN_VERSION: u8 = 1;
const MAX_VERSION: u8 = 40;
const MAX_SIZE: usize = (MAX_VERSION as usize) * 4 + 17;
const MAX_BUFFER_LEN: usize = (MAX_SIZE * MAX_SIZE).div_ceil(8) + 1;
const MAX_BINARY_LEN: u32 = 2953;
const MAX_TEXT_LEN: u32 = 7089;
const MAX_ECC_LEN: usize = 30;

const ERR_NULL_POINTER: i32 = -1;
const ERR_OUTPUT_TOO_SHORT: i32 = -2;
const ERR_INVALID_ECC: i32 = -3;
const ERR_INVALID_VERSION: i32 = -4;
const ERR_INVALID_MASK: i32 = -5;
const ERR_DATA_TOO_LONG: i32 = -6;
const ERR_INVALID_UTF8: i32 = -7;
const ERR_INPUT_TOO_LONG: i32 = -8;

#[unsafe(no_mangle)]
pub extern "C" fn kyuaru_encode_text_utf8(
    data: *const u8,
    data_len: u32,
    ecl: u8,
    min_version: u8,
    max_version: u8,
    mask: i8,
    boost_ecl: u8,
    out: *mut u8,
    out_len: u32,
) -> i32 {
    encode_ffi(
        data,
        data_len,
        ecl,
        min_version,
        max_version,
        mask,
        boost_ecl,
        out,
        out_len,
        true,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn kyuaru_encode_binary(
    data: *const u8,
    data_len: u32,
    ecl: u8,
    min_version: u8,
    max_version: u8,
    mask: i8,
    boost_ecl: u8,
    out: *mut u8,
    out_len: u32,
) -> i32 {
    encode_ffi(
        data,
        data_len,
        ecl,
        min_version,
        max_version,
        mask,
        boost_ecl,
        out,
        out_len,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn encode_ffi(
    data: *const u8,
    data_len: u32,
    ecl: u8,
    min_version: u8,
    max_version: u8,
    mask: i8,
    boost_ecl: u8,
    out: *mut u8,
    out_len: u32,
    text_mode: bool,
) -> i32 {
    let Some(ecl) = QrCodeEcc::from_ffi(ecl) else {
        return ERR_INVALID_ECC;
    };
    let Some(min_version) = Version::try_new(min_version) else {
        return ERR_INVALID_VERSION;
    };
    let Some(max_version) = Version::try_new(max_version) else {
        return ERR_INVALID_VERSION;
    };
    if min_version > max_version {
        return ERR_INVALID_VERSION;
    }
    let Some(mask) = Mask::from_ffi(mask) else {
        return ERR_INVALID_MASK;
    };
    if out.is_null() || (data.is_null() && data_len != 0) {
        return ERR_NULL_POINTER;
    }
    if data_len
        > if text_mode {
            MAX_TEXT_LEN
        } else {
            MAX_BINARY_LEN
        }
    {
        return ERR_INPUT_TOO_LONG;
    }

    let needed_out_len = max_version.buffer_len();
    if usize::try_from(out_len).map_or(true, |len| len < needed_out_len) {
        return ERR_OUTPUT_TOO_SHORT;
    }

    let input = if data_len == 0 {
        &[]
    } else {
        // SAFETY: The FFI caller supplies a pointer to at least data_len bytes.
        // The public API bounds data_len to the QR standard maximum before this.
        unsafe { slice::from_raw_parts(data, data_len as usize) }
    };

    if text_mode && str::from_utf8(input).is_err() {
        return ERR_INVALID_UTF8;
    }

    // SAFETY: The output pointer is checked for null and out_len is checked
    // against the version-specific capacity required by the encoder.
    let out = unsafe { slice::from_raw_parts_mut(out, out_len as usize) };
    let mut temp = [0u8; MAX_BUFFER_LEN];

    match encode_data(
        input,
        text_mode,
        &mut temp,
        out,
        ecl,
        min_version,
        max_version,
        mask,
        boost_ecl != 0,
    ) {
        Ok(size) => i32::from(size),
        Err(DataTooLong::SegmentTooLong | DataTooLong::DataOverCapacity) => ERR_DATA_TOO_LONG,
    }
}

#[allow(clippy::too_many_arguments)]
fn encode_data(
    data: &[u8],
    text_mode: bool,
    tempbuffer: &mut [u8; MAX_BUFFER_LEN],
    outbuffer: &mut [u8],
    ecl: QrCodeEcc,
    minversion: Version,
    maxversion: Version,
    mask: Option<Mask>,
    boostecl: bool,
) -> Result<u8, DataTooLong> {
    let buflen = maxversion.buffer_len();
    let tempbuffer = &mut tempbuffer[..buflen];
    let outbuffer = &mut outbuffer[..buflen];

    let (datacodewordslen, ecl, version) = if data.is_empty() {
        QrCode::encode_segments_to_codewords(&[], outbuffer, ecl, minversion, maxversion, boostecl)?
    } else {
        let text_segment_mode = text_mode.then(|| classify_text_segment_mode(data));
        let segment = if text_segment_mode == Some(QrSegmentMode::Numeric)
            && QrSegment::calc_buffer_size(QrSegmentMode::Numeric, data.len())
                .is_some_and(|len| len <= tempbuffer.len())
        {
            QrSegment::make_numeric(data, tempbuffer)
        } else if text_segment_mode == Some(QrSegmentMode::Alphanumeric)
            && QrSegment::calc_buffer_size(QrSegmentMode::Alphanumeric, data.len())
                .is_some_and(|len| len <= tempbuffer.len())
        {
            QrSegment::make_alphanumeric(data, tempbuffer)
        } else if QrSegment::calc_buffer_size(QrSegmentMode::Byte, data.len())
            .is_some_and(|len| len <= outbuffer.len())
        {
            QrSegment::make_bytes(data)
        } else {
            return Err(DataTooLong::SegmentTooLong);
        };
        QrCode::encode_segments_to_codewords(
            core::slice::from_ref(&segment),
            outbuffer,
            ecl,
            minversion,
            maxversion,
            boostecl,
        )?
    };

    let qr = QrCode::encode_codewords(outbuffer, datacodewordslen, tempbuffer, ecl, version, mask);
    Ok(qr.size())
}

struct QrCode<'a> {
    size: &'a mut u8,
    modules: &'a mut [u8],
}

impl<'a> QrCode<'a> {
    fn encode_segments_to_codewords(
        segs: &[QrSegment<'_>],
        outbuffer: &'a mut [u8],
        mut ecl: QrCodeEcc,
        minversion: Version,
        maxversion: Version,
        boostecl: bool,
    ) -> Result<(usize, QrCodeEcc, Version), DataTooLong> {
        debug_assert!(minversion <= maxversion);
        debug_assert!(outbuffer.len() >= QrCode::get_num_data_codewords(maxversion, ecl));

        let mut version = minversion;
        let datausedbits = loop {
            let datacapacitybits = QrCode::get_num_data_codewords(version, ecl) * 8;
            let dataused = QrSegment::get_total_bits(segs, version);
            if dataused.is_some_and(|bits| bits <= datacapacitybits) {
                break dataused.unwrap();
            } else if version >= maxversion {
                return Err(match dataused {
                    None => DataTooLong::SegmentTooLong,
                    Some(_) => DataTooLong::DataOverCapacity,
                });
            } else {
                version = Version::new(version.value() + 1);
            }
        };

        for &newecl in &[QrCodeEcc::Medium, QrCodeEcc::Quartile, QrCodeEcc::High] {
            if boostecl && datausedbits <= QrCode::get_num_data_codewords(version, newecl) * 8 {
                ecl = newecl;
            }
        }

        let datacapacitybits = QrCode::get_num_data_codewords(version, ecl) * 8;
        let mut bb = BitBuffer::new(&mut outbuffer[..datacapacitybits / 8]);
        for seg in segs {
            bb.append_bits(seg.mode.mode_bits(), 4);
            bb.append_bits(
                u32::try_from(seg.numchars).unwrap(),
                seg.mode.num_char_count_bits(version),
            );
            bb.append_data(seg.data, seg.bitlength);
        }
        debug_assert_eq!(bb.length, datausedbits);

        let numzerobits = cmp::min(4, datacapacitybits - bb.length);
        bb.append_bits(0, u8::try_from(numzerobits).unwrap());
        let numzerobits = bb.length.wrapping_neg() & 7;
        bb.append_bits(0, u8::try_from(numzerobits).unwrap());
        debug_assert_eq!(bb.length % 8, 0);

        let mut padbyte = 0xEC;
        while bb.length < datacapacitybits {
            bb.append_bits(padbyte, 8);
            padbyte ^= 0xEC ^ 0x11;
        }

        Ok((bb.length / 8, ecl, version))
    }

    fn encode_codewords<'b>(
        mut datacodewordsandoutbuffer: &'a mut [u8],
        datacodewordslen: usize,
        mut tempbuffer: &'b mut [u8],
        ecl: QrCodeEcc,
        version: Version,
        mut mask: Option<Mask>,
    ) -> QrCode<'a> {
        datacodewordsandoutbuffer = &mut datacodewordsandoutbuffer[..version.buffer_len()];
        tempbuffer = &mut tempbuffer[..version.buffer_len()];

        let rawcodewords = QrCode::get_num_raw_data_modules(version) / 8;
        debug_assert!(datacodewordslen <= rawcodewords);
        let (data, temp) = datacodewordsandoutbuffer.split_at_mut(datacodewordslen);
        let allcodewords = Self::add_ecc_and_interleave(data, version, ecl, temp, tempbuffer);

        let mut result = QrCode::function_modules_marked(datacodewordsandoutbuffer, version);
        result.draw_codewords(allcodewords);
        result.draw_light_function_modules();
        let funcmods = QrCode::function_modules_marked(tempbuffer, version);

        if mask.is_none() {
            let mut minpenalty = i32::MAX;
            for i in 0..8 {
                let candidate = Mask::new(i);
                result.apply_mask(&funcmods, candidate);
                result.draw_format_bits(ecl, candidate);
                let penalty = result.get_penalty_score();
                if penalty < minpenalty {
                    mask = Some(candidate);
                    minpenalty = penalty;
                }
                result.apply_mask(&funcmods, candidate);
            }
        }

        let mask = mask.unwrap();
        result.apply_mask(&funcmods, mask);
        result.draw_format_bits(ecl, mask);
        result
    }

    #[inline]
    fn version(&self) -> Version {
        Version::new((*self.size - 17) / 4)
    }

    #[inline]
    fn size(&self) -> u8 {
        *self.size
    }

    #[inline]
    fn get_module_bounded(&self, x: u8, y: u8) -> bool {
        debug_assert!(x < *self.size && y < *self.size);
        let index = usize::from(y) * usize::from(*self.size) + usize::from(x);
        get_bit(u32::from(self.modules[index >> 3]), (index & 7) as u8)
    }

    #[inline]
    fn set_module_unbounded(&mut self, x: i32, y: i32, isdark: bool) {
        if (0..i32::from(*self.size)).contains(&x) && (0..i32::from(*self.size)).contains(&y) {
            self.set_module_bounded(x as u8, y as u8, isdark);
        }
    }

    #[inline]
    fn set_module_bounded(&mut self, x: u8, y: u8, isdark: bool) {
        debug_assert!(x < *self.size && y < *self.size);
        let index = usize::from(y) * usize::from(*self.size) + usize::from(x);
        let mask = 1u8 << (index & 7);
        let byte = &mut self.modules[index >> 3];
        if isdark {
            *byte |= mask;
        } else {
            *byte &= !mask;
        }
    }

    #[inline]
    fn flip_module_bounded(&mut self, x: u8, y: u8) {
        debug_assert!(x < *self.size && y < *self.size);
        let index = usize::from(y) * usize::from(*self.size) + usize::from(x);
        self.modules[index >> 3] ^= 1u8 << (index & 7);
    }

    fn add_ecc_and_interleave<'b>(
        data: &[u8],
        ver: Version,
        ecl: QrCodeEcc,
        temp: &mut [u8],
        resultbuf: &'b mut [u8],
    ) -> &'b [u8] {
        debug_assert_eq!(data.len(), QrCode::get_num_data_codewords(ver, ecl));

        let numblocks = QrCode::table_get(&NUM_ERROR_CORRECTION_BLOCKS, ver, ecl);
        let blockecclen = QrCode::table_get(&ECC_CODEWORDS_PER_BLOCK, ver, ecl);
        let rawcodewords = QrCode::get_num_raw_data_modules(ver) / 8;
        let numshortblocks = numblocks - rawcodewords % numblocks;
        let shortblockdatalen = rawcodewords / numblocks - blockecclen;
        let result = &mut resultbuf[..rawcodewords];
        let divisor = &RS_GENERATORS[blockecclen][..blockecclen];
        let ecc = &mut temp[..blockecclen];
        let mut dat = data;

        for i in 0..numblocks {
            let datlen = shortblockdatalen + usize::from(i >= numshortblocks);
            reed_solomon_remainder(&dat[..datlen], divisor, ecc);

            let mut k = i;
            for (j, &value) in dat[..datlen].iter().enumerate() {
                if j == shortblockdatalen {
                    k -= numshortblocks;
                }
                result[k] = value;
                k += numblocks;
            }

            k = data.len() + i;
            for &value in ecc.iter() {
                result[k] = value;
                k += numblocks;
            }
            dat = &dat[datlen..];
        }
        debug_assert!(dat.is_empty());
        result
    }

    fn function_modules_marked(outbuffer: &'a mut [u8], ver: Version) -> Self {
        debug_assert_eq!(outbuffer.len(), ver.buffer_len());
        let (size_byte, modules) = outbuffer.split_first_mut().unwrap();
        let mut result = Self {
            size: size_byte,
            modules,
        };
        let size = ver.value() * 4 + 17;
        *result.size = size;
        result.modules.fill(0);

        result.fill_rectangle(6, 0, 1, size);
        result.fill_rectangle(0, 6, size, 1);
        result.fill_rectangle(0, 0, 9, 9);
        result.fill_rectangle(size - 8, 0, 8, 9);
        result.fill_rectangle(0, size - 8, 9, 8);

        let mut alignpatposbuf = [0u8; 7];
        let alignpatpos = result.get_alignment_pattern_positions(&mut alignpatposbuf);
        for (i, &pos0) in alignpatpos.iter().enumerate() {
            for (j, &pos1) in alignpatpos.iter().enumerate() {
                if !alignment_pattern_overlaps_finder(i, j, alignpatpos.len() - 1) {
                    result.fill_rectangle(pos0 - 2, pos1 - 2, 5, 5);
                }
            }
        }

        if ver.value() >= 7 {
            result.fill_rectangle(size - 11, 0, 3, 6);
            result.fill_rectangle(0, size - 11, 6, 3);
        }
        result
    }

    fn draw_light_function_modules(&mut self) {
        let size = *self.size;
        for i in (7..size - 7).step_by(2) {
            self.set_module_bounded(6, i, false);
            self.set_module_bounded(i, 6, false);
        }

        for dy in -4i32..=4 {
            for dx in -4i32..=4 {
                let dist = dx.abs().max(dy.abs());
                if dist == 2 || dist == 4 {
                    self.set_module_unbounded(3 + dx, 3 + dy, false);
                    self.set_module_unbounded(i32::from(size) - 4 + dx, 3 + dy, false);
                    self.set_module_unbounded(3 + dx, i32::from(size) - 4 + dy, false);
                }
            }
        }

        let mut alignpatposbuf = [0u8; 7];
        let alignpatpos = self.get_alignment_pattern_positions(&mut alignpatposbuf);
        for (i, &pos0) in alignpatpos.iter().enumerate() {
            for (j, &pos1) in alignpatpos.iter().enumerate() {
                if alignment_pattern_overlaps_finder(i, j, alignpatpos.len() - 1) {
                    continue;
                }
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        self.set_module_bounded(
                            (i32::from(pos0) + dx) as u8,
                            (i32::from(pos1) + dy) as u8,
                            dx == 0 && dy == 0,
                        );
                    }
                }
            }
        }

        let ver = u32::from(self.version().value());
        if ver >= 7 {
            let bits = {
                let mut rem = ver;
                for _ in 0..12 {
                    rem = (rem << 1) ^ ((rem >> 11) * 0x1F25);
                }
                ver << 12 | rem
            };
            for i in 0u8..18 {
                let bit = get_bit(bits, i);
                let a = size - 11 + i % 3;
                let b = i / 3;
                self.set_module_bounded(a, b, bit);
                self.set_module_bounded(b, a, bit);
            }
        }
    }

    fn draw_format_bits(&mut self, ecl: QrCodeEcc, mask: Mask) {
        let bits = {
            let data = u32::from(ecl.format_bits() << 3 | mask.value());
            let mut rem = data;
            for _ in 0..10 {
                rem = (rem << 1) ^ ((rem >> 9) * 0x537);
            }
            (data << 10 | rem) ^ 0x5412
        };

        for i in 0..6 {
            self.set_module_bounded(8, i, get_bit(bits, i));
        }
        self.set_module_bounded(8, 7, get_bit(bits, 6));
        self.set_module_bounded(8, 8, get_bit(bits, 7));
        self.set_module_bounded(7, 8, get_bit(bits, 8));
        for i in 9..15 {
            self.set_module_bounded(14 - i, 8, get_bit(bits, i));
        }

        let size = *self.size;
        for i in 0..8 {
            self.set_module_bounded(size - 1 - i, 8, get_bit(bits, i));
        }
        for i in 8..15 {
            self.set_module_bounded(8, size - 15 + i, get_bit(bits, i));
        }
        self.set_module_bounded(8, size - 8, true);
    }

    fn fill_rectangle(&mut self, left: u8, top: u8, width: u8, height: u8) {
        for y in top..top + height {
            for x in left..left + width {
                self.set_module_bounded(x, y, true);
            }
        }
    }

    fn draw_codewords(&mut self, data: &[u8]) {
        debug_assert_eq!(
            data.len(),
            QrCode::get_num_raw_data_modules(self.version()) / 8
        );
        let size = i32::from(*self.size);
        let mut i = 0usize;
        let mut right = size - 1;

        while right >= 1 {
            if right == 6 {
                right = 5;
            }
            for vert in 0..size {
                for j in 0..2 {
                    let x = (right - j) as u8;
                    let upward = (right + 1) & 2 == 0;
                    let y = if upward { size - 1 - vert } else { vert } as u8;
                    if !self.get_module_bounded(x, y) && i < data.len() * 8 {
                        self.set_module_bounded(
                            x,
                            y,
                            get_bit(u32::from(data[i >> 3]), 7 - ((i as u8) & 7)),
                        );
                        i += 1;
                    }
                }
            }
            right -= 2;
        }
        debug_assert_eq!(i, data.len() * 8);
    }

    fn apply_mask(&mut self, functionmodules: &QrCode<'_>, mask: Mask) {
        match mask.value() {
            0 => self.apply_mask_with(functionmodules, |x, y| {
                (u16::from(x) + u16::from(y)) & 1 == 0
            }),
            1 => self.apply_mask_with(functionmodules, |_, y| y & 1 == 0),
            2 => self.apply_mask_with(functionmodules, |x, _| x % 3 == 0),
            3 => self.apply_mask_with(functionmodules, |x, y| {
                (u16::from(x) + u16::from(y)) % 3 == 0
            }),
            4 => self.apply_mask_with(functionmodules, |x, y| (x / 3 + y / 2) & 1 == 0),
            5 => self.apply_mask_with(functionmodules, |x, y| {
                let xy = u16::from(x) * u16::from(y);
                xy % 2 + xy % 3 == 0
            }),
            6 => self.apply_mask_with(functionmodules, |x, y| {
                let xy = u16::from(x) * u16::from(y);
                (xy % 2 + xy % 3) & 1 == 0
            }),
            7 => self.apply_mask_with(functionmodules, |x, y| {
                let xy = u16::from(x) * u16::from(y);
                (((u16::from(x) + u16::from(y)) & 1) + xy % 3) & 1 == 0
            }),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn apply_mask_with<F>(&mut self, functionmodules: &QrCode<'_>, should_invert: F)
    where
        F: Fn(u8, u8) -> bool,
    {
        let size = *self.size;
        for y in 0..size {
            for x in 0..size {
                if !functionmodules.get_module_bounded(x, y) && should_invert(x, y) {
                    self.flip_module_bounded(x, y);
                }
            }
        }
    }

    fn get_penalty_score(&self) -> i32 {
        let mut result = 0;
        let size = *self.size;

        for y in 0..size {
            let mut runcolor = false;
            let mut runx = 0;
            let mut runhistory = FinderPenalty::new(size);
            for x in 0..size {
                if self.get_module_bounded(x, y) == runcolor {
                    runx += 1;
                    if runx == 5 {
                        result += PENALTY_N1;
                    } else if runx > 5 {
                        result += 1;
                    }
                } else {
                    runhistory.add_history(runx);
                    if !runcolor {
                        result += runhistory.count_patterns() * PENALTY_N3;
                    }
                    runcolor = self.get_module_bounded(x, y);
                    runx = 1;
                }
            }
            result += runhistory.terminate_and_count(runcolor, runx) * PENALTY_N3;
        }

        for x in 0..size {
            let mut runcolor = false;
            let mut runy = 0;
            let mut runhistory = FinderPenalty::new(size);
            for y in 0..size {
                if self.get_module_bounded(x, y) == runcolor {
                    runy += 1;
                    if runy == 5 {
                        result += PENALTY_N1;
                    } else if runy > 5 {
                        result += 1;
                    }
                } else {
                    runhistory.add_history(runy);
                    if !runcolor {
                        result += runhistory.count_patterns() * PENALTY_N3;
                    }
                    runcolor = self.get_module_bounded(x, y);
                    runy = 1;
                }
            }
            result += runhistory.terminate_and_count(runcolor, runy) * PENALTY_N3;
        }

        for y in 0..size - 1 {
            for x in 0..size - 1 {
                let color = self.get_module_bounded(x, y);
                if color == self.get_module_bounded(x + 1, y)
                    && color == self.get_module_bounded(x, y + 1)
                    && color == self.get_module_bounded(x + 1, y + 1)
                {
                    result += PENALTY_N2;
                }
            }
        }

        let dark = self
            .modules
            .iter()
            .map(|value| value.count_ones())
            .sum::<u32>() as i32;
        let total = i32::from(size) * i32::from(size);
        let k = ((dark * 20 - total * 10).abs() + total - 1) / total - 1;
        result + k * PENALTY_N4
    }

    fn get_alignment_pattern_positions<'b>(&self, resultbuf: &'b mut [u8; 7]) -> &'b [u8] {
        let ver = self.version().value();
        if ver == 1 {
            &resultbuf[..0]
        } else {
            let numalign = ver / 7 + 2;
            let step = u8::try_from(
                (i32::from(ver) * 8 + i32::from(numalign) * 3 + 5) / (i32::from(numalign) * 4 - 4)
                    * 2,
            )
            .unwrap();
            let result = &mut resultbuf[..usize::from(numalign)];
            for i in 0..numalign - 1 {
                result[usize::from(i)] = *self.size - 7 - i * step;
            }
            *result.last_mut().unwrap() = 6;
            result.reverse();
            result
        }
    }

    fn get_num_raw_data_modules(ver: Version) -> usize {
        let ver = usize::from(ver.value());
        let mut result = (16 * ver + 128) * ver + 64;
        if ver >= 2 {
            let numalign = ver / 7 + 2;
            result -= (25 * numalign - 10) * numalign - 55;
            if ver >= 7 {
                result -= 36;
            }
        }
        result
    }

    fn get_num_data_codewords(ver: Version, ecl: QrCodeEcc) -> usize {
        QrCode::get_num_raw_data_modules(ver) / 8
            - QrCode::table_get(&ECC_CODEWORDS_PER_BLOCK, ver, ecl)
                * QrCode::table_get(&NUM_ERROR_CORRECTION_BLOCKS, ver, ecl)
    }

    #[inline]
    fn table_get(table: &'static [[i16; 41]; 4], ver: Version, ecl: QrCodeEcc) -> usize {
        table[ecl.ordinal()][usize::from(ver.value())] as usize
    }
}

struct QrSegment<'a> {
    mode: QrSegmentMode,
    numchars: usize,
    data: &'a [u8],
    bitlength: usize,
}

impl<'a> QrSegment<'a> {
    #[inline]
    fn make_bytes(data: &'a [u8]) -> Self {
        Self::new(
            QrSegmentMode::Byte,
            data.len(),
            data,
            data.len().checked_mul(8).unwrap(),
        )
    }

    fn make_numeric(text: &[u8], buf: &'a mut [u8]) -> Self {
        debug_assert_eq!(classify_text_segment_mode(text), QrSegmentMode::Numeric);
        let mut bb = BitBuffer::new(buf);
        for chunk in text.chunks(3) {
            let value = chunk
                .iter()
                .fold(0u32, |acc, &b| acc * 10 + u32::from(b - b'0'));
            bb.append_bits(value, (chunk.len() as u8) * 3 + 1);
        }
        Self::new(QrSegmentMode::Numeric, text.len(), bb.data, bb.length)
    }

    fn make_alphanumeric(text: &[u8], buf: &'a mut [u8]) -> Self {
        debug_assert!(matches!(
            classify_text_segment_mode(text),
            QrSegmentMode::Numeric | QrSegmentMode::Alphanumeric
        ));
        let mut bb = BitBuffer::new(buf);
        for chunk in text.chunks(2) {
            let value = chunk.iter().fold(0u32, |acc, &b| {
                acc * 45 + u32::from(alphanumeric_value(b).unwrap())
            });
            bb.append_bits(value, (chunk.len() as u8) * 5 + 1);
        }
        Self::new(QrSegmentMode::Alphanumeric, text.len(), bb.data, bb.length)
    }

    #[inline]
    fn new(mode: QrSegmentMode, numchars: usize, data: &'a [u8], bitlength: usize) -> Self {
        debug_assert!(bitlength == 0 || (bitlength - 1) / 8 < data.len());
        Self {
            mode,
            numchars,
            data,
            bitlength,
        }
    }

    fn calc_buffer_size(mode: QrSegmentMode, numchars: usize) -> Option<usize> {
        let bits = Self::calc_bit_length(mode, numchars)?;
        Some(bits.div_ceil(8))
    }

    fn calc_bit_length(mode: QrSegmentMode, numchars: usize) -> Option<usize> {
        let mul_frac_ceil = |numer: usize, denom: usize| {
            numchars
                .checked_mul(numer)
                .and_then(|x| x.checked_add(denom - 1))
                .map(|x| x / denom)
        };

        match mode {
            QrSegmentMode::Numeric => mul_frac_ceil(10, 3),
            QrSegmentMode::Alphanumeric => mul_frac_ceil(11, 2),
            QrSegmentMode::Byte => mul_frac_ceil(8, 1),
        }
    }

    fn get_total_bits(segs: &[Self], version: Version) -> Option<usize> {
        let mut result = 0usize;
        for seg in segs {
            let ccbits = seg.mode.num_char_count_bits(version);
            if let Some(limit) = 1usize.checked_shl(u32::from(ccbits))
                && seg.numchars >= limit
            {
                return None;
            }
            result = result.checked_add(4 + usize::from(ccbits))?;
            result = result.checked_add(seg.bitlength)?;
        }
        Some(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QrSegmentMode {
    Numeric,
    Alphanumeric,
    Byte,
}

impl QrSegmentMode {
    #[inline]
    fn mode_bits(self) -> u32 {
        match self {
            Self::Numeric => 0x1,
            Self::Alphanumeric => 0x2,
            Self::Byte => 0x4,
        }
    }

    #[inline]
    fn num_char_count_bits(self, ver: Version) -> u8 {
        let index = usize::from((ver.value() + 7) / 17);
        match self {
            Self::Numeric => [10, 12, 14][index],
            Self::Alphanumeric => [9, 11, 13][index],
            Self::Byte => [8, 16, 16][index],
        }
    }
}

struct BitBuffer<'a> {
    data: &'a mut [u8],
    length: usize,
}

impl<'a> BitBuffer<'a> {
    #[inline]
    fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            data: buffer,
            length: 0,
        }
    }

    #[inline]
    fn append_bits(&mut self, val: u32, len: u8) {
        debug_assert!(len <= 31 && val >> len == 0);

        let mut remaining = usize::from(len);
        while remaining > 0 {
            let index = self.length >> 3;
            let bit_offset = self.length & 7;
            let free_bits = 8 - bit_offset;
            let write_bits = remaining.min(free_bits);
            let shift = remaining - write_bits;
            let mask = if write_bits == 8 {
                u8::MAX
            } else {
                (1u8 << write_bits) - 1
            };
            let bits = ((val >> shift) as u8) & mask;

            if bit_offset == 0 {
                self.data[index] = 0;
            }
            self.data[index] |= bits << (free_bits - write_bits);
            self.length += write_bits;
            remaining -= write_bits;
        }
    }

    fn append_data(&mut self, data: &[u8], bitlength: usize) {
        debug_assert!(bitlength == 0 || (bitlength - 1) / 8 < data.len());

        let full_bytes = bitlength >> 3;
        for &byte in &data[..full_bytes] {
            self.append_bits(u32::from(byte), 8);
        }

        let remaining_bits = (bitlength & 7) as u8;
        if remaining_bits != 0 {
            self.append_bits(
                u32::from(data[full_bytes] >> (8 - remaining_bits)),
                remaining_bits,
            );
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum QrCodeEcc {
    Low,
    Medium,
    Quartile,
    High,
}

impl QrCodeEcc {
    #[inline]
    fn from_ffi(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Low),
            1 => Some(Self::Medium),
            2 => Some(Self::Quartile),
            3 => Some(Self::High),
            _ => None,
        }
    }

    #[inline]
    fn ordinal(self) -> usize {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::Quartile => 2,
            Self::High => 3,
        }
    }

    #[inline]
    fn format_bits(self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 0,
            Self::Quartile => 3,
            Self::High => 2,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version(u8);

impl Version {
    #[inline]
    const fn new(version: u8) -> Self {
        debug_assert!(version >= MIN_VERSION && version <= MAX_VERSION);
        Self(version)
    }

    #[inline]
    fn try_new(version: u8) -> Option<Self> {
        if (MIN_VERSION..=MAX_VERSION).contains(&version) {
            Some(Self(version))
        } else {
            None
        }
    }

    #[inline]
    const fn value(self) -> u8 {
        self.0
    }

    #[inline]
    const fn buffer_len(self) -> usize {
        let sidelen = (self.0 as usize) * 4 + 17;
        (sidelen * sidelen).div_ceil(8) + 1
    }
}

#[derive(Clone, Copy)]
struct Mask(u8);

impl Mask {
    #[inline]
    const fn new(mask: u8) -> Self {
        debug_assert!(mask <= 7);
        Self(mask)
    }

    #[inline]
    fn from_ffi(mask: i8) -> Option<Option<Self>> {
        match mask {
            -1 => Some(None),
            0..=7 => Some(Some(Self(mask as u8))),
            _ => None,
        }
    }

    #[inline]
    const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
enum DataTooLong {
    SegmentTooLong,
    DataOverCapacity,
}

fn classify_text_segment_mode(data: &[u8]) -> QrSegmentMode {
    let mut numeric = true;
    for &b in data {
        if !b.is_ascii_digit() {
            numeric = false;
            if alphanumeric_value(b).is_none() {
                return QrSegmentMode::Byte;
            }
        }
    }
    if numeric {
        QrSegmentMode::Numeric
    } else {
        QrSegmentMode::Alphanumeric
    }
}

#[inline]
fn alphanumeric_value(value: u8) -> Option<u8> {
    if value < 128 {
        let mapped = ALPHANUMERIC_VALUES[usize::from(value)];
        if mapped >= 0 {
            return Some(mapped as u8);
        }
    }
    None
}

#[inline]
fn alignment_pattern_overlaps_finder(i: usize, j: usize, last: usize) -> bool {
    (i == 0 && (j == 0 || j == last)) || (i == last && j == 0)
}

#[inline]
fn get_bit(x: u32, i: u8) -> bool {
    (x >> i) & 1 != 0
}

fn reed_solomon_remainder(data: &[u8], divisor: &[u8], result: &mut [u8]) {
    debug_assert_eq!(divisor.len(), result.len());
    result.fill(0);
    for &byte in data {
        let factor = byte ^ result[0];
        for i in 0..result.len() - 1 {
            result[i] = result[i + 1];
        }
        *result.last_mut().unwrap() = 0;
        if factor != 0 {
            for (x, &y) in result.iter_mut().zip(divisor) {
                *x ^= gf_multiply(y, factor);
            }
        }
    }
}

#[inline]
fn gf_multiply(x: u8, y: u8) -> u8 {
    if x == 0 || y == 0 {
        0
    } else {
        GF_EXP[usize::from(GF_LOG[usize::from(x)]) + usize::from(GF_LOG[usize::from(y)])]
    }
}

const fn gf_multiply_const(x: u8, y: u8) -> u8 {
    let mut z = 0u8;
    let mut i = 8u8;
    while i > 0 {
        i -= 1;
        z = (z << 1) ^ ((z >> 7) * 0x1D);
        z ^= ((y >> i) & 1) * x;
    }
    z
}

const fn build_gf_tables() -> ([u8; 512], [u8; 256]) {
    let mut exp = [0u8; 512];
    let mut log = [0u8; 256];
    let mut x = 1u16;
    let mut i = 0usize;

    while i < 255 {
        exp[i] = x as u8;
        log[x as usize] = i as u8;
        x <<= 1;
        if x & 0x100 != 0 {
            x ^= 0x11D;
        }
        i += 1;
    }
    while i < 512 {
        exp[i] = exp[i - 255];
        i += 1;
    }

    (exp, log)
}

const fn build_rs_generator(degree: usize) -> [u8; MAX_ECC_LEN] {
    let mut divisor = [0u8; MAX_ECC_LEN];
    if degree == 0 {
        return divisor;
    }

    divisor[degree - 1] = 1;
    let mut root = 1u8;
    let mut i = 0usize;
    while i < degree {
        let mut j = 0usize;
        while j < degree {
            divisor[j] = gf_multiply_const(divisor[j], root);
            if j + 1 < degree {
                divisor[j] ^= divisor[j + 1];
            }
            j += 1;
        }
        root = gf_multiply_const(root, 0x02);
        i += 1;
    }

    divisor
}

const fn build_rs_generators() -> [[u8; MAX_ECC_LEN]; MAX_ECC_LEN + 1] {
    let mut result = [[0u8; MAX_ECC_LEN]; MAX_ECC_LEN + 1];
    let mut degree = 1usize;
    while degree <= MAX_ECC_LEN {
        result[degree] = build_rs_generator(degree);
        degree += 1;
    }
    result
}

const fn build_alphanumeric_values() -> [i8; 128] {
    let charset = *b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";
    let mut result = [-1i8; 128];
    let mut i = 0usize;
    while i < charset.len() {
        result[charset[i] as usize] = i as i8;
        i += 1;
    }
    result
}

struct FinderPenalty {
    qr_size: i32,
    run_history: [i32; 7],
}

impl FinderPenalty {
    #[inline]
    fn new(size: u8) -> Self {
        Self {
            qr_size: i32::from(size),
            run_history: [0; 7],
        }
    }

    fn add_history(&mut self, mut currentrunlength: i32) {
        if self.run_history[0] == 0 {
            currentrunlength += self.qr_size;
        }
        let len = self.run_history.len();
        self.run_history.copy_within(0..len - 1, 1);
        self.run_history[0] = currentrunlength;
    }

    fn count_patterns(&self) -> i32 {
        let rh = &self.run_history;
        let n = rh[1];
        let core = n > 0 && rh[2] == n && rh[3] == n * 3 && rh[4] == n && rh[5] == n;
        i32::from(core && rh[0] >= n * 4 && rh[6] >= n)
            + i32::from(core && rh[6] >= n * 4 && rh[0] >= n)
    }

    fn terminate_and_count(mut self, currentruncolor: bool, mut currentrunlength: i32) -> i32 {
        if currentruncolor {
            self.add_history(currentrunlength);
            currentrunlength = 0;
        }
        currentrunlength += self.qr_size;
        self.add_history(currentrunlength);
        self.count_patterns()
    }
}

const PENALTY_N1: i32 = 3;
const PENALTY_N2: i32 = 3;
const PENALTY_N3: i32 = 40;
const PENALTY_N4: i32 = 10;

const GF_TABLES: ([u8; 512], [u8; 256]) = build_gf_tables();
const GF_EXP: [u8; 512] = GF_TABLES.0;
const GF_LOG: [u8; 256] = GF_TABLES.1;
const RS_GENERATORS: [[u8; MAX_ECC_LEN]; MAX_ECC_LEN + 1] = build_rs_generators();
const ALPHANUMERIC_VALUES: [i8; 128] = build_alphanumeric_values();

static ECC_CODEWORDS_PER_BLOCK: [[i16; 41]; 4] = [
    [
        -1, 7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28,
        30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
    [
        -1, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28,
        28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28,
    ],
    [
        -1, 13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30,
        30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
    [
        -1, 17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24,
        30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
];

static NUM_ERROR_CORRECTION_BLOCKS: [[i16; 41]; 4] = [
    [
        -1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12,
        13, 14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25,
    ],
    [
        -1, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21,
        23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49,
    ],
    [
        -1, 1, 1, 2, 2, 4, 4, 6, 6, 8, 8, 8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27,
        29, 34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59, 62, 65, 68,
    ],
    [
        -1, 1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32,
        35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81,
    ],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_text_with_finder_modules() {
        let input = b"HELLO WORLD";
        let mut out = [0u8; MAX_BUFFER_LEN];
        let size = kyuaru_encode_text_utf8(
            input.as_ptr(),
            input.len() as u32,
            1,
            1,
            40,
            -1,
            1,
            out.as_mut_ptr(),
            out.len() as u32,
        );

        assert_eq!(size, 21);
        assert_eq!(out[0], 21);
        assert!(module_at(&out, 21, 0, 0));
        assert!(module_at(&out, 21, 6, 6));
        assert!(!module_at(&out, 21, 7, 7));
    }

    #[test]
    fn rejects_invalid_text_utf8_but_accepts_binary() {
        let input = [0xFF];
        let mut out = [0u8; MAX_BUFFER_LEN];
        let text = kyuaru_encode_text_utf8(
            input.as_ptr(),
            input.len() as u32,
            1,
            1,
            40,
            -1,
            1,
            out.as_mut_ptr(),
            out.len() as u32,
        );
        let binary = kyuaru_encode_binary(
            input.as_ptr(),
            input.len() as u32,
            1,
            1,
            40,
            -1,
            1,
            out.as_mut_ptr(),
            out.len() as u32,
        );

        assert_eq!(text, ERR_INVALID_UTF8);
        assert_eq!(binary, 21);
    }

    #[test]
    fn rejects_too_small_output_buffer() {
        let input = b"abc";
        let mut out = [0u8; 8];
        let result = kyuaru_encode_text_utf8(
            input.as_ptr(),
            input.len() as u32,
            1,
            1,
            40,
            -1,
            1,
            out.as_mut_ptr(),
            out.len() as u32,
        );

        assert_eq!(result, ERR_OUTPUT_TOO_SHORT);
    }

    fn module_at(out: &[u8], size: usize, x: usize, y: usize) -> bool {
        let index = y * size + x;
        (out[1 + (index >> 3)] >> (index & 7)) & 1 != 0
    }

    fn run_text(input: &[u8], ecl: u8, min_v: u8, max_v: u8, mask: i8, boost: u8) -> i32 {
        let mut out = [0u8; MAX_BUFFER_LEN];
        kyuaru_encode_text_utf8(
            input.as_ptr(),
            input.len() as u32,
            ecl,
            min_v,
            max_v,
            mask,
            boost,
            out.as_mut_ptr(),
            out.len() as u32,
        )
    }

    fn run_binary(input: &[u8], ecl: u8, min_v: u8, max_v: u8, mask: i8, boost: u8) -> i32 {
        let mut out = [0u8; MAX_BUFFER_LEN];
        kyuaru_encode_binary(
            input.as_ptr(),
            input.len() as u32,
            ecl,
            min_v,
            max_v,
            mask,
            boost,
            out.as_mut_ptr(),
            out.len() as u32,
        )
    }

    #[test]
    fn rejects_invalid_ecc_above_3() {
        assert_eq!(run_text(b"hi", 4, 1, 40, -1, 0), ERR_INVALID_ECC);
        assert_eq!(run_text(b"hi", 255, 1, 40, -1, 0), ERR_INVALID_ECC);
    }

    #[test]
    fn rejects_invalid_version() {
        assert_eq!(run_text(b"hi", 1, 0, 40, -1, 0), ERR_INVALID_VERSION);
        assert_eq!(run_text(b"hi", 1, 1, 41, -1, 0), ERR_INVALID_VERSION);
        // min > max
        assert_eq!(run_text(b"hi", 1, 10, 5, -1, 0), ERR_INVALID_VERSION);
    }

    #[test]
    fn rejects_invalid_mask() {
        assert_eq!(run_text(b"hi", 1, 1, 40, -2, 0), ERR_INVALID_MASK);
        assert_eq!(run_text(b"hi", 1, 1, 40, 8, 0), ERR_INVALID_MASK);
    }

    #[test]
    fn rejects_data_too_long_for_version_range() {
        let input = vec![b'x'; 1000];
        // 1000 bytes far exceeds v5 byte capacity.
        assert_eq!(run_text(&input, 0, 1, 5, -1, 0), ERR_DATA_TOO_LONG);
    }

    #[test]
    fn rejects_input_too_long_above_spec_maximum() {
        // Text mode caps at MAX_TEXT_LEN = 7089.
        let input = vec![b'0'; (MAX_TEXT_LEN + 1) as usize];
        assert_eq!(run_text(&input, 1, 1, 40, -1, 0), ERR_INPUT_TOO_LONG);

        // Binary mode caps at MAX_BINARY_LEN = 2953.
        let bin = vec![0u8; (MAX_BINARY_LEN + 1) as usize];
        assert_eq!(run_binary(&bin, 1, 1, 40, -1, 0), ERR_INPUT_TOO_LONG);
    }

    #[test]
    fn rejects_null_data_pointer_when_len_nonzero() {
        let mut out = [0u8; MAX_BUFFER_LEN];
        let result = kyuaru_encode_text_utf8(
            core::ptr::null(),
            5,
            1,
            1,
            40,
            -1,
            0,
            out.as_mut_ptr(),
            out.len() as u32,
        );
        assert_eq!(result, ERR_NULL_POINTER);
    }

    #[test]
    fn rejects_null_output_pointer() {
        let input = b"hi";
        let result = kyuaru_encode_text_utf8(
            input.as_ptr(),
            input.len() as u32,
            1,
            1,
            40,
            -1,
            0,
            core::ptr::null_mut(),
            MAX_BUFFER_LEN as u32,
        );
        assert_eq!(result, ERR_NULL_POINTER);
    }

    #[test]
    fn empty_input_yields_version_1() {
        assert_eq!(run_text(b"", 1, 1, 40, -1, 0), 21);
        assert_eq!(run_binary(b"", 1, 1, 40, -1, 0), 21);
    }

    #[test]
    fn all_four_ecls_encode_short_text() {
        for ecl in 0u8..=3 {
            let size = run_text(b"hi", ecl, 1, 40, -1, 0);
            assert!(size >= 21, "ECL {ecl}: got size {size}");
        }
    }

    #[test]
    fn all_eight_masks_explicit() {
        for mask in 0i8..=7 {
            let size = run_text(b"hi", 1, 1, 40, mask, 0);
            assert!(size >= 21, "mask {mask}: got size {size}");
        }
    }

    #[test]
    fn boost_ecl_never_decreases_capacity() {
        // For a short input that easily fits L, boostEcl=true should produce
        // the same size (version) and ideally a higher ECL.
        let without = run_text(b"hi", 0, 1, 40, -1, 0);
        let with = run_text(b"hi", 0, 1, 40, -1, 1);
        assert_eq!(
            without, with,
            "boost_ecl shouldn't change the picked version"
        );
    }

    #[test]
    fn numeric_segment_packs_tighter_than_byte_mode() {
        // Same length in numeric vs byte-mode should pick a smaller version in numeric.
        let numeric_size = run_text(b"12345678901234567890", 1, 1, 40, -1, 0);
        let bytemode_size = run_text(b"abcdefghijklmnopqrst", 1, 1, 40, -1, 0);
        assert!(
            numeric_size <= bytemode_size,
            "numeric segment should fit in <= the version a byte segment needs"
        );
    }

    #[test]
    fn alphanumeric_segment_used_for_uppercase_ascii() {
        // "HELLO" is alphanumeric; should fit in a small version.
        let size = run_text(b"HELLO", 1, 1, 40, -1, 0);
        assert_eq!(size, 21);
    }

    #[test]
    fn boundary_input_at_text_spec_max_succeeds_at_max_version() {
        let input = vec![b'0'; MAX_TEXT_LEN as usize];
        let size = run_text(&input, 0, 1, 40, -1, 0);
        // Spec says MAX_TEXT_LEN numerics at ECL L fit in version 40 (size 177).
        assert_eq!(size, 177);
    }

    proptest::proptest! {
        // Random binary inputs at random ECL/version-range/mask. The encoder
        // either succeeds with a valid size, or returns a documented error
        // code. Either way it must not crash or return garbage.
        #[test]
        fn encode_binary_returns_valid_size_or_documented_error(
            bytes in proptest::collection::vec(proptest::num::u8::ANY, 0..=200),
            ecl in 0u8..=3,
            min_v in 1u8..=40,
            max_v in 1u8..=40,
            mask in -1i8..=7,
            boost in proptest::bool::ANY,
        ) {
            let (lo, hi) = if min_v <= max_v { (min_v, max_v) } else { (max_v, min_v) };
            let result = run_binary(&bytes, ecl, lo, hi, mask, boost as u8);
            if result >= 0 {
                // Successful encode: size must be a valid QR side length
                // (4v+17 for v ∈ [lo, hi]).
                proptest::prop_assert!(result >= 21 && result <= 177);
                let v = ((result as u8) - 17) / 4;
                proptest::prop_assert!(v >= lo && v <= hi);
            } else {
                // Must be one of the documented error codes.
                proptest::prop_assert!(
                    result == ERR_NULL_POINTER
                        || result == ERR_OUTPUT_TOO_SHORT
                        || result == ERR_INVALID_ECC
                        || result == ERR_INVALID_VERSION
                        || result == ERR_INVALID_MASK
                        || result == ERR_DATA_TOO_LONG
                        || result == ERR_INVALID_UTF8
                        || result == ERR_INPUT_TOO_LONG,
                    "unexpected error code: {result}"
                );
            }
        }

        // Encoding the same input twice yields byte-equal output.
        #[test]
        fn determinism(
            bytes in proptest::collection::vec(proptest::num::u8::ANY, 0..=100),
            ecl in 0u8..=3,
            mask in 0i8..=7,
        ) {
            let mut out1 = [0u8; MAX_BUFFER_LEN];
            let mut out2 = [0u8; MAX_BUFFER_LEN];
            let s1 = kyuaru_encode_binary(
                bytes.as_ptr(), bytes.len() as u32, ecl, 1, 40, mask, 0,
                out1.as_mut_ptr(), out1.len() as u32,
            );
            let s2 = kyuaru_encode_binary(
                bytes.as_ptr(), bytes.len() as u32, ecl, 1, 40, mask, 0,
                out2.as_mut_ptr(), out2.len() as u32,
            );
            proptest::prop_assert_eq!(s1, s2);
            if s1 >= 0 {
                proptest::prop_assert_eq!(out1, out2);
            }
        }
    }
}
