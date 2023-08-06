// use std::sync::{Arc, Mutex};
//
// use serde::{
// 	de::{Deserialize, Visitor},
// 	Deserializer,
// 	Serialize,
// 	Serializer,
// };
//
// struct State {}
//
// struct Test {
// 	state: Arc<Mutex<State>>,
// }
//
// fn transcode<'de, T: Deserialize<'de>>(from: impl Serialize) -> T {
// 	let test = Test { from };
//
// 	std::thread::spawn(|| from.serialize(test));
//
// 	T::deserialize(test)
// }
//
// impl Serializer for Test {
// 	type Ok = ();
// 	type Error = ();
// 	type SerializeSeq = ();
// 	type SerializeTuple = ();
// 	type SerializeTupleStruct = ();
// 	type SerializeTupleVariant = ();
// 	type SerializeMap = ();
// 	type SerializeStruct = ();
// 	type SerializeStructVariant = ();
//
// 	fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
// 	where
// 		T: Serialize,
// 	{
// 		todo!()
// 	}
//
// 	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_unit_variant(
// 		self,
// 		name: &'static str,
// 		variant_index: u32,
// 		variant: &'static str,
// 	) -> Result<Self::Ok, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_newtype_struct<T: ?Sized>(
// 		self,
// 		name: &'static str,
// 		value: &T,
// 	) -> Result<Self::Ok, Self::Error>
// 	where
// 		T: Serialize,
// 	{
// 		todo!()
// 	}
//
// 	fn serialize_newtype_variant<T: ?Sized>(
// 		self,
// 		name: &'static str,
// 		variant_index: u32,
// 		variant: &'static str,
// 		value: &T,
// 	) -> Result<Self::Ok, Self::Error>
// 	where
// 		T: Serialize,
// 	{
// 		todo!()
// 	}
//
// 	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_tuple_struct(
// 		self,
// 		name: &'static str,
// 		len: usize,
// 	) -> Result<Self::SerializeTupleStruct, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_tuple_variant(
// 		self,
// 		name: &'static str,
// 		variant_index: u32,
// 		variant: &'static str,
// 		len: usize,
// 	) -> Result<Self::SerializeTupleVariant, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_struct(
// 		self,
// 		name: &'static str,
// 		len: usize,
// 	) -> Result<Self::SerializeStruct, Self::Error> {
// 		todo!()
// 	}
//
// 	fn serialize_struct_variant(
// 		self,
// 		name: &'static str,
// 		variant_index: u32,
// 		variant: &'static str,
// 		len: usize,
// 	) -> Result<Self::SerializeStructVariant, Self::Error> {
// 		todo!()
// 	}
// }
//
// impl Deserializer for Test {
// 	type Error = ();
//
// 	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_unit_struct<V>(
// 		self,
// 		name: &'static str,
// 		visitor: V,
// 	) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_newtype_struct<V>(
// 		self,
// 		name: &'static str,
// 		visitor: V,
// 	) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_tuple_struct<V>(
// 		self,
// 		name: &'static str,
// 		len: usize,
// 		visitor: V,
// 	) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_struct<V>(
// 		self,
// 		name: &'static str,
// 		fields: &'static [&'static str],
// 		visitor: V,
// 	) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_enum<V>(
// 		self,
// 		name: &'static str,
// 		variants: &'static [&'static str],
// 		visitor: V,
// 	) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
//
// 	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
// 	where
// 		V: Visitor<'de>,
// 	{
// 		todo!()
// 	}
// }
