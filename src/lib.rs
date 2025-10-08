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
		KeyboardEnhancementFlags,
		KeyCode,
		KeyEvent,
		KeyModifiers,
		MediaKeyCode,
		ModifierKeyCode,
		PushKeyboardEnhancementFlags,
		PopKeyboardEnhancementFlags,
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

fn keycode_to_string(keycode: &KeyCode) -> String {
	macro_rules! plain_variants {
		( $variable:ident, $base:path, [ $( $variant:ident ),+ $(,)? ] $(, $default:expr)? ) => {
			match $variable {
				$( <$base>::$variant => {
					return ::std::stringify!($variant).to_string();
				}, )+
				$( _ => $default )?
			}
		};
	}

	plain_variants!(keycode, KeyCode, [
		Backspace,
		Enter,
		Left,
		Right,
		Up,
		Down,
		Home,
		End,
		PageUp,
		PageDown,
		Tab,
		BackTab,
		Delete,
		Insert,
		Null,
		Esc,
		CapsLock,
		ScrollLock,
		NumLock,
		PrintScreen,
		Pause,
		Menu,
		KeypadBegin
	], {});

	match keycode {
		KeyCode::F(i) => format!("F{i}"),
		KeyCode::Char(c) => c.to_string(),
		KeyCode::Media(m) => plain_variants!(m, MediaKeyCode, [
			Play,
			Pause,
			PlayPause,
			Reverse,
			Stop,
			FastForward,
			Rewind,
			TrackNext,
			TrackPrevious,
			Record,
			LowerVolume,
			RaiseVolume,
			MuteVolume,
		], unreachable!()),
		KeyCode::Modifier(m) => plain_variants!(m, ModifierKeyCode, [
			LeftShift,
			LeftControl,
			LeftAlt,
			LeftSuper,
			LeftHyper,
			LeftMeta,
			RightShift,
			RightControl,
			RightAlt,
			RightSuper,
			RightHyper,
			RightMeta,
			IsoLevel3Shift,
			IsoLevel5Shift,
		], unreachable!()),
		_ => unreachable!(),
	}
}

fn event_to_lua(lua: &Lua, event: &Event) -> LuaResult<LuaValue> {
	match event {
		Event::FocusGained => todo!(),
		Event::FocusLost => todo!(),
		Event::Key(key) => table!(lua, {
			code = keycode_to_string(&key.code),
			//modifiers = table!(lua, {})?,
			//kind = table!(lua, {})?,
			//state = table!(lua, {})?,
		})?.into_lua(lua),
		Event::Mouse(mouse) => todo!(),
		Event::Paste(text) => todo!(),
		Event::Resize(width, height) => todo!(),
	}
}

fn raw_read() -> std::io::Result<Event> {
	let mut stdout = std::io::stdout();

	enable_raw_mode();
	let result = execute!(
		 stdout,
		 EnableBracketedPaste,
		 EnableFocusChange,
		 EnableMouseCapture,
		 PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::all()),
	).and_then(|_| read());
	let cleanup = execute!(
		 stdout,
		 DisableBracketedPaste,
		 DisableFocusChange,
		 DisableMouseCapture,
		 PopKeyboardEnhancementFlags,
	);
	disable_raw_mode();

	cleanup.and(result)
}

fn input_loop(lua: &Lua, callback: LuaFunction) -> LuaResult<()> {
	let mut last_key: Option<KeyEvent> = None;

	loop {
		let event = raw_read().map_err(LuaError::runtime)?;
		eprintln!("{event:#?}");
		callback.call::<()>(event_to_lua(lua, &event)?)?;

		let Event::Key(key) = event else {
			continue;
		};

		if matches!(key, KeyEvent {
			code: KeyCode::Char('c'),
			modifiers: KeyModifiers::CONTROL,
			..
		}) && (last_key == Some(key)) {
			break;
		};

		last_key = Some(key);
	}

	Ok(())
}

#[mlua::lua_module(name = "term")]
fn lua_term(lua: &Lua) -> LuaResult<LuaValue> {
	let exports: LuaTable = table!(lua, {
		input_loop = lua.create_function(input_loop)?,
	})?;

	exports.into_lua(lua)
}
