#!/usr/bin/env bash

readonly -a known_hash_algos=(
	ck
	md5
	sha1
	sha224
	sha256
	sha384
	sha512
	b2
)

readonly -a base_pkgbuild_vars=(
	arch
	backup
	changelog
	checkdepends
	conflicts
	depends
	groups
	epoch
	install
	license
	makedepends
	noextract
	optdepends
	options
	pkgbase
	pkgdesc
	pkgname
	pkgrel
	pkgver
	provides
	replaces
	source
	url
	validpgpkeys
	"${known_hash_algos[@]/%/sums}"
)

readonly -a pkgbuild_functions=(
	pkgver
	verify
	prepare
	build
	check
	package
)

pkgbuild_vars=( "${base_pkgbuild_vars[@]}" )

source_safe() {
	local file="$1"
	local shellopts
	shellopts=$(shopt -p extglob)
	shopt -u extglob

  # shellcheck source=/usr/share/pacman/PKGBUILD.proto
	if ! source "$file"; then
		exit 1
	fi

	eval "$shellopts"
}

escape() {
	local val="$1"
	val="${val//\\/\\\\}"
	val="${val//\"/\\\"}"
	val="${val//$'\n'/\\\\n}"
	printf -- "%s" "$val"
}

expand_pkgbuild_vars() {
	local a

	if [[ $(typeof_var arch) == ARRAY ]]; then
		for a in "${arch[@]}"; do
			pkgbuild_vars+=( "${base_pkgbuild_vars[@]/%/_$a}" )
		done
	fi

	readonly -a pkgbuild_vars
}

typeof_var() {
	local type
	type=$(declare -p "$1" 2>/dev/null)

	if [[ "$type" == "declare --"* ]]; then
		printf "STRING"
	elif [[ "$type" == "declare -a"* ]]; then
		printf "ARRAY"
	elif [[ "$type" == "declare -A"* ]]; then
		printf "MAP"
	else
		printf "NONE"
	fi
}

dump_string() {
	local varname=$1 prefix="$2"
	local val
	val="$(escape "${!varname}")"

	printf -- '%s STRING %s "%s"\n' "$prefix" "$varname" "$val"

}

dump_array() {
	local val varname=$1 prefix="$2"
	local arr="$varname"'[@]'

	printf -- '%s ARRAY %s' "$prefix" "$varname"

	for val in "${!arr}"; do
		val="$(escape "$val")"
		printf -- ' "%s"' "$val"
	done

	printf '\n'
}

dump_map() {
	local key varname=$1 prefix="$2"
	declare -n map=$varname

	printf -- '%s MAP %s' "$prefix" "$varname"

	for key in "${!map[@]}"; do
		val="${map[$key]}"

		key="$(escape "$key")"
		val="$(escape "$val")"

		printf -- ' "%s" "%s"' "$key" "$val"
	done

	printf '\n'
}

dump_var() {
	local varname=$1 prefix="${2:-"VAR GLOBAL"}"
	local type
	type=$(typeof_var "$varname")

	if [[ $type == STRING ]]; then
		dump_string "$varname" "$prefix"
	elif [[ $type == ARRAY ]]; then
		dump_array "$varname" "$prefix"
	elif [[ $type == MAP ]]; then
		dump_map "$varname" "$prefix"
	fi
}

grep_function() {
	local funcname="$1" regex="$2"

	declare -f "$funcname" 2>/dev/null | grep -E "$regex"
}

dump_function_vars() {
	local funcname="$1" varname attr_regex decl new_vars
	declare -A new_vars
	printf -v attr_regex '^[[:space:]]* [a-z1-9_]*\+?='

	if ! function_exists "$funcname"; then
		return
	fi

	# this function requires extglob - save current status to restore later
	local shellopts
	shellopts=$(shopt -p extglob)
	shopt -s extglob

	while read -r; do
		# strip leading whitespace and any usage of declare
		decl=${REPLY##*([[:space:]])}
		varname=${decl%%[+=]*}

		local -I "$varname"
		new_vars[$varname]=1
		eval "$decl"
	done < <(grep_function "$funcname" "$attr_regex")

	for varname in "${pkgbuild_vars[@]}"; do
		if [[ -v "new_vars[$varname]" ]]; then
			dump_var "$varname" "VAR FUNCTION $funcname"
		fi
	done

	eval "$shellopts"
}

dump_functions_vars() {
	local name

	dump_function_vars package

	for name in "${pkgname[@]}"; do
		dump_function_vars "package_${name}"
	done
}


dump_global_vars() {
	local varname

	for varname in "${pkgbuild_vars[@]}"; do
		dump_var "$varname"
	done
}

function_exists() {
	declare -f "$1" >/dev/null
}

dump_function_name() {
	local funcname="$1"

	if function_exists "$funcname"; then
		printf -- "FUNCTION %s\n" "$funcname"
	fi
}

dump_function_names() {
	local name funcname

	for funcname in "${pkgbuild_functions[@]}"; do
		dump_function_name "$funcname"
	done

	for name in "${pkgname[@]}"; do
		dump_function_name "package_${name}"
	done
}

dump_pkgbuild() {
	source_safe "$1"

	expand_pkgbuild_vars
	dump_global_vars
	dump_functions_vars
	dump_function_names
}

# usage:
# pkgbuild dump <path/to/pkgbuild>

if [[ "$1" == dump ]]; then
	shift
	dump_pkgbuild "$@"
fi

exit 0
