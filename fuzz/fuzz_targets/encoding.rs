#![no_main]
use libfuzzer_sys::fuzz_target;

use core::fmt::Debug;

use kv_3d_storage::*;
use kv_3d_storage_fuzz::*;

fuzz_target!(|data: (u8, u8, u8, u8, u8, u8)| {
    let (x1, y1, z1, x2, y2, z2) = data;

    let x1_fw = U8FixedWidth(x1);
    let y1_fw = U8FixedWidth(y1);
    let z1_fw = U8FixedWidth(z1);
    let x2_fw = U8FixedWidth(x2);
    let y2_fw = U8FixedWidth(y2);
    let z2_fw = U8FixedWidth(z2);

    let x1_vw = U8VariableWidth(x1);
    let y1_vw = U8VariableWidth(y1);
    let z1_vw = U8VariableWidth(z1);
    let x2_vw = U8VariableWidth(x2);
    let y2_vw = U8VariableWidth(y2);
    let z2_vw = U8VariableWidth(z2);

    assert_dimension_works(&x1_fw, &y1_fw);
    assert_dimension_works(&x1_vw, &y1_vw);

    // All fixed-width
    assert_point3d_works(
        &Point3d {
            x: x1_fw,
            y: y1_fw,
            z: z1_fw,
        },
        &Point3d {
            x: x2_fw,
            y: y2_fw,
            z: z2_fw,
        },
    );

    // All variable-width
    assert_point3d_works(
        &Point3d {
            x: x1_vw,
            y: y1_vw,
            z: z1_vw,
        },
        &Point3d {
            x: x2_vw,
            y: y2_vw,
            z: z2_vw,
        },
    );

    // One fixed-width, two variable width.
    assert_point3d_works(
        &Point3d {
            x: x1_fw,
            y: y1_vw,
            z: z1_vw,
        },
        &Point3d {
            x: x2_fw,
            y: y2_vw,
            z: z2_vw,
        },
    );

    // Two fixed-width, one variable width.
    assert_point3d_works(
        &Point3d {
            x: x1_fw,
            y: y1_fw,
            z: z1_vw,
        },
        &Point3d {
            x: x2_fw,
            y: y2_fw,
            z: z2_vw,
        },
    );
});

/// Check that the encodings of two values of a dimension do not violate the contracts of the Dimension trait.
pub fn assert_dimension_works<D: Dimension + Debug>(v1: &D, v2: &D) {
    let mut v1_buf = vec![];
    v1_buf.resize(D::HOMOMORPHIC_ENCODING_MAX_LENGTH, 0);

    let v1_encoding_len = v1.homomorphic_encode(&mut v1_buf);

    if D::IS_FIXED_WIDTH_ENCODING {
        assert_eq!(
            v1_encoding_len,
            D::HOMOMORPHIC_ENCODING_MAX_LENGTH,
            "\n\nDimension claims to produce fixed-width encodings, but got an encoding of length other than the claimed fixed width.
value: {:?}
encoding: {:?}
actual encoding length: {:?}
claimed fixed width (HOMOMORPHIC_ENCODING_MAX_LENGTH): {:?}\n\n", v1, &v1_buf[0..v1_encoding_len], v1_encoding_len, D::HOMOMORPHIC_ENCODING_MAX_LENGTH
        );
    } else {
        assert!(
            v1_encoding_len <= D::HOMOMORPHIC_ENCODING_MAX_LENGTH,
            "\n\nOverlong encoding.
value: {:?}
encoding: {:?}
encoding length: {:?}
claimed maximum length (HOMOMORPHIC_ENCODING_MAX_LENGTH): {:?}\n\n",
            v1,
            &v1_buf[0..v1_encoding_len],
            v1_encoding_len,
            D::HOMOMORPHIC_ENCODING_MAX_LENGTH
        );

        for i in 0..v1_encoding_len {
            if i > 0 && v1_buf[i] == 0 && v1_buf[i - 1] == 0 {
                panic!(
                    "A variable-width encoding must not contain consecutive zero bytes.
value: {:?}
encoding: {:?}
index of first of the consecutive zero bytes: {:?}\n\n",
                    v1,
                    &v1_buf[0..v1_encoding_len],
                    i - 1
                );
            }
        }
    }

    let (v1_decoded, v1_num_decoded_bytes) = D::homomorphic_decode(&v1_buf).unwrap();

    assert_eq!(
        &v1_decoded,
        v1,
        "\n\nDecoding the encoding did not yield the original value.
value: {:?}
encoding: {:?}
decoded: {:?}
number of decoded bytes by the decoding function: {:?}\n\n",
        v1,
        &v1_buf[0..v1_encoding_len],
        v1_decoded,
        v1_num_decoded_bytes
    );

    assert_eq!(
        v1_num_decoded_bytes,
        v1_encoding_len,
        "\n\nDecoding reported a different length than the encoding process.
value: {:?}
encoding: {:?}
encoding length as reported by the encoding function: {:?}
number of decoded bytes by the decoding function: {:?}\n\n",
        v1,
        &v1_buf[0..v1_encoding_len],
        v1_encoding_len,
        v1_num_decoded_bytes
    );

    let mut v2_buf = vec![];
    v2_buf.resize(D::HOMOMORPHIC_ENCODING_MAX_LENGTH, 0);

    let v2_encoding_len = v2.homomorphic_encode(&mut v2_buf);

    // Test that the encoding is homomorphic.
    assert_eq!(
        v1.cmp(&v2),
        v1_buf[0..v1_encoding_len].cmp(&v2_buf[0..v2_encoding_len]),
        "\n\nEncoding is not homomorphic:
v1: {:?}
v2: {:?}
v1.cmp(v2): {:?}
encoding of v1: {:?}
encoding of v2: {:?}
v1_enc.cmp(v2.enc): {:?}\n\n",
        v1,
        v2,
        v1.cmp(&v2),
        &v1_buf[0..v1_encoding_len],
        &v2_buf[0..v2_encoding_len],
        v1_buf[0..v1_encoding_len].cmp(&v2_buf[0..v2_encoding_len])
    );
}

// Check that the encodings of two 3d points work and are homomorphic.
pub fn assert_point3d_works<X: Dimension + Debug, Y: Dimension + Debug, Z: Dimension + Debug>(
    v1: &Point3d<X, Y, Z>,
    v2: &Point3d<X, Y, Z>,
) {
    /*
     * Test xyz ordering.
     */
    let mut v1_xyz_buf = vec![];
    v1_xyz_buf.resize(Point3d::<X, Y, Z>::max_encoding_len_xyz(), 0);

    let v1_xyz_encoding_len = v1.encode_xyz(&mut v1_xyz_buf);

    if X::IS_FIXED_WIDTH_ENCODING && Y::IS_FIXED_WIDTH_ENCODING && Z::IS_FIXED_WIDTH_ENCODING {
        assert_eq!(
                v1_xyz_encoding_len,
                Point3d::<X, Y, Z>::max_encoding_len_xyz(),
                "\n\nPoint3d should produce fixed-width encodings, but got an encoding of length other than the claimed length.
value: {:?}
encoding: {:?}
actual encoding length: {:?}
claimed fixed width (HOMOMORPHIC_ENCODING_MAX_LENGTH): {:?}\n\n", v1, &v1_xyz_buf[0..v1_xyz_encoding_len], v1_xyz_encoding_len, Point3d::<X, Y, Z>::max_encoding_len_xyz()
            );
    } else {
        assert!(
            v1_xyz_encoding_len <= Point3d::<X, Y, Z>::max_encoding_len_xyz(),
            "\n\nOverlong encoding.
value: {:?}
encoding: {:?}
encoding length: {:?}
claimed maximum length (HOMOMORPHIC_ENCODING_MAX_LENGTH): {:?}\n\n",
            v1,
            &v1_xyz_buf[0..v1_xyz_encoding_len],
            v1_xyz_encoding_len,
            Point3d::<X, Y, Z>::max_encoding_len_xyz()
        );
    }

    let (v1_xyz_decoded, v1_xyz_num_decoded_bytes) =
        Point3d::<X, Y, Z>::decode_xyz(&v1_xyz_buf).unwrap();

    assert_eq!(
        &v1_xyz_decoded,
        v1,
        "\n\nDecoding the encoding did not yield the original point.
value: {:?}
encoding: {:?}
decoded: {:?}
number of decoded bytes by the decoding function: {:?}\n\n",
        v1,
        &v1_xyz_buf[0..v1_xyz_encoding_len],
        v1_xyz_decoded,
        v1_xyz_num_decoded_bytes
    );

    assert_eq!(
        v1_xyz_num_decoded_bytes,
        v1_xyz_encoding_len,
        "\n\nDecoding reported a different length than the encoding process.
value: {:?}
encoding: {:?}
encoding length as reported by the encoding function: {:?}
number of decoded bytes by the decoding function: {:?}\n\n",
        v1,
        &v1_xyz_buf[0..v1_xyz_encoding_len],
        v1_xyz_encoding_len,
        v1_xyz_num_decoded_bytes
    );

    let mut v2_xyz_buf = vec![];
    v2_xyz_buf.resize(Point3d::<X, Y, Z>::max_encoding_len_xyz(), 0);

    let v2_xyz_encoding_len = v2.encode_xyz(&mut v2_xyz_buf);

    // Test that the encoding is homomorphic.
    assert_eq!(
        v1.cmp_xyz(&v2),
        v1_xyz_buf[0..v1_xyz_encoding_len].cmp(&v2_xyz_buf[0..v2_xyz_encoding_len]),
        "\n\nEncoding is not homomorphic:
v1: {:?}
v2: {:?}
v1.cmp_xyz(v2): {:?}
encoding of v1: {:?}
encoding of v2: {:?}
v1_xyz_enc.cmp(v2.enc): {:?}\n\n",
        v1,
        v2,
        v1.cmp_xyz(&v2),
        &v1_xyz_buf[0..v1_xyz_encoding_len],
        &v2_xyz_buf[0..v2_xyz_encoding_len],
        v1_xyz_buf[0..v1_xyz_encoding_len].cmp(&v2_xyz_buf[0..v2_xyz_encoding_len])
    );

    /*
     * Test yzx ordering.
     */
    let mut v1_yzx_buf = vec![];
    v1_yzx_buf.resize(Point3d::<X, Y, Z>::max_encoding_len_yzx(), 0);

    let v1_yzx_encoding_len = v1.encode_yzx(&mut v1_yzx_buf);

    if X::IS_FIXED_WIDTH_ENCODING && Y::IS_FIXED_WIDTH_ENCODING && Z::IS_FIXED_WIDTH_ENCODING {
        assert_eq!(
                v1_yzx_encoding_len,
                Point3d::<X, Y, Z>::max_encoding_len_yzx(),
                "\n\nPoint3d should produce fixed-width encodings, but got an encoding of length other than the claimed length.
value: {:?}
encoding: {:?}
actual encoding length: {:?}
claimed fixed width (HOMOMORPHIC_ENCODING_MAX_LENGTH): {:?}\n\n", v1, &v1_yzx_buf[0..v1_yzx_encoding_len], v1_yzx_encoding_len, Point3d::<X, Y, Z>::max_encoding_len_yzx()
            );
    } else {
        assert!(
            v1_yzx_encoding_len <= Point3d::<X, Y, Z>::max_encoding_len_yzx(),
            "\n\nOverlong encoding.
value: {:?}
encoding: {:?}
encoding length: {:?}
claimed maximum length (HOMOMORPHIC_ENCODING_MAX_LENGTH): {:?}\n\n",
            v1,
            &v1_yzx_buf[0..v1_yzx_encoding_len],
            v1_yzx_encoding_len,
            Point3d::<X, Y, Z>::max_encoding_len_yzx()
        );
    }

    let (v1_yzx_decoded, v1_yzx_num_decoded_bytes) =
        Point3d::<X, Y, Z>::decode_yzx(&v1_yzx_buf).unwrap();

    assert_eq!(
        &v1_yzx_decoded,
        v1,
        "\n\nDecoding the encoding did not yield the original point.
value: {:?}
encoding: {:?}
decoded: {:?}
number of decoded bytes by the decoding function: {:?}\n\n",
        v1,
        &v1_yzx_buf[0..v1_yzx_encoding_len],
        v1_yzx_decoded,
        v1_yzx_num_decoded_bytes
    );

    assert_eq!(
        v1_yzx_num_decoded_bytes,
        v1_yzx_encoding_len,
        "\n\nDecoding reported a different length than the encoding process.
value: {:?}
encoding: {:?}
encoding length as reported by the encoding function: {:?}
number of decoded bytes by the decoding function: {:?}\n\n",
        v1,
        &v1_yzx_buf[0..v1_yzx_encoding_len],
        v1_yzx_encoding_len,
        v1_yzx_num_decoded_bytes
    );

    let mut v2_yzx_buf = vec![];
    v2_yzx_buf.resize(Point3d::<X, Y, Z>::max_encoding_len_yzx(), 0);

    let v2_yzx_encoding_len = v2.encode_yzx(&mut v2_yzx_buf);

    // Test that the encoding is homomorphic.
    assert_eq!(
        v1.cmp_yzx(&v2),
        v1_yzx_buf[0..v1_yzx_encoding_len].cmp(&v2_yzx_buf[0..v2_yzx_encoding_len]),
        "\n\nEncoding is not homomorphic:
v1: {:?}
v2: {:?}
v1.cmp_yzx(v2): {:?}
encoding of v1: {:?}
encoding of v2: {:?}
v1_yzx_enc.cmp(v2.enc): {:?}\n\n",
        v1,
        v2,
        v1.cmp_yzx(&v2),
        &v1_yzx_buf[0..v1_yzx_encoding_len],
        &v2_yzx_buf[0..v2_yzx_encoding_len],
        v1_yzx_buf[0..v1_yzx_encoding_len].cmp(&v2_yzx_buf[0..v2_yzx_encoding_len])
    );

    /*
     * Test zxy ordering.
     */
    let mut v1_zxy_buf = vec![];
    v1_zxy_buf.resize(Point3d::<X, Y, Z>::max_encoding_len_zxy(), 0);

    let v1_zxy_encoding_len = v1.encode_zxy(&mut v1_zxy_buf);

    if X::IS_FIXED_WIDTH_ENCODING && Y::IS_FIXED_WIDTH_ENCODING && Z::IS_FIXED_WIDTH_ENCODING {
        assert_eq!(
                v1_zxy_encoding_len,
                Point3d::<X, Y, Z>::max_encoding_len_zxy(),
                "\n\nPoint3d should produce fixed-width encodings, but got an encoding of length other than the claimed length.
value: {:?}
encoding: {:?}
actual encoding length: {:?}
claimed fixed width (HOMOMORPHIC_ENCODING_MAX_LENGTH): {:?}\n\n", v1, &v1_zxy_buf[0..v1_zxy_encoding_len], v1_zxy_encoding_len, Point3d::<X, Y, Z>::max_encoding_len_zxy()
            );
    } else {
        assert!(
            v1_zxy_encoding_len <= Point3d::<X, Y, Z>::max_encoding_len_zxy(),
            "\n\nOverlong encoding.
value: {:?}
encoding: {:?}
encoding length: {:?}
claimed maximum length (HOMOMORPHIC_ENCODING_MAX_LENGTH): {:?}\n\n",
            v1,
            &v1_zxy_buf[0..v1_zxy_encoding_len],
            v1_zxy_encoding_len,
            Point3d::<X, Y, Z>::max_encoding_len_zxy()
        );
    }

    let (v1_zxy_decoded, v1_zxy_num_decoded_bytes) =
        Point3d::<X, Y, Z>::decode_zxy(&v1_zxy_buf).unwrap();

    assert_eq!(
        &v1_zxy_decoded,
        v1,
        "\n\nDecoding the encoding did not yield the original point.
value: {:?}
encoding: {:?}
decoded: {:?}
number of decoded bytes by the decoding function: {:?}\n\n",
        v1,
        &v1_zxy_buf[0..v1_zxy_encoding_len],
        v1_zxy_decoded,
        v1_zxy_num_decoded_bytes
    );

    assert_eq!(
        v1_zxy_num_decoded_bytes,
        v1_zxy_encoding_len,
        "\n\nDecoding reported a different length than the encoding process.
value: {:?}
encoding: {:?}
encoding length as reported by the encoding function: {:?}
number of decoded bytes by the decoding function: {:?}\n\n",
        v1,
        &v1_zxy_buf[0..v1_zxy_encoding_len],
        v1_zxy_encoding_len,
        v1_zxy_num_decoded_bytes
    );

    let mut v2_zxy_buf = vec![];
    v2_zxy_buf.resize(Point3d::<X, Y, Z>::max_encoding_len_zxy(), 0);

    let v2_zxy_encoding_len = v2.encode_zxy(&mut v2_zxy_buf);

    // Test that the encoding is homomorphic.
    assert_eq!(
        v1.cmp_zxy(&v2),
        v1_zxy_buf[0..v1_zxy_encoding_len].cmp(&v2_zxy_buf[0..v2_zxy_encoding_len]),
        "\n\nEncoding is not homomorphic:
v1: {:?}
v2: {:?}
v1.cmp_zxy(v2): {:?}
encoding of v1: {:?}
encoding of v2: {:?}
v1_zxy_enc.cmp(v2.enc): {:?}\n\n",
        v1,
        v2,
        v1.cmp_zxy(&v2),
        &v1_zxy_buf[0..v1_zxy_encoding_len],
        &v2_zxy_buf[0..v2_zxy_encoding_len],
        v1_zxy_buf[0..v1_zxy_encoding_len].cmp(&v2_zxy_buf[0..v2_zxy_encoding_len])
    );    
}
