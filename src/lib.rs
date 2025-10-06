// SPDX-license-identifier: MIT
#![allow(unused)]

use mlua::prelude::*;

macro_rules! table {
	( $lua:ident , { $( $key:ident = $value:expr ),* $(,)? } $(,)? ) => {
		::mlua::Result::and_then(::mlua::Lua::create_table($lua), |__table__| {
			$( ::mlua::Table::raw_set(
				&__table__,
				::std::stringify!($key),
				::mlua::IntoLua::into_lua($value, &$lua)?,
			)?; )*

			::mlua::Table::set_metatable(&__table__, ::core::option::Option::Some({
				let __mt__ = ::mlua::Lua::create_table(&$lua)?;

				::mlua::Table::raw_set(&__mt__, "__index", ::std::borrow::ToOwned::to_owned(&__table__))?;

				::mlua::Table::raw_set(&__mt__, "__tostring", ::mlua::Lua::create_function(&$lua, |lua, this: ::mlua::Table| {
					::mlua::IntoLua::into_lua(::std::format!("{this:#?}"), &lua)
				})?)?;

				__mt__
			}));

			::mlua::Result::Ok(__table__)
		})
	};
}

#[mlua::lua_module(name = "term")]
fn lua_term(lua: &Lua) -> LuaResult<LuaValue> {
	let exports: LuaTable = table!(lua, {
		foo = "bar"
	})?;

	exports.into_lua(lua)
}
