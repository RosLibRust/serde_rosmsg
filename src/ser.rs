//! Serialize a Rust data structure into ROSMSG binary data.
//!
//! Data types supported by ROSMSG are supported as well. This results in the
//! lack of support for:
//!
//! * Enums of any type, including `Option`
//! * `char`, so use one character `String`s instead
//! * Maps that can't be boiled down to `<String, String>`

use super::error::{Error, ErrorKind, Result};
use byteorder::{LittleEndian, WriteBytesExt};
use error_chain::bail;
use serde::ser::{self, Impossible};
use std::io;

/// A structure for serializing Rust values into ROSMSG binary data.
///
/// The structure does not write the object size prefix.
/// It's the user's responsibility to write the object size themselves.
///
/// Prefer using `to_writer` and `to_vec`.
pub struct Serializer<W> {
    writer: W,
}

impl<W> Serializer<W>
where
    W: io::Write,
{
    /// Creates a new ROSMSG serializer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate roslibrust_serde_rosmsg;
    /// # use roslibrust_serde_rosmsg::ser::Serializer;
    /// # extern crate serde;
    /// # fn main() {
    /// use serde::ser::Serialize;
    ///
    /// let mut cursor = std::io::Cursor::new(Vec::new());
    /// String::from("Hello, World!").serialize(
    ///         &mut Serializer::new(&mut cursor)).unwrap();
    ///
    /// assert_eq!(cursor.into_inner(), b"\x0d\0\0\0Hello, World!");
    /// # }
    /// ```
    pub fn new(writer: W) -> Self {
        Serializer { writer: writer }
    }

    /// Unwrap the `Writer` from the `Serializer`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate roslibrust_serde_rosmsg;
    /// # use roslibrust_serde_rosmsg::ser::Serializer;
    /// # extern crate serde;
    /// # fn main() {
    /// use serde::ser::Serialize;
    ///
    /// let mut ser = Serializer::new(std::io::Cursor::new(Vec::new()));
    /// String::from("Hello, World!").serialize(&mut ser).unwrap();
    ///
    /// // Get the cursor that was passed to the serializer
    /// let cursor: std::io::Cursor<Vec<u8>> = ser.into_inner();
    ///
    /// // Get the data from the cursor. This method is unrelated to
    /// // Serializer's into_inner(), and is part of Cursor
    /// let data = cursor.into_inner();
    ///
    /// assert_eq!(data, b"\x0d\0\0\0Hello, World!");
    /// # }
    /// ```
    pub fn into_inner(self) -> W {
        self.writer
    }

    #[inline]
    fn write_size(&mut self, len: usize) -> io::Result<()> {
        self.writer.write_u32::<LittleEndian>(len as u32)
    }
}

type SerializerResult = Result<()>;

macro_rules! impl_nums {
    ($ty:ty, $ser_method:ident, $writer_method:ident) => {
        #[inline]
        fn $ser_method(self, v: $ty) -> SerializerResult {
            self.writer
                .$writer_method::<LittleEndian>(v)
                .map_err(|v| v.into())
        }
    };
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a, W>;
    type SerializeTuple = Compound<'a, W>;
    type SerializeTupleStruct = Compound<'a, W>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = CompoundMap<'a, W>;
    type SerializeStruct = Compound<'a, W>;
    type SerializeStructVariant = Impossible<(), Error>;

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }

    #[inline]
    fn serialize_bool(self, v: bool) -> SerializerResult {
        self.writer
            .write_u8(if v { 1 } else { 0 })
            .map_err(|v| v.into())
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> SerializerResult {
        self.writer.write_i8(v).map_err(|v| v.into())
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> SerializerResult {
        self.writer.write_u8(v).map_err(|v| v.into())
    }

    impl_nums!(u16, serialize_u16, write_u16);
    impl_nums!(u32, serialize_u32, write_u32);
    impl_nums!(u64, serialize_u64, write_u64);
    impl_nums!(i16, serialize_i16, write_i16);
    impl_nums!(i32, serialize_i32, write_i32);
    impl_nums!(i64, serialize_i64, write_i64);
    impl_nums!(f32, serialize_f32, write_f32);
    impl_nums!(f64, serialize_f64, write_f64);

    #[inline]
    fn serialize_char(self, _v: char) -> SerializerResult {
        bail!(ErrorKind::UnsupportedCharType)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> SerializerResult {
        self.serialize_bytes(value.as_bytes())
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> SerializerResult {
        self.write_size(value.len())
            .and_then(|_| self.writer.write_all(value))
            .map_err(|v| v.into())
    }

    #[inline]
    fn serialize_none(self) -> SerializerResult {
        bail!(ErrorKind::UnsupportedEnumType)
    }

    #[inline]
    fn serialize_some<T: ?Sized + ser::Serialize>(self, _value: &T) -> SerializerResult {
        bail!(ErrorKind::UnsupportedEnumType)
    }

    #[inline]
    fn serialize_unit(self) -> SerializerResult {
        self.serialize_tuple(0).and(Ok(()))
    }

    #[inline]
    fn serialize_unit_struct(self, name: &'static str) -> SerializerResult {
        self.serialize_tuple_struct(name, 0).and(Ok(()))
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> SerializerResult {
        bail!(ErrorKind::UnsupportedEnumType)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> SerializerResult {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> SerializerResult {
        bail!(ErrorKind::UnsupportedEnumType)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        match len {
            Some(len) => self.serialize_u32(len as u32)?,
            None => bail!(ErrorKind::VariableArraySizeAnnotation),
        };
        Ok(Compound::new(self))
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(Compound::new(self))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        bail!(ErrorKind::UnsupportedEnumType)
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(CompoundMap::new(self))
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_tuple(len)
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        bail!(ErrorKind::UnsupportedEnumType)
    }
}

#[doc(hidden)]
pub struct Compound<'a, W: 'a> {
    ser: &'a mut Serializer<W>,
}

impl<'a, W> Compound<'a, W> {
    #[inline]
    fn new(ser: &'a mut Serializer<W>) -> Compound<'a, W> {
        Compound { ser: ser }
    }
}

impl<'a, W> ser::SerializeSeq for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTuple for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleStruct for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStruct for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

#[doc(hidden)]
pub struct CompoundMap<'a, W: 'a> {
    ser: &'a mut Serializer<W>,
    item: Vec<u8>,
}

impl<'a, W> CompoundMap<'a, W> {
    #[inline]
    fn new(ser: &'a mut Serializer<W>) -> CompoundMap<'a, W> {
        CompoundMap {
            ser: ser,
            item: Vec::new(),
        }
    }
}

impl<'a, W> ser::SerializeMap for CompoundMap<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        self.item = Vec::<u8>::new();
        let mut buffer = Vec::<u8>::new();
        key.serialize(&mut Serializer::new(&mut buffer))?;
        self.item.extend(buffer.into_iter().skip(4));
        self.item.push(b'=');
        Ok(())
    }

    #[inline]
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        use serde::Serializer as SerializerTrait;
        let mut buffer = Vec::<u8>::new();
        value.serialize(&mut Serializer::new(&mut buffer))?;
        self.item.extend(buffer.into_iter().skip(4));
        self.ser.serialize_bytes(&self.item)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::Error for Error {
    #[inline]
    fn custom<T: ::std::fmt::Display>(msg: T) -> Self {
        format!("{}", msg).into()
    }
}

/// Serialize the given data structure `T` as ROSMSG into a seekable IO stream.
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail. It can also fail if the structure contains unsupported elements.
///
/// Finally, it can also fail due to writer or seek failure.
///
/// This function writes a placeholder length, serializes the data directly to the writer,
/// then seeks back to write the correct length. This avoids intermediate buffer allocation.
///
/// # Examples
///
/// ```rust
/// # use roslibrust_serde_rosmsg::ser::to_writer;
/// # use std;
/// let mut cursor = std::io::Cursor::new(Vec::new());
/// to_writer(&mut cursor, &String::from("Hello, World!")).unwrap();
/// assert_eq!(cursor.into_inner(), b"\x11\0\0\0\x0d\0\0\0Hello, World!");
/// ```
pub fn to_writer<W, T>(writer: &mut W, value: &T) -> Result<()>
where
    W: io::Write + io::Seek,
    T: ser::Serialize,
{
    // Write placeholder for length (will be overwritten)
    let start_pos = writer.stream_position()?;
    writer.write_u32::<LittleEndian>(0)?;

    // Serialize the value
    let data_start = writer.stream_position()?;
    let mut serializer = Serializer::new(writer);
    value.serialize(&mut serializer)?;
    let writer = serializer.into_inner();
    let end_pos = writer.stream_position()?;

    // Calculate the data length
    let length = (end_pos - data_start) as u32;

    // Seek back and write the actual length
    writer.seek(io::SeekFrom::Start(start_pos))?;
    writer.write_u32::<LittleEndian>(length)?;

    // Seek back to end
    writer.seek(io::SeekFrom::Start(end_pos))?;

    Ok(())
}

/// Variant of [to_writer] where the 4 bytes for the overall message length are skipped
/// and the overall length is determined by some other means.
pub fn to_writer_skip_length<W, T>(writer: &mut W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ser::Serialize,
{
    value.serialize(&mut Serializer::new(writer))
}

/// Serialize the given data structure `T` as a ROSMSG byte vector.
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail. It can also fail if the structure contains unsupported elements.
///
/// # Examples
///
/// ```rust
/// # use roslibrust_serde_rosmsg::ser::to_vec;
/// let data = to_vec(&String::from("Hello, World!")).unwrap();
/// assert_eq!(data, b"\x11\0\0\0\x0d\0\0\0Hello, World!");
/// ```
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize,
{
    // Note we don't know the length until we've serialized
    let mut buffer = Vec::with_capacity(128);
    let mut cursor = io::Cursor::new(&mut buffer);
    // Skip the first 4 bytes so we have somewhere to write length
    cursor.set_position(4);
    // Serialize the value to the back of the Vec
    value.serialize(&mut Serializer::new(&mut cursor))?;
    // Note: we need to subtract 4 from the length due to the skipped 4 bytes
    // But we also want and empty message to not have a negative length
    // Therefore we use saturating_sub
    let length = (cursor.get_ref().len().saturating_sub(4)) as u32;
    // Reset cursor to the start of the Vec and write the length there
    cursor.set_position(0);
    cursor.write_u32::<LittleEndian>(length)?;
    Ok(buffer)
}

/// Variant of [to_vec] where the 4 bytes for the overall message length are skipped
/// and the overall length is determined by some other means.
pub fn to_vec_skip_length<T>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize,
{
    let mut writer = Vec::with_capacity(128);
    to_writer_skip_length(&mut writer, value)?;
    Ok(writer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use std::collections::HashMap;

    #[test]
    fn writes_u8() {
        assert_eq!(vec![1, 0, 0, 0, 150], to_vec(&150u8).unwrap());
    }

    #[test]
    fn writes_u16() {
        assert_eq!(vec![2, 0, 0, 0, 0x34, 0xA2], to_vec(&0xA234u16).unwrap());
    }

    #[test]
    fn writes_u32() {
        assert_eq!(
            vec![4, 0, 0, 0, 0x45, 0x23, 1, 0xCD],
            to_vec(&0xCD012345u32).unwrap()
        );
    }

    #[test]
    fn writes_u64() {
        assert_eq!(
            vec![8, 0, 0, 0, 0xBB, 0xAA, 0x10, 0x32, 0x54, 0x76, 0x98, 0xAB],
            to_vec(&0xAB9876543210AABBu64).unwrap()
        );
    }

    #[test]
    fn writes_i8() {
        assert_eq!(vec![1, 0, 0, 0, 156], to_vec(&-100i8).unwrap());
    }

    #[test]
    fn writes_i16() {
        assert_eq!(vec![2, 0, 0, 0, 0xD0, 0x8A], to_vec(&-30000i16).unwrap());
    }

    #[test]
    fn writes_i32() {
        assert_eq!(
            vec![4, 0, 0, 0, 0x00, 0x6C, 0xCA, 0x88],
            to_vec(&-2000000000i32).unwrap()
        );
    }

    #[test]
    fn writes_i64() {
        assert_eq!(
            vec![8, 0, 0, 0, 0x00, 0x00, 0x7c, 0x1d, 0xaf, 0x93, 0x19, 0x83],
            to_vec(&-9000000000000000000i64).unwrap()
        );
    }

    #[test]
    fn writes_f32() {
        assert_eq!(
            vec![4, 0, 0, 0, 0x00, 0x70, 0x7b, 0x44],
            to_vec(&1005.75f32).unwrap()
        );
    }

    #[test]
    fn writes_f64() {
        assert_eq!(
            vec![8, 0, 0, 0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6e, 0x8f, 0x40],
            to_vec(&1005.75f64).unwrap()
        );
    }

    #[test]
    fn writes_bool() {
        assert_eq!(vec![1, 0, 0, 0, 1], to_vec(&true).unwrap());
        assert_eq!(vec![1, 0, 0, 0, 0], to_vec(&false).unwrap());
    }

    #[test]
    fn writes_string() {
        assert_eq!(vec![4, 0, 0, 0, 0, 0, 0, 0], to_vec(&"").unwrap());
        assert_eq!(
            vec![
                17, 0, 0, 0, 13, 0, 0, 0, 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100,
                33
            ],
            to_vec(&"Hello, World!").unwrap()
        );
    }

    #[test]
    fn writes_array() {
        assert_eq!(
            vec![8, 0, 0, 0, 7, 0, 1, 4, 33, 0, 57, 0],
            to_vec(&[7i16, 1025, 33, 57]).unwrap()
        );
    }

    #[test]
    fn writes_vector() {
        assert_eq!(
            vec![12, 0, 0, 0, 4, 0, 0, 0, 7, 0, 1, 4, 33, 0, 57, 0],
            to_vec(&vec![7i16, 1025, 33, 57]).unwrap()
        );
    }

    #[test]
    fn writes_tuple() {
        assert_eq!(
            vec![
                22, 0, 0, 0, 2, 8, 1, 7, 6, 0, 0, 0, 65, 66, 67, 48, 49, 50, 4, 0, 0, 0, 1, 0, 0, 1
            ],
            to_vec(&(2050i16, true, 7u8, "ABC012", vec![true, false, false, true])).unwrap()
        );
    }

    #[derive(Serialize)]
    struct TestStructOne {
        a: i16,
        b: bool,
        c: u8,
        d: String,
        e: Vec<bool>,
    }

    #[test]
    fn writes_simple_struct() {
        let v = TestStructOne {
            a: 2050i16,
            b: true,
            c: 7u8,
            d: String::from("ABC012"),
            e: vec![true, false, false, true],
        };
        assert_eq!(
            vec![
                22, 0, 0, 0, 2, 8, 1, 7, 6, 0, 0, 0, 65, 66, 67, 48, 49, 50, 4, 0, 0, 0, 1, 0, 0, 1
            ],
            to_vec(&v).unwrap()
        );
    }

    #[test]
    fn writes_simple_struct_no_length() {
        let v = TestStructOne {
            a: 2050i16,
            b: true,
            c: 7u8,
            d: String::from("ABC012"),
            e: vec![true, false, false, true],
        };
        assert_eq!(
            vec![
                //22, 0, 0, 0,
                2, 8, 1, 7, 6, 0, 0, 0, 65, 66, 67, 48, 49, 50, 4, 0, 0, 0, 1, 0, 0, 1
            ],
            to_vec_skip_length(&v).unwrap()
        );
    }

    #[derive(Serialize)]
    struct TestStructPart {
        a: String,
        b: bool,
    }

    #[derive(Serialize)]
    struct TestStructBig {
        a: Vec<TestStructPart>,
        b: String,
    }

    #[test]
    fn writes_complex_struct() {
        let mut parts = Vec::new();
        parts.push(TestStructPart {
            a: String::from("ABC"),
            b: true,
        });
        parts.push(TestStructPart {
            a: String::from("1!!!!"),
            b: true,
        });
        parts.push(TestStructPart {
            a: String::from("234b"),
            b: false,
        });
        let v = TestStructBig {
            a: parts,
            b: String::from("EEe"),
        };
        assert_eq!(
            vec![
                38, 0, 0, 0, 3, 0, 0, 0, 3, 0, 0, 0, 65, 66, 67, 1, 5, 0, 0, 0, 49, 33, 33, 33, 33,
                1, 4, 0, 0, 0, 50, 51, 52, 98, 0, 3, 0, 0, 0, 69, 69, 101
            ],
            to_vec(&v).unwrap()
        );
    }

    #[test]
    fn writes_empty_string_string_map() {
        let data = HashMap::<String, String>::new();
        assert_eq!(vec![0, 0, 0, 0], to_vec(&data).unwrap());
    }

    #[test]
    fn writes_single_item_string_string_map() {
        let mut data = HashMap::<String, String>::new();
        data.insert(String::from("abc"), String::from("123"));
        assert_eq!(
            vec![11, 0, 0, 0, 7, 0, 0, 0, 97, 98, 99, 61, 49, 50, 51],
            to_vec(&data).unwrap()
        );
    }

    #[test]
    fn writes_multiple_item_string_string_map() {
        let mut data = HashMap::<String, String>::new();
        data.insert(String::from("abc"), String::from("123"));
        data.insert(String::from("AAA"), String::from("B0"));
        let answer = to_vec(&data).unwrap();
        assert!(
            vec![
                21, 0, 0, 0, 7, 0, 0, 0, 97, 98, 99, 61, 49, 50, 51, 6, 0, 0, 0, 65, 65, 65, 61,
                66, 48
            ] == answer
                || vec![
                    21, 0, 0, 0, 6, 0, 0, 0, 65, 65, 65, 61, 66, 48, 7, 0, 0, 0, 97, 98, 99, 61,
                    49, 50, 51
                ] == answer
        );
    }

    #[test]
    fn to_writer_basic() {
        let mut cursor = io::Cursor::new(Vec::new());
        to_writer(&mut cursor, &String::from("Hello, World!")).unwrap();
        assert_eq!(cursor.into_inner(), b"\x11\0\0\0\x0d\0\0\0Hello, World!");
    }

    #[test]
    fn to_writer_same_as_to_vec() {
        let test_value = vec![1u32, 2u32, 3u32, 4u32, 5u32];

        let expected = to_vec(&test_value).unwrap();

        let mut cursor = io::Cursor::new(Vec::new());
        to_writer(&mut cursor, &test_value).unwrap();

        assert_eq!(cursor.into_inner(), expected);
    }

    #[test]
    fn to_writer_multiple_times() {
        let mut cursor = io::Cursor::new(Vec::new());

        // First write
        to_writer(&mut cursor, &String::from("Hello")).unwrap();

        // Second write appends
        to_writer(&mut cursor, &String::from("World")).unwrap();

        let result = cursor.into_inner();

        // Should contain both messages
        assert_eq!(&result[0..13], b"\x09\0\0\0\x05\0\0\0Hello");
        assert_eq!(&result[13..26], b"\x09\0\0\0\x05\0\0\0World");
    }
}
