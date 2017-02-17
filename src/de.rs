//! Deserialize ROSMSG binary data to a Rust data structure.

use byteorder::{LittleEndian, ReadBytesExt};
use serde::de;
use super::error::{Error, ErrorKind, Result, ResultExt};
use std::io;

/// A structure for deserializing ROSMSG into Rust values
pub struct Deserializer<R> {
    reader: R,
}

impl<R> Deserializer<R>
    where R: io::Read
{
    /// Create a new ROSMSG deserializer.
    #[inline]
    pub fn new(reader: R) -> Self {
        Deserializer { reader: reader }
    }

    /// Unwrap the `Reader` from the `Deserializer`.
    #[inline]
    pub fn into_inner(self) -> R {
        self.reader
    }

    #[inline]
    fn pop_length(&mut self) -> io::Result<u32> {
        self.reader.read_u32::<LittleEndian>()
    }

    #[inline]
    fn get_string(&mut self) -> Result<(u32, String)> {
        let length = self.pop_length().chain_err(|| ErrorKind::EndOfBuffer)?;
        let mut buffer = vec![0; length as usize];
        self.reader.read_exact(&mut buffer).chain_err(|| ErrorKind::EndOfBuffer)?;
        String::from_utf8(buffer).chain_err(|| ErrorKind::BadStringData).map(|v| (length + 4, v))
    }
}

macro_rules! impl_nums {
    ($ty:ty, $dser_method:ident, $visitor_method:ident, $reader_method:ident) => {
        #[inline]
        fn $dser_method<V>(self, visitor: V) -> Result<V::Value>
            where V: de::Visitor,
        {
            let value = self.reader.$reader_method::<LittleEndian>()
                .chain_err(|| ErrorKind::EndOfBuffer)?;
            visitor.$visitor_method(value)
        }
    }
}

impl<'a, R: io::Read> de::Deserializer for &'a mut Deserializer<R> {
    type Error = Error;

    #[inline]
    fn deserialize<V>(self, _visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        bail!(ErrorKind::UnsupportedDeserializerMethod("deserialize".into()))
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        let value = self.reader.read_u8().chain_err(|| ErrorKind::EndOfBuffer).map(|v| v != 0)?;
        visitor.visit_bool(value)
    }

    #[inline]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        let value = self.reader.read_u8().chain_err(|| ErrorKind::EndOfBuffer)?;
        visitor.visit_u8(value)
    }

    #[inline]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        let value = self.reader.read_i8().chain_err(|| ErrorKind::EndOfBuffer)?;
        visitor.visit_i8(value)
    }

    impl_nums!(u16, deserialize_u16, visit_u16, read_u16);
    impl_nums!(u32, deserialize_u32, visit_u32, read_u32);
    impl_nums!(u64, deserialize_u64, visit_u64, read_u64);
    impl_nums!(i16, deserialize_i16, visit_i16, read_i16);
    impl_nums!(i32, deserialize_i32, visit_i32, read_i32);
    impl_nums!(i64, deserialize_i64, visit_i64, read_i64);
    impl_nums!(f32, deserialize_f32, visit_f32, read_f32);
    impl_nums!(f64, deserialize_f64, visit_f64, read_f64);

    #[inline]
    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        bail!(ErrorKind::UnsupportedCharType)
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_str(&self.get_string()?.1)
    }

    #[inline]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_string(self.get_string()?.1)
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        bail!(ErrorKind::UnsupportedEnumType)
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        let _size = self.pop_length().chain_err(|| ErrorKind::EndOfBuffer)?;
        let len = self.pop_length().chain_err(|| ErrorKind::EndOfBuffer)? as usize;

        visitor.visit_seq(SeqVisitor {
            deserializer: self,
            len: len,
        })
    }

    #[inline]
    fn deserialize_seq_fixed_size<V>(self, len: usize, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        let _size = self.pop_length().chain_err(|| ErrorKind::EndOfBuffer)?;

        visitor.visit_seq(SeqVisitor {
            deserializer: self,
            len: len,
        })
    }

    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        let _size = self.pop_length().chain_err(|| ErrorKind::EndOfBuffer)?;
        visitor.visit_seq(TupleVisitor(self))
    }

    #[inline]
    fn deserialize_tuple_struct<V>(self,
                                   _name: &'static str,
                                   len: usize,
                                   visitor: V)
                                   -> Result<V::Value>
        where V: de::Visitor
    {
        self.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        let size = self.pop_length().chain_err(|| ErrorKind::EndOfBuffer)?;

        visitor.visit_map(MapVisitor {
            deserializer: self,
            size: size,
            key: Vec::new(),
            value: Vec::new(),
        })
    }

    #[inline]
    fn deserialize_struct<V>(self,
                             _name: &'static str,
                             fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value>
        where V: de::Visitor
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    #[inline]
    fn deserialize_struct_field<V>(self, _visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        bail!(ErrorKind::UnsupportedDeserializerMethod("deserialize_struct_field".into()))
    }

    #[inline]
    fn deserialize_enum<V>(self,
                           _name: &'static str,
                           _variants: &'static [&'static str],
                           _visitor: V)
                           -> Result<V::Value>
        where V: de::Visitor
    {
        bail!(ErrorKind::UnsupportedEnumType)
    }

    #[inline]
    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        bail!(ErrorKind::UnsupportedDeserializerMethod("deserialize_ignored_any".into()))
    }
}

struct SeqVisitor<'a, R: io::Read + 'a> {
    deserializer: &'a mut Deserializer<R>,
    len: usize,
}

impl<'a, 'b: 'a, R: io::Read + 'b> de::SeqVisitor for SeqVisitor<'a, R> {
    type Error = Error;

    #[inline]
    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: de::DeserializeSeed
    {
        if self.len > 0 {
            self.len -= 1;
            let value = de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

struct TupleVisitor<'a, R: io::Read + 'a>(&'a mut Deserializer<R>);

impl<'a, 'b: 'a, R: io::Read + 'b> de::SeqVisitor for TupleVisitor<'a, R> {
    type Error = Error;

    #[inline]
    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: de::DeserializeSeed
    {
        let value = de::DeserializeSeed::deserialize(seed, &mut *self.0)?;
        Ok(Some(value))
    }
}

impl de::Error for Error {
    #[inline]
    fn custom<T: ::std::fmt::Display>(msg: T) -> Self {
        format!("{}", msg).into()
    }
}

struct MapVisitor<'a, R: io::Read + 'a> {
    deserializer: &'a mut Deserializer<R>,
    key: Vec<u8>,
    value: Vec<u8>,
    size: u32,
}

impl<'a, R: io::Read + 'a> MapVisitor<'a, R> {
    #[inline]
    fn pop_item(&mut self) -> Result<()> {
        let (len, data) = self.deserializer.get_string()?;
        if self.size < len {
            bail!(ErrorKind::BadMapEntry)
        }
        self.size -= len;
        let mut data = data.splitn(2, '=');
        self.key = match data.next() {
            Some(v) => Self::value_into_bytes(v)?,
            None => bail!(ErrorKind::BadMapEntry),
        };
        self.value = match data.next() {
            Some(v) => Self::value_into_bytes(v)?,
            None => bail!(ErrorKind::BadMapEntry),
        };
        Ok(())
    }

    #[inline]
    fn value_into_bytes(val: &str) -> Result<Vec<u8>> {
        use super::Serializer;
        use serde::Serialize;
        let mut answer = Vec::<u8>::new();
        val.serialize(&mut Serializer::new(&mut answer))?;
        Ok(answer)
    }
}

impl<'a, 'b: 'a, R: io::Read + 'b> de::MapVisitor for MapVisitor<'a, R> {
    type Error = Error;

    #[inline]
    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where K: de::DeserializeSeed
    {
        if self.size > 0 {
            self.pop_item()?;
            let mut deserializer = Deserializer::new(io::Cursor::new(&self.key));
            let key = de::DeserializeSeed::deserialize(seed, &mut deserializer)?;
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where V: de::DeserializeSeed
    {
        let mut deserializer = Deserializer::new(io::Cursor::new(&self.value));
        let value = de::DeserializeSeed::deserialize(seed, &mut deserializer)?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std;
    use serde::Deserialize;

    fn push_data(data: Vec<u8>) -> Deserializer<std::io::Cursor<Vec<u8>>> {
        Deserializer::new(std::io::Cursor::new(data))
    }

    #[test]
    fn reads_u8() {
        let mut decoder = push_data(vec![150]);
        assert_eq!(150, u8::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_u16() {
        let mut decoder = push_data(vec![0x34, 0xA2]);
        assert_eq!(0xA234, u16::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_u32() {
        let mut decoder = push_data(vec![0x45, 0x23, 1, 0xCD]);
        assert_eq!(0xCD012345, u32::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_u64() {
        let mut decoder = push_data(vec![0xBB, 0xAA, 0x10, 0x32, 0x54, 0x76, 0x98, 0xAB]);
        assert_eq!(0xAB9876543210AABB, u64::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_i8() {
        let mut decoder = push_data(vec![156]);
        assert_eq!(-100, i8::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_i16() {
        let mut decoder = push_data(vec![0xD0, 0x8A]);
        assert_eq!(-30000, i16::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_i32() {
        let mut decoder = push_data(vec![0x00, 0x6C, 0xCA, 0x88]);
        assert_eq!(-2000000000, i32::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_i64() {
        let mut decoder = push_data(vec![0x00, 0x00, 0x7c, 0x1d, 0xaf, 0x93, 0x19, 0x83]);
        assert_eq!(-9000000000000000000,
                   i64::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_f32() {
        let mut decoder = push_data(vec![0x00, 0x70, 0x7b, 0x44]);
        assert_eq!(1005.75, f32::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_f64() {
        let mut decoder = push_data(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x6e, 0x8f, 0x40]);
        assert_eq!(1005.75, f64::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_bool() {
        let mut decoder = push_data(vec![1]);
        assert_eq!(true, bool::deserialize(&mut decoder).unwrap());
        let mut decoder = push_data(vec![0]);
        assert_eq!(false, bool::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_string() {
        let mut decoder = push_data(vec![0, 0, 0, 0]);
        assert_eq!("", String::deserialize(&mut decoder).unwrap());
        let mut decoder = push_data(vec![13, 0, 0, 0, 72, 101, 108, 108, 111, 44, 32, 87, 111,
                                         114, 108, 100, 33]);
        assert_eq!("Hello, World!", String::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_array_struct() {
        #[derive(Debug,Deserialize,PartialEq)]
        struct TestArray([i16; 4]);
        let mut decoder = push_data(vec![8, 0, 0, 0, 7, 0, 1, 4, 33, 0, 57, 0]);
        assert_eq!(TestArray([7, 1025, 33, 57]),
                   TestArray::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_tuple_struct() {
        #[derive(Debug,Deserialize,PartialEq)]
        struct TestTuple(i16, bool, u8, String);
        let mut decoder = push_data(vec![14, 0, 0, 0, 2, 8, 1, 7, 6, 0, 0, 0, 65, 66, 67, 48, 49,
                                         50]);
        assert_eq!(TestTuple(2050, true, 7, String::from("ABC012")),
                   TestTuple::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_vector() {
        let mut decoder = push_data(vec![12, 0, 0, 0, 4, 0, 0, 0, 7, 0, 1, 4, 33, 0, 57, 0]);
        assert_eq!(vec![7, 1025, 33, 57],
                   Vec::<i16>::deserialize(&mut decoder).unwrap());
    }

    #[derive(Debug,Deserialize,PartialEq)]
    struct TestStructOne {
        a: i16,
        b: bool,
        c: u8,
        d: String,
        e: Vec<bool>,
    }

    #[test]
    fn reads_simple_struct() {
        let v = TestStructOne {
            a: 2050i16,
            b: true,
            c: 7u8,
            d: String::from("ABC012"),
            e: vec![true, false, false, true],
        };
        let mut decoder = push_data(vec![26, 0, 0, 0, 2, 8, 1, 7, 6, 0, 0, 0, 65, 66, 67, 48, 49,
                                         50, 8, 0, 0, 0, 4, 0, 0, 0, 1, 0, 0, 1]);
        assert_eq!(v, TestStructOne::deserialize(&mut decoder).unwrap());
    }

    #[derive(Debug,Deserialize,PartialEq)]
    struct TestStructPart {
        a: String,
        b: bool,
    }

    #[derive(Debug,Deserialize,PartialEq)]
    struct TestStructBig {
        a: Vec<TestStructPart>,
        b: String,
    }

    #[test]
    fn reads_complex_struct() {
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
        let mut decoder = push_data(vec![54, 0, 0, 0, 43, 0, 0, 0, 3, 0, 0, 0, 8, 0, 0, 0, 3, 0,
                                         0, 0, 65, 66, 67, 1, 10, 0, 0, 0, 5, 0, 0, 0, 49, 33,
                                         33, 33, 33, 1, 9, 0, 0, 0, 4, 0, 0, 0, 50, 51, 52, 98,
                                         0, 3, 0, 0, 0, 69, 69, 101]);
        assert_eq!(v, TestStructBig::deserialize(&mut decoder).unwrap());
    }

    #[test]
    fn reads_empty_string_string_map() {
        let input = vec![0, 0, 0, 0];
        let mut decoder = push_data(input);
        let data = std::collections::HashMap::<String, String>::deserialize(&mut decoder).unwrap();
        assert_eq!(0, data.len());
    }

    #[test]
    fn reads_single_element_string_string_map() {
        let input = vec![11, 0, 0, 0, 7, 0, 0, 0, 97, 98, 99, 61, 49, 50, 51];
        let mut decoder = push_data(input);
        let data = std::collections::HashMap::<String, String>::deserialize(&mut decoder).unwrap();
        assert_eq!(1, data.len());
        assert_eq!(Some(&String::from("123")), data.get("abc"));
    }

    #[test]
    fn reads_typical_header() {
        let input = vec![0xb0, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x6d, 0x65, 0x73, 0x73,
                         0x61, 0x67, 0x65, 0x5f, 0x64, 0x65, 0x66, 0x69, 0x6e, 0x69, 0x74, 0x69,
                         0x6f, 0x6e, 0x3d, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x20, 0x64, 0x61,
                         0x74, 0x61, 0x0a, 0x0a, 0x25, 0x00, 0x00, 0x00, 0x63, 0x61, 0x6c, 0x6c,
                         0x65, 0x72, 0x69, 0x64, 0x3d, 0x2f, 0x72, 0x6f, 0x73, 0x74, 0x6f, 0x70,
                         0x69, 0x63, 0x5f, 0x34, 0x37, 0x36, 0x37, 0x5f, 0x31, 0x33, 0x31, 0x36,
                         0x39, 0x31, 0x32, 0x37, 0x34, 0x31, 0x35, 0x35, 0x37, 0x0a, 0x00, 0x00,
                         0x00, 0x6c, 0x61, 0x74, 0x63, 0x68, 0x69, 0x6e, 0x67, 0x3d, 0x31, 0x27,
                         0x00, 0x00, 0x00, 0x6d, 0x64, 0x35, 0x73, 0x75, 0x6d, 0x3d, 0x39, 0x39,
                         0x32, 0x63, 0x65, 0x38, 0x61, 0x31, 0x36, 0x38, 0x37, 0x63, 0x65, 0x63,
                         0x38, 0x63, 0x38, 0x62, 0x64, 0x38, 0x38, 0x33, 0x65, 0x63, 0x37, 0x33,
                         0x63, 0x61, 0x34, 0x31, 0x64, 0x31, 0x0e, 0x00, 0x00, 0x00, 0x74, 0x6f,
                         0x70, 0x69, 0x63, 0x3d, 0x2f, 0x63, 0x68, 0x61, 0x74, 0x74, 0x65, 0x72,
                         0x14, 0x00, 0x00, 0x00, 0x74, 0x79, 0x70, 0x65, 0x3d, 0x73, 0x74, 0x64,
                         0x5f, 0x6d, 0x73, 0x67, 0x73, 0x2f, 0x53, 0x74, 0x72, 0x69, 0x6e, 0x67];
        let mut decoder = push_data(input);
        let data = std::collections::HashMap::<String, String>::deserialize(&mut decoder).unwrap();
        assert_eq!(6, data.len());
        assert_eq!(Some(&String::from("string data\n\n")),
                   data.get("message_definition"));
        assert_eq!(Some(&String::from("/rostopic_4767_1316912741557")),
                   data.get("callerid"));
        assert_eq!(Some(&String::from("1")), data.get("latching"));
        assert_eq!(Some(&String::from("992ce8a1687cec8c8bd883ec73ca41d1")),
                   data.get("md5sum"));
        assert_eq!(Some(&String::from("/chatter")), data.get("topic"));
        assert_eq!(Some(&String::from("std_msgs/String")), data.get("type"));
    }
}
