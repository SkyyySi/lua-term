#!/usr/bin/env sh
unset LUA_PATH LUA_CPATH

eval "$(luarocks path --lua-version "$(command -- lua -e 'do
	local _VERSION = _VERSION
	assert(type(_VERSION) == "string")
	local version = _VERSION:match("^Lua (%d+%.%d+)$")
	assert(type(version) == "string")
	print(version)
end')")"

if [ -n "${ZSH_NAME:-}" ]; then
	eval 'LUA_CPATH="$(dirname -- "${(%):-%x}")/target/debug/lib?.so;$LUA_CPATH"'
elif [ -n "${BASH_SOURCE:-}" ]; then
	# shellcheck disable=2155,3028
	eval 'LUA_CPATH="$(dirname -- "${BASH_SOURCE:-$0}")/target/debug/lib?.so;$LUA_CPATH"'
fi

export LUA_CPATH
