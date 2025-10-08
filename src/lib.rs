// SPDX-license-identifier: MIT
#![allow(unused)]

use crossterm::{
	event::{
		read,
		DisableBracketedPaste,
		DisableFocusChange,
		DisableMouseCapture,
		EnableBracketedPaste,
		EnableFocusChange,
		EnableMouseCapture,
		Event,
		KeyCode,
		KeyEvent,
		KeyModifiers,
	},
	terminal::{
		disable_raw_mode,
		enable_raw_mode,
	},
	execute,
};
use mlua::prelude::*;

#[macro_export(local_inner_macros)]
macro_rules! table {
	( $lua:ident, $body:tt $(,)? ) => {
		::mlua::Result::and_then($crate::table!(;;PARSE_BODY;; $lua, $body), |__table__| {
			::mlua::Table::set_metatable(&__table__, ::core::option::Option::Some($crate::table!(;;PARSE_BODY;; $lua, {
				__index = ::std::borrow::ToOwned::to_owned(&__table__),
				__tostring = ::mlua::Lua::create_function(&$lua, |lua, this: ::mlua::Table| {
					::mlua::IntoLua::into_lua(::std::format!("{this:#?}"), &lua)
				})?,
			})?));

			::mlua::Result::Ok(__table__)
		})
	};

	( ;;PARSE_BODY;; $lua:ident, { $( $key:tt = $value:expr ),* $(,)? } ) => {
		::mlua::Result::and_then(::mlua::Lua::create_table($lua), |__table__| {
			$( ::mlua::Table::raw_set(
				&__table__,
				::mlua::IntoLua::into_lua($crate::table!(;;PARSE_KEY;; $key), &$lua)?,
				::mlua::IntoLua::into_lua($value, &$lua)?,
			)?; )*

			::mlua::Result::Ok(__table__)
		})
	};

	( ;;PARSE_KEY;; $key:ident ) => {
		::std::stringify!($key)
	};

	( ;;PARSE_KEY;; [ $key:expr ] ) => {
		$key
	};

	( ;;PARSE_KEY;; $key:literal ) => {
		$key
	};
}

#[mlua::lua_module(name = "term")]
fn lua_term(lua: &Lua) -> LuaResult<LuaValue> {
	let exports: LuaTable = table!(lua, {
		with_raw_mode = lua.create_function(|_lua, (f, args): (LuaFunction, LuaMultiValue)| {
			let mut stdout = std::io::stdout();

			enable_raw_mode();
			let result: LuaResult<LuaMultiValue> = execute!(
				 stdout,
				 EnableBracketedPaste,
				 EnableFocusChange,
				 EnableMouseCapture
			).map_err(LuaError::runtime).and_then(|_| f.call(args));
			let cleanup = execute!(
				 stdout,
				 DisableBracketedPaste,
				 DisableFocusChange,
				 DisableMouseCapture
			);
			disable_raw_mode();

			cleanup.map_err(LuaError::runtime).and(result)
		})?,
		input_loop = lua.create_function(|_lua, callback: LuaFunction| {
			let mut last_key: Option<KeyEvent> = None;

			while let Ok(event) = crossterm::event::read() {
				eprintln!("{}\r", format!("{event:#?}").replace("\n", "\n\r"));

				if !matches!(event, Event::Key(KeyEvent {
					code: KeyCode::Char('c'),
					modifiers: KeyModifiers::CONTROL,
					..
				})) {
					continue;
				};

				/*if !matches!(last_key, Some(Event::Key(event))) {
					continue;
				};*/
			}

			Ok(())
		})?,
	})?;

	exports.into_lua(lua)
}
