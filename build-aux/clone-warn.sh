#!/bin/sh
linenum=0 # Line we are currently reading
clonestartline=0 # Line which starts clone! call
inclone=0 # 1->we passed a clone! call but didn't get a closure
clonelines=0 # How many lines this clone! call has
prefix="" # All the whitespace required to reach same indent level as clone!
ret=0

while IFS= read -r line; do
	linenum=$((linenum + 1))

	# Our line has `clone!` lets work on it
	if printf "%s\\n" "$line" | grep -q '^.*clone!'; then
		# Check if we are in a clone! oneliner and skip everything
		printf "%s\\n" "$line" | grep -E -q 'clone!\(.*\)(|,|;)$' && continue

		# We are inside a clone!
		inclone=$((inclone + 1))

		# The starting line is the one we currently ara
		clonestartline=$linenum

		# Count the first line
		clonelines=1

		prefix="$(printf "%s\\n" "$line" | grep -o '^[[:space:]]*')"

		continue
	fi

	# We are inside a clone! call
	if [ $inclone -gt 0 ]; then
		clonelines=$((clonelines + 1))

		# Check for closing bracers
		if printf "%s\\n" "$line" | grep -E -q "^${prefix}[^ ]*)(|,|;)$"; then
			# It would be `2` since 3 is the minimum we want to trigger but
			# we have to account for the fact that there is an opening line
			# and a closing line
			if [ $clonelines -gt 5 ]; then
				# Easier than adding '>&2' to every single call
				(
					printf "clone! in line %s to %s is too long (%s lines)!\\n" \
						"$clonestartline" $linenum $((clonelines - 2))
					printf -- '```rust\n' 
					sed -n "$clonestartline,${linenum}p" "$1"
					printf -- '```\n\n'
				) >&2
				ret=$((ret + 1))
			fi
			# Reset everything for next run
			inclone=$((inclone - 1))
			clonelines=0
			prefix=""
			clonestartline=0
		fi
	fi
done < "${1:?'please give a file'}"
exit $ret
